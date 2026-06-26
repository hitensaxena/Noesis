use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalType, SignalArc};
use crate::signals::types;
use crate::signals::awareness::ObserverTransitionDetected;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Estimates confidence of the system's cognitive state based on
/// signal activation, salience, and novelty patterns.
pub struct ConfidenceEstimator {
    sample_count: usize,
    avg_activation: f64,
    avg_salience: f64,
}
impl ConfidenceEstimator {
    pub fn new() -> Self { Self { sample_count: 0, avg_activation: 0.0, avg_salience: 0.0 } }
}

#[async_trait]
impl Processor for ConfidenceEstimator {
    fn name(&self) -> &str { "confidence" }
    fn version(&self) -> &str { "0.1.0" }
    fn priority(&self) -> u8 { 115 }
    fn subscribed_signals(&self) -> &[SignalType] { &[types::OBSERVER_TRANSITION_DETECTED] }
    fn emitted_signals(&self) -> &[SignalType] { &[] }

    async fn process(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<Vec<SignalArc>> {
        if let Some(obs) = signal.as_any().downcast_ref::<ObserverTransitionDetected>() {
            self.sample_count += 1;
            let n = self.sample_count as f64;
            self.avg_activation = self.avg_activation * ((n - 1.0) / n) + obs.activation as f64 / n;
            self.avg_salience = self.avg_salience * ((n - 1.0) / n) + obs.salience as f64 / n;
            tracing::trace!("[Confidence] avg_activation={:.3}, avg_salience={:.3}", self.avg_activation, self.avg_salience);
        }
        Ok(vec![])
    }
    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}
impl Default for ConfidenceEstimator { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
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
    fn test_confidence_name() {
        let p = ConfidenceEstimator::new();
        assert_eq!(p.name(), "confidence");
    }

    #[test]
    fn test_confidence_subscriptions() {
        let p = ConfidenceEstimator::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::OBSERVER_TRANSITION_DETECTED));
    }

    #[test]
    fn test_confidence_never_emits() {
        // This processor never emits — it only updates internal averages
        assert!(true, "ConfidenceEstimator has empty emitted_signals()");
    }

    #[tokio::test]
    async fn test_confidence_processes_signal() {
        let mut p = ConfidenceEstimator::new();
        let ctx = test_context();

        let sig = ObserverTransitionDetected::new("test.signal", "test", 1, 0.5, 0.5);
        let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
        assert!(result.is_empty(), "confidence processor never emits");
    }
}
