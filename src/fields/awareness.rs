use std::any::Any;
use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tracing;

use crate::eventbus::signal::SignalArc;
use crate::field::field::Field;
use crate::field::context::FieldContext;
use crate::signals::types;

/// The current focus of the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusItem {
    pub topic: String,
    pub salience: f32,
    pub reason: String,
}

/// State of the Awareness Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwarenessFieldState {
    pub focus_stack: Vec<FocusItem>,
    pub salience_map: std::collections::HashMap<String, f32>,
}

/// The Awareness Field — tracks current focus and salience.
pub struct AwarenessField {
    state: AwarenessFieldState,
}

impl AwarenessField {
    pub fn new() -> Self {
        Self {
            state: AwarenessFieldState {
                focus_stack: Vec::new(),
                salience_map: std::collections::HashMap::new(),
            },
        }
    }
}

#[async_trait]
impl Field for AwarenessField {
    fn name(&self) -> &str {
        "awareness"
    }

    async fn init(&mut self, _ctx: &FieldContext) -> Result<()> {
        tracing::info!("[AwarenessField] initialized");
        Ok(())
    }

    async fn handle_signal(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<()> {
        if signal.signal_type() == types::ATTENTION_SHIFTED {
            tracing::debug!("[AwarenessField] attention shifted");
        } else if signal.signal_type() == types::CURIOSITY_DETECTED {
            tracing::debug!("[AwarenessField] curiosity detected");
        }
        Ok(())
    }

    fn state(&self) -> Box<dyn Any> {
        Box::new(self.state.clone())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("[AwarenessField] shutting down");
        Ok(())
    }
}

impl Default for AwarenessField {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::field::field::Field;
    use crate::field::context::FieldContext;
    use crate::storage::memory_store::MemoryStore;
    use crate::eventbus::bus::EventBus;
    use crate::signals::AttentionShifted;

    #[tokio::test]
    async fn test_awareness_field_init() {
        let storage = Arc::new(MemoryStore::new());
        let bus = Arc::new(EventBus::new());
        let ctx = FieldContext::new(bus, storage);

        let mut field = AwarenessField::new();
        field.init(&ctx).await.unwrap();
        assert_eq!(field.name(), "awareness");
    }

    #[tokio::test]
    async fn test_awareness_field_handles_attention() {
        let storage = Arc::new(MemoryStore::new());
        let bus = Arc::new(EventBus::new());
        let ctx = FieldContext::new(bus, storage);

        let mut field = AwarenessField::new();
        field.init(&ctx).await.unwrap();

        let sig = AttentionShifted::new(
            "Working on Noesis", 0.9, "New episode arrived",
        );
        let result = field.handle_signal(&ctx, Arc::new(sig)).await;
        assert!(result.is_ok(), "should handle attention signals");
    }
}
