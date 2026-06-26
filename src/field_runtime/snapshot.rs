//! SnapshotManager — periodic field state snapshots for REST API + persistence.
//!
//! In the architecture, this periodically serializes field state so external
//! interfaces (REST, TUI) can observe field state without blocking processors.
//! For now, a thin wrapper around the existing FieldStateCache pattern.

use std::sync::Arc;
use dashmap::DashMap;
use serde_json::Value;

/// Manages periodic snapshots of field state.
pub struct SnapshotManager {
    cache: Arc<DashMap<String, Value>>,
}

impl SnapshotManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
        }
    }

    /// Update the cached snapshot for a field.
    pub fn update(&self, field_name: &str, state: Value) {
        self.cache.insert(field_name.to_string(), state);
    }

    /// Get the current cached snapshot for a field.
    pub fn get(&self, field_name: &str) -> Option<Value> {
        self.cache.get(field_name).map(|e| e.value().clone())
    }

    /// Get all field snapshots.
    pub fn all(&self) -> Vec<(String, Value)> {
        self.cache
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }

    /// Get the underlying cache for injection into API state.
    pub fn cache(&self) -> Arc<DashMap<String, Value>> {
        self.cache.clone()
    }
}

impl Default for SnapshotManager {
    fn default() -> Self {
        Self::new()
    }
}
