use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct GracefulShutdown {
    pub shutdown_tx: broadcast::Sender<()>,
    pub is_shutting_down: Arc<AtomicBool>,
    pub active_tasks: Arc<AtomicUsize>,
}

impl Clone for GracefulShutdown {
    fn clone(&self) -> Self {
        Self {
            shutdown_tx: self.shutdown_tx.clone(),
            is_shutting_down: self.is_shutting_down.clone(),
            active_tasks: self.active_tasks.clone(),
        }
    }
}

impl Default for GracefulShutdown {
    fn default() -> Self {
        Self::new()
    }
}

impl GracefulShutdown {
    pub fn new() -> Self {
        let (shutdown_tx, _) = broadcast::channel(1);
        Self {
            shutdown_tx,
            is_shutting_down: Arc::new(AtomicBool::new(false)),
            active_tasks: Arc::new(AtomicUsize::new(0)),
        }
    }
    pub fn get_shutdown_rx(&self) -> broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }
    pub fn is_shutting_down(&self) -> bool {
        self.is_shutting_down.load(Ordering::Acquire)
    }
    pub fn shutdown(&self) {
        tracing::info!("Shutting down gracefully...");
        self.is_shutting_down.store(true, Ordering::Release);
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
                let count = self.active_tasks.load(Ordering::Acquire);
                if count == 0 {
                    tracing::info!("All active tasks have finished.");
                    break;
                }
                tracing::info!("Waiting for {} active tasks to finish...", count);
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
    pub fn task_started(&self) {
        self.active_tasks.fetch_add(1, Ordering::AcqRel);
    }
    pub fn task_finished(&self) {
        self.active_tasks.fetch_sub(1, Ordering::AcqRel);
    }
    pub fn active_task_count(&self) -> usize {
        self.active_tasks.load(Ordering::Acquire)
    }
}

pub struct TaskGuard {
    shutdown: Arc<GracefulShutdown>,
}

impl TaskGuard {
    pub fn new(shutdown: Arc<GracefulShutdown>) -> Option<Self> {
        if shutdown.is_shutting_down() {
            return None;
        }
        shutdown.task_started();
        Some(Self { shutdown })
    }
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        self.shutdown.task_finished();
    }
}
