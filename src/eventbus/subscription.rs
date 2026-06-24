use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A handle that allows unsubscribing from the event bus.
#[derive(Clone)]
pub struct Subscription {
    pub(crate) active: Arc<AtomicBool>,
    pub(crate) name: String,
}

impl Subscription {
    pub fn new(name: &str) -> Self {
        Self {
            active: Arc::new(AtomicBool::new(true)),
            name: name.to_string(),
        }
    }

    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::SeqCst)
    }

    pub fn unsubscribe(&self) {
        self.active.store(false, Ordering::SeqCst);
        tracing::info!("[Subscription] {} unsubscribed", self.name);
    }
}
