use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A reasoning chain — a sequence of steps from evidence to conclusion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningChain {
    pub id: Uuid,
    pub premises: Vec<String>,
    pub conclusion: String,
    pub confidence: f32,
}
