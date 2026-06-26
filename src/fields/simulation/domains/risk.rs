use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A risk assessment for a plan or scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub id: Uuid,
    pub description: String,
    pub probability: f32,
    pub impact: f32,
    pub mitigation: Option<String>,
}
