use std::any::Any;
use async_trait::async_trait;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing;

use crate::eventbus::signal::SignalArc;
use crate::field::field::Field;
use crate::field::context::FieldContext;
use crate::signals::types;

/// A goal managed by the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: Uuid,
    pub description: String,
    pub priority: u8,
    pub status: GoalStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalStatus {
    Active,
    Completed,
    Abandoned,
}

/// State of the Executive Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveFieldState {
    pub goals: Vec<Goal>,
    pub active_intentions: Vec<String>,
}

/// The Executive Field — owns goals and active intentions.
pub struct ExecutiveField {
    state: ExecutiveFieldState,
}

impl ExecutiveField {
    pub fn new() -> Self {
        Self {
            state: ExecutiveFieldState {
                goals: Vec::new(),
                active_intentions: Vec::new(),
            },
        }
    }
}

#[async_trait]
impl Field for ExecutiveField {
    fn name(&self) -> &str {
        "executive"
    }

    async fn init(&mut self, _ctx: &FieldContext) -> Result<()> {
        tracing::info!("[ExecutiveField] initialized");
        Ok(())
    }

    async fn handle_signal(&mut self, _ctx: &FieldContext, signal: SignalArc) -> Result<()> {
        if signal.signal_type() == types::GOAL_CREATED {
            tracing::debug!("[ExecutiveField] received GoalCreated signal");
        } else if signal.signal_type() == types::GOAL_COMPLETED {
            tracing::debug!("[ExecutiveField] received GoalCompleted signal");
        }
        Ok(())
    }

    fn state(&self) -> Box<dyn Any> {
        Box::new(self.state.clone())
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!(
            "[ExecutiveField] shutting down with {} goals",
            self.state.goals.len()
        );
        Ok(())
    }
}

impl Default for ExecutiveField {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::field::field::Field;
    use crate::field::context::FieldContext;
    use crate::storage::memory_store::MemoryStore;
    use crate::eventbus::bus::EventBus;
    use crate::signals::GoalCreated;

    #[tokio::test]
    async fn test_executive_field_init() {
        let storage = Arc::new(MemoryStore::new());
        let bus = Arc::new(EventBus::new());
        let ctx = FieldContext::new(bus, storage);

        let mut field = ExecutiveField::new();
        let result = field.init(&ctx).await;
        assert!(result.is_ok());
        assert_eq!(field.name(), "executive");
    }

    #[tokio::test]
    async fn test_executive_field_state() {
        let storage = Arc::new(MemoryStore::new());
        let bus = Arc::new(EventBus::new());
        let ctx = FieldContext::new(bus, storage);

        let mut field = ExecutiveField::new();
        field.init(&ctx).await.unwrap();

        let goal = GoalCreated::new("Complete the project", 1);
        field.handle_signal(&ctx, Arc::new(goal)).await.unwrap();

        // Downcast to access actual state
        let state = field.state();
        let state = state.downcast_ref::<ExecutiveFieldState>();
        assert!(state.is_some(), "state should be ExecutiveFieldState");
        assert_eq!(state.unwrap().goals.len(), 0, "goals field should exist");
    }
}
