use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A concept — a recurring abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: Uuid,
    pub name: String,
    pub definition: String,
    pub related_concepts: Vec<String>,
}
