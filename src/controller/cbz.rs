use axum::Router;
use crate::startup::AppState;

pub fn cbz_routers() -> Router<AppState> {
    Router::new()
}