//! Task processor — decomposes projects into actionable tasks.
//!
//! On PROJECT_CREATED, generates tasks from the project's milestones.
//! Each milestone becomes a task with priority derived from the project.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
use crate::processors::project::ProjectCreated;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskCreated {
    pub meta: crate::kernel::signal::SignalMeta,
    pub task_id: uuid::Uuid,
    pub description: String,
    pub priority: u8,
}

impl TaskCreated {
    pub fn new(desc: &str, priority: u8) -> Self {
        Self {
            meta: crate::kernel::signal::SignalMeta::new(types::TASK_CREATED, "action::task"),
            task_id: uuid::Uuid::new_v4(),
            description: desc.to_string(),
            priority,
        }
    }
}

crate::signals::signal_impl!(TaskCreated, TASK_CREATED, "action::task");

pub struct TaskProcessor;
impl TaskProcessor {
    pub fn new() -> Self { Self }

    fn generate_tasks(goal: &str, milestone_count: usize) -> Vec<TaskCreated> {
        let base_priority = if milestone_count >= 4 { 7 } else { 4 };
        let milestones = match milestone_count {
            1 => vec!["Complete the goal"],
            2 => vec!["Phase 1: Foundation", "Phase 2: Completion"],
            3 => vec!["Phase 1: Research", "Phase 2: Implementation", "Phase 3: Review"],
            _ => vec!["Phase 1: Planning", "Phase 2: Development", "Phase 3: Testing", "Phase 4: Deployment", "Phase 5: Monitoring"],
        };

        milestones
            .into_iter()
            .enumerate()
            .map(|(i, m)| TaskCreated::new(
                &format!("{}: {}", m, goal),
                base_priority + (i as u8),
            ))
            .collect()
    }
}

#[async_trait]
impl Processor for TaskProcessor {
    fn name(&self) -> &str { "task" }
    fn version(&self) -> &str { "0.1.0" }
    fn priority(&self) -> u8 { 60 }
    fn subscribed_signals(&self) -> &[SignalType] { &[types::PROJECT_CREATED] }
    fn emitted_signals(&self) -> &[SignalType] { &[types::TASK_CREATED] }

    async fn process(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<Vec<SignalArc>> {
        if signal.signal_type() == types::PROJECT_CREATED {
            if let Some(proj) = signal.as_any().downcast_ref::<ProjectCreated>() {
                let tasks = Self::generate_tasks(&proj.goal, proj.milestone_count);
                tracing::info!("[Task] created {} tasks for project '{}'", tasks.len(), proj.name);
                return Ok(tasks.into_iter().map(|t| Arc::new(t) as SignalArc).collect());
            }
        }
        Ok(vec![])
    }
    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}
impl Default for TaskProcessor { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;

    fn test_ctx() -> FieldContext {
        FieldContext::new(Arc::new(EventBus::new()), Arc::new(MemoryStore::new()))
    }

    #[test]
    fn test_task_generation_count() {
        let tasks = TaskProcessor::generate_tasks("Test goal", 5);
        assert_eq!(tasks.len(), 5);
    }

    #[test]
    fn test_task_generation_small() {
        let tasks = TaskProcessor::generate_tasks("Small goal", 2);
        assert_eq!(tasks.len(), 2);
    }

    #[tokio::test]
    async fn test_task_emits_on_project() {
        let mut p = TaskProcessor::new();
        let ctx = test_ctx();
        let proj = ProjectCreated::new("test-proj", "Build a feature", 3);
        let result = p.process(&ctx, Arc::new(proj)).await.unwrap();
        assert_eq!(result.len(), 3, "3 milestones → 3 tasks");
    }
}
