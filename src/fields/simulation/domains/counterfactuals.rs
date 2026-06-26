use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A what-if counterfactual scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Counterfactual {
    pub id: Uuid,
    pub actual_outcome: String,
    pub alternative: String,
    pub likelihood: f32,
}
