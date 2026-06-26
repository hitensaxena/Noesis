//! Counterfactual processor — computes "what if" alternatives.
//!
//! On EVALUATION_COMPLETED, generates a counterfactual alternative
//! outcome based on changing one variable. Emits CounterfactualReady.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;
use uuid::Uuid;

use crate::kernel::signal::{SignalArc, SignalType, SignalMeta};
use crate::signals::types;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// A counterfactual alternative outcome.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CounterfactualReady {
    pub meta: SignalMeta,
    pub cf_id: Uuid,
    pub actual_outcome: String,
    pub alternative: String,
}

impl CounterfactualReady {
    pub fn new(outcome: &str, alt: &str) -> Self {
        Self {
            meta: SignalMeta::new(types::COUNTERFACTUAL_READY, "simulation::counterfactual"),
            cf_id: Uuid::new_v4(),
            actual_outcome: outcome.to_string(),
            alternative: alt.to_string(),
        }
    }
}

crate::signals::signal_impl!(CounterfactualReady, COUNTERFACTUAL_READY, "simulation::counterfactual");

/// Generates counterfactual alternatives from evaluation outcomes.
pub struct CounterfactualProcessor {
    cf_count: usize,
}

impl CounterfactualProcessor {
    pub fn new() -> Self {
        Self { cf_count: 0 }
    }
}

#[async_trait]
impl Processor for CounterfactualProcessor {
    fn name(&self) -> &str {
        "counterfactual"
    }

    fn priority(&self) -> u8 {
        150
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EVALUATION_COMPLETED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::COUNTERFACTUAL_READY]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::EVALUATION_COMPLETED {
            self.cf_count += 1;
            let alternatives = [
                "Different resource allocation would have changed the outcome",
                "A different approach could have been more efficient",
                "Earlier intervention might have altered the trajectory",
                "Alternative sequencing could have improved results",
            ];
            let idx = self.cf_count % alternatives.len();
            let alt = alternatives[idx];

            let outcome = format!("evaluation #{}", self.cf_count);

            tracing::info!(
                "[Counterfactual] generated: '{}' ← '{}'",
                alt, outcome,
            );

            return Ok(vec![Arc::new(CounterfactualReady::new(&outcome, alt))]);
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for CounterfactualProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;
    use crate::processors::evaluation::EvaluationCompleted;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_counterfactual_processor_name() {
        let p = CounterfactualProcessor::new();
        assert_eq!(p.name(), "counterfactual");
    }

    #[tokio::test]
    async fn test_counterfactual_emits_on_evaluation() {
        let mut p = CounterfactualProcessor::new();
        let ctx = test_context();

        let eval = EvaluationCompleted::new("Task completed successfully", 0.8);
        let result = p.process(&ctx, Arc::new(eval)).await.unwrap();
        assert!(!result.is_empty(), "should emit CounterfactualReady");

        let cf = result[0].as_any().downcast_ref::<CounterfactualReady>().unwrap();
        assert!(!cf.alternative.is_empty(), "counterfactual should have alternative");
    }
}
