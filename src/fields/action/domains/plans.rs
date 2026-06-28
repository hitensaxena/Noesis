use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A plan — a structured sequence of steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: Uuid,
    pub project_id: Uuid,
    pub steps: Vec<String>,
    pub status: PlanStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanStatus {
    Draft,
    Active,
    Completed,
    Abandoned,
}
