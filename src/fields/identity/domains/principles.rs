use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A guiding principle derived from experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Principle {
    pub id: Uuid,
    pub principle: String,
    pub source: String,
    pub weight: f32,
}
