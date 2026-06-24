use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::eventbus::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{IdentityUpdated, GoalCreated};
use crate::processor::processor::Processor;
use crate::field::context::FieldContext;

/// Manages goal lifecycle — creates goals in response to identity updates.
pub struct GoalProcessor {
    goal_count: usize,
}

impl GoalProcessor {
    pub fn new() -> Self {
        Self { goal_count: 0 }
    }
}

#[async_trait]
impl Processor for GoalProcessor {
    fn name(&self) -> &str {
        "goal"
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::IDENTITY_UPDATED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::GOAL_CREATED, types::GOAL_COMPLETED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        if let Some(iu) = signal.as_any().downcast_ref::<IdentityUpdated>() {
            self.goal_count += 1;
            tracing::info!(
                "[GoalProcessor] identity updated (v{}), considering new goals",
                iu.identity_version
            );

            let goal = GoalCreated::new(
                &format!("Explore implications of identity v{}", iu.identity_version),
                50,
            );

            tracing::debug!("[GoalProcessor] emitted GoalCreated: {}", goal.description);
            return Ok(vec![Arc::new(goal)]);
        }

        Ok(vec![])
    }
}
