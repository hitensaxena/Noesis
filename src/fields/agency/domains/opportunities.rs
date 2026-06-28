use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An opportunity — a detected possibility worth pursuing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    pub id: Uuid,
    pub description: String,
    pub potential_value: f32,
    pub effort_estimate: f32,
}
