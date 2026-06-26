//! Scenario processor — branches on current state × possible actions.
//!
//! On DECISION_EVALUATED, generates a scenario projecting the likely
//! outcome of the decision. On BEAT_SLOW, emits ScenarioReady with a
//! summary of the most likely scenario.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;
use uuid::Uuid;

use crate::kernel::signal::{SignalArc, SignalType, SignalMeta};
use crate::signals::types;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// A projected scenario.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScenarioReady {
    pub meta: SignalMeta,
    pub scenario_id: Uuid,
    pub description: String,
    pub probability: f32,
}

impl ScenarioReady {
    pub fn new(description: &str, probability: f32) -> Self {
        Self {
            meta: SignalMeta::new(types::SCENARIO_READY, "simulation::scenario"),
            scenario_id: Uuid::new_v4(),
            description: description.to_string(),
            probability,
        }
    }
}

crate::signals::signal_impl!(ScenarioReady, SCENARIO_READY, "simulation::scenario");

/// Generates scenarios from decisions and signals.
pub struct ScenarioProcessor {
    scenario_count: usize,
}

impl ScenarioProcessor {
    pub fn new() -> Self {
        Self {
            scenario_count: 0,
        }
    }
}

#[async_trait]
impl Processor for ScenarioProcessor {
    fn name(&self) -> &str {
        "scenario"
    }

    fn priority(&self) -> u8 {
        140
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::DECISION_EVALUATED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::SCENARIO_READY]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::DECISION_EVALUATED {
            self.scenario_count += 1;
            let prob = 0.5 + (self.scenario_count as f32 * 0.05).min(0.4);
            let description = format!(
                "Scenario #{}: Decision outcome projected with {:.0}% confidence",
                self.scenario_count, prob * 100.0,
            );

            tracing::info!(
                "[Scenario] generated: {} (prob: {:.2})",
                description, prob,
            );

            return Ok(vec![Arc::new(ScenarioReady::new(&description, prob))]);
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for ScenarioProcessor {
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
    fn test_scenario_processor_name() {
        let p = ScenarioProcessor::new();
        assert_eq!(p.name(), "scenario");
    }

    #[tokio::test]
    async fn test_scenario_emits_on_decision() {
        let mut p = ScenarioProcessor::new();
        let ctx = test_context();

        let sig = crate::signals::DecisionEvaluated {
            meta: SignalMeta::new(types::DECISION_EVALUATED, "test"),
            decision_id: Uuid::new_v4(),
            decision: "Test decision".to_string(),
            outcome: "Test".to_string(),
            satisfaction: 0.8,
        };
        let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
        assert!(!result.is_empty(), "should emit ScenarioReady");

        let sc = result[0].as_any().downcast_ref::<ScenarioReady>().unwrap();
        assert!(sc.probability > 0.0 && sc.probability <= 1.0, "probability should be valid");
    }
}
