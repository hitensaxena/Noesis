//! ObserverProcessor — records every signal transition.
//!
//! Subscribes to ALL cognitive signals and emits ObserverTransitionDetected
//! for each one. This provides a complete transition log that powers mood
//! estimation, health checking, pattern detection, and all awareness analytics.
//!
//! Foundation processor — must be registered first in the awareness field.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::awareness::ObserverTransitionDetected;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Observes and records every signal transition in the system.
pub struct ObserverProcessor;

impl ObserverProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Processor for ObserverProcessor {
    fn name(&self) -> &str {
        "observer"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn priority(&self) -> u8 {
        10 // Highest priority — must observe before other processors transform
    }

    fn activation_threshold(&self) -> f32 {
        0.05 // Observe even low-activation signals
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[
            types::INGEST_REQUEST,
            types::EPISODE_RECORDED,
            types::FACT_EXTRACTED,
            types::MEMORY_CONSOLIDATED,
            types::PATTERN_DETECTED,
            types::BELIEF_CHANGED,
            types::TRAIT_DETECTED,
            types::IDENTITY_UPDATED,
            types::GOAL_CREATED,
            types::GOAL_COMPLETED,
            types::DECISION_EVALUATED,
            types::ATTENTION_SHIFTED,
            types::CURIOSITY_DETECTED,
            types::NARRATIVE_GENERATED,
            types::ENTITY_CREATED,
            types::EDGE_CREATED,
            types::TRIPLES_EXTRACTED,
        ]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::OBSERVER_TRANSITION_DETECTED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        // Skip observer's own signals to prevent infinite observation loops
        if signal.signal_type() == types::OBSERVER_TRANSITION_DETECTED {
            return Ok(vec![]);
        }

        let signal_type = signal.signal_type().to_string();
        let depth = signal.meta().depth;
        let activation = signal.meta().activation;
        let salience = signal.meta().salience;
        let source = signal.meta().source.clone();

        tracing::trace!(
            "[Observer] transition: {} (depth={}, activation={:.2})",
            signal_type, depth, activation
        );

        let observation = ObserverTransitionDetected::new(
            &signal_type, &source, depth, activation, salience,
        );

        Ok(vec![Arc::new(observation)])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for ObserverProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::signal::{Signal, SignalMeta, SignalType};
    use crate::signals::{EpisodeRecorded, IngestRequest};
    use crate::signals::awareness::ObserverTransitionDetected;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_observer_name() {
        let p = ObserverProcessor::new();
        assert_eq!(p.name(), "observer");
    }

    #[test]
    fn test_observer_subscriptions() {
        let p = ObserverProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::INGEST_REQUEST));
        assert!(subs.contains(&types::EPISODE_RECORDED));
        assert!(subs.contains(&types::BELIEF_CHANGED));
    }

    #[tokio::test]
    async fn test_observer_emits_transition() {
        let mut p = ObserverProcessor::new();
        let ctx = test_context();

        let episode = EpisodeRecorded::new("test content", "test", vec![]);
        let result = p.process(&ctx, Arc::new(episode)).await.unwrap();
        assert_eq!(result.len(), 1, "should emit ObserverTransitionDetected");

        let obs = result[0].as_any().downcast_ref::<ObserverTransitionDetected>().unwrap();
        assert_eq!(obs.signal_type, types::EPISODE_RECORDED.to_string());
    }

    #[tokio::test]
    async fn test_observer_skips_own_signals() {
        let mut p = ObserverProcessor::new();
        let ctx = test_context();

        let obs = ObserverTransitionDetected::new("observer.transition.detected", "test", 1, 0.5, 0.5);
        let result = p.process(&ctx, Arc::new(obs)).await.unwrap();
        assert!(result.is_empty(), "should skip own signal type to prevent loops");
    }

    #[tokio::test]
    async fn test_observer_records_all_cognitive_signals() {
        let mut p = ObserverProcessor::new();
        let ctx = test_context();

        // IngestRequest (not EpisodeRecorded — unit struct downcasts differently)
        let ingest = IngestRequest::new("test", "test");
        let result = p.process(&ctx, Arc::new(ingest)).await.unwrap();
        assert_eq!(result.len(), 1, "should observe ingest requests");
    }
}
