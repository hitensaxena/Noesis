use std::sync::Arc;

use crate::kernel::bus::EventBus;
use crate::kernel::capabilities::CapabilityRegistry;
use crate::kernel::metrics::MetricsCollector;
use crate::kernel::signal::SignalArc;
use crate::storage::store::Storage;

/// The processor's local cognitive environment.
///
/// Provides everything a processor needs without exposing kernel internals.
/// Processors operate inside this context — they never touch Tokio, the
/// EventBus directly, or any kernel infrastructure.
#[derive(Clone)]
pub struct FieldContext {
    pub event_bus: Arc<EventBus>,
    pub storage: Arc<dyn Storage>,
    pub metrics: Arc<MetricsCollector>,
    pub capabilities: Arc<CapabilityRegistry>,
    pub field_name: &'static str,
}

impl FieldContext {
    /// Create a new FieldContext with the given event bus and storage.
    /// Other fields (metrics, capabilities, field_name) are set to defaults.
    pub fn new(event_bus: Arc<EventBus>, storage: Arc<dyn Storage>) -> Self {
        Self {
            event_bus,
            storage,
            metrics: Arc::new(MetricsCollector::new()),
            capabilities: Arc::new(CapabilityRegistry::new()),
            field_name: "",
        }
    }

    /// Create a fully-configured FieldContext with all fields set.
    pub fn new_with(
        event_bus: Arc<EventBus>,
        storage: Arc<dyn Storage>,
        metrics: Arc<MetricsCollector>,
        capabilities: Arc<CapabilityRegistry>,
        field_name: &'static str,
    ) -> Self {
        Self {
            event_bus,
            storage,
            metrics,
            capabilities,
            field_name,
        }
    }

    /// Emit a signal onto the event bus.
    pub fn emit(&self, signal: SignalArc) {
        self.event_bus.publish(signal);
    }

    /// Log a message prefixed with the field name.
    pub fn log(&self, msg: &str) {
        if self.field_name.is_empty() {
            tracing::info!("[?] {}", msg);
        } else {
            tracing::info!("[{}] {}", self.field_name, msg);
        }
    }
}
