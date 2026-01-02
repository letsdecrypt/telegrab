use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TaskType {
    HtmlParse { id: i32 },
    PicDownload { id: i32 },
    CbzArchive { id: i32 },
    ScanDir,
    RemoveCbz { id: i32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Processing,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub task_type: TaskType,
    pub status: TaskStatus,
    pub created_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub completed_at: Option<OffsetDateTime>,
    pub result: Option<String>,
    pub error: Option<String>,
}

impl Task {
    pub fn new_html_parse_task(doc_id: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_type: TaskType::HtmlParse { id: doc_id },
            status: TaskStatus::Pending,
            created_at: OffsetDateTime::now_utc(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        }
    }
    pub fn new_pic_download_task(doc_id: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_type: TaskType::PicDownload { id: doc_id },
            status: TaskStatus::Pending,
            created_at: OffsetDateTime::now_utc(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        }
    }
    pub fn new_cbz_archive_task(doc_id: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_type: TaskType::CbzArchive { id: doc_id },
            status: TaskStatus::Pending,
            created_at: OffsetDateTime::now_utc(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        }
    }
    pub fn new_scan_dir_task() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_type: TaskType::ScanDir,
            status: TaskStatus::Pending,
            created_at: OffsetDateTime::now_utc(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        }
    }
    pub fn new_remove_cbz_task(cbz_id: i32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_type: TaskType::RemoveCbz { id: cbz_id },
            status: TaskStatus::Pending,
            created_at: OffsetDateTime::now_utc(),
            started_at: None,
            completed_at: None,
            result: None,
            error: None,
        }
    }
    pub fn mark_processing(&mut self) {
        self.status = TaskStatus::Processing;
        self.started_at = Some(OffsetDateTime::now_utc());
    }
    pub fn mark_completed(&mut self, result: Option<String>) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(OffsetDateTime::now_utc());
        self.result = result;
    }
    pub fn mark_failed(&mut self, error: String) {
        self.status = TaskStatus::Failed(error.clone());
        self.completed_at = Some(OffsetDateTime::now_utc());
        self.error = Some(error);
    }
    pub fn description(&self) -> String {
        match &self.task_type {
            TaskType::HtmlParse { id: doc_id } => format!("Parse doc: {}", doc_id),
            TaskType::PicDownload { id: pic_id } => {
                format!("Download pic: {}", pic_id)
            }
            TaskType::CbzArchive { id: doc_id } => {
                format!("Archive doc: {}", doc_id)
            }
            TaskType::ScanDir => "Scan dir".to_string(),
            TaskType::RemoveCbz { id: cbz_id } => {
                format!("Remove cbz: {}", cbz_id)
            }
        }
    }
}

impl Into<String> for TaskType {
    fn into(self) -> String {
        match self {
            TaskType::HtmlParse { id } => format!("HtmlParse: {}", id),
            TaskType::PicDownload { id } => format!("PicDownload: {}", id),
            TaskType::CbzArchive { id } => format!("CbzArchive: {}", id),
            TaskType::ScanDir => "ScanDir".to_string(),
            TaskType::RemoveCbz { id } => format!("RemoveCbz: {}", id),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnqueueRequest {
    pub id: i32,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnqueueResponse {
    pub task_id: String,
    pub task_type: String,
    pub message: String,
    pub queue_size: usize,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueInfo {
    pub queue_size: usize,
    pub all_tasks: Vec<Task>,
    pub stats: QueueStats,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueStats {
    pub pending: usize,
    pub processing: usize,
    pub completed: usize,
    pub failed: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTaskInfo {
    pub task_id: String,
    pub task_type: TaskType,
    pub description: String,
    pub worker_id: usize,
    pub started_at: OffsetDateTime,
    pub duration_secs: f64,
    pub progress: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveTaskResponse {
    pub count: usize,
    pub tasks: Vec<ActiveTaskInfo>,
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum QueueEvent {
    TaskAdded(Task),
    TaskRemoved(String),
    TaskUpdated(Task),
    QueueCleared,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupRequest {
    #[serde(default = "default_keep_count")]
    pub keep_recent: usize,
}

fn default_keep_count() -> usize {
    100
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResponse {
    pub message: String,
    pub removed_count: usize,
    pub remaining_completed: usize,
    pub total_tasks: usize,
}
