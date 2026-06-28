use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An execution record — tracks the running of a task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    pub id: Uuid,
    pub task_id: Uuid,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub result: Option<String>,
}
