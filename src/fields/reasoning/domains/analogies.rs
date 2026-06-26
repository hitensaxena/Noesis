use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An analogy mapping between two domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analogy {
    pub id: Uuid,
    pub source: String,
    pub target: String,
    pub mapping: String,
    pub strength: f32,
}
