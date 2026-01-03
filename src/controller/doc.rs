use crate::errors;
use crate::format;
use crate::model::dto;
use crate::model::dto::doc::UpdateDocReq;
use crate::model::dto::pagination::PaginationQuery;
use crate::model::dto::AffectedRows;
use crate::service;
use crate::state::AppState;
use crate::Result;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;
use crate::model::entity::task::{EnqueueResponse, Task};

pub fn routers() -> Router<AppState> {
    Router::new()
        .route("/parsed", get(get_parsed_docs_handler))
        .route("/parse_all", post(parse_all_doc_handler))
        .route("/", get(get_docs_handler))
        .route("/", post(create_doc_handler))
        .route("/{id}", get(get_doc_handler))
        .route("/{id}/pics", get(get_pics_by_doc_id_handler))
        .route("/{id}", patch(update_doc_handler))
        .route("/{id}", delete(delete_doc_handler))
}

#[derive(Deserialize)]
pub struct NewDocData {
    pub url: String,
}
async fn create_doc_handler(
    State(state): State<AppState>,
    Json(params): Json<NewDocData>,
) -> Result<Response> {
    let new_doc = params.try_into()?;
    let doc = service::doc::create_doc(&state.db_pool, new_doc).await?;
    let response = (StatusCode::CREATED, Json(doc)).into_response();
    Ok(response)
}

async fn parse_all_doc_handler(State(state): State<AppState>) -> impl IntoResponse {
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
    if state.queue_state.is_parse_all_active().await {
        return (
            StatusCode::CONFLICT,
            Json(EnqueueResponse {
                task_id: "".to_string(),
                task_type: "".to_string(),
                message: "HtmlParseAll is active, no new HtmlParseAll tasks accepted".to_string(),
                queue_size: 0,
            }),
        );
    }
    let task =  Task::new_html_parse_all_task();state.queue_state.enqueue(task.clone()).await;
    let queue_size = state.queue_state.size().await;

    let response = EnqueueResponse {
        task_id: task.id.clone(),
        task_type: task.task_type.into(),
        message: "Task added to queue".to_string(),
        queue_size,
    };
    (StatusCode::CREATED, Json(response))
}
async fn get_parsed_docs_handler(State(state): State<AppState>) -> Result<Response> {
    let docs = service::doc::get_parsed_docs(&state.db_pool).await?;
    format::json(docs)
}
async fn get_docs_handler(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Response> {
    let docs = service::doc::get_docs(&state.db_pool, &query).await?;
    let mut headers = HeaderMap::new();
    headers.insert("x-total-count", docs.total.to_string().parse()?);
    let json = Json(docs.data);
    Ok((headers, json).into_response())
}
async fn get_doc_handler(State(state): State<AppState>, Path(id): Path<i32>) -> Result<Response> {
    let doc = service::doc::get_doc_by_id(&state.db_pool, id).await?;
    format::json(doc)
}

async fn get_pics_by_doc_id_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Response> {
    let pics = service::pic::get_pics_by_doc_id(&state.db_pool, id).await?;
    format::json(pics)
}

async fn update_doc_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(params): Json<UpdateDocReq>,
) -> Result<Response> {
    let doc = service::doc::update_doc(&state.db_pool, id, params).await?;
    format::json(doc)
}
async fn delete_doc_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Response> {
    let count = service::doc::delete_doc_by_id(&state.db_pool, id).await?;
    let affected_rows = AffectedRows::new(count);
    format::json(affected_rows)
}

impl TryFrom<NewDocData> for dto::doc::CreateDocReq {
    type Error = errors::Error;

    fn try_from(value: NewDocData) -> Result<Self, Self::Error> {
        if validator::ValidateUrl::validate_url(&value.url) {
            Ok(Self { url: value.url })
        } else {
            Err(errors::Error::Message(format!(
                "Invalid url: {}",
                value.url
            )))
        }
    }
}
