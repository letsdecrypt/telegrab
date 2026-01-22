use std::sync::Arc;
use tokio::sync::{RwLock, broadcast};

#[derive(Debug, Clone)]
pub struct GracefulShutdown {
    pub shutdown_tx: broadcast::Sender<()>,
    pub shutdown_rx: Arc<RwLock<broadcast::Receiver<()>>>,
    pub is_shutting_down: Arc<RwLock<bool>>,
    pub active_tasks: Arc<RwLock<usize>>,
}

impl Default for GracefulShutdown {
    fn default() -> Self {
        Self::new()
    }
}

impl GracefulShutdown {
    pub fn new() -> Self {
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        Self {
            shutdown_tx,
            shutdown_rx: Arc::new(RwLock::new(shutdown_rx)),
            is_shutting_down: Arc::new(RwLock::new(false)),
            active_tasks: Arc::new(RwLock::new(0)),
        }
    }
    pub async fn get_shutdown_rx(&self) -> broadcast::Receiver<()> {
        let rx = self.shutdown_rx.read().await;
        rx.resubscribe()
    }
    pub async fn is_shutting_down(&self) -> bool {
        let is_shutting_down = self.is_shutting_down.read().await;
        *is_shutting_down
    }
    pub async fn shutdown(&self) {
        tracing::info!("Shutting down gracefully...");
        {
            let mut shutting_down = self.is_shutting_down.write().await;
            *shutting_down = true
        }
        if let Err(e) = self.shutdown_tx.send(()) {
            tracing::error!("send shutdown signal failed: {:?}", e);
        }
        tracing::info!("Shutdown signal sent, waiting for active tasks to finish...");
    }
    pub async fn wait_for_completion(&self, timeout_secs: u64) -> bool {
        use tokio::time::{Duration, sleep, timeout};
        tracing::info!(
            "Waiting for {} seconds for active tasks to finish...",
            timeout_secs
        );
        let timeout_duration = Duration::from_secs(timeout_secs);
        match timeout(timeout_duration, async {
            loop {
                let active_tasks = self.active_tasks.read().await;
                if *active_tasks == 0 {
                    tracing::info!("All active tasks have finished.");
                    break;
                }
                tracing::info!("Waiting for {} active tasks to finish...", *active_tasks);
                sleep(Duration::from_millis(500)).await;
            }
        })
        .await
        {
            Ok(_) => {
                tracing::info!("Graceful shutdown completed.");
                true
            }
            Err(_) => {
                tracing::warn!("Timeout reached, active tasks may not have finished.");
                false
            }
        }
    }
    pub async fn task_started(&self) {
        let mut tasks = self.active_tasks.write().await;
        *tasks += 1;
    }
    pub async fn task_finished(&self) {
        let mut tasks = self.active_tasks.write().await;
        if *tasks > 0 {
            *tasks -= 1;
        }
    }
    pub async fn active_task_count(&self) -> usize {
        let tasks = self.active_tasks.read().await;
        *tasks
    }
}

pub struct TaskGuard {
    shutdown: Arc<GracefulShutdown>,
}

impl TaskGuard {
    pub async fn new(shutdown: Arc<GracefulShutdown>) -> Option<Self> {
        if shutdown.is_shutting_down().await {
            return None;
        }
        shutdown.task_started().await;
        Some(Self { shutdown })
    }
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        let shutdown = self.shutdown.clone();
        tokio::spawn(async move {
            shutdown.task_finished().await;
        });
    }
}
