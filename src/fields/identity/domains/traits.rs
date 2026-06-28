use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A detected personality trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trait {
    pub id: Uuid,
    pub name: String,
    pub strength: f32,
}
