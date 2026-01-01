use crate::configuration::Settings;
use crate::graceful::GracefulShutdown;
use crate::http_client::HttpClientManager;
use crate::model::entity::task::{ActiveTaskInfo, QueueEvent, Task, TaskStatus, TaskType};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::sync::{broadcast, Notify, RwLock};

#[derive(Debug, Clone)]
pub struct QueueState {
    pub tasks: Arc<RwLock<VecDeque<Task>>>,
    pub active_tasks: Arc<RwLock<HashMap<String, ActiveTaskInfo>>>,
    pub task_store: Arc<RwLock<HashMap<String, Task>>>,
    pub sender: broadcast::Sender<QueueEvent>,
    pub notify: Arc<Notify>,
}

impl Default for QueueState {
    fn default() -> Self {
        Self::new()
    }
}

impl QueueState {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1024);
        Self {
            tasks: Arc::new(RwLock::new(VecDeque::new())),
            active_tasks: Arc::new(RwLock::new(HashMap::new())),
            task_store: Arc::new(RwLock::new(HashMap::new())),
            sender,
            notify: Arc::new(Notify::new()),
        }
    }
    pub async fn register_active_task(&self, task: &Task, worker_id: usize) {
        let mut active_tasks = self.active_tasks.write().await;

        let task_type_str = match task.task_type {
            TaskType::HtmlParse { .. } => "html".to_string(),
            TaskType::PicDownload { .. } => "pic".to_string(),
            TaskType::CbzArchive { .. } => "cbz".to_string(),
        };
        let active_task = ActiveTaskInfo {
            task_id: task.id.clone(),
            task_type: task_type_str,
            description: task.description(),
            worker_id,
            started_at: OffsetDateTime::now_utc(),
            duration_secs: 0.0,
            progress: None,
        };
        active_tasks.insert(task.id.clone(), active_task);
        tracing::debug!("register active task {} (worker {})", task.id, worker_id);
    }
    pub async fn unregister_active_task(&self, task_id: &str) -> bool {
        let mut active_tasks = self.active_tasks.write().await;
        let removed = active_tasks.remove(task_id).is_some();
        if removed {
            tracing::debug!("unregister active task {}", task_id);
        }
        removed
    }
    pub async fn update_task_progress(&self, task_id: &str, progress: f64) -> bool {
        let mut active_tasks = self.active_tasks.write().await;
        if let Some(active_task) = active_tasks.get_mut(task_id) {
            active_task.progress = Some(progress);
            let diff = (OffsetDateTime::now_utc() - active_task.started_at).whole_milliseconds();
            active_task.duration_secs = diff as f64 / 1000.0;
            true
        } else {
            false
        }
    }
    pub async fn get_active_tasks(&self) -> Vec<ActiveTaskInfo> {
        let active_tasks = self.active_tasks.read().await;
        let now = OffsetDateTime::now_utc();
        active_tasks
            .values()
            .map(|t| {
                let mut task = t.clone();
                let diff = (now - task.started_at).whole_milliseconds();
                task.duration_secs = diff as f64 / 1000.0;
                task
            })
            .collect()
    }
    pub async fn active_task_count(&self) -> usize {
        let active_tasks = self.active_tasks.read().await;
        active_tasks.len()
    }
    pub async fn size(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.len()
    }
    pub async fn get_tasks(&self) -> Vec<Task> {
        let task_store = self.task_store.read().await;
        task_store.values().cloned().collect()
    }
    pub async fn get_task(&self, task_id: &str) -> Option<Task> {
        let task_store = self.task_store.read().await;
        task_store.get(task_id).cloned()
    }
    pub async fn update_task(&self, updated_task: Task) -> bool {
        let mut task_store = self.task_store.write().await;
        task_store.insert(updated_task.id.clone(), updated_task.clone());
        if let Err(e) = self.sender.send(QueueEvent::TaskUpdated(updated_task)) {
            tracing::warn!("send task updated event failed: {:?}", e);
        }
        true
    }
    pub async fn enqueue(&self, task: Task) {
        let task_clone = task.clone();
        let mut tasks = self.tasks.write().await;
        tasks.push_back(task.clone());
        let mut task_store = self.task_store.write().await;
        task_store.insert(task.id.clone(), task);
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
            && let Err(e) = self.sender.send(QueueEvent::QueueCleared) {
                tracing::warn!("send tasks cleared event failed: {:?}", e);
            }
        cleared
    }
    pub async fn cleanup_completed_tasks(&self, keep_recent: usize) -> usize {
        let mut task_store = self.task_store.write().await;
        let mut completed_tasks: Vec<String> = Vec::new();
        for (id, task) in task_store.iter() {
            if let TaskStatus::Completed = task.status {
                completed_tasks.push(id.clone());
            }
        }
        let mut removed_count = 0;
        if completed_tasks.len() > keep_recent {
            completed_tasks.sort_by(|a, b| {
                let a = task_store.get(a).unwrap();
                let b = task_store.get(b).unwrap();
                b.created_at.cmp(&a.created_at)
            });
            let to_remove = completed_tasks.len() - keep_recent;
            for id in completed_tasks.iter().take(to_remove) {
                task_store.remove(id);
                removed_count += 1;
            }
        }
        removed_count
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub queue_state: QueueState,
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
        let queue_state = QueueState::new();
        let db_pool = Arc::new(
            PgPoolOptions::new()
                .acquire_timeout(std::time::Duration::from_secs(2))
                .connect_lazy_with(configuration.database.with_db()),
        );
        let shutdown = Arc::new(GracefulShutdown::new());

        let http_client = Arc::new(HttpClientManager::new(Some(
            configuration.http_client.clone(),
        )));

        {
            //db migration
            sqlx::migrate!()
                .run(&*db_pool)
                .await
                .expect("Could not run database migrations.");
        }

        Self {
            queue_state,
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
