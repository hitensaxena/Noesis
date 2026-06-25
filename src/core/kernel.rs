use std::sync::Arc;
use anyhow::Result;
use tracing;

use crate::core::registry::Registry;
use crate::core::lifecycle::Lifecycle;
use crate::core::runtime::Runtime;
use crate::eventbus::bus::EventBus;

/// The Kernel is the top-level orchestrator for the Noesis system.
///
/// It owns the EventBus, Registry, Runtime, and Lifecycle.
/// It does NOT route signals or make decisions — it only wires components
/// and manages their lifecycle. No god objects.
pub struct Kernel {
    pub event_bus: Arc<EventBus>,
    pub registry: Registry,
    pub lifecycle: Lifecycle,
    pub runtime: Runtime,
}

impl Kernel {
    pub fn new() -> Self {
        Self {
            event_bus: Arc::new(EventBus::new()),
            registry: Registry::new(),
            lifecycle: Lifecycle::new(),
            runtime: Runtime::new(),
        }
    }

    /// Initialize the kernel: run startup hooks.
    pub async fn init(&mut self) -> Result<()> {
        tracing::info!("[Kernel] initializing...");
        self.lifecycle.run_startup()?;
        tracing::info!("[Kernel] initialized");
        Ok(())
    }

    /// Shut down the kernel gracefully: shutdown hooks, cancel tasks.
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("[Kernel] shutting down...");
        self.lifecycle.run_shutdown()?;
        self.runtime.shutdown().await;
        tracing::info!("[Kernel] shutdown complete");
        Ok(())
    }
}

impl Default for Kernel {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kernel_initialization() {
        let mut kernel = Kernel::new();
        assert_eq!(kernel.registry.list_fields().len(), 0);
        assert_eq!(kernel.registry.list_processors().len(), 0);

        let result = kernel.init().await;
        assert!(result.is_ok(), "kernel init should succeed");
    }

    #[tokio::test]
    async fn test_kernel_shutdown() {
        let mut kernel = Kernel::new();
        kernel.init().await.unwrap();
        let result = kernel.shutdown().await;
        assert!(result.is_ok(), "kernel shutdown should succeed");
    }
}
