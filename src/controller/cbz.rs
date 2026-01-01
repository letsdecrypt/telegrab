use crate::model::dto::cbz::{DeleteCbzReq, UpdateCbzReq};
use crate::model::dto::pagination::PaginationQuery;
use crate::model::dto::AffectedRows;
use crate::state::AppState;
use crate::{format, service, Result};
use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post, put};
use axum::{Json, Router};

pub fn routers() -> Router<AppState> {
    Router::new()
        .route("/", get(get_cbz_page_handler))
        .route("/", post(scan_cbz_handler))
        .route("/", put(toggle_auto_scan_or_fs_notify_handler))
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
    "scan_cbz_handler"
}

pub async fn toggle_auto_scan_or_fs_notify_handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    "toggle_auto_scan_or_fs_notify_handler"
}
pub async fn remove_cbz_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(params): Json<DeleteCbzReq>,
) -> Result<Response> {
    // todo: remove link, and remove file
    let delete_file = params.delete_file;
    let count = service::cbz::remove_cbz_by_id(&state.db_pool, id).await?;
    let affected_rows = AffectedRows::new(count);
    format::json(affected_rows)
}

pub async fn update_cbz_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(params): Json<UpdateCbzReq>,
) -> Result<Response> {
    let cbz = service::cbz::update_cbz(&state.db_pool, id, params.doc_id).await?;
    format::json(cbz)
}
