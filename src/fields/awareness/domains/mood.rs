use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A mood sample — the system's estimated affective state at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodSample {
    pub id: Uuid,
    pub valence: f32,
    pub arousal: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
