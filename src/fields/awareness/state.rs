use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export domain types for cohesive state access
pub use super::domains::attention::{FocusItem, FocusStack};
pub use super::domains::curiosity::CuriosityItem;
pub use super::domains::observer::TransitionRecord;
pub use super::domains::open_loops::OpenLoop;
pub use super::domains::mood::MoodSample;
pub use super::domains::health::HealthStatus;
pub use super::domains::analytics::AnalyticsSnapshot;

/// State of the Awareness Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwarenessFieldState {
    pub focus_stack: Vec<FocusItem>,
    pub salience_map: HashMap<String, f32>,
    pub curiosity_items: Vec<CuriosityItem>,
    pub recent_transitions: Vec<TransitionRecord>,
    pub total_transitions: usize,
}
