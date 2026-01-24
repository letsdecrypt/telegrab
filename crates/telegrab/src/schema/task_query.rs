use crate::model::entity::task::{ActiveTaskInfo, Task, TaskStatus, TaskType};
use crate::schema::helper::{to_global_id, ArcStates, RelayTy};
use async_graphql::{Context, Enum, Object, Result, SimpleObject};
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Enum)]
#[graphql(name = "TaskType")]
pub enum GTaskType {
    AlbumParse,
    AlbumDownload,
    ImageDownload,
    CbzArchive,
    ScanDir,
    RemoveCbz,
    FSCbzAdded,
    FSCbzRemoved,
    HtmlParseAll,
}
#[derive(Debug, Clone, SimpleObject)]
#[graphql(name = "Task")]
pub struct GTask {
    pub id: String,
    pub task_type: GTaskType,
    pub inner_id: Option<String>,
    pub status: TaskStatus,
    pub created_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub completed_at: Option<OffsetDateTime>,
    pub result: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, SimpleObject)]
#[graphql(name = "ActiveTask")]
pub struct GActiveTask {
    pub task_id: String,
    pub task_type: GTaskType,
    pub description: String,
    pub worker_id: usize,
    pub started_at: OffsetDateTime,
    pub duration_secs: f64,
    pub progress: Option<f64>,
}

fn task_type_to_g(task_type: TaskType)->(Option<String>, GTaskType){
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
#[derive(Default)]
pub struct TaskQuery;

#[Object]
impl TaskQuery {
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
    async fn active_tasks(&self, ctx: &Context<'_>) -> Result<Vec<GActiveTask>>{
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
