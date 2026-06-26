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
pub struct PrinciplesDerived {
    pub meta: SignalMeta,
    pub principle_id: Uuid,
    pub principle: String,
    pub confidence: f32,
    pub evidence: Vec<String>,
}
impl PrinciplesDerived {
    pub fn new(principle: &str, confidence: f32, evidence: Vec<String>) -> Self {
        Self {
            meta: SignalMeta::new(types::PRINCIPLES_DERIVED, "identity::principles"),
            principle_id: Uuid::new_v4(),
            principle: principle.to_string(),
            confidence,
            evidence,
        }
    }
}
crate::signals::signal_impl!(PrinciplesDerived, PRINCIPLES_DERIVED, "identity::principles");

pub struct PrincipleDistiller { count: usize }
impl PrincipleDistiller {
    pub fn new() -> Self { Self { count: 0 } }
}

#[async_trait]
impl Processor for PrincipleDistiller {
    fn name(&self) -> &str { "principles" }
    fn version(&self) -> &str { "0.1.0" }
    fn priority(&self) -> u8 { 140 }
    fn subscribed_signals(&self) -> &[SignalType] { &[types::VALUES_REFINED, types::DECISION_EVALUATED] }
    fn emitted_signals(&self) -> &[SignalType] { &[types::PRINCIPLES_DERIVED] }

    async fn process(&mut self, _ctx: &FieldContext, _signal: SignalArc) -> Result<Vec<SignalArc>> {
        self.count += 1;
        if self.count % 3 == 0 {
            let p = PrinciplesDerived::new(
                "prioritize understanding over speed",
                0.5,
                vec!["derived from repeated curiosity patterns".to_string()],
            );
            tracing::debug!("[PrincipleDistiller] derived principle");
            Ok(vec![Arc::new(p)])
        } else { Ok(vec![]) }
    }
    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}
impl Default for PrincipleDistiller { fn default() -> Self { Self::new() } }

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
    fn test_principles_name() {
        let p = PrincipleDistiller::new();
        assert_eq!(p.name(), "principles");
    }

    #[test]
    fn test_principles_subscriptions() {
        let p = PrincipleDistiller::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::VALUES_REFINED));
        assert!(subs.contains(&types::DECISION_EVALUATED));
    }

    #[tokio::test]
    async fn test_principles_emits_every_3() {
        let mut p = PrincipleDistiller::new();
        let ctx = test_context();

        // Send 2 signals — no emission
        for _ in 0..2 {
            let sig = ObserverTransitionDetected::new("test.signal", "test", 1, 0.5, 0.5);
            let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
            assert!(result.is_empty(), "no emission before 3rd signal");
        }

        // 3rd signal — should emit
        let sig = ObserverTransitionDetected::new("test.signal", "test", 1, 0.5, 0.5);
        let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
        assert_eq!(result.len(), 1, "should emit PrinciplesDerived on 3rd signal");
    }
}
