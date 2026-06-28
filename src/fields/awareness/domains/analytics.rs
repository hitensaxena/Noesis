use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An analytics snapshot — signal rates, patterns, and derived statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsSnapshot {
    pub signal_rate_per_sec: f64,
    pub total_signals: usize,
    pub signal_type_counts: HashMap<String, usize>,
}
