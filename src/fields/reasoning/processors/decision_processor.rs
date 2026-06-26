//! Decision processor — evaluates conclusions into actionable decisions.
//!
//! On CONCLUSION_READY, translates high-confidence conclusions into
//! formal DecisionMade signals with a reasoned recommendation.
//! Higher confidence conclusions produce higher-quality decisions.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::reasoning::{ConclusionReady, DecisionMade};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Minimum confidence to form a decision.
const MIN_CONFIDENCE: f32 = 0.5;

/// Translates conclusions into decisions.
pub struct DecisionProcessor;

impl DecisionProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Processor for DecisionProcessor {
    fn name(&self) -> &str {
        "decision"
    }

    fn priority(&self) -> u8 {
        120
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::CONCLUSION_READY]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::DECISION_EVALUATED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::CONCLUSION_READY {
            if let Some(conc) = signal.as_any().downcast_ref::<ConclusionReady>() {
                if conc.confidence < MIN_CONFIDENCE {
                    tracing::trace!(
                        "[DecisionProcessor] confidence too low ({:.2}), skipping",
                        conc.confidence,
                    );
                    return Ok(vec![]);
                }

                let choice = format!("Act on: {}", conc.conclusion);
                let reasoning = format!(
                    "Decision based on conclusion (confidence: {:.2})",
                    conc.confidence,
                );

                tracing::info!(
                    "[DecisionProcessor] decision: {} (confidence: {:.2})",
                    choice, conc.confidence,
                );

                return Ok(vec![Arc::new(DecisionMade::new(&choice, &reasoning))]);
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for DecisionProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_decision_processor_name() {
        let p = DecisionProcessor::new();
        assert_eq!(p.name(), "decision");
    }

    #[tokio::test]
    async fn test_decision_emits_on_high_confidence() {
        let mut p = DecisionProcessor::new();
        let ctx = test_context();

        let conc = ConclusionReady::new("The system should prioritize memory consolidation", 0.85);
        let result = p.process(&ctx, Arc::new(conc)).await.unwrap();
        assert!(!result.is_empty(), "high confidence should produce decision");

        let sig = result[0].as_any().downcast_ref::<DecisionMade>().unwrap();
        assert!(sig.choice.contains("consolidation"), "decision should reference the conclusion");
    }

    #[tokio::test]
    async fn test_decision_skips_low_confidence() {
        let mut p = DecisionProcessor::new();
        let ctx = test_context();

        let conc = ConclusionReady::new("Maybe do something", 0.3);
        let result = p.process(&ctx, Arc::new(conc)).await.unwrap();
        assert!(result.is_empty(), "low confidence should not produce decision");
    }
}
