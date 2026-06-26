use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A formal decision with supporting reasoning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: Uuid,
    pub choice: String,
    pub alternatives: Vec<String>,
    pub reasoning: String,
    pub outcome: Option<String>,
}
