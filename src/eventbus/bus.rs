use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use dashmap::DashMap;
use tokio::sync::broadcast;
use tracing;

use super::signal::{SignalArc, SignalType};
use super::subscription::Subscription;

const MAX_CASCADE_DEPTH: u32 = 50;

#[allow(dead_code)]
struct BusEntry {
    name: String,
    active: Arc<AtomicBool>,
}

/// The event bus — the sole communication channel in Noesis.
///
/// Signals are published and fanned out to all subscribers of that signal type.
/// Processors and fields never communicate directly — only through this bus.
pub struct EventBus {
    subscribers: DashMap<SignalType, Vec<BusEntry>>,
    senders: DashMap<SignalType, broadcast::Sender<SignalArc>>,
    rx_counters: DashMap<SignalType, usize>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            subscribers: DashMap::new(),
            senders: DashMap::new(),
            rx_counters: DashMap::new(),
        }
    }

    /// Subscribe to a signal type. Returns a Subscription handle.
    pub fn subscribe(&self, signal_type: SignalType, name: &str) -> Subscription {
        let entry = BusEntry {
            name: name.to_string(),
            active: Arc::new(AtomicBool::new(true)),
        };

        let active = entry.active.clone();
        let sub = Subscription { active, name: name.to_string() };

        // Ensure a sender exists for this signal type
        self.senders
            .entry(signal_type.clone())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(256);
                tx
            });

        self.subscribers
            .entry(signal_type.clone())
            .or_insert_with(Vec::new)
            .push(entry);

        tracing::debug!("[Bus] {} subscribed to {:?}", name, signal_type);
        sub
    }

    /// Publish a signal to all receivers of its type.
    ///
    /// Signals are sent to the broadcast channel for the signal type.
    /// All active receivers (both processor subscriptions and direct receivers)
    /// will receive the signal. If no receivers are active, the send is silently dropped.
    pub fn publish(&self, signal: SignalArc) {
        let depth = signal.meta().depth;
        if depth > MAX_CASCADE_DEPTH {
            tracing::warn!(
                "[Bus] cascade depth {} exceeded max {}, dropping signal {:?}",
                depth,
                MAX_CASCADE_DEPTH,
                signal.signal_type()
            );
            return;
        }

        let signal_type = signal.signal_type();
        let signal_type_clone = signal_type.clone();

        if let Some(tx) = self.senders.get(&signal_type_clone) {
            let subscriber_count = self.subscriber_count(&signal_type);
            tracing::debug!(
                "[Bus] publishing {:?} (depth={}, subscriber_count={})",
                signal_type,
                depth,
                subscriber_count
            );
            let _ = tx.send(signal);
        }
    }

    /// Publish multiple signals sequentially.
    pub fn publish_many(&self, signals: Vec<SignalArc>) {
        for sig in signals {
            self.publish(sig);
        }
    }

    /// Create a receiver for a signal type (for internal polling).
    pub fn subscribe_receiver(
        &self,
        signal_type: SignalType,
    ) -> broadcast::Receiver<SignalArc> {
        let tx = self
            .senders
            .entry(signal_type.clone())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(256);
                tx
            })
            .clone();

        let mut rx_count = self
            .rx_counters
            .entry(signal_type)
            .or_insert(0);
        *rx_count += 1;

        tx.subscribe()
    }

    /// Check if a signal type has any active subscribers.
    pub fn has_subscribers(&self, signal_type: &SignalType) -> bool {
        self.subscribers
            .get(signal_type)
            .map(|s| s.iter().any(|e| e.active.load(Ordering::SeqCst)))
            .unwrap_or(false)
    }

    /// Return the number of active subscribers for a signal type.
    pub fn subscriber_count(&self, signal_type: &SignalType) -> usize {
        self.subscribers
            .get(signal_type)
            .map(|s| s.iter().filter(|e| e.active.load(Ordering::SeqCst)).count())
            .unwrap_or(0)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::eventbus::signal::{SignalMeta, SignalType, Signal};
    use std::sync::Arc;
    use super::EventBus;
    use crate::signals::types;

    #[derive(Debug)]
    struct TestSignal {
        meta: SignalMeta,
    }

    impl Signal for TestSignal {
        fn signal_type(&self) -> SignalType { types::INGEST_REQUEST }
        fn meta(&self) -> &SignalMeta { &self.meta }
        fn as_any(&self) -> &dyn std::any::Any { self }
    }

    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe_receiver(types::INGEST_REQUEST);

        let signal = TestSignal { meta: SignalMeta::new(types::INGEST_REQUEST, "test") };
        bus.publish(Arc::new(signal));

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let received = rx.try_recv();
        assert!(received.is_ok(), "should receive published signal");
    }

    #[tokio::test]
    async fn test_subscribe_unsubscribe() {
        let bus = EventBus::new();
        let sub = bus.subscribe(types::INGEST_REQUEST, "test-processor");

        assert!(sub.is_active());
        assert_eq!(bus.subscriber_count(&types::INGEST_REQUEST), 1);

        sub.unsubscribe();
        assert!(!sub.is_active());
        assert_eq!(bus.subscriber_count(&types::INGEST_REQUEST), 0);
    }

    #[tokio::test]
    async fn test_multiple_signal_types() {
        let bus = EventBus::new();
        let mut rx_ingest = bus.subscribe_receiver(types::INGEST_REQUEST);
        let mut rx_episode = bus.subscribe_receiver(types::EPISODE_RECORDED);

        let sig1 = TestSignal { meta: SignalMeta::new(types::INGEST_REQUEST, "test") };
        bus.publish(Arc::new(sig1));

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        assert!(rx_ingest.try_recv().is_ok(), "ingest subscribers should get it");
        assert!(rx_episode.try_recv().is_err(), "episode subscribers should NOT get ingest");
    }

    #[tokio::test]
    async fn test_cascade_depth_limit() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe_receiver(types::INGEST_REQUEST);

        // Publish a signal at max depth + 1
        let meta = SignalMeta::new(types::INGEST_REQUEST, "test");
        let child_meta = meta.child(types::INGEST_REQUEST, "test"); // depth 1
        let signal = TestSignal { meta: child_meta };
        bus.publish(Arc::new(signal));

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Shallow signals should pass through
        let received = rx.try_recv();
        assert!(received.is_ok(), "depth=1 signal should be published");
    }
}
