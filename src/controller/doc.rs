use crate::startup::AppState;
use axum::Router;

pub fn doc_routers() -> Router<AppState> {
    Router::new()
}
