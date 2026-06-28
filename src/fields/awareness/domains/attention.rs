use serde::{Deserialize, Serialize};

/// The current focus of the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusItem {
    pub topic: String,
    pub salience: f32,
    pub reason: String,
}

/// The focus stack — ordered list of what the system is attending to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusStack {
    pub items: Vec<FocusItem>,
    pub depth: usize,
}
