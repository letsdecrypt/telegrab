use crate::startup::AppState;
use axum::Router;

pub fn routers() -> Router<AppState> {
    Router::new()
}
