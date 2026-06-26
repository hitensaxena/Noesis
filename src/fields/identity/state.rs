use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A single belief held by the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub id: Uuid,
    pub belief: String,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

/// A detected personality trait.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trait {
    pub id: Uuid,
    pub name: String,
    pub strength: f32,
}

/// State of the Identity Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityFieldState {
    pub beliefs: Vec<Belief>,
    pub traits: Vec<Trait>,
    pub identity_version: u32,
}
