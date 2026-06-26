//! Risk processor — assesses risk from plans and decisions.
//!
//! On PLANNING_PLAN_READY, evaluates the plan for potential risks based
//! on the number of steps and goal complexity. Emits RiskAssessed with
//! probability and impact scores.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
use crate::processors::planning::PlanReady;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RiskAssessed {
    pub meta: crate::kernel::signal::SignalMeta,
    pub risk_id: uuid::Uuid,
    pub description: String,
    pub probability: f32,
    pub impact: f32,
}

impl RiskAssessed {
    pub fn new(desc: &str, probability: f32, impact: f32) -> Self {
        Self {
            meta: crate::kernel::signal::SignalMeta::new(types::RISK_ASSESSED, "action::risk"),
            risk_id: uuid::Uuid::new_v4(),
            description: desc.to_string(),
            probability,
            impact,
        }
    }
}

crate::signals::signal_impl!(RiskAssessed, RISK_ASSESSED, "action::risk");

pub struct RiskProcessor;

impl RiskProcessor {
    pub fn new() -> Self { Self }

    fn assess(step_count: usize, goal_description: &str) -> (String, f32, f32) {
        let base_prob = 0.2 + (step_count as f32 * 0.05).min(0.4);
        let base_impact = 0.3 + (step_count as f32 * 0.08).min(0.5);
        let desc = format!(
            "Risk from plan '{}': {} steps may introduce complexity",
            goal_description, step_count,
        );
        (desc, base_prob, base_impact)
    }
}

#[async_trait]
impl Processor for RiskProcessor {
    fn name(&self) -> &str { "risk" }
    fn version(&self) -> &str { "0.1.0" }
    fn priority(&self) -> u8 { 170 }
    fn subscribed_signals(&self) -> &[SignalType] { &[types::PLANNING_PLAN_READY] }
    fn emitted_signals(&self) -> &[SignalType] { &[types::RISK_ASSESSED] }

    async fn process(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<Vec<SignalArc>> {
        if signal.signal_type() == types::PLANNING_PLAN_READY {
            if let Some(plan) = signal.as_any().downcast_ref::<PlanReady>() {
                let (desc, prob, impact) = Self::assess(plan.steps.len(), &plan.goal);
                tracing::info!("[Risk] assessed: {} (prob: {:.2}, impact: {:.2})", desc, prob, impact);
                return Ok(vec![Arc::new(RiskAssessed::new(&desc, prob, impact))]);
            }
        }
        Ok(vec![])
    }
    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}
impl Default for RiskProcessor { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;

    fn test_ctx() -> FieldContext {
        FieldContext::new(Arc::new(EventBus::new()), Arc::new(MemoryStore::new()))
    }

    #[test]
    fn test_risk_processor_name() { assert_eq!(RiskProcessor::new().name(), "risk"); }

    #[test]
    fn test_risk_assessment() {
        let (desc, prob, impact) = RiskProcessor::assess(5, "Build a complex system");
        assert!(desc.contains("complex"), "description should reference the goal");
        assert!(prob > 0.2);
        assert!(impact > 0.3);
    }

    #[tokio::test]
    async fn test_risk_emits_on_plan() {
        let mut p = RiskProcessor::new();
        let ctx = test_ctx();
        let plan = PlanReady::new("Test goal", vec!["step1".into(), "step2".into()], "~1d");
        let result = p.process(&ctx, Arc::new(plan)).await.unwrap();
        assert!(!result.is_empty());
        let risk = result[0].as_any().downcast_ref::<RiskAssessed>().unwrap();
        assert!(risk.probability > 0.0);
    }
}
