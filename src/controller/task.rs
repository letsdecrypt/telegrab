use crate::model::entity::task::{
    ActiveTaskResponse, EnqueueRequest, EnqueueResponse, QueueInfo, QueueStats, Task,
    TaskStatus, TaskType,
};
use crate::state::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive};
use axum::response::{IntoResponse, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::stream::StreamExt;
use futures_util::{Stream, stream};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;

pub fn routers() -> Router<AppState> {
    Router::new()
        .route("/active", get(active_tasks))
        .route("/queued", get(queued_tasks))
        .route("/enqueue", post(enqueue_task))
        .route("/sse", get(sse_handler))
}

async fn active_tasks(State(state): State<AppState>) -> impl IntoResponse {
    let tasks = state.queue_state.get_active_tasks().await;
    let count = tasks.len();
    let response = ActiveTaskResponse {
        tasks,
        count,
        workers: state.worker_count,
    };
    Json(response)
}

async fn queued_tasks(State(state): State<AppState>) -> impl IntoResponse {
    let tasks = state.queue_state.get_tasks().await;
    let size = tasks.len();
    let mut stats = QueueStats {
        pending: 0,
        completed: 0,
        processing: 0,
        failed: 0,
    };
    for task in &tasks {
        match task.status {
            TaskStatus::Pending => stats.pending += 1,
            TaskStatus::Processing => stats.processing += 1,
            TaskStatus::Completed => stats.completed += 1,
            TaskStatus::Failed(_) => stats.failed += 1,
        }
    }
    let response = QueueInfo { size, tasks, stats };
    Json(response)
}
async fn enqueue_task(
    State(state): State<AppState>,
    Json(payload): Json<EnqueueRequest>,
) -> impl IntoResponse {
    if state.shutdown.is_shutting_down().await {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(EnqueueResponse {
                task_id: "".to_string(),
                task_type: "".to_string(),
                message: "Server is shutting down, no new tasks accepted".to_string(),
                queue_size: 0,
            }),
        );
    }
    let task = match payload.task_type.as_str() {
        "HtmlParse" => Task::new_html_parse_task(payload.id),
        "PicDownload" => Task::new_pic_download_task(payload.id),
        "CbzArchive" => Task::new_cbz_archive_task(payload.id),
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(EnqueueResponse {
                    task_id: "".to_string(),
                    task_type: payload.task_type.clone(),
                    message: "Invalid task type".to_string(),
                    queue_size: 0,
                }),
            );
        }
    };
    state.queue_state.enqueue(task.clone()).await;
    let queue_size = state.queue_state.size().await;
    let task_type_str = match task.task_type {
        TaskType::HtmlParse { id: _ } => "HtmlParse",
        TaskType::PicDownload { id: _ } => "PicDownload",
        TaskType::CbzArchive { id: _ } => "CbzArchive",
    };

    let response = EnqueueResponse {
        task_id: task.id.clone(),
        task_type: task_type_str.to_string(),
        message: "Task added to queue".to_string(),
        queue_size,
    };
    (StatusCode::CREATED, Json(response))
}
pub async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let rx = state.queue_state.sender.subscribe();
    let mut shutdown_rx = state.shutdown.get_shutdown_rx().await;
    let stream = StreamExt::chain(
        StreamExt::filter_map(
            BroadcastStream::new(rx).take_until(async move {
                let _ = shutdown_rx.recv().await;
                tracing::info!("Shutdown signal received. Closing SSE stream.");
            }),
            |event| async move {
                match event {
                    Ok(event) => {
                        let json = serde_json::to_string(&event).unwrap();
                        Some(Ok(Event::default().data(json)))
                    }
                    Err(_) => None,
                }
            },
        ),
        stream::once(async {
            Ok(Event::default()
                .event("connected")
                .data("{\"message\":\"Connected to SSE stream\"}"))
        }),
    );

    Sse::new(stream).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive"),
    )
}
