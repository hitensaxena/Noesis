use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A task tracked by the Action field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub priority: u8,
}
