use crate::model::entity::task::{ActiveTaskInfo, Task, TaskStatus, TaskType};
use crate::schema::helper::{ArcStates, RelayTy, to_global_id};
use async_graphql::{Context, Enum, Object, Result, SimpleObject};
use time::OffsetDateTime;

/// Type of background task
#[derive(Debug, Clone, Copy, Eq, PartialEq, Enum)]
#[graphql(name = "TaskType")]
pub enum GTaskType {
    /// Parse album HTML page
    AlbumParse,
    /// Download all images in an album
    AlbumDownload,
    /// Download a single image
    ImageDownload,
    /// Create CBZ archive for an album
    CbzArchive,
    /// Remove a CBZ archive
    RemoveCbz,
    /// Scan directory for new files
    ScanDir,
    /// Filesystem event: CBZ file added
    FSCbzAdded,
    /// Filesystem event: CBZ file removed
    FSCbzRemoved,
    /// Parse all album HTML pages
    HtmlParseAll,
}

/// A background task record
#[derive(Debug, Clone, SimpleObject)]
#[graphql(name = "Task")]
pub struct GTask {
    /// Unique task ID
    pub id: String,
    /// Type of the task
    pub task_type: GTaskType,
    /// Global ID of the associated entity (album or image), if applicable
    pub inner_id: Option<String>,
    /// Current status of the task
    pub status: TaskStatus,
    /// When the task was created
    pub created_at: OffsetDateTime,
    /// When the task started execution
    pub started_at: Option<OffsetDateTime>,
    /// When the task completed (success or failure)
    pub completed_at: Option<OffsetDateTime>,
    /// Result message on success
    pub result: Option<String>,
    /// Error message on failure
    pub error: Option<String>,
}

/// Information about a currently running task
#[derive(Debug, Clone, SimpleObject)]
#[graphql(name = "ActiveTask")]
pub struct GActiveTask {
    /// Unique task ID
    pub task_id: String,
    /// Type of the task
    pub task_type: GTaskType,
    /// Human-readable description of what the task is doing
    pub description: String,
    /// ID of the worker executing this task
    pub worker_id: usize,
    /// When the task started execution
    pub started_at: OffsetDateTime,
    /// Duration since task started, in seconds
    pub duration_secs: f64,
    /// Progress percentage (0.0 - 1.0), if available
    pub progress: Option<f64>,
}

fn task_type_to_g(task_type: TaskType) -> (Option<String>, GTaskType) {
    match task_type {
        TaskType::HtmlParse { id } => (
            Some(to_global_id(RelayTy::Album, id as usize)),
            GTaskType::AlbumParse,
        ),
        TaskType::DocDownload { id } => (
            Some(to_global_id(RelayTy::Album, id as usize)),
            GTaskType::AlbumDownload,
        ),
        TaskType::PicDownload { id } => (
            Some(to_global_id(RelayTy::Image, id as usize)),
            GTaskType::ImageDownload,
        ),
        TaskType::CbzArchive { id } => (
            Some(to_global_id(RelayTy::Album, id as usize)),
            GTaskType::CbzArchive,
        ),
        TaskType::RemoveCbz { id } => (
            Some(to_global_id(RelayTy::Cbz, id as usize)),
            GTaskType::RemoveCbz,
        ),
        TaskType::ScanDir => (None, GTaskType::ScanDir),
        TaskType::FSCbzAdded { .. } => (None, GTaskType::FSCbzAdded),
        TaskType::FSCbzRemoved { .. } => (None, GTaskType::FSCbzRemoved),
        TaskType::HtmlParseAll => (None, GTaskType::HtmlParseAll),
    }
}

impl From<Task> for GTask {
    fn from(val: Task) -> Self {
        let (inner_id, task_type) = task_type_to_g(val.task_type);
        Self {
            id: val.id,
            task_type,
            inner_id,
            status: val.status,
            created_at: val.created_at,
            started_at: val.started_at,
            completed_at: val.completed_at,
            result: val.result,
            error: val.error,
        }
    }
}

impl From<ActiveTaskInfo> for GActiveTask {
    fn from(val: ActiveTaskInfo) -> Self {
        let (_inner_id, task_type) = task_type_to_g(val.task_type);
        Self {
            task_id: val.task_id,
            task_type,
            description: val.description,
            worker_id: val.worker_id,
            started_at: val.started_at,
            duration_secs: val.duration_secs,
            progress: val.progress,
        }
    }
}

/// Root query for task-related operations
#[derive(Default)]
pub struct TaskQuery;

#[Object]
impl TaskQuery {
    /// Get all tasks (including completed)
    async fn tasks(&self, ctx: &Context<'_>) -> Result<Vec<GTask>> {
        let states = ctx.data::<ArcStates>()?;
        let tasks = states
            .get_tasks()
            .await
            .iter()
            .map(|task| task.clone().into())
            .collect();
        Ok(tasks)
    }

    /// Get currently active (running) tasks
    async fn active_tasks(&self, ctx: &Context<'_>) -> Result<Vec<GActiveTask>> {
        let states = ctx.data::<ArcStates>()?;
        let tasks = states
            .get_active_tasks()
            .await
            .iter()
            .map(|task| task.clone().into())
            .collect();
        Ok(tasks)
    }
}
