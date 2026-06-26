use tokio_util::sync::CancellationToken;
use tracing;

/// Manages the tokio task set for graceful shutdown of all spawned tasks.
pub struct Runtime {
    cancellation_token: CancellationToken,
    task_handles: Vec<tokio::task::JoinHandle<()>>,
}

impl Runtime {
    pub fn new() -> Self {
        Self {
            cancellation_token: CancellationToken::new(),
            task_handles: Vec::new(),
        }
    }

    /// Spawn a tokio task that will be cancelled on shutdown.
    pub fn spawn<F>(&mut self, name: &str, future: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let token = self.cancellation_token.clone();
        let name_owned = name.to_string();
        let handle = tokio::spawn(async move {
            tokio::select! {
                _ = future => {},
                _ = token.cancelled() => {
                    tracing::debug!("[Runtime] task '{}' cancelled", name_owned);
                }
            }
        });
        self.task_handles.push(handle);
        tracing::debug!("[Runtime] spawned task: {}", name);
    }

    /// Return a reference to the shutdown token.
    pub fn shutdown_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    /// Signal shutdown and await all spawned tasks.
    pub async fn shutdown(&mut self) {
        tracing::info!(
            "[Runtime] shutting down {} tasks",
            self.task_handles.len()
        );
        self.cancellation_token.cancel();
        for handle in self.task_handles.drain(..) {
            let _ = handle.await;
        }
        tracing::info!("[Runtime] all tasks stopped");
    }
}

impl Default for Runtime {
    fn default() -> Self {
        Self::new()
    }
}
