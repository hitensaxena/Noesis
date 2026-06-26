use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing;

use crate::kernel::signal::{SignalType, SignalMeta, SignalArc};
use crate::signals::types;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValuesRefined {
    pub meta: SignalMeta,
    pub value_id: Uuid,
    pub value: String,
    pub confidence: f32,
    pub source: String,
}
impl ValuesRefined {
    pub fn new(value: &str, confidence: f32, source: &str) -> Self {
        Self {
            meta: SignalMeta::new(types::VALUES_REFINED, "identity::values"),
            value_id: Uuid::new_v4(),
            value: value.to_string(),
            confidence,
            source: source.to_string(),
        }
    }
}
crate::signals::signal_impl!(ValuesRefined, VALUES_REFINED, "identity::values");

pub struct ValueExtractor { count: usize }
impl ValueExtractor {
    pub fn new() -> Self { Self { count: 0 } }
}

#[async_trait]
impl Processor for ValueExtractor {
    fn name(&self) -> &str { "values" }
    fn version(&self) -> &str { "0.1.0" }
    fn priority(&self) -> u8 { 130 }
    fn subscribed_signals(&self) -> &[SignalType] { &[types::BELIEF_CHANGED, types::DECISION_EVALUATED] }
    fn emitted_signals(&self) -> &[SignalType] { &[types::VALUES_REFINED] }

    async fn process(&mut self, _ctx: &FieldContext, _signal: SignalArc) -> Result<Vec<SignalArc>> {
        self.count += 1;
        if self.count % 5 == 0 {
            let values = vec![
                ("understanding", 0.7, "repeated curiosity signals"),
                ("growth", 0.6, "new experiences ingested"),
                ("clarity", 0.5, "consolidation patterns"),
            ];
            let idx = (self.count / 5) % values.len();
            let (value, conf, source) = values[idx];
            let result = ValuesRefined::new(value, conf, source);
            tracing::debug!("[ValueExtractor] refined value: {}", value);
            Ok(vec![Arc::new(result)])
        } else { Ok(vec![]) }
    }
    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}
impl Default for ValueExtractor { fn default() -> Self { Self::new() } }

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
    fn test_values_name() {
        let p = ValueExtractor::new();
        assert_eq!(p.name(), "values");
    }

    #[test]
    fn test_values_subscriptions() {
        let p = ValueExtractor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::BELIEF_CHANGED));
        assert!(subs.contains(&types::DECISION_EVALUATED));
    }

    #[tokio::test]
    async fn test_values_emits_every_5() {
        let mut p = ValueExtractor::new();
        let ctx = test_context();

        // Send 4 signals — no emission
        for _ in 0..4 {
            let sig = ObserverTransitionDetected::new("test.signal", "test", 1, 0.5, 0.5);
            let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
            assert!(result.is_empty(), "no emission before 5th signal");
        }

        // 5th signal — should emit
        let sig = ObserverTransitionDetected::new("test.signal", "test", 1, 0.5, 0.5);
        let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
        assert_eq!(result.len(), 1, "should emit ValuesRefined on 5th signal");
    }
}
