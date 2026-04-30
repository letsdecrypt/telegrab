use crate::model::entity::task::{Task, TaskStatus};
use crate::schema::helper::{ArcStates, RelayTy, from_global_id};
use crate::schema::task_query::GTask;
use async_graphql::{Context, InputObject, Object, Result, SimpleObject};

/// Input for enqueuing a task for an album or image
#[derive(InputObject, Debug, Clone)]
struct EnqueueTaskInput {
    /// Global ID of the entity to process (album or image)
    pub id: String,
    /// Client mutation ID for Relay support
    pub client_mutation_id: Option<String>,
}

/// Payload returned after enqueuing a task
#[derive(SimpleObject, Debug, Clone)]
struct EnqueueTaskPayload {
    /// The enqueued task
    pub task: GTask,
    /// Client mutation ID echoed back for Relay support
    pub client_mutation_id: Option<String>,
}

/// Input for cleaning up completed tasks
#[derive(InputObject, Debug, Clone)]
struct CleanUpInput {
    /// Number of most recent completed tasks to keep
    pub keep_recent: usize,
    /// Client mutation ID for Relay support
    pub client_mutation_id: Option<String>,
}

/// Payload returned after cleaning up completed tasks
#[derive(SimpleObject, Debug, Clone)]
struct CleanUpPayload {
    /// Number of tasks removed
    pub removed_count: usize,
    /// Number of completed tasks remaining
    pub remaining_completed: usize,
    /// Client mutation ID echoed back for Relay support
    pub client_mutation_id: Option<String>,
}

/// Mutations for task management operations
#[derive(Default)]
pub struct TaskMutation;

#[Object]
impl TaskMutation {
    /// Enqueue a new task for processing an album or image
    async fn enqueue_task(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Input for enqueuing a task")] input: EnqueueTaskInput,
    ) -> Result<EnqueueTaskPayload> {
        let states = ctx.data::<ArcStates>()?;
        let client_mutation_id = input.client_mutation_id.clone();
        let (ty, id) = from_global_id(input.id.as_str())?;
        match ty {
            RelayTy::Album => {
                if let Some(task) = states.find_doc_in_queue(id as i32).await {
                    return Ok(EnqueueTaskPayload {
                        task: task.into(),
                        client_mutation_id,
                    });
                }
                let task = Task::new_html_parse_task(id as i32);
                states.enqueue(task.clone()).await;
                let g_task = task.into();
                Ok(EnqueueTaskPayload {
                    task: g_task,
                    client_mutation_id,
                })
            }
            RelayTy::Image => {
                if let Some(task) = states.find_pic_in_queue(id as i32).await {
                    return Ok(EnqueueTaskPayload {
                        task: task.into(),
                        client_mutation_id,
                    });
                }
                let task = Task::new_pic_download_task(id as i32);
                states.enqueue(task.clone()).await;
                let g_task = task.into();
                Ok(EnqueueTaskPayload {
                    task: g_task,
                    client_mutation_id,
                })
            }
            _ => Err("Invalid type".into()),
        }
    }

    /// Remove completed tasks, keeping only the most recent ones
    async fn cleanup_completed(
        &self,
        ctx: &Context<'_>,
        #[graphql(desc = "Input for cleaning up completed tasks")] input: CleanUpInput,
    ) -> Result<CleanUpPayload> {
        let states = ctx.data::<ArcStates>()?;
        let client_mutation_id = input.client_mutation_id.clone();
        let removed_count = states.cleanup_completed_tasks(input.keep_recent).await;
        let remaining_completed = states
            .task_store
            .iter()
            .filter(|r| matches!(r.value().status, TaskStatus::Completed))
            .count();
        Ok(CleanUpPayload {
            removed_count,
            remaining_completed,
            client_mutation_id,
        })
    }
}
