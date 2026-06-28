use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A risk assessment for a plan or project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub id: Uuid,
    pub plan_id: Uuid,
    pub risk: String,
    pub likelihood: f32,
    pub impact: f32,
}
