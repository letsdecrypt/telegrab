use crate::configuration::Settings;
use crate::graceful::{GracefulShutdown, TaskGuard};
use crate::http_client::HttpClientManager;
use crate::model::entity::task::{QueueEvent, TaskType};
use crate::state::{AppState, QueueState};
use crate::{service, Result};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TaskWorker {
    queue_state: QueueState,
    shutdown: Arc<GracefulShutdown>,
    http_client: Arc<HttpClientManager>,
    db_pool: Arc<PgPool>,
    worker_id: usize,
}

impl TaskWorker {
    pub fn new(app_state: &AppState, worker_id: usize) -> Self {
        Self {
            queue_state: app_state.queue_state.clone(),
            shutdown: app_state.shutdown.clone(),
            http_client: app_state.http_client.clone(),
            db_pool: app_state.db_pool.clone(),
            worker_id,
        }
    }
    pub async fn start(&self) {
        tracing::info!("Worker {} started", self.worker_id);
        let mut shutdown_rx = self.shutdown.get_shutdown_rx().await;
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("Worker {} received shutdown signal, stop to receive new tasks", self.worker_id);
                    break;
                }
                _ = async {
                    loop{
                        if self.shutdown.is_shutting_down().await{
                            tracing::info!("Worker {} is shutting down, no more waiting  for tasks", self.worker_id);
                            return;
                        }
                        let has_task = self.queue_state.wait_for_task(Some(Duration::from_secs(5))).await;
                        if has_task {
                            break;
                        } else {
                            continue;
                        }

                    }
                    match self.process_queue_with_guard().await{
                        Ok(Some(true)) => {
                            tracing::info!("Worker {} processed a task", self.worker_id);
                        }
                        Ok(Some(false)) => {
                            // empty queue, wait for next task
                        }
                        Ok(None) => {
                            tracing::info!("Worker {} is shutting down, no more new tasks", self.worker_id);
                        }
                        Err(e) => {
                            tracing::error!("Worker {} encountered an error on queue: {}", self.worker_id, e);
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }=>{}
            }
        }
        tracing::info!(
            "Worker {} waiting for current tasks to finish",
            self.worker_id
        );
        self.wait_for_current_tasks().await;
        tracing::info!("Worker {} stopped", self.worker_id);
    }
    async fn process_queue_with_guard(&self) -> Result<Option<bool>, String> {
        let _guard = match TaskGuard::new(self.shutdown.clone()).await {
            Some(guard) => guard,
            None => return Ok(None),
        };

        let task = self.queue_state.dequeue().await;
        match task {
            Some(mut task) => {
                tracing::info!("Worker {} processing task: {:?}", self.worker_id, task);
                task.mark_processing();

                /*if !self.queue_state.update_task(task.clone()).await {
                    tracing::warn!(
                        "Worker {} can not update task {}, it may be processed by other worker",
                        self.worker_id,
                        task.id
                    );
                    return Ok(Some(false));
                }*/
                self.queue_state
                    .register_active_task(&task, self.worker_id)
                    .await;
                let result = match &task.task_type {
                    TaskType::HtmlParse { id: doc_id } => {
                        self.process_html_parse_task(doc_id).await
                    }
                    TaskType::PicDownload { id: pic_id } => {
                        self.process_pic_download_task(pic_id).await
                    }
                    TaskType::CbzArchive { id: doc_id } => {
                        self.process_cbz_archive_task(doc_id).await
                    }
                };
                self.queue_state.unregister_active_task(&task.id).await;
                match result {
                    Ok(task_result) => {
                        task.mark_completed(task_result);
                        tracing::info!(
                            "Worker {} processed task {} successfully",
                            self.worker_id,
                            task.id
                        );
                    }
                    Err(err) => {
                        task.mark_failed(err.to_string());
                        tracing::warn!(
                            "Worker {} processed task {} failed: {}",
                            self.worker_id,
                            task.id,
                            err
                        );
                    }
                }
                /*if !self.queue_state.update_task(task.clone()).await {
                    tracing::warn!(
                        "Worker {} can not update task {} to final state",
                        self.worker_id,
                        task.id
                    );
                }*/
                if let Err(err) = self
                    .queue_state
                    .sender
                    .send(QueueEvent::TaskRemoved(task.id.clone()))
                {
                    tracing::warn!(
                        "Worker {} send TaskRemoved event {} failed: {}",
                        self.worker_id,
                        task.id,
                        err
                    );
                }
                Ok(Some(true))
            }
            None => Ok(Some(false)),
        }
    }
    async fn process_html_parse_task(&self, id: &i32) -> Result<Option<String>> {
        let doc = service::doc::get_doc_by_id(&self.db_pool, *id).await?;
        if doc.status==1 && doc.page_title.is_some(){
            return Ok(doc.page_title)
        }
        let telegraph_post = self.http_client.parse_telegraph_post(&doc.url).await?;
        let doc = service::doc::update_parsed_doc(&self.db_pool, *id, telegraph_post).await?;
        Ok(doc.page_title)
    }
    async fn process_pic_download_task(&self, pic_id: &i32) -> Result<Option<String>> {
        todo!("Download pic {}", pic_id);
    }
    async fn process_cbz_archive_task(&self, doc_id: &i32) -> Result<Option<String>> {
        todo!("Archive html doc {}", doc_id);
    }
    async fn wait_for_current_tasks(&self) {
        let active_tasks = self.queue_state.active_task_count().await;
        if active_tasks > 0 {
            tracing::info!(
                "Worker {} waiting for {} active tasks to finish",
                self.worker_id,
                active_tasks
            );
            for i in 0..30 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let remaining = self.queue_state.active_task_count().await;
                if remaining == 0 {
                    tracing::info!("Worker {} all active tasks finished", self.worker_id);
                    return;
                }
                tracing::info!(
                    "Worker {} awaiting...({}/30s), remaining tasks: {}",
                    self.worker_id,
                    i,
                    remaining
                );
            }
            tracing::warn!(
                "Worker {} timed out waiting, force shutdown",
                self.worker_id
            );
        } else {
            tracing::info!("Worker {} no active task", self.worker_id);
        }
    }
}

pub async fn start_background_workers(state: AppState, configuration: Settings) {
    let queue_state = state.queue_state.clone();
    let shutdown = state.shutdown.clone();
    let http_client = state.http_client.clone();
    let db_pool = state.db_pool.clone();
    let worker_count = configuration.worker.count;
    tracing::info!("Start {} worker(s)", worker_count);
    for worker_id in 0..worker_count {
        let worker = TaskWorker {
            queue_state: queue_state.clone(),
            shutdown: shutdown.clone(),
            http_client: http_client.clone(),
            db_pool: db_pool.clone(),
            worker_id,
        };
        tokio::spawn(async move {
            worker.start().await;
        });
    }
}
