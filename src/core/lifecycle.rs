use std::sync::atomic::{AtomicU8, Ordering};
use anyhow::Result;
use tracing;

/// Phases of the Noesis lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Init = 0,
    RegisterFields = 1,
    RegisterProcessors = 2,
    StartEventBus = 3,
    StartScheduler = 4,
    Running = 5,
    Shutdown = 6,
}

impl From<u8> for Phase {
    fn from(v: u8) -> Self {
        match v {
            0 => Phase::Init,
            1 => Phase::RegisterFields,
            2 => Phase::RegisterProcessors,
            3 => Phase::StartEventBus,
            4 => Phase::StartScheduler,
            5 => Phase::Running,
            _ => Phase::Shutdown,
        }
    }
}

/// Manages ordered startup and shutdown of all system components.
pub struct Lifecycle {
    current_phase: AtomicU8,
    startup_hooks: Vec<Box<dyn Fn() -> Result<()> + Send + Sync>>,
    shutdown_hooks: Vec<Box<dyn Fn() -> Result<()> + Send + Sync>>,
}

impl Lifecycle {
    pub fn new() -> Self {
        Self {
            current_phase: AtomicU8::new(0),
            startup_hooks: Vec::new(),
            shutdown_hooks: Vec::new(),
        }
    }

    pub fn add_startup_hook(
        &mut self,
        hook: Box<dyn Fn() -> Result<()> + Send + Sync>,
    ) {
        self.startup_hooks.push(hook);
    }

    pub fn add_shutdown_hook(
        &mut self,
        hook: Box<dyn Fn() -> Result<()> + Send + Sync>,
    ) {
        self.shutdown_hooks.push(hook);
    }

    pub fn run_startup(&self) -> Result<()> {
        tracing::info!("[Lifecycle] running {} startup hooks", self.startup_hooks.len());
        for hook in &self.startup_hooks {
            hook()?;
            let phase: Phase = self.phase().into();
            tracing::debug!("[Lifecycle] completed phase {:?}", phase);
        }
        self.current_phase.store(Phase::Running as u8, Ordering::SeqCst);
        tracing::info!("[Lifecycle] startup complete, system is running");
        Ok(())
    }

    pub fn run_shutdown(&self) -> Result<()> {
        self.current_phase
            .store(Phase::Shutdown as u8, Ordering::SeqCst);
        tracing::info!(
            "[Lifecycle] running {} shutdown hooks",
            self.shutdown_hooks.len()
        );
        for hook in self.shutdown_hooks.iter().rev() {
            hook()?;
        }
        tracing::info!("[Lifecycle] shutdown complete");
        Ok(())
    }

    pub fn phase(&self) -> u8 {
        self.current_phase.load(Ordering::SeqCst)
    }
}

impl Default for Lifecycle {
    fn default() -> Self {
        Self::new()
    }
}
