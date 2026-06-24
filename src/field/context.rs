use std::sync::Arc;

use crate::eventbus::bus::EventBus;
use crate::eventbus::signal::SignalArc;
use crate::storage::store::Storage;

/// Context provided to fields and processors during initialization and operation.
pub struct FieldContext {
    pub event_bus: Arc<EventBus>,
    pub storage: Arc<dyn Storage>,
}

impl FieldContext {
    pub fn new(event_bus: Arc<EventBus>, storage: Arc<dyn Storage>) -> Self {
        Self { event_bus, storage }
    }

    /// Emit a signal onto the event bus.
    pub fn emit(&self, signal: SignalArc) {
        self.event_bus.publish(signal);
    }
}
