use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::eventbus::signal::SignalMeta;
use crate::signals::types;
use crate::signals::signal_impl;

/// A new goal was created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalCreated {
    pub meta: SignalMeta,
    pub goal_id: Uuid,
    pub description: String,
    pub priority: u8,
    pub deadline: Option<DateTime<Utc>>,
}

impl GoalCreated {
    pub fn new(description: &str, priority: u8) -> Self {
        Self {
            meta: SignalMeta::new(types::GOAL_CREATED, "noesis::signals"),
            goal_id: Uuid::new_v4(),
            description: description.to_string(),
            priority,
            deadline: None,
        }
    }
}

signal_impl!(GoalCreated, GOAL_CREATED, "noesis::signals");

/// A goal was completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalCompleted {
    pub meta: SignalMeta,
    pub goal_id: Uuid,
    pub description: String,
    pub success: bool,
    pub outcome: String,
}

signal_impl!(GoalCompleted, GOAL_COMPLETED, "noesis::signals");

/// A decision was evaluated after its outcome was observed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionEvaluated {
    pub meta: SignalMeta,
    pub decision_id: Uuid,
    pub decision: String,
    pub outcome: String,
    pub satisfaction: f32,
}

signal_impl!(DecisionEvaluated, DECISION_EVALUATED, "noesis::signals");
