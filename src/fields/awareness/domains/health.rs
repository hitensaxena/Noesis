use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Health status of the system at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub id: Uuid,
    pub component: String,
    pub status: String,
    pub last_checked: chrono::DateTime<chrono::Utc>,
}
