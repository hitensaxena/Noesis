use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single belief held by the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub id: Uuid,
    pub belief: String,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}
