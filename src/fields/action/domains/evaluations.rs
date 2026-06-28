use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An outcome evaluation — how well a task or project went.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evaluation {
    pub id: Uuid,
    pub target_id: Uuid,
    pub score: f32,
    pub notes: String,
}
