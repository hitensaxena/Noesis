//! Shared system state — provides a bridge between the cascade loop
//! and external interfaces (REST API, TUI, Hermes plugin).
//!
//! Fields write their state to the FieldStateCache during signal processing.
//! The REST API reads from this cache when serving /api/memories, /api/graph, etc.

use std::sync::Arc;
use dashmap::DashMap;
use serde_json::Value;
use tokio::sync::Mutex;

/// Cache of field state snapshots, updated periodically by the cascade loop.
///
/// Key: field name (e.g., "memory", "identity", "knowledge_graph")
/// Value: JSON-serialized field state
pub type FieldStateCache = Arc<DashMap<String, Value>>;

/// Creates a new field state cache.
pub fn new_field_cache() -> FieldStateCache {
    Arc::new(DashMap::new())
}

/// System state shared between the kernel and external interfaces.
pub struct SystemState {
    pub fields: FieldStateCache,
    pub last_cascade_time: Mutex<Option<std::time::Instant>>,
    pub total_signals_processed: std::sync::atomic::AtomicU64,
}

impl SystemState {
    pub fn new(field_cache: FieldStateCache) -> Self {
        Self {
            fields: field_cache,
            last_cascade_time: Mutex::new(None),
            total_signals_processed: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Record that a signal was processed and update the cascade timestamp.
    pub fn record_signal(&self) {
        self.total_signals_processed.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// Get the total number of signals processed.
    pub fn signal_count(&self) -> u64 {
        self.total_signals_processed.load(std::sync::atomic::Ordering::Relaxed)
    }
}
