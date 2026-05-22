use crate::Result;
use axum::{
    Json,
    response::{IntoResponse, Response},
};
use serde::Serialize;

pub fn json<T: Serialize>(t: T) -> Result<Response> {
    Ok(Json(t).into_response())
}
