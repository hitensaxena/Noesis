use std::any::Any;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::SignalArc;
use crate::field_runtime::field::Field;
use crate::field_runtime::context::FieldContext;
use crate::signals::types;
use crate::signals::awareness::{AttentionShifted, CuriosityDetected, ObserverTransitionDetected};
use chrono::Utc;

pub mod state;
pub mod processors;
pub use state::{AwarenessFieldState, FocusItem, CuriosityItem, TransitionRecord};

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
                curiosity_items: Vec::new(),
                recent_transitions: Vec::with_capacity(100),
                total_transitions: 0,
            },
        }
    }
}

#[async_trait]
impl Field for AwarenessField {
    fn name(&self) -> &str { "awareness" }

    async fn init(&mut self, _ctx: &FieldContext) -> Result<()> {
        tracing::info!("[AwarenessField] initialized");
        Ok(())
    }

    async fn handle_signal(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<()> {
        let signal_type = signal.signal_type();

        if signal_type == types::ATTENTION_SHIFTED {
            if let Some(attn) = signal.as_any().downcast_ref::<AttentionShifted>() {
                let item = FocusItem {
                    topic: attn.new_focus.clone(),
                    salience: attn.salience,
                    reason: attn.reason.clone(),
                };
                self.state.focus_stack.push(item);
                self.state.salience_map
                    .entry(attn.new_focus.clone())
                    .and_modify(|s| *s = (*s + attn.salience) / 2.0)
                    .or_insert(attn.salience);
                tracing::debug!("[AwarenessField] attention shifted to '{}' (salience: {:.2})",
                    attn.new_focus, attn.salience);
            }
        } else if signal_type == types::CURIOSITY_DETECTED {
            if let Some(cur) = signal.as_any().downcast_ref::<CuriosityDetected>() {
                let item = CuriosityItem {
                    id: cur.curiosity_id,
                    topic: cur.topic.clone(),
                    gap_description: cur.gap_description.clone(),
                    intensity: cur.intensity,
                };
                self.state.curiosity_items.push(item);
                tracing::debug!("[AwarenessField] stored curiosity: '{}' (intensity: {:.2})",
                    cur.topic, cur.intensity);
            }
        } else if signal_type == types::OBSERVER_TRANSITION_DETECTED {
            if let Some(obs) = signal.as_any().downcast_ref::<ObserverTransitionDetected>() {
                let record = TransitionRecord {
                    signal_type: obs.signal_type.clone(),
                    source: obs.source.clone(),
                    depth: obs.depth,
                    activation: obs.activation,
                    salience: obs.salience,
                    timestamp: Utc::now(),
                };
                self.state.recent_transitions.push(record);
                if self.state.recent_transitions.len() > 100 {
                    self.state.recent_transitions.remove(0);
                }
                self.state.total_transitions += 1;
            }
        }
        Ok(())
    }

    fn state(&self) -> Box<dyn Any + Send> {
        Box::new(self.state.clone())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("[AwarenessField] shutting down");
        Ok(())
    }
}

impl Default for AwarenessField {
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
        let sig = AttentionShifted::new("Working on Noesis", 0.9, "New episode arrived");
        let result = field.handle_signal(&ctx, Arc::new(sig)).await;
        assert!(result.is_ok(), "should handle attention signals");
    }
}
