use std::any::Any;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::SignalArc;
use crate::field_runtime::field::Field;
use crate::field_runtime::context::FieldContext;

pub mod state;
pub mod domains;
pub mod processors;
pub use state::{ActionFieldState, Project, Task};

/// The Action Field — executes plans, manages tasks, evaluates outcomes.
///
/// Receives signals from Agency (goals) and produces execution signals.
/// Currently a stub — full implementation will handle projects, planning,
/// task dispatch, execution, evaluation, risk, and recovery.
pub struct ActionField {
    state: ActionFieldState,
}

impl ActionField {
    pub fn new() -> Self {
        Self {
            state: ActionFieldState {
                projects: Vec::new(),
                tasks: Vec::new(),
            },
        }
    }
}

#[async_trait]
impl Field for ActionField {
    fn name(&self) -> &str { "action" }

    async fn init(&mut self, _ctx: &FieldContext) -> Result<()> {
        tracing::info!("[ActionField] initialized");
        Ok(())
    }

    async fn handle_signal(&mut self, _ctx: &FieldContext, _signal: SignalArc) -> Result<()> {
        Ok(())
    }

    fn state(&self) -> Box<dyn Any + Send> {
        Box::new(self.state.clone())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("[ActionField] shutting down with {} projects, {} tasks",
            self.state.projects.len(), self.state.tasks.len());
        Ok(())
    }
}

impl Default for ActionField {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::field_runtime::field::Field;
    use crate::field_runtime::context::FieldContext;
    use crate::storage::memory_store::MemoryStore;
    use crate::kernel::bus::EventBus;

    #[tokio::test]
    async fn test_action_field_init() {
        let storage = Arc::new(MemoryStore::new());
        let bus = Arc::new(EventBus::new());
        let ctx = FieldContext::new(bus, storage);
        let mut field = ActionField::new();
        field.init(&ctx).await.unwrap();
        assert_eq!(field.name(), "action");
    }
}
