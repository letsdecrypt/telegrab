use crate::model::entity::task::QueueEvent;
use crate::schema::helper::ArcStates;
use crate::schema::task_query::GTask;
use async_graphql::{Context, Enum, Interface, Result, SimpleObject, Subscription};
use futures_util::stream::{Stream, StreamExt};
use std::convert::Infallible;
use std::ops::{Deref, DerefMut};
use tokio_stream::wrappers::BroadcastStream;

/// Type of task queue event
#[derive(Debug, Copy, Clone, Eq, PartialEq, Enum)]
pub enum TaskEventType {
    /// A new task was added to the queue
    TaskAdded,
    /// An existing task was updated
    TaskUpdated,
    /// A task was removed from the queue
    TaskRemoved,
    /// A task reported progress
    TaskProgress,
    /// The entire queue was cleared
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

/// Event payload when a task is added
#[derive(Debug, Clone, SimpleObject)]
pub struct TaskAdded {
    /// Event type
    pub r#type: TaskEventType,
    /// The added task
    pub task: GTask,
}

/// Event payload when a task is updated
#[derive(Debug, Clone, SimpleObject)]
pub struct TaskUpdated {
    /// Event type
    pub r#type: TaskEventType,
    /// The updated task
    pub task: GTask,
}

/// Event payload when a task is removed
#[derive(Debug, Clone, SimpleObject)]
pub struct TaskRemoved {
    /// Event type
    pub r#type: TaskEventType,
    /// ID of the removed task
    pub task_id: String,
}

/// Event payload for task progress updates
#[derive(Debug, Clone, SimpleObject)]
pub struct TaskProgress {
    /// Event type
    pub r#type: TaskEventType,
    /// ID of the task reporting progress
    pub task_id: String,
    /// Progress value (0.0 - 1.0)
    pub progress: f64,
}

/// Event payload when the queue is cleared
#[derive(Debug, Clone, SimpleObject)]
pub struct QueueCleared {
    /// Event type
    pub r#type: TaskEventType,
}

/// Union of all possible task queue events
#[derive(Interface)]
#[graphql(field(name = "type", ty = "TaskEventType", desc = "The type of a task event"))]
pub enum TaskEvent {
    /// A task was added
    TaskAdded(TaskAdded),
    /// A task was updated
    TaskUpdated(TaskUpdated),
    /// A task was removed
    TaskRemoved(TaskRemoved),
    /// A task reported progress
    TaskProgress(TaskProgress),
    /// The queue was cleared
    QueueCleared(QueueCleared),
}

/// Subscription for real-time task queue events
#[derive(Default)]
pub struct TaskSubscription;

#[Subscription]
impl TaskSubscription {
    /// Subscribe to all task queue events (add, update, remove, progress, clear)
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
