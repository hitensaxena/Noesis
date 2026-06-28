use serde::{Deserialize, Serialize};

// Re-export domain types for cohesive state access
pub use super::domains::goals::{Goal, GoalStatus};
pub use super::domains::priorities::PriorityItem;
pub use super::domains::strategy::Strategy;
pub use super::domains::opportunities::Opportunity;
pub use super::domains::purpose::MissionStatement;

/// State of the Agency Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgencyFieldState {
    pub goals: Vec<Goal>,
    pub active_pursuits: Vec<String>,
}
