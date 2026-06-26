use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Synthesized knowledge combining multiple facts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synthesis {
    pub id: Uuid,
    pub topic: String,
    pub content: String,
    pub sources: Vec<String>,
}
