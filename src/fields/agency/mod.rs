use std::any::Any;
use async_trait::async_trait;
use anyhow::Result;
use chrono::Utc;
use tracing;

use crate::kernel::signal::SignalArc;
use crate::field_runtime::field::Field;
use crate::field_runtime::context::FieldContext;
use crate::signals::types;
use crate::signals::agency::{GoalCreated, GoalCompleted};

pub mod state;
pub mod domains;
pub mod processors;
pub use state::{AgencyFieldState, Goal, GoalStatus};

pub struct AgencyField {
    state: AgencyFieldState,
}

impl AgencyField {
    pub fn new() -> Self {
        Self {
            state: AgencyFieldState {
                goals: Vec::new(),
                active_pursuits: Vec::new(),
            },
        }
    }
}

#[async_trait]
impl Field for AgencyField {
    fn name(&self) -> &str { "agency" }

    async fn init(&mut self, _ctx: &FieldContext) -> Result<()> {
        tracing::info!("[AgencyField] initialized");
        Ok(())
    }

    async fn handle_signal(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<()> {
        let signal_type = signal.signal_type();

        if signal_type == types::GOAL_CREATED {
            if let Some(gc) = signal.as_any().downcast_ref::<GoalCreated>() {
                let goal = Goal {
                    id: gc.goal_id,
                    description: gc.description.clone(),
                    priority: gc.priority,
                    status: GoalStatus::Active,
                    created_at: Utc::now(),
                    completed_at: None,
                };
                self.state.goals.push(goal);
                tracing::debug!("[AgencyField] stored goal: {} (priority: {})", gc.description, gc.priority);
            }
        } else if signal_type == types::GOAL_COMPLETED {
            if let Some(gc) = signal.as_any().downcast_ref::<GoalCompleted>() {
                for goal in self.state.goals.iter_mut() {
                    if goal.id == gc.goal_id {
                        goal.status = GoalStatus::Completed;
                        goal.completed_at = Some(Utc::now());
                        tracing::debug!("[AgencyField] completed goal: {} (success: {})", gc.description, gc.success);
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn state(&self) -> Box<dyn Any + Send> {
        Box::new(self.state.clone())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("[AgencyField] shutting down with {} goals", self.state.goals.len());
        Ok(())
    }
}

impl Default for AgencyField {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::field_runtime::field::Field;
    use crate::field_runtime::context::FieldContext;
    use crate::storage::memory_store::MemoryStore;
    use crate::kernel::bus::EventBus;
    use crate::signals::GoalCreated;

    #[tokio::test]
    async fn test_agency_field_init() {
        let storage = Arc::new(MemoryStore::new());
        let bus = Arc::new(EventBus::new());
        let ctx = FieldContext::new(bus, storage);
        let mut field = AgencyField::new();
        let result = field.init(&ctx).await;
        assert!(result.is_ok());
        assert_eq!(field.name(), "agency");
    }

    #[tokio::test]
    async fn test_agency_field_stores_goals() {
        let storage = Arc::new(MemoryStore::new());
        let bus = Arc::new(EventBus::new());
        let ctx = FieldContext::new(bus, storage);
        let mut field = AgencyField::new();
        field.init(&ctx).await.unwrap();
        let goal = GoalCreated::new("Complete the project", 1);
        field.handle_signal(&ctx, Arc::new(goal)).await.unwrap();
        let state = field.state();
        let state = state.downcast_ref::<AgencyFieldState>();
        assert!(state.is_some(), "state should be AgencyFieldState");
        assert_eq!(state.unwrap().goals.len(), 1, "should have 1 goal stored");
    }
}
