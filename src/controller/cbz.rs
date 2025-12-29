use crate::state::AppState;
use axum::Router;

pub fn routers() -> Router<AppState> {
    Router::new()
}
