use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::kernel::signal::SignalMeta;
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

/// Goal priorities were reordered.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityReordered {
    pub meta: SignalMeta,
    pub priority_id: Uuid,
    pub goal_id: Uuid,
    pub new_priority: u8,
}

impl PriorityReordered {
    pub fn new(goal: Uuid, priority: u8) -> Self {
        Self {
            meta: SignalMeta::new(types::PRIORITY_REORDERED, "agency::priority"),
            priority_id: Uuid::new_v4(),
            goal_id: goal,
            new_priority: priority,
        }
    }
}

signal_impl!(PriorityReordered, PRIORITY_REORDERED, "agency::priority");

/// The system's strategy was updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyUpdated {
    pub meta: SignalMeta,
    pub strategy_id: Uuid,
    pub description: String,
    pub priority: u8,
}

impl StrategyUpdated {
    pub fn new(desc: &str, priority: u8) -> Self {
        Self {
            meta: SignalMeta::new(types::STRATEGY_UPDATED, "agency::strategy"),
            strategy_id: Uuid::new_v4(),
            description: desc.to_string(),
            priority,
        }
    }
}

signal_impl!(StrategyUpdated, STRATEGY_UPDATED, "agency::strategy");

/// An opportunity was detected.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpportunityDetected {
    pub meta: SignalMeta,
    pub opportunity_id: Uuid,
    pub description: String,
    pub potential: f32,
}

impl OpportunityDetected {
    pub fn new(desc: &str, potential: f32) -> Self {
        Self {
            meta: SignalMeta::new(types::OPPORTUNITY_DETECTED, "agency::opportunity"),
            opportunity_id: Uuid::new_v4(),
            description: desc.to_string(),
            potential,
        }
    }
}

signal_impl!(OpportunityDetected, OPPORTUNITY_DETECTED, "agency::opportunity");
