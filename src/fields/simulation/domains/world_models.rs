use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A causal model of some part of the world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldModel {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub confidence: f32,
}
