use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A personality profile containing trait scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityProfile {
    pub id: Uuid,
    pub openness: f32,
    pub conscientiousness: f32,
    pub extraversion: f32,
    pub agreeableness: f32,
    pub neuroticism: f32,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}
