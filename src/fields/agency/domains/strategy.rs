use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A strategic plan — a high-level approach to achieving goals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Strategy {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub confidence: f32,
}
