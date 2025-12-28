use crate::startup::AppState;
use axum::routing::get;
use axum::{
    Router,
    extract::State,
    response::sse::{Event, Sse},
};
use futures_util::stream::{self, Stream};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt as _;

pub async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // 实现 SSE 流逻辑
    let stream = stream::repeat_with(|| Event::default().data("hi!"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

pub fn routers() -> Router<AppState> {
    Router::new().route("/sse", get(sse_handler))
}
