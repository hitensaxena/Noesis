use serde::{Deserialize, Serialize};

/// A recorded state transition from the ObserverProcessor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransitionRecord {
    pub signal_type: String,
    pub source: String,
    pub depth: u32,
    pub activation: f32,
    pub salience: f32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}
