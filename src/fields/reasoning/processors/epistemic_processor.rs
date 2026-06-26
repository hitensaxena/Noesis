//! Epistemic classifier — categorizes observed signals by epistemic status.
//!
//! On OBSERVER_TRANSITION_DETECTED, classifies the signal using activation
//! and salience heuristics. Emits EpistemicClassified with categories:
//! known, inferred, believed, or uncertain.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalType, SignalArc};
use crate::signals::types;
use crate::signals::awareness::ObserverTransitionDetected;
use crate::signals::reasoning::EpistemicClassified;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

pub struct EpistemicClassifier;

impl EpistemicClassifier {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Processor for EpistemicClassifier {
    fn name(&self) -> &str { "epistemic" }
    fn version(&self) -> &str { "0.1.0" }
    fn priority(&self) -> u8 { 110 }
    fn subscribed_signals(&self) -> &[SignalType] { &[types::OBSERVER_TRANSITION_DETECTED] }
    fn emitted_signals(&self) -> &[SignalType] { &[types::EPISTEMICS_CLASSIFIED] }

    async fn process(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<Vec<SignalArc>> {
        if let Some(obs) = signal.as_any().downcast_ref::<ObserverTransitionDetected>() {
            let (classification, confidence) = classify(&obs.signal_type, obs.activation, obs.salience);
            tracing::trace!("[Epistemic] {} -> {}", obs.signal_type, classification);
            let result = EpistemicClassified::new(&obs.signal_type, classification, confidence);
            Ok(vec![Arc::new(result)])
        } else { Ok(vec![]) }
    }
    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}
impl Default for EpistemicClassifier { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::signal::SignalType;
    use crate::signals::awareness::ObserverTransitionDetected;
    use crate::signals::reasoning::EpistemicClassified;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_epistemic_name() {
        let p = EpistemicClassifier::new();
        assert_eq!(p.name(), "epistemic");
    }

    #[test]
    fn test_epistemic_subscriptions() {
        let p = EpistemicClassifier::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::OBSERVER_TRANSITION_DETECTED));
    }

    #[tokio::test]
    async fn test_epistemic_classifies_signal() {
        let mut p = EpistemicClassifier::new();
        let ctx = test_context();

        let sig = ObserverTransitionDetected::new("test.signal", "test", 1, 0.9, 0.9);
        let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
        assert_eq!(result.len(), 1, "epistemic classifier always emits");
        let classified = result[0].as_any().downcast_ref::<EpistemicClassified>().unwrap();
        assert_eq!(classified.signal_type, "test.signal");
    }

    #[tokio::test]
    async fn test_epistemic_high_activation_is_known() {
        let mut p = EpistemicClassifier::new();
        let ctx = test_context();

        let sig = ObserverTransitionDetected::new("test.high", "test", 1, 0.9, 0.5);
        let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
        let classified = result[0].as_any().downcast_ref::<EpistemicClassified>().unwrap();
        assert_eq!(classified.classification, "known");
    }
}

fn classify(signal_type: &str, activation: f32, salience: f32) -> (&'static str, f32) {
    if activation > 0.8 { ("known", 0.9) }
    else if signal_type.contains("extract") || signal_type.contains("record") { ("inferred", 0.7) }
    else if salience > 0.7 { ("believed", 0.6) }
    else if activation > 0.4 { ("inferred", 0.5) }
    else { ("uncertain", 0.3) }
}
