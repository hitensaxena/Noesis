//! Plan processor — decomposes goals into actionable plans with steps.
//!
//! On GOAL_CREATED, analyzes the goal description to generate a structured
//! plan with steps and estimated duration. Higher-priority goals get
//! shorter timeliness estimates.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::agency::GoalCreated;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// A plan with actionable steps.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanReady {
    pub meta: crate::kernel::signal::SignalMeta,
    pub plan_id: uuid::Uuid,
    pub goal: String,
    pub steps: Vec<String>,
    pub estimated_duration: String,
}

impl PlanReady {
    pub fn new(goal: &str, steps: Vec<String>, duration: &str) -> Self {
        Self {
            meta: crate::kernel::signal::SignalMeta::new(types::PLANNING_PLAN_READY, "action::planning"),
            plan_id: uuid::Uuid::new_v4(),
            goal: goal.to_string(),
            steps,
            estimated_duration: duration.to_string(),
        }
    }
}

crate::signals::signal_impl!(PlanReady, PLANNING_PLAN_READY, "action::planning");

/// Decomposes goals into plans.
pub struct PlanDecomposer {
    plan_count: usize,
}

impl PlanDecomposer {
    pub fn new() -> Self {
        Self { plan_count: 0 }
    }

    fn decompose_goal(description: &str, priority: u8) -> (Vec<String>, String) {
        // Generate context-aware steps based on goal content
        let lower = description.to_lowercase();
        let steps = if lower.contains("learn") || lower.contains("study") {
            vec![
                "Research foundational knowledge".to_string(),
                "Practice with hands-on exercises".to_string(),
                "Build a small project".to_string(),
                "Review and refine understanding".to_string(),
            ]
        } else if lower.contains("build") || lower.contains("create") || lower.contains("implement") {
            vec![
                "Define requirements and scope".to_string(),
                "Design architecture".to_string(),
                "Implement core functionality".to_string(),
                "Test and validate".to_string(),
                "Deploy and monitor".to_string(),
            ]
        } else if lower.contains("fix") || lower.contains("repair") {
            vec![
                "Diagnose root cause".to_string(),
                "Identify solution options".to_string(),
                "Apply fix".to_string(),
                "Verify resolution".to_string(),
            ]
        } else {
            vec![
                "Analyze requirements".to_string(),
                "Plan approach".to_string(),
                "Execute plan".to_string(),
                "Review results".to_string(),
            ]
        };

        let duration = match priority {
            0..=3 => "~2 weeks",
            4..=6 => "~1 week",
            7..=10 => "~2 days",
            _ => "~1 week",
        };

        (steps, duration.to_string())
    }
}

#[async_trait]
impl Processor for PlanDecomposer {
    fn name(&self) -> &str { "planning" }
    fn version(&self) -> &str { "0.1.0" }
    fn priority(&self) -> u8 { 160 }
    fn subscribed_signals(&self) -> &[SignalType] { &[types::GOAL_CREATED] }
    fn emitted_signals(&self) -> &[SignalType] { &[types::PLANNING_PLAN_READY] }

    async fn process(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<Vec<SignalArc>> {
        if signal.signal_type() == types::GOAL_CREATED {
            if let Some(gc) = signal.as_any().downcast_ref::<GoalCreated>() {
                self.plan_count += 1;
                let (steps, duration) = Self::decompose_goal(&gc.description, gc.priority);
                let plan = PlanReady::new(&gc.description, steps, &duration);
                tracing::info!("[Plan] plan ready for '{}' ({} steps)", gc.description, plan.steps.len());
                return Ok(vec![Arc::new(plan)]);
            }
        }
        Ok(vec![])
    }
    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}
impl Default for PlanDecomposer { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;

    fn test_ctx() -> FieldContext {
        FieldContext::new(Arc::new(EventBus::new()), Arc::new(MemoryStore::new()))
    }

    #[test]
    fn test_plan_decomposer_name() { assert_eq!(PlanDecomposer::new().name(), "planning"); }

    #[test]
    fn test_decompose_build_goal() {
        let (steps, dur) = PlanDecomposer::decompose_goal("Build a web app", 7);
        assert!(steps.len() >= 4);
        assert!(dur.contains("2 days"));
    }

    #[test]
    fn test_decompose_learn_goal() {
        let (steps, _) = PlanDecomposer::decompose_goal("Learn Rust programming", 5);
        assert!(steps.iter().any(|s| s.contains("Research") || s.contains("Practice")));
    }

    #[tokio::test]
    async fn test_plan_emits_on_goal() {
        let mut p = PlanDecomposer::new();
        let ctx = test_ctx();
        let g = GoalCreated::new("Build a memory system", 8);
        let result = p.process(&ctx, Arc::new(g)).await.unwrap();
        assert!(!result.is_empty());
        let plan = result[0].as_any().downcast_ref::<PlanReady>().unwrap();
        assert_eq!(plan.goal, "Build a memory system");
        assert!(!plan.steps.is_empty());
    }
}
