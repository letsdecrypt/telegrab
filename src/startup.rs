use crate::shutdown_signal::shutdown_signal;
use crate::state::AppState;
use crate::{
    Result,
    configuration::Settings,
    controller::{cbz, doc, health_check, pic, task},
    middleware::{TeleGrabRequestId, request_id_middleware},
};
use axum::{Router, http, routing::get};
use axum_messages::MessagesManagerLayer;
use axum_session::{SessionConfig, SessionLayer, SessionStore};
use axum_session_redispool::SessionRedisPool;
use listenfd::ListenFd;
use redis_pool::RedisPool;
use secrecy::ExposeSecret;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tower_sessions::{MemoryStore, SessionManagerLayer};

pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health_check::health))
        .nest("/api/doc", doc::routers())
        .nest("/api/pic", pic::routers())
        .nest("/api/cbz", cbz::routers())
        .nest("/api/task", task::routers())
        .with_state(state)
}

pub async fn run_app_until_stopped(state: AppState, configuration: Settings) -> Result<()> {
    let app = register_layer(app(state.clone()), &configuration).await;

    let listener = init_listener(&configuration).await;
    let server = axum::serve(listener, app);
    let graceful = server.with_graceful_shutdown(shutdown_signal(state.clone()));
    graceful.await?;
    Ok(())
}

async fn init_listener(configuration: &Settings) -> TcpListener {
    let mut listen_fd = ListenFd::from_env();
    match listen_fd.take_tcp_listener(0) {
        Ok(Some(listener)) => {
            listener
                .set_nonblocking(true)
                .expect("Failed to set nonblocking");
            let l = TcpListener::from_std(listener)
                .expect("Failed to convert tcp listener to axum tcp listener");
            let b = l
                .local_addr()
                .expect("tcp listener to be bound to a socket address.");
            tracing::info!("Starting API server with ListenFd: {} ...", b);
            l
        }
        Ok(None) | Err(_) => {
            let listener = TcpListener::bind(configuration.application.address())
                .await
                .expect("Failed to bind to address");
            let b = listener
                .local_addr()
                .expect("tcp listener to be bound to a socket address.");
            tracing::info!("Starting API server with address: {} ...", b);
            listener
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
