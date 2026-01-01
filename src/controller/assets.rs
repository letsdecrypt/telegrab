use crate::state::AppState;
use axum::Router;
use tower_http::services::ServeDir;

pub fn routers(state: &AppState) -> Router<AppState> {
    Router::new()
        .nest_service("/pic", ServeDir::new(state.pic_dir.as_str()))
        .nest_service("/cbz", ServeDir::new(state.cbz_dir.as_str()))
}
