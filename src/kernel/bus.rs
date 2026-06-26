use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use dashmap::DashMap;
use tokio::sync::broadcast;
use tracing;

use super::signal::{SignalArc, SignalType};
use super::subscription::Subscription;

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
        // Drop signals whose activation has decayed below the minimum threshold.
        // This guarantees cascade convergence via energy dissipation — no matter
        // how many hops a cascade runs, activation eventually reaches zero and
        // the signal is discarded.
        if !signal.meta().is_alive() {
            tracing::trace!(
                "[Bus] dropping dead signal {:?} (activation={:.3})",
                signal.signal_type(),
                signal.meta().activation,
            );
            return;
        }

        let signal_type = signal.signal_type();
        let signal_type_clone = signal_type.clone();

        if let Some(tx) = self.senders.get(&signal_type_clone) {
            let subscriber_count = self.subscriber_count(&signal_type);
            tracing::debug!(
                "[Bus] publishing {:?} (activation={:.3}, subscriber_count={})",
                signal_type,
                signal.meta().activation,
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
    use crate::kernel::signal::{SignalMeta, SignalType, Signal};
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
    async fn test_activation_cutoff() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe_receiver(types::INGEST_REQUEST);

        // A signal with activation above 0.01 should be published
        let meta = SignalMeta::new(types::INGEST_REQUEST, "test").with_activation(0.5);
        let signal = TestSignal { meta };
        bus.publish(Arc::new(signal));

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let received = rx.try_recv();
        assert!(received.is_ok(), "alive signal (activation=0.5) should be published");

        // A signal with activation <= 0.01 should be dropped (dead)
        let dead_meta = SignalMeta::new(types::INGEST_REQUEST, "test").with_activation(0.005);
        let dead_signal = TestSignal { meta: dead_meta };

        // Clear any residual signals
        let _ = rx.try_recv();

        bus.publish(Arc::new(dead_signal));

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let received_dead = rx.try_recv();
        assert!(received_dead.is_err(), "dead signal (activation=0.005) should be dropped");
    }

    #[tokio::test]
    async fn test_child_decays_activation() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe_receiver(types::INGEST_REQUEST);

        // Create a parent signal with activation=1.0, decay=0.5
        let parent = SignalMeta::new(types::INGEST_REQUEST, "parent").with_decay(0.5);
        let child = parent.child(types::INGEST_REQUEST, "child");

        // Child activation should be 1.0 * 0.5 = 0.5
        assert!((child.activation - 0.5).abs() < f32::EPSILON, "child activation should be 0.5");
        assert!(child.depth == 1, "child depth should be 1");
        assert!(child.is_alive(), "child with activation 0.5 should be alive");

        // A grandchild at decay=0.5: 0.5 * 0.5 = 0.25
        let grandchild = child.child(types::INGEST_REQUEST, "grandchild");
        assert!((grandchild.activation - 0.25).abs() < f32::EPSILON, "grandchild activation should be 0.25");

        // Publish the child and verify it routes correctly
        let test_signal = TestSignal { meta: child };
        bus.publish(Arc::new(test_signal));

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let received = rx.try_recv();
        assert!(received.is_ok(), "decayed child signal should still be published");
    }
}
