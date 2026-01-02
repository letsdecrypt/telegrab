use crate::model::dto::cbz::UpdateCbzReq;
use crate::model::dto::pagination::PaginationQuery;
use crate::model::entity::task::{EnqueueResponse, Task};
use crate::state::AppState;
use crate::{format, service, Result};
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};

pub fn routers() -> Router<AppState> {
    Router::new()
        .route("/", get(get_cbz_page_handler))
        .route("/", post(scan_cbz_handler))
        .route("/{id}", get(get_cbz_handler))
        .route("/{id}", delete(remove_cbz_handler))
        .route("/{id}", patch(update_cbz_handler))
}

pub async fn get_cbz_page_handler(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Response> {
    let cbz_page = service::cbz::get_cbz_page(&state.db_pool, &query).await?;
    let mut headers = HeaderMap::new();
    headers.insert("x-total-count", cbz_page.total.to_string().parse()?);
    let json = Json(cbz_page.data);
    Ok((headers, json).into_response())
}
pub async fn get_cbz_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Response> {
    let cbz = service::cbz::get_cbz_by_id(&state.db_pool, id).await?;
    format::json(cbz)
}

pub async fn scan_cbz_handler(State(state): State<AppState>) -> impl IntoResponse {
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
    if state.queue_state.is_scan_active().await {
        return (
            StatusCode::CONFLICT,
            Json(EnqueueResponse {
                task_id: "".to_string(),
                task_type: "".to_string(),
                message: "Scan is active, no new scan tasks accepted".to_string(),
                queue_size: 0,
            }),
        );
    }
    let task = Task::new_scan_dir_task();
    state.queue_state.enqueue(task.clone()).await;
    let queue_size = state.queue_state.size().await;

    let response = EnqueueResponse {
        task_id: task.id.clone(),
        task_type: task.task_type.into(),
        message: "Scan directory task enqueued".to_string(),
        queue_size,
    };
    (StatusCode::CREATED, Json(response))
}

pub async fn remove_cbz_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Response> {
    let task = Task::new_remove_cbz_task(id);
    state.queue_state.enqueue(task.clone()).await;
    let queue_size = state.queue_state.size().await;
    let response = EnqueueResponse {
        task_id: task.id.clone(),
        task_type: task.task_type.into(),
        message: "Remove cbz task enqueued".to_string(),
        queue_size,
    };
    format::json(response)
}

pub async fn update_cbz_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(params): Json<UpdateCbzReq>,
) -> Result<Response> {
    let cbz = service::cbz::update_cbz(&state.db_pool, id, params.doc_id).await?;
    format::json(cbz)
}
