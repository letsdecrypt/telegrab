use crate::model::entity::task::QueueEvent;
use crate::schema::helper::ArcStates;
use crate::schema::task_query::GTask;
use async_graphql::{Context, Enum, Interface, Result, SimpleObject, Subscription};
use futures_util::stream::{Stream, StreamExt};
use std::convert::Infallible;
use std::ops::{Deref, DerefMut};
use tokio_stream::wrappers::BroadcastStream;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
pub enum TaskEventType {
    TaskAdded,
    TaskUpdated,
    TaskRemoved,
    TaskProgress,
    QueueCleared,
}
impl AsRef<TaskEventType> for TaskEventType {
    fn as_ref(&self) -> &Self {
        self
    }
}
impl Deref for TaskEventType {
    type Target = Self;
    fn deref(&self) -> &Self::Target {
        self
    }
}
impl DerefMut for TaskEventType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self
    }
}
impl From<&TaskEventType> for TaskEventType {
    fn from(val: &TaskEventType) -> Self {
        *val
    }
}
#[derive(Debug, Clone, SimpleObject)]
pub struct TaskAdded {
    pub r#type: TaskEventType,
    pub task: GTask,
}
#[derive(Debug, Clone, SimpleObject)]
pub struct TaskUpdated {
    pub r#type: TaskEventType,
    pub task: GTask,
}
#[derive(Debug, Clone, SimpleObject)]
pub struct TaskRemoved {
    pub r#type: TaskEventType,
    pub task_id: String,
}
#[derive(Debug, Clone, SimpleObject)]
pub struct TaskProgress {
    pub r#type: TaskEventType,
    pub task_id: String,
    pub progress: f64,
}
#[derive(Debug, Clone, SimpleObject)]
pub struct QueueCleared {
    pub r#type: TaskEventType,
}

#[derive(Interface)]
#[graphql(field(name = "type", ty = "TaskEventType", desc = "The type of a task event"))]
pub enum TaskEvent {
    TaskAdded(TaskAdded),
    TaskUpdated(TaskUpdated),
    TaskRemoved(TaskRemoved),
    TaskProgress(TaskProgress),
    QueueCleared(QueueCleared),
}

#[derive(Default)]
pub struct TaskSubscription;

#[Subscription]
impl TaskSubscription {
    async fn events(&self, ctx: &Context<'_>) -> impl Stream<Item = Result<TaskEvent, Infallible>> {
        let states = ctx.data_unchecked::<ArcStates>();
        let rx = states.sender.subscribe();
        let stream = BroadcastStream::new(rx);

        stream.filter_map(|result| async move {
            match result {
                Ok(q_event) => {
                    let t_event = match q_event {
                        QueueEvent::TaskAdded(task) => TaskEvent::TaskAdded(TaskAdded {
                            r#type: TaskEventType::TaskAdded,
                            task: task.into(),
                        }),
                        QueueEvent::TaskUpdated(task) => TaskEvent::TaskUpdated(TaskUpdated {
                            r#type: TaskEventType::TaskUpdated,
                            task: task.into(),
                        }),
                        QueueEvent::TaskRemoved(task_id) => TaskEvent::TaskRemoved(TaskRemoved {
                            r#type: TaskEventType::TaskRemoved,
                            task_id,
                        }),
                        QueueEvent::TaskProgress(task_id, progress) => {
                            TaskEvent::TaskProgress(TaskProgress {
                                r#type: TaskEventType::TaskProgress,
                                task_id,
                                progress,
                            })
                        }
                        QueueEvent::QueueCleared => TaskEvent::QueueCleared(QueueCleared {
                            r#type: TaskEventType::QueueCleared,
                        }),
                    };
                    Some(Ok(t_event))
                }
                Err(_) => None,
            }
        })
    }
}
