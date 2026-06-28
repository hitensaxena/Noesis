use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An open loop — a cognitive task awaiting resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenLoop {
    pub id: Uuid,
    pub description: String,
    pub opened_at: chrono::DateTime<chrono::Utc>,
    pub closed_at: Option<chrono::DateTime<chrono::Utc>>,
}
