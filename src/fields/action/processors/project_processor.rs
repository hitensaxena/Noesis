//! Project processor — creates structured projects from goals.
//!
//! On GOAL_CREATED, creates a project with name, goal, and milestone
//! count based on goal priority. Higher-priority goals get more milestones.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::agency::GoalCreated;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectCreated {
    pub meta: crate::kernel::signal::SignalMeta,
    pub project_id: uuid::Uuid,
    pub name: String,
    pub goal: String,
    pub milestone_count: usize,
}

impl ProjectCreated {
    pub fn new(name: &str, goal: &str, milestones: usize) -> Self {
        Self {
            meta: crate::kernel::signal::SignalMeta::new(types::PROJECT_CREATED, "action::project"),
            project_id: uuid::Uuid::new_v4(),
            name: name.to_string(),
            goal: goal.to_string(),
            milestone_count: milestones,
        }
    }
}

crate::signals::signal_impl!(ProjectCreated, PROJECT_CREATED, "action::project");

pub struct ProjectProcessor {
    count: usize,
}

impl ProjectProcessor {
    pub fn new() -> Self { Self { count: 0 } }

    fn create_project_name(description: &str) -> String {
        let words: Vec<&str> = description.split_whitespace().collect();
        if words.len() >= 3 {
            format!("{}-{}-{}", words[0], words[1], words[2])
        } else if !words.is_empty() {
            words.join("-")
        } else {
            "New-Project".to_string()
        }
    }

    fn milestone_count(priority: u8) -> usize {
        match priority {
            0..=3 => 2,
            4..=6 => 3,
            7..=10 => 5,
            _ => 3,
        }
    }
}

#[async_trait]
impl Processor for ProjectProcessor {
    fn name(&self) -> &str { "project" }
    fn version(&self) -> &str { "0.1.0" }
    fn priority(&self) -> u8 { 50 }
    fn subscribed_signals(&self) -> &[SignalType] { &[types::GOAL_CREATED] }
    fn emitted_signals(&self) -> &[SignalType] { &[types::PROJECT_CREATED] }

    async fn process(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<Vec<SignalArc>> {
        if signal.signal_type() == types::GOAL_CREATED {
            if let Some(gc) = signal.as_any().downcast_ref::<GoalCreated>() {
                self.count += 1;
                let name = Self::create_project_name(&gc.description);
                let milestones = Self::milestone_count(gc.priority);
                let p = ProjectCreated::new(&name, &gc.description, milestones);
                tracing::info!("[Project] created '{}' ({} milestones)", name, milestones);
                return Ok(vec![Arc::new(p)]);
            }
        }
        Ok(vec![])
    }
    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}
impl Default for ProjectProcessor { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;

    fn test_ctx() -> FieldContext {
        FieldContext::new(Arc::new(EventBus::new()), Arc::new(MemoryStore::new()))
    }

    #[test]
    fn test_project_name_from_goal() {
        assert_eq!(ProjectProcessor::create_project_name("Build a web app"), "Build-a-web");
    }

    #[test]
    fn test_milestone_count() {
        assert_eq!(ProjectProcessor::milestone_count(1), 2);
        assert_eq!(ProjectProcessor::milestone_count(5), 3);
        assert_eq!(ProjectProcessor::milestone_count(9), 5);
    }

    #[tokio::test]
    async fn test_project_emits_on_goal() {
        let mut p = ProjectProcessor::new();
        let ctx = test_ctx();
        let g = GoalCreated::new("Deploy Noesis to production", 9);
        let result = p.process(&ctx, Arc::new(g)).await.unwrap();
        assert!(!result.is_empty());
        let proj = result[0].as_any().downcast_ref::<ProjectCreated>().unwrap();
        assert!(proj.name.contains("Deploy"));
        assert_eq!(proj.milestone_count, 5);
    }
}
