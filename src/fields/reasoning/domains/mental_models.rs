use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A mental model — a simplified representation of how something works.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentalModel {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
