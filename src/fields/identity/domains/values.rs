use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A value derived from decisions and reflections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Value {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub strength: f32,
}
