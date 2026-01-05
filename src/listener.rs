use crate::configuration::{ListenerConfig, ListenerType, Settings};
use crate::errors::Error::ListenerError;
use crate::graceful::GracefulShutdown;
use crate::Result;
use axum::Router;
use listenfd::ListenFd;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Arc;
use tokio::net::{TcpListener, UnixListener};
use tokio::task::JoinHandle;

pub type ListenerHandle = JoinHandle<Result<()>>;

pub async fn start_listeners(
    app: Router,
    configuration: &Settings,
    shutdown: Arc<GracefulShutdown>,
) -> Result<Vec<ListenerHandle>> {
    let mut handles = Vec::new();
    let mut listen_fd = ListenFd::from_env();
    // use fd listeners, usually on dev
    if listen_fd.len() > 0 {
        let fd_handles = start_listen_on_fd(app.clone(), &mut listen_fd, shutdown.clone()).await?;
        if !fd_handles.is_empty() {
            return Ok(fd_handles);
        }
    }
    // use configuration listeners, usually on production
    for listener in configuration.application.listeners.iter() {
        let handle = start_listener(app.clone(), listener, shutdown.clone()).await?;
        handles.push(handle);
    }

    Ok(handles)
}
pub async fn start_listener(
    app: Router,
    config: &ListenerConfig,
    shutdown: Arc<GracefulShutdown>,
) -> Result<ListenerHandle> {
    match config.listener_type {
        ListenerType::Tcp => start_tcp_listener(app, config, shutdown).await,
        ListenerType::Uds => start_uds_listener(app, config, shutdown).await,
    }
}

pub async fn start_tcp_listener(
    app: Router,
    config: &ListenerConfig,
    shutdown: Arc<GracefulShutdown>,
) -> Result<ListenerHandle> {
    let config_address = config.address.clone();
    let listener = TcpListener::bind(&config_address).await.map_err(|e| {
        ListenerError(format!(
            "Failed to bind TCP listener to {}: {}",
            &config_address, e
        ))
    })?;
    let local_address = listener.local_addr().map_err(|e| {
        ListenerError(format!(
            "Failed to get local address for {}: {}",
            &config_address, e
        ))
    })?;

    tracing::info!("[Listener] Starting TCP server on {}...", local_address);
    let mut shutdown_rx = shutdown.get_shutdown_rx().await;
    let join_handle = tokio::spawn(async move {
        let config_address_graceful = config_address.clone();
        let config_address_stop = config_address.clone();
        let server = axum::serve(listener, app);
        let graceful = server.with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
            tracing::info!(
                "[Listener] Received shutdown signal on {}...",
                &config_address_graceful
            );
        });
        match graceful.await {
            Ok(_) => {
                tracing::info!(
                    "[Listener] TCP Server on {} stopped gracefully.",
                    &config_address_stop
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    "[Listener] TCP Server on {} failed: {}",
                    &config_address_stop,
                    e
                );
                Err(ListenerError(format!(
                    "TCP Server on {} failed: {}",
                    &config_address_stop, e
                )))
            }
        }
    });

    Ok(join_handle)
}
pub async fn start_uds_listener(
    app: Router,
    config: &ListenerConfig,
    shutdown: Arc<GracefulShutdown>,
) -> Result<ListenerHandle> {
    let socket_path = Path::new(&config.address).to_path_buf();
    let address = config.address.clone();
    if socket_path.exists() {
        std::fs::remove_file(&socket_path).map_err(|e| {
            ListenerError(format!(
                "Failed to remove existing socket file {}: {}",
                &address, e
            ))
        })?
    }
    if let Some(parent_dir) = socket_path.parent() {
        std::fs::create_dir_all(parent_dir).map_err(|e| {
            ListenerError(format!(
                "Failed to create parent directory for socket file {}: {}",
                &address, e
            ))
        })?
    }
    let listener = UnixListener::bind(&address).map_err(|e| {
        ListenerError(format!(
            "Failed to bind UDS listener to {}: {}",
            &address, e
        ))
    })?;
    {
        let mut perms = std::fs::metadata(&socket_path)?.permissions();
        let desired_mode = 0o666;
        let current_mode = perms.mode();
        if current_mode != desired_mode {
            tracing::warn!(
                "[Listener] Permissions for socket file {} are not set correctly, {:o}. Setting them to {:o}.",
                &address,
                current_mode,
                desired_mode
            );
            perms.set_mode(desired_mode);
            if let Err(e) = std::fs::set_permissions(&socket_path, perms) {
                tracing::warn!(
                    "Failed to set permissions for socket file {}: {}",
                    &address,
                    e
                )
            }
        }
    };
    tracing::info!("[Listener] Starting UDS server on {}...", &address);
    let mut shutdown_rx = shutdown.get_shutdown_rx().await;
    let join_handle = tokio::spawn(async move {
        let config_address_graceful = address.clone();
        let config_address_stop = address.clone();
        let server = axum::serve(listener, app);
        let graceful = server.with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
            tracing::info!(
                "[Listener] Received shutdown signal on {}...",
                &config_address_graceful
            )
        });
        match graceful.await {
            Ok(_) => {
                tracing::info!(
                    "[Listener] UDS Server on {} stopped gracefully.",
                    &config_address_stop
                );
                Ok(())
            }
            Err(e) => {
                tracing::error!(
                    "[Listener] UDS Server on {} failed: {}",
                    &config_address_stop,
                    e
                );
                Err(ListenerError(format!(
                    "UDS Server on {} failed: {}",
                    &config_address_stop, e
                )))
            }
        }
    });
    Ok(join_handle)
}

async fn start_listen_on_fd(
    app: Router,
    fd: &mut ListenFd,
    shutdown: Arc<GracefulShutdown>,
) -> Result<Vec<ListenerHandle>> {
    let mut handles = Vec::new();
    for i in 0..fd.len() {
        let app_clone = app.clone();
        if let Ok(Some(listener)) = fd.take_tcp_listener(i) {
            listener.set_nonblocking(true).map_err(|e| {
                ListenerError(format!(
                    "Failed to set non-blocking on listen_fd {}: {}",
                    i, e
                ))
            })?;
            let address = listener.local_addr().map_err(|e| {
                ListenerError(format!(
                    "Failed to get local address for listen_fd {}: {}",
                    i, e
                ))
            })?;
            let l = TcpListener::from_std(listener).map_err(|e| {
                ListenerError(format!(
                    "Failed to convert std listener to tokio listener on listen_fd {}: {}",
                    i, e
                ))
            })?;
            tracing::info!("[Listener] Starting FD TCP server on {}...", &address);
            let mut shutdown_rx = shutdown.get_shutdown_rx().await;
            let join_handle = tokio::spawn(async move {
                let server = axum::serve(l, app_clone);
                let graceful = server.with_graceful_shutdown(async move {
                    let _ = shutdown_rx.recv().await;
                    tracing::info!("[Listener] FD TCP Received shutdown signal...");
                });
                match graceful.await {
                    Ok(_) => {
                        tracing::info!("[Listener] FD TCP Server stopped gracefully.",);
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("[Listener] FD TCP Server failed: {}", e);
                        Err(ListenerError(format!("FD TCP Server failed: {}", e)))
                    }
                }
            });
            handles.push(join_handle);
        } else if let Ok(Some(listener)) = fd.take_unix_listener(i) {
            listener.set_nonblocking(true).map_err(|e| {
                ListenerError(format!(
                    "Failed to set non-blocking on listen_fd {}: {:?}",
                    i, e
                ))
            })?;
            let address = listener.local_addr().map_err(|e| {
                ListenerError(format!(
                    "Failed to get local address for listen_fd {}: {}",
                    i, e
                ))
            })?;
            let l = UnixListener::from_std(listener).map_err(|e| {
                ListenerError(format!(
                    "Failed to convert std listener to tokio listener on listen_fd {}: {}",
                    i, e
                ))
            })?;
            tracing::info!("[Listener] Starting FD UDS server on {:?}...", &address);
            let mut shutdown_rx = shutdown.get_shutdown_rx().await;
            let join_handle = tokio::spawn(async move {
                let server = axum::serve(l, app_clone);
                let graceful = server.with_graceful_shutdown(async move {
                    let _ = shutdown_rx.recv().await;
                    tracing::info!("[Listener] FD UDS Received shutdown signal...");
                });
                match graceful.await {
                    Ok(_) => {
                        tracing::info!("[Listener] FD UDS Server stopped gracefully.",);
                        Ok(())
                    }
                    Err(e) => {
                        tracing::error!("[Listener] FD UDS Server failed: {}", e);
                        Err(ListenerError(format!("FD UDS Server failed: {}", e)))
                    }
                }
            });
            handles.push(join_handle);
        }
    }
    Ok(handles)
}
