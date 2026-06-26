use std::any::Any;
use async_trait::async_trait;
use anyhow::Result;
use chrono::Utc;
use tracing;

use crate::kernel::signal::SignalArc;
use crate::field_runtime::field::Field;
use crate::field_runtime::context::FieldContext;
use crate::signals::types;
use crate::signals::BeliefChanged;

pub mod state;
pub mod processors;
pub use state::{IdentityFieldState, Belief, Trait};

pub struct IdentityField {
    state: IdentityFieldState,
}

impl IdentityField {
    pub fn new() -> Self {
        Self {
            state: IdentityFieldState {
                beliefs: Vec::new(),
                traits: Vec::new(),
                identity_version: 0,
            },
        }
    }

    /// Return a structured self-model summary.
    pub fn self_model(&self) -> serde_json::Value {
        serde_json::json!({
            "version": self.state.identity_version,
            "beliefs_count": self.state.beliefs.len(),
            "traits_count": self.state.traits.len(),
            "traits": self.state.traits.iter().map(|t| serde_json::json!({
                "name": t.name,
                "strength": t.strength,
            })).collect::<Vec<_>>(),
        })
    }

    /// Return all active beliefs.
    pub fn beliefs(&self) -> Vec<Belief> {
        self.state.beliefs.clone()
    }
}

#[async_trait]
impl Field for IdentityField {
    fn name(&self) -> &str { "identity" }

    async fn init(&mut self, _ctx: &FieldContext) -> Result<()> {
        tracing::info!("[IdentityField] initialized");
        Ok(())
    }

    async fn handle_signal(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<()> {
        if signal.signal_type() == types::BELIEF_CHANGED {
            if let Some(bc) = signal.as_any().downcast_ref::<BeliefChanged>() {
                let belief = Belief {
                    id: bc.belief_id,
                    belief: bc.belief.clone(),
                    confidence: bc.confidence,
                    created_at: Utc::now(),
                    is_active: true,
                };
                self.state.beliefs.push(belief);
                self.state.identity_version += 1;
                tracing::debug!("[IdentityField] stored belief (v{}, total: {})",
                    self.state.identity_version, self.state.beliefs.len());
            }
        }
        Ok(())
    }

    fn state(&self) -> Box<dyn Any + Send> {
        Box::new(self.state.clone())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("[IdentityField] shutting down with {} beliefs, {} traits",
            self.state.beliefs.len(), self.state.traits.len());
        Ok(())
    }
}

impl Default for IdentityField {
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
    use crate::signals::{BeliefChanged, BeliefChangeType};

    #[tokio::test]
    async fn test_identity_field_stores_beliefs() {
        let storage = Arc::new(MemoryStore::new());
        let bus = Arc::new(EventBus::new());
        let ctx = FieldContext::new(bus, storage);
        let mut field = IdentityField::new();
        field.init(&ctx).await.unwrap();
        let bc = BeliefChanged::new("I value deep thinking", BeliefChangeType::Created, 0.9);
        field.handle_signal(&ctx, Arc::new(bc)).await.unwrap();
        let state = field.state();
        let state = state.downcast_ref::<IdentityFieldState>();
        assert!(state.is_some(), "state should be IdentityFieldState");
        assert_eq!(state.unwrap().beliefs.len(), 1, "should have 1 belief");
    }
}
