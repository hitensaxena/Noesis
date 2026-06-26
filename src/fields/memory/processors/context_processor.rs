//! ContextConstructor — assembles reasoning context from recent memory.
//!
//! Subscribes to ObserverTransitionDetected and builds a context window
//! containing the most recent signals and their relationships. This context
//! is used by reasoning processors to ground their conclusions in recent
//! cognitive state. Emits ContextAssembled periodically.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalType, SignalMeta, SignalArc};
use crate::signals::types;
use crate::signals::awareness::ObserverTransitionDetected;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// ContextAssembled signal (defined locally since it's a memory-specific signal)
// ---------------------------------------------------------------------------

/// A context window assembled from recent cognitive state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAssembled {
    pub meta: SignalMeta,
    pub context_id: Uuid,
    pub recent_signals: Vec<String>,
    pub signal_count: usize,
    pub attention_focus: Option<String>,
}

impl ContextAssembled {
    pub fn new(recent_signals: Vec<String>, signal_count: usize, attention_focus: Option<String>) -> Self {
        Self {
            meta: SignalMeta::new(types::CONTEXT_ASSEMBLED, "memory::context_processor"),
            context_id: Uuid::new_v4(),
            recent_signals,
            signal_count,
            attention_focus,
        }
    }
}

crate::signals::signal_impl!(ContextAssembled, CONTEXT_ASSEMBLED, "memory::context_processor");

// ---------------------------------------------------------------------------
// ContextConstructor processor
// ---------------------------------------------------------------------------

/// Assembles reasoning context from recent signal observations.
pub struct ContextConstructor {
    signal_buffer: Vec<String>,
    cycle_count: usize,
}

impl ContextConstructor {
    pub fn new() -> Self {
        Self {
            signal_buffer: Vec::with_capacity(50),
            cycle_count: 0,
        }
    }
}

#[async_trait]
impl Processor for ContextConstructor {
    fn name(&self) -> &str { "context" }
    fn version(&self) -> &str { "0.1.0" }
    fn priority(&self) -> u8 { 120 }
    fn activation_threshold(&self) -> f32 { 0.1 }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::OBSERVER_TRANSITION_DETECTED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::CONTEXT_ASSEMBLED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        if let Some(obs) = signal.as_any().downcast_ref::<ObserverTransitionDetected>() {
            self.signal_buffer.push(obs.signal_type.clone());
            if self.signal_buffer.len() > 50 {
                self.signal_buffer.remove(0);
            }
            self.cycle_count += 1;

            // Assemble context every 30 observations
            if self.cycle_count % 30 == 0 {
                // Deduplicate and take top types
                let mut type_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
                for sig in &self.signal_buffer {
                    *type_counts.entry(sig.clone()).or_insert(0) += 1;
                }
                let mut ranked: Vec<_> = type_counts.into_iter().collect();
                ranked.sort_by(|a, b| b.1.cmp(&a.1));
                let top_signals: Vec<String> = ranked.into_iter().take(5).map(|(t, _)| t).collect();

                // Check for attention-related signals
                let attention_focus = if top_signals.iter().any(|s| s.contains("attention")) {
                    Some("attention signal present".to_string())
                } else {
                    None
                };

                tracing::debug!("[ContextConstructor] assembled context with {} signal types", top_signals.len());

                let context = ContextAssembled::new(
                    top_signals,
                    self.signal_buffer.len(),
                    attention_focus,
                );
                return Ok(vec![Arc::new(context)]);
            }
        }
        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}

impl Default for ContextConstructor {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::signal::SignalType;
    use crate::signals::awareness::ObserverTransitionDetected;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_context_name() {
        let p = ContextConstructor::new();
        assert_eq!(p.name(), "context");
    }

    #[test]
    fn test_context_subscriptions() {
        let p = ContextConstructor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::OBSERVER_TRANSITION_DETECTED));
    }

    #[tokio::test]
    async fn test_context_emits_every_30() {
        let mut p = ContextConstructor::new();
        let ctx = test_context();

        for _ in 0..29 {
            let sig = ObserverTransitionDetected::new("test.signal", "test", 1, 0.5, 0.5);
            let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
            assert!(result.is_empty(), "no emission before 30th");
        }

        let sig = ObserverTransitionDetected::new("test.signal", "test", 1, 0.5, 0.5);
        let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
        assert_eq!(result.len(), 1, "should emit ContextAssembled on 30th");
    }

    #[tokio::test]
    async fn test_context_collects_signal_types() {
        let mut p = ContextConstructor::new();
        let ctx = test_context();

        for i in 0..30 {
            let sig = ObserverTransitionDetected::new(&format!("signal.{}", i % 3), "test", 1, 0.5, 0.5);
            let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
            if i < 29 {
                assert!(result.is_empty());
            } else {
                assert_eq!(result.len(), 1, "30th signal triggers context emission");
            }
        }
    }
}
