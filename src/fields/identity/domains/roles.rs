use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A role the system identifies with.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub is_active: bool,
}
