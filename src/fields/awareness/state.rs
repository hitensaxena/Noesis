use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// The current focus of the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusItem {
    pub topic: String,
    pub salience: f32,
    pub reason: String,
}

/// A detected curiosity / knowledge gap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuriosityItem {
    pub id: Uuid,
    pub topic: String,
    pub gap_description: String,
    pub intensity: f32,
}

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

/// State of the Awareness Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwarenessFieldState {
    pub focus_stack: Vec<FocusItem>,
    pub salience_map: std::collections::HashMap<String, f32>,
    pub curiosity_items: Vec<CuriosityItem>,
    pub recent_transitions: Vec<TransitionRecord>,
    pub total_transitions: usize,
}
