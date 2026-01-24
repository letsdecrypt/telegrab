use crate::{
    Result,
    configuration::Settings,
    controller::{assets, cbz, doc, health_check, pic, task, gallery},
    errors::Error::ListenerError,
    listener,
    middleware::{TeleGrabRequestId, request_id_middleware},
    shutdown_signal::shutdown_signal,
    state::AppState,
};
use axum::{Router, http, routing::get};
use tower_http::trace::TraceLayer;

pub fn app(state: AppState) -> Router {
    Router::new()
        .nest("/resource", assets::routers(&state))
        .nest("/graphql", gallery::routers(&state))
        .route("/api/health", get(health_check::health))
        .nest("/api/doc", doc::routers())
        .nest("/api/pic", pic::routers())
        .nest("/api/cbz", cbz::routers())
        .nest("/api/task", task::routers())
        .with_state(state)
}

pub async fn run_app_until_stopped(state: AppState, configuration: Settings) -> Result<()> {
    let app = register_layer(app(state.clone()), &configuration).await;

    let listener_handles =
        listener::start_listeners(app, &configuration, state.shutdown.clone()).await?;
    tracing::info!("Started {} listener(s)", listener_handles.len());
    let shutdown_signal = shutdown_signal(state.clone());

    tokio::select! {
            _ = shutdown_signal => {
                tracing::info!("All listeners stopped gracefully");
                Ok(())
            }
        _ = async {
            let mut results = Vec::new();
            for handle in listener_handles {
                let result = handle.await;
                results.push(result);
            }
            for result in results {
                if let Err(e) = result {
                    tracing::error!("Listener error {:?}", e);
                    return Err(ListenerError("One or more listeners failed".to_string()));
                }
            }
            Ok(())
        } => {
            Err(ListenerError("One or more listeners failed".to_string()))
        }
    }
}
pub async fn register_layer(app: Router, _configuration: &Settings) -> Router {
    app.layer(
        TraceLayer::new_for_http().make_span_with(|request: &http::Request<_>| {
            let ext = request.extensions();
            let request_id = ext
                .get::<TeleGrabRequestId>()
                .map_or_else(|| "req-id-none".to_string(), |r| r.get().to_string());
            let user_agent = request
                .headers()
                .get(axum::http::header::USER_AGENT)
                .map_or("", |h| h.to_str().unwrap_or(""));

            tracing::error_span!(
                "http-request",
                "http.method" = tracing::field::display(request.method()),
                "http.uri" = tracing::field::display(request.uri()),
                "http.version" = tracing::field::debug(request.version()),
                "http.user_agent" = tracing::field::display(user_agent),
                request_id = tracing::field::display(request_id),
            )
        }),
    )
    .layer(axum::middleware::from_fn(request_id_middleware))
}
