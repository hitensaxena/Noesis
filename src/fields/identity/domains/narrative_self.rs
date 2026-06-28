use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A coherent life narrative — the story the system tells about itself.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeSelf {
    pub id: Uuid,
    pub narrative: String,
    pub chapter: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}
