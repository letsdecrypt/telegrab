use crate::model::entity::task::{Task, TaskStatus};
use crate::schema::helper::{from_global_id, ArcStates, RelayTy};
use crate::schema::task_query::GTask;
use async_graphql::{Context, InputObject, Object, Result, SimpleObject};

#[derive(InputObject, Debug, Clone)]
struct EnqueueTaskInput {
    pub id: String,
    pub client_mutation_id: Option<String>,
}

#[derive(SimpleObject, Debug, Clone)]
struct EnqueueTaskPayload {
    pub task: GTask,
    pub client_mutation_id: Option<String>,
}
#[derive(InputObject, Debug, Clone)]
struct CleanUpInput {
    pub keep_recent: usize,
    pub client_mutation_id: Option<String>,
}
#[derive(SimpleObject, Debug, Clone)]
struct CleanUpPayload {
    pub removed_count: usize,
    pub remaining_completed: usize,
    pub client_mutation_id: Option<String>,
}

#[derive(Default)]
pub struct TaskMutation;

#[Object]
impl TaskMutation {
    async fn enqueue_task(
        &self,
        ctx: &Context<'_>,
        input: EnqueueTaskInput,
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
    async fn cleanup_completed(
        &self,
        ctx: &Context<'_>,
        input: CleanUpInput,
    ) -> Result<CleanUpPayload> {
        let states = ctx.data::<ArcStates>()?;
        let client_mutation_id = input.client_mutation_id.clone();
        let removed_count = states.cleanup_completed_tasks(input.keep_recent).await;
        let tasks = states.task_store.read().await;
        let remaining_completed = tasks
            .iter()
            .filter(|(_, task)| matches!(task.status, TaskStatus::Completed))
            .count();
        Ok(CleanUpPayload {
            removed_count,
            remaining_completed,
            client_mutation_id,
        })
    }
}
