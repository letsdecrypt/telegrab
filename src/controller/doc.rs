use crate::Result;
use crate::errors;
use crate::format;
use crate::model::dto;
use crate::model::dto::AffectedRows;
use crate::model::dto::doc::UpdateDocReq;
use crate::model::dto::pagination::PaginationQuery;
use crate::service;
use crate::state::AppState;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;

pub fn routers() -> Router<AppState> {
    Router::new()
        .route("/parsed", get(get_parsed_docs_handler))
        .route("/", get(get_docs_handler))
        .route("/", post(create_doc_handler))
        .route("/{id}", get(get_doc_handler))
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
    format::json(doc)
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
    let json = Json(docs.data);
    let mut response = json.into_response();
    response.headers_mut().insert(
        "x-total-count",
        axum::http::HeaderValue::from_str(&docs.total.to_string()).unwrap(),
    );
    Ok(response)
}
async fn get_doc_handler(State(state): State<AppState>, Path(id): Path<i32>) -> Result<Response> {
    let doc = service::doc::get_doc_by_id(&state.db_pool, id).await?;
    format::json(doc)
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
