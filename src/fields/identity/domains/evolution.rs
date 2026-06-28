use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A projected identity state — where the system expects to be.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityProjection {
    pub id: Uuid,
    pub description: String,
    pub target_version: u32,
    pub confidence: f32,
}
