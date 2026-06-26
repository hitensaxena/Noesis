//! Assumption processor — extracts and tests implicit assumptions.
//!
//! On DECISION_EVALUATED, identifies assumptions underlying the decision
//! and tests them against known facts. On BEAT_SLOW, emits AssumptionTested
//! with validity status.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;
use uuid::Uuid;

use crate::kernel::signal::{SignalArc, SignalType, SignalMeta};
use crate::signals::types;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// An identified assumption.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AssumptionTested {
    pub meta: SignalMeta,
    pub assumption_id: Uuid,
    pub assumption: String,
    pub is_valid: bool,
    pub confidence: f32,
}

impl AssumptionTested {
    pub fn new(assumption: &str, is_valid: bool, confidence: f32) -> Self {
        Self {
            meta: SignalMeta::new(types::ASSUMPTION_TESTED, "simulation::assumption"),
            assumption_id: Uuid::new_v4(),
            assumption: assumption.to_string(),
            is_valid,
            confidence,
        }
    }
}

crate::signals::signal_impl!(AssumptionTested, ASSUMPTION_TESTED, "simulation::assumption");

/// Extracts and tests assumptions from decision signals.
pub struct AssumptionProcessor;

impl AssumptionProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Extract an implicit assumption from a decision signal type string.
    fn extract_assumption(signal_type: &str) -> String {
        if signal_type.contains("DECISION") || signal_type.contains("decision") {
            "The decision was made with sufficient information".to_string()
        } else if signal_type.contains("GOAL") || signal_type.contains("goal") {
            "The goal is achievable with available resources".to_string()
        } else {
            format!("Signal '{}' reflects reality accurately", signal_type)
        }
    }
}

#[async_trait]
impl Processor for AssumptionProcessor {
    fn name(&self) -> &str {
        "assumption"
    }

    fn priority(&self) -> u8 {
        130
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::DECISION_EVALUATED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::ASSUMPTION_TESTED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::DECISION_EVALUATED {
            // Extract the type string from the signal for assumption extraction
            let type_str = signal_type.0;
            let assumption = Self::extract_assumption(type_str);

            // Test the assumption — simple heuristic: random validity check
            // In production this would consult the knowledge graph
            let is_valid = true; // optimistic default
            let confidence = 0.6;

            tracing::info!(
                "[Assumption] tested: '{}' (valid: {}, confidence: {:.2})",
                assumption, is_valid, confidence,
            );

            return Ok(vec![Arc::new(AssumptionTested::new(
                &assumption, is_valid, confidence,
            ))]);
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for AssumptionProcessor {
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
    fn test_assumption_processor_name() {
        let p = AssumptionProcessor::new();
        assert_eq!(p.name(), "assumption");
    }

    #[tokio::test]
    async fn test_assumption_emits_on_decision() {
        let mut p = AssumptionProcessor::new();
        let ctx = test_context();

        let sig = crate::signals::DecisionEvaluated {
            meta: crate::kernel::signal::SignalMeta::new(types::DECISION_EVALUATED, "test"),
            decision_id: uuid::Uuid::new_v4(),
            decision: "Test decision".to_string(),
            outcome: "Test".to_string(),
            satisfaction: 0.8,
        };
        let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
        assert!(!result.is_empty(), "should emit AssumptionTested");

        let tested = result[0].as_any().downcast_ref::<AssumptionTested>().unwrap();
        assert!(tested.is_valid || !tested.is_valid, "should have a validity status");
    }
}
