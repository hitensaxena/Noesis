use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An entry in the identity evolution timeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub id: Uuid,
    pub event: String,
    pub identity_version: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
