use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A priority item — what the system should focus on and how important it is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityItem {
    pub id: Uuid,
    pub goal_id: Uuid,
    pub rank: u32,
    pub label: String,
}
