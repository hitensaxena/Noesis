use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A prediction about a future state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Forecast {
    pub id: Uuid,
    pub prediction: String,
    pub probability: f32,
    pub horizon: String,
}
