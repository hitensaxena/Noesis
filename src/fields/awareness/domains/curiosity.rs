use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A detected curiosity / knowledge gap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuriosityItem {
    pub id: Uuid,
    pub topic: String,
    pub gap_description: String,
    pub intensity: f32,
}
