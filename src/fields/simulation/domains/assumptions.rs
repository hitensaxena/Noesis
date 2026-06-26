use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An assumption the system is operating under.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assumption {
    pub id: Uuid,
    pub assumption: String,
    pub confidence: f32,
    pub is_active: bool,
}
