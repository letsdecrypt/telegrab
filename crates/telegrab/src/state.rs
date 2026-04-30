use crate::configuration::Settings;
use crate::graceful::GracefulShutdown;
use crate::http_client::HttpClientManager;
use crate::model::entity::task::{ActiveTaskInfo, QueueEvent, Task, TaskStatus, TaskType};
use dashmap::DashMap;
use sqlx_postgres::{PgPool, PgPoolOptions};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::sync::{Mutex, Notify, RwLock, broadcast};

#[derive(Debug, Clone)]
pub struct QueueState {
    pub tasks: Arc<RwLock<VecDeque<Task>>>,
    pub active_tasks: Arc<DashMap<String, ActiveTaskInfo>>,
    pub task_store: Arc<DashMap<String, Task>>,
    pub sender: broadcast::Sender<QueueEvent>,
    pub notify: Arc<Notify>,
    pub max_total_tasks: usize,
}

impl Default for QueueState {
    fn default() -> Self {
        Self::new(1000)
    }
}

impl QueueState {
    pub fn new(max_total_tasks: usize) -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self {
            tasks: Arc::new(RwLock::new(VecDeque::new())),
            active_tasks: Arc::new(DashMap::new()),
            task_store: Arc::new(DashMap::new()),
            sender,
            notify: Arc::new(Notify::new()),
            max_total_tasks,
        }
    }
    pub async fn register_active_task(&self, task: &Task, worker_id: usize) {
        let active_task = ActiveTaskInfo {
            task_id: task.id.clone(),
            task_type: task.task_type.clone(),
            description: task.description(),
            worker_id,
            started_at: OffsetDateTime::now_utc(),
            duration_secs: 0.0,
            progress: None,
        };
        self.active_tasks.insert(task.id.clone(), active_task);
        tracing::debug!("register active task {} (worker {})", task.id, worker_id);
    }
    pub async fn unregister_active_task(&self, task_id: &str) -> bool {
        let removed = self.active_tasks.remove(task_id).is_some();
        if removed {
            tracing::debug!("unregister active task {}", task_id);
        }
        removed
    }
    pub async fn update_task_progress(&self, task_id: &str, progress: f64) -> bool {
        if let Some(mut active_task) = self.active_tasks.get_mut(task_id) {
            active_task.progress = Some(progress);
            let diff = (OffsetDateTime::now_utc() - active_task.started_at).whole_milliseconds();
            active_task.duration_secs = diff as f64 / 1000.0;
            drop(active_task); // release the lock before sending event
            if let Err(e) = self
                .sender
                .send(QueueEvent::TaskProgress(task_id.to_string(), progress))
            {
                tracing::warn!("send task progress event failed: {:?}", e);
            }
            true
        } else {
            false
        }
    }
    pub async fn get_active_tasks(&self) -> Vec<ActiveTaskInfo> {
        let now = OffsetDateTime::now_utc();
        self.active_tasks
            .iter()
            .map(|r| {
                let mut task = r.value().clone();
                let diff = (now - task.started_at).whole_milliseconds();
                task.duration_secs = diff as f64 / 1000.0;
                task
            })
            .collect()
    }
    pub async fn active_task_count(&self) -> usize {
        self.active_tasks.len()
    }
    pub async fn find_doc_in_queue(&self, doc_id: i32) -> Option<Task> {
        let tasks = self.tasks.read().await;
        tasks
            .iter()
            .find(|t| match t.task_type {
                TaskType::HtmlParse { id } => id == doc_id,
                TaskType::DocDownload { id } => id == doc_id,
                TaskType::CbzArchive { id } => id == doc_id,
                _ => false,
            })
            .cloned()
    }
    pub async fn find_pic_in_queue(&self, pic_id: i32) -> Option<Task> {
        let tasks = self.tasks.read().await;
        tasks
            .iter()
            .find(|t| match t.task_type {
                TaskType::PicDownload { id } => id == pic_id,
                _ => false,
            })
            .cloned()
    }
    pub async fn is_doc_active(&self, doc_id: i32) -> bool {
        self.active_tasks.iter().any(|r| match r.value().task_type {
            TaskType::HtmlParse { id } => id == doc_id,
            TaskType::DocDownload { id } => id == doc_id,
            TaskType::CbzArchive { id } => id == doc_id,
            _ => false,
        })
    }
    pub async fn is_pic_active(&self, pic_id: i32) -> bool {
        self.active_tasks.iter().any(|r| match r.value().task_type {
            TaskType::PicDownload { id } => id == pic_id,
            _ => false,
        })
    }
    pub async fn is_scan_active(&self) -> bool {
        self.active_tasks
            .iter()
            .any(|r| matches!(r.value().task_type, TaskType::ScanDir))
    }
    pub async fn is_parse_all_active(&self) -> bool {
        self.active_tasks
            .iter()
            .any(|r| matches!(r.value().task_type, TaskType::HtmlParseAll))
    }
    pub async fn size(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.len()
    }
    pub async fn get_tasks(&self) -> Vec<Task> {
        self.task_store.iter().map(|r| r.value().clone()).collect()
    }
    pub async fn get_task(&self, task_id: &str) -> Option<Task> {
        self.task_store.get(task_id).map(|r| r.value().clone())
    }
    pub async fn update_task(&self, updated_task: Task) -> bool {
        self.task_store
            .insert(updated_task.id.clone(), updated_task.clone());
        if let Err(e) = self.sender.send(QueueEvent::TaskUpdated(updated_task)) {
            tracing::warn!("send task updated event failed: {:?}", e);
        }
        true
    }
    pub async fn enqueue(&self, task: Task) {
        let task_clone = task.clone();
        let mut tasks = self.tasks.write().await;
        tasks.push_back(task.clone());
        self.task_store.insert(task.id.clone(), task);
        self.notify.notify_one();
        if let Err(e) = self.sender.send(QueueEvent::TaskAdded(task_clone)) {
            tracing::warn!("send task enqueued event failed: {:?}", e);
        }
    }
    pub async fn dequeue(&self) -> Option<Task> {
        let mut tasks = self.tasks.write().await;
        tasks.pop_front()
    }
    pub async fn wait_for_task(&self, timeout: Option<Duration>) -> bool {
        {
            let tasks = self.tasks.read().await;
            if !tasks.is_empty() {
                return true;
            }
        }
        match timeout {
            Some(t) => {
                tokio::select! {
                    _ = tokio::time::sleep(t) => false,
                    _ = self.notify.notified() => true,
                }
            }
            None => {
                self.notify.notified().await;
                true
            }
        }
    }
    pub async fn clear(&self) -> Vec<Task> {
        let mut tasks = self.tasks.write().await;
        let cleared: Vec<Task> = tasks.drain(..).collect();
        if !cleared.is_empty()
            && let Err(e) = self.sender.send(QueueEvent::QueueCleared)
        {
            tracing::warn!("send tasks cleared event failed: {:?}", e);
        }
        cleared
    }
    pub async fn cleanup_completed_tasks(&self, keep_recent: usize) -> usize {
        let mut completed_tasks: Vec<String> = Vec::new();
        let mut failed_tasks: Vec<String> = Vec::new();
        for r in self.task_store.iter() {
            match r.value().status {
                TaskStatus::Completed => completed_tasks.push(r.key().clone()),
                TaskStatus::Failed => failed_tasks.push(r.key().clone()),
                _ => {}
            }
        }
        let mut removed_count = 0;
        // Prune oldest completed tasks beyond keep_recent
        if completed_tasks.len() > keep_recent {
            completed_tasks.sort_by(|a, b| {
                let a = self.task_store.get(a).unwrap();
                let b = self.task_store.get(b).unwrap();
                a.created_at.cmp(&b.created_at)
            });
            let to_remove = completed_tasks.len() - keep_recent;
            for id in completed_tasks.iter().take(to_remove) {
                self.task_store.remove(id);
                removed_count += 1;
            }
        }
        // If still over max_total_tasks, prune oldest failed tasks
        let excess = self.task_store.len().saturating_sub(self.max_total_tasks);
        if excess > 0 && !failed_tasks.is_empty() {
            failed_tasks.sort_by(|a, b| {
                let a = self.task_store.get(a).unwrap();
                let b = self.task_store.get(b).unwrap();
                a.created_at.cmp(&b.created_at)
            });
            for id in failed_tasks.iter().take(excess) {
                if self.task_store.remove(id).is_some() {
                    removed_count += 1;
                }
            }
        }
        removed_count
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub queue_state: Arc<QueueState>,
    pub fs_watcher: Arc<Mutex<Option<notify::RecommendedWatcher>>>,
    pub shutdown: Arc<GracefulShutdown>,
    pub db_pool: Arc<PgPool>,
    pub http_client: Arc<HttpClientManager>,
    pub base_url: String,
    pub worker_count: usize,
    pub pic_dir: String,
    pub cbz_dir: String,
}

impl AppState {
    pub async fn build(configuration: &Settings) -> Self {
        let queue_state = Arc::new(QueueState::new(configuration.worker.max_total_tasks));
        let db_pool = Arc::new(
            PgPoolOptions::new()
                .acquire_timeout(Duration::from_secs(5))
                .max_connections(configuration.database.max_connections)
                .min_connections(configuration.database.min_connections)
                .connect_with(configuration.database.with_db())
                .await
                .expect("Failed to connect to database"),
        );
        let shutdown = Arc::new(GracefulShutdown::new());

        let http_client = Arc::new(HttpClientManager::new(Some(
            configuration.http_client.clone(),
        )));

        if configuration.database.auto_migrate {
            sqlx::migrate!("../../migrations")
                .run(&*db_pool)
                .await
                .expect("Could not run database migrations.");
        }

        Self {
            queue_state,
            fs_watcher: Arc::new(Mutex::new(None)),
            shutdown,
            db_pool,
            http_client,
            base_url: configuration.application.base_url.clone(),
            worker_count: configuration.worker.count,
            pic_dir: configuration.pic_dir.clone(),
            cbz_dir: configuration.cbz_dir.clone(),
        }
    }
}
