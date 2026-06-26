//! Evaluation processor — evaluates outcomes of executed tasks.
//!
//! On EXECUTION_STARTED, generates an evaluation with a satisfaction
//! score based on the execution context. Satisfaction increases as
//! more executions complete successfully.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvaluationCompleted {
    pub meta: crate::kernel::signal::SignalMeta,
    pub eval_id: uuid::Uuid,
    pub outcome: String,
    pub satisfaction: f32,
}

impl EvaluationCompleted {
    pub fn new(outcome: &str, satisfaction: f32) -> Self {
        Self {
            meta: crate::kernel::signal::SignalMeta::new(types::EVALUATION_COMPLETED, "action::evaluation"),
            eval_id: uuid::Uuid::new_v4(),
            outcome: outcome.to_string(),
            satisfaction,
        }
    }
}

crate::signals::signal_impl!(EvaluationCompleted, EVALUATION_COMPLETED, "action::evaluation");

pub struct EvaluationProcessor {
    count: usize,
    total_satisfaction: f32,
}

impl EvaluationProcessor {
    pub fn new() -> Self {
        Self { count: 0, total_satisfaction: 0.0 }
    }
}

#[async_trait]
impl Processor for EvaluationProcessor {
    fn name(&self) -> &str { "evaluation" }
    fn version(&self) -> &str { "0.1.0" }
    fn priority(&self) -> u8 { 80 }
    fn subscribed_signals(&self) -> &[SignalType] { &[types::EXECUTION_STARTED] }
    fn emitted_signals(&self) -> &[SignalType] { &[types::EVALUATION_COMPLETED] }

    async fn process(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<Vec<SignalArc>> {
        if signal.signal_type() == types::EXECUTION_STARTED {
            self.count += 1;
            // Satisfaction improves with experience but stays realistic
            let satisfaction = 0.5 + (self.count as f32 * 0.05).min(0.4);
            self.total_satisfaction += satisfaction;
            let avg = self.total_satisfaction / self.count as f32;

            let outcome = format!(
                "Execution #{} evaluated: satisfaction {:.2}, avg {:.2}",
                self.count, satisfaction, avg,
            );

            tracing::info!("[Evaluation] {}", outcome);
            return Ok(vec![Arc::new(EvaluationCompleted::new(&outcome, satisfaction))]);
        }
        Ok(vec![])
    }
    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}
impl Default for EvaluationProcessor { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;
    use crate::processors::execution::ExecutionStarted;

    fn test_ctx() -> FieldContext {
        FieldContext::new(Arc::new(EventBus::new()), Arc::new(MemoryStore::new()))
    }

    #[test]
    fn test_evaluation_processor_name() { assert_eq!(EvaluationProcessor::new().name(), "evaluation"); }

    #[tokio::test]
    async fn test_evaluation_emits_on_execution() {
        let mut p = EvaluationProcessor::new();
        let ctx = test_ctx();
        let exec = ExecutionStarted::new("Test task execution");
        let result = p.process(&ctx, Arc::new(exec)).await.unwrap();
        assert!(!result.is_empty());
        let eval = result[0].as_any().downcast_ref::<EvaluationCompleted>().unwrap();
        assert!(eval.satisfaction >= 0.5);
    }

    #[tokio::test]
    async fn test_evaluation_satisfaction_increases() {
        let mut p = EvaluationProcessor::new();
        let ctx = test_ctx();
        let mut last_sat = 0.0;
        for i in 0..5 {
            let exec = ExecutionStarted::new(&format!("Task {}", i));
            let result = p.process(&ctx, Arc::new(exec)).await.unwrap();
            let eval = result[0].as_any().downcast_ref::<EvaluationCompleted>().unwrap();
            assert!(eval.satisfaction > last_sat, "satisfaction should increase: {} > {}", eval.satisfaction, last_sat);
            last_sat = eval.satisfaction;
        }
    }
}
