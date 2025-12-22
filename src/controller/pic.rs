use axum::Router;
use crate::startup::AppState;

pub fn pic_routers() -> Router<AppState> {
    Router::new()
}