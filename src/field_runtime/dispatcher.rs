//! SignalDispatcher — routes incoming signals to the correct field's processors.
//!
//! Routes signals by prefix (e.g. `memory.*` → MemoryField, `identity.*` → IdentityField).
//! This is the bridge between the Kernel's EventBus and the Field Runtime.

use std::sync::Arc;
use anyhow::Result;
use dashmap::DashMap;
use tracing;

use crate::field_runtime::context::FieldContext;
use crate::field_runtime::field::Field;
use crate::kernel::signal::SignalArc;

/// Routes signals to field processors based on signal type prefix.
pub struct SignalDispatcher {
    fields: Arc<DashMap<String, Box<dyn Field + Send>>>,
}

impl SignalDispatcher {
    pub fn new() -> Self {
        Self {
            fields: Arc::new(DashMap::new()),
        }
    }

    /// Register a field for signal dispatch.
    pub fn register_field(&self, name: &str, field: Box<dyn Field + Send>) {
        self.fields.insert(name.to_string(), field);
        tracing::debug!("[SignalDispatcher] registered field: {}", name);
    }

    /// Dispatch a signal to the matching field.
    /// Returns the field name if dispatched, None if no field matched.
    pub async fn dispatch(&self, signal: &SignalArc, ctx: &FieldContext) -> Result<Option<String>> {
        let signal_type = signal.signal_type().to_string();

        // Extract the field prefix (e.g., "memory" from "memory.capture.recorded")
        let prefix = signal_type.split('.').next().unwrap_or("");

        if let Some(mut entry) = self.fields.get_mut(prefix) {
            tracing::trace!(
                "[SignalDispatcher] routing {} to {}",
                signal_type,
                entry.name()
            );
            entry.handle_signal(ctx, signal.clone()).await?;
            Ok(Some(entry.name().to_string()))
        } else {
            tracing::trace!(
                "[SignalDispatcher] no field registered for prefix: {}",
                prefix
            );
            Ok(None)
        }
    }

    /// List all registered field names.
    pub fn field_names(&self) -> Vec<String> {
        self.fields.iter().map(|e| e.key().clone()).collect()
    }

    /// Collect state snapshots from all registered fields.
    /// Returns (field_name, state) pairs for the background snapshot task.
    pub fn snapshot_states(&self) -> Vec<(String, Box<dyn std::any::Any + Send>)> {
        self.fields.iter().map(|entry| {
            let name = entry.key().clone();
            let state = entry.value().state();
            (name, state)
        }).collect()
    }
}

impl Default for SignalDispatcher {
    fn default() -> Self {
        Self::new()
    }
}
