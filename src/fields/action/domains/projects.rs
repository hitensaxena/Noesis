use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A project tracked by the Action field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub description: String,
}
