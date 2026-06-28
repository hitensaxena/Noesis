use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Core identity record — who the system believes it is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModel {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub version: u32,
}
