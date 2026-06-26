//! Execution processor — tracks task execution state.
//!
//! On TASK_CREATED, starts tracking the task and emits ExecutionStarted.
//! Maintains a list of running tasks for monitoring.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
use crate::processors::task::TaskCreated;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionStarted {
    pub meta: crate::kernel::signal::SignalMeta,
    pub execution_id: uuid::Uuid,
    pub task: String,
}

impl ExecutionStarted {
    pub fn new(task: &str) -> Self {
        Self {
            meta: crate::kernel::signal::SignalMeta::new(types::EXECUTION_STARTED, "action::execution"),
            execution_id: uuid::Uuid::new_v4(),
            task: task.to_string(),
        }
    }
}

crate::signals::signal_impl!(ExecutionStarted, EXECUTION_STARTED, "action::execution");

pub struct ExecutionProcessor {
    running_tasks: Vec<String>,
    completed_count: usize,
}

impl ExecutionProcessor {
    pub fn new() -> Self {
        Self { running_tasks: Vec::new(), completed_count: 0 }
    }
}

#[async_trait]
impl Processor for ExecutionProcessor {
    fn name(&self) -> &str { "execution" }
    fn version(&self) -> &str { "0.1.0" }
    fn priority(&self) -> u8 { 70 }
    fn subscribed_signals(&self) -> &[SignalType] { &[types::TASK_CREATED] }
    fn emitted_signals(&self) -> &[SignalType] { &[types::EXECUTION_STARTED] }

    async fn process(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<Vec<SignalArc>> {
        if signal.signal_type() == types::TASK_CREATED {
            if let Some(task) = signal.as_any().downcast_ref::<TaskCreated>() {
                self.running_tasks.push(format!("{} (p{})", task.description, task.priority));
                self.completed_count += 1;
                tracing::info!("[Execution] started task '{}' (running: {})", task.description, self.running_tasks.len());
                return Ok(vec![Arc::new(ExecutionStarted::new(&task.description))]);
            }
        }
        Ok(vec![])
    }
    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("[Execution] shutdown with {} running tasks", self.running_tasks.len());
        Ok(())
    }
}
impl Default for ExecutionProcessor { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;
    use crate::processors::task::TaskCreated;

    fn test_ctx() -> FieldContext {
        FieldContext::new(Arc::new(EventBus::new()), Arc::new(MemoryStore::new()))
    }

    #[test]
    fn test_execution_processor_name() { assert_eq!(ExecutionProcessor::new().name(), "execution"); }

    #[tokio::test]
    async fn test_execution_emits_on_task() {
        let mut p = ExecutionProcessor::new();
        let ctx = test_ctx();
        let task = TaskCreated::new("Implement feature X", 5);
        let result = p.process(&ctx, Arc::new(task)).await.unwrap();
        assert!(!result.is_empty());
        let exec = result[0].as_any().downcast_ref::<ExecutionStarted>().unwrap();
        assert!(exec.task.contains("Implement feature X"));
    }

    #[tokio::test]
    async fn test_execution_tracks_running_tasks() {
        let mut p = ExecutionProcessor::new();
        let ctx = test_ctx();
        for i in 0..3 {
            let task = TaskCreated::new(&format!("Task {}", i), 5);
            p.process(&ctx, Arc::new(task)).await.unwrap();
        }
        assert_eq!(p.running_tasks.len(), 3);
        assert_eq!(p.completed_count, 3);
    }
}
