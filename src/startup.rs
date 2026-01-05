use crate::{
    Result,
    configuration::Settings,
    controller::{assets, cbz, doc, health_check, pic, task},
    errors::Error::ListenerError,
    listener,
    middleware::{TeleGrabRequestId, request_id_middleware},
    shutdown_signal::shutdown_signal,
    state::AppState,
};
use axum::{Router, http, routing::get};
use axum_messages::MessagesManagerLayer;
use axum_session::{SessionConfig, SessionLayer, SessionStore};
use axum_session_redispool::SessionRedisPool;
use redis_pool::RedisPool;
use secrecy::ExposeSecret;
use tower_http::trace::TraceLayer;
use tower_sessions::{MemoryStore, SessionManagerLayer};

pub fn app(state: AppState) -> Router {
    Router::new()
        .nest("/resource", assets::routers(&state))
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
async fn init_session_store(redis_url: &str) -> SessionStore<SessionRedisPool> {
    let client =
        redis::Client::open(redis_url).expect("Failed when trying to open the redis connection");
    let pool = RedisPool::from(client);
    let session_config = SessionConfig::default();
    SessionStore::<SessionRedisPool>::new(Some(pool.clone().into()), session_config)
        .await
        .expect("Failed to init session store")
}

pub async fn register_layer(app: Router, configuration: &Settings) -> Router {
    let session_store = init_session_store(configuration.redis_uri.expose_secret()).await;
    let memory_session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(memory_session_store).with_secure(false);

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
    .layer(MessagesManagerLayer)
    .layer(SessionLayer::new(session_store))
    .layer(session_layer)
}
