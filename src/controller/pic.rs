use crate::Result;
use crate::model::dto::AffectedRows;
use crate::model::dto::pagination::PaginationQuery;
use crate::model::dto::pic::MutatePicReq;
use crate::state::AppState;
use crate::{format, service};
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use serde::Deserialize;

pub fn routers() -> Router<AppState> {
    Router::new()
        .route("/", get(get_pics_handler))
        .route("/", post(create_pic_handler))
        .route("/{id}", get(get_pic_handler))
        .route("/{id}", patch(update_pic_handler))
        .route("/{id}", delete(delete_pic_handler))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PicQuery {
    pub doc_id: Option<i32>,
}
async fn get_pics_handler(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
    Query(pic_query): Query<PicQuery>,
) -> Result<Response> {
    let pics = service::pic::get_pics(&state.db_pool, &query, &pic_query).await?;
    let json = Json(pics.data);
    let mut response = json.into_response();
    response.headers_mut().insert(
        "x-total-count",
        axum::http::HeaderValue::from_str(&pics.total.to_string()).unwrap(),
    );
    Ok(response)
}
async fn get_pic_handler(State(state): State<AppState>, Path(id): Path<i32>) -> Result<Response> {
    let pic = service::pic::get_pic_by_id(&state.db_pool, id).await?;
    format::json(pic)
}

async fn create_pic_handler(
    State(state): State<AppState>,
    Json(params): Json<MutatePicReq>,
) -> Result<Response> {
    let pic = service::pic::create_pic(&state.db_pool, params).await?;
    format::json(pic)
}

async fn update_pic_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(params): Json<MutatePicReq>,
) -> Result<Response> {
    let pic = service::pic::update_pic_by_id(&state.db_pool, id, params).await?;
    format::json(pic)
}
async fn delete_pic_handler(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Response> {
    let count = service::pic::delete_pic_by_id(&state.db_pool, id).await?;
    let affected_rows = AffectedRows::new(count);
    format::json(affected_rows)
}
