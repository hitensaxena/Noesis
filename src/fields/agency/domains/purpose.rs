use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A mission or purpose statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MissionStatement {
    pub id: Uuid,
    pub statement: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
