use std::any::Any;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::SignalArc;
use crate::field_runtime::field::Field;
use crate::field_runtime::context::FieldContext;
use crate::signals::types;
use crate::fields::reasoning::state::ReasoningFieldState;

pub mod state;
pub mod processors;

/// The Reasoning Field — answers "What do I conclude?"
///
/// Owns reasoning chains, mental models, metacognitive insights,
/// decisions, hypotheses, analogies, epistemic classifications,
/// syntheses, and concepts.
pub struct ReasoningField {
    state: ReasoningFieldState,
}

impl ReasoningField {
    pub fn new() -> Self {
        Self {
            state: ReasoningFieldState::default(),
        }
    }
}

#[async_trait]
impl Field for ReasoningField {
    fn name(&self) -> &str { "reasoning" }

    async fn init(&mut self, _ctx: &FieldContext) -> Result<()> {
        tracing::info!("[ReasoningField] initialized");
        Ok(())
    }

    async fn handle_signal(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<()> {
        let signal_type = signal.signal_type();

        // Handle metacognitive insights
        if signal_type == types::METACOGNITION_INSIGHT {
            // MetaProcessor emits these; store in metacognitive_insights
            tracing::debug!("[ReasoningField] received metacognition.insight");
        }

        Ok(())
    }

    fn state(&self) -> Box<dyn Any + Send> {
        Box::new(self.state.clone())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("[ReasoningField] shutting down with {} insights, {} decisions",
            self.state.insight_count, self.state.decision_count);
        Ok(())
    }
}

impl Default for ReasoningField {
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
    async fn test_reasoning_field_init() {
        let storage = Arc::new(MemoryStore::new());
        let bus = Arc::new(EventBus::new());
        let ctx = FieldContext::new(bus, storage);
        let mut field = ReasoningField::new();
        field.init(&ctx).await.unwrap();
        assert_eq!(field.name(), "reasoning");
    }
}
