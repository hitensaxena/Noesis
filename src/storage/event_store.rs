//! Event persistence — durable storage for CloudEvents.
//!
//! Provides an EventStore trait with an in-memory implementation,
//! plus a bridge that automatically persists events published on the EventBus.

use std::sync::atomic::AtomicU64;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::eventbus::cloud_event::CloudEvent;
use crate::eventbus::signal::SignalArc;

/// A stored event record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub seq: u64,
    pub id: String,
    pub event_type: String,
    pub source: String,
    pub subject: String,
    pub time: DateTime<Utc>,
    pub data: serde_json::Value,
    pub actor: String,
    pub scope: serde_json::Value,
}

/// Event persistence abstraction.
///
/// Events are the source of truth. The EventStore records every event
/// durably, and projections (fields, knowledge graph, etc.) consume
/// the event stream to maintain their state.
#[async_trait]
pub trait EventStore: Send + Sync {
    /// Store an event and return its sequence number.
    async fn append(&self, event: CloudEvent) -> u64;

    /// Retrieve an event by its sequence number.
    async fn get(&self, seq: u64) -> Option<StoredEvent>;

    /// Retrieve an event by its CloudEvents ID.
    async fn get_by_id(&self, id: &str) -> Option<StoredEvent>;

    /// List events from a sequence offset, with optional type filter.
    async fn list(
        &self,
        from_seq: u64,
        limit: u64,
        event_type: Option<&str>,
    ) -> Vec<StoredEvent>;

    /// Get events for a specific subject.
    async fn list_by_subject(&self, subject: &str, limit: u64) -> Vec<StoredEvent>;

    /// Get the total number of stored events.
    async fn count(&self) -> u64;

    /// Get the latest sequence number.
    async fn latest_seq(&self) -> u64;

    /// List all unique event types in the store.
    async fn list_types(&self) -> Vec<String>;
}

/// In-memory event store using a Vec protected by a mutex.
pub struct MemoryEventStore {
    events: tokio::sync::Mutex<Vec<StoredEvent>>,
    seq: AtomicU64,
}

impl MemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: tokio::sync::Mutex::new(Vec::new()),
            seq: AtomicU64::new(0),
        }
    }
}

impl Default for MemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventStore for MemoryEventStore {
    async fn append(&self, event: CloudEvent) -> u64 {
        let seq = self.seq.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        let stored = StoredEvent {
            seq,
            id: event.id,
            event_type: event.event_type,
            source: event.source,
            subject: event.subject,
            time: Utc::now(),
            data: event.data,
            actor: event.actor,
            scope: event.scope,
        };
        let mut events = self.events.lock().await;
        events.push(stored);
        seq
    }

    async fn get(&self, seq: u64) -> Option<StoredEvent> {
        let events = self.events.lock().await;
        // seq is 1-based, vec is 0-based
        if seq == 0 || seq > events.len() as u64 {
            return None;
        }
        events.get((seq - 1) as usize).cloned()
    }

    async fn get_by_id(&self, id: &str) -> Option<StoredEvent> {
        let events = self.events.lock().await;
        events.iter().find(|e| e.id == id).cloned()
    }

    async fn list(
        &self,
        from_seq: u64,
        limit: u64,
        event_type: Option<&str>,
    ) -> Vec<StoredEvent> {
        let events = self.events.lock().await;
        events
            .iter()
            .skip((from_seq.saturating_sub(1)) as usize)
            .filter(|e| {
                event_type
                    .map(|t| e.event_type.contains(t))
                    .unwrap_or(true)
            })
            .take(limit as usize)
            .cloned()
            .collect()
    }

    async fn list_by_subject(&self, subject: &str, limit: u64) -> Vec<StoredEvent> {
        let events = self.events.lock().await;
        events
            .iter()
            .filter(|e| e.subject == subject)
            .rev()
            .take(limit as usize)
            .cloned()
            .collect()
    }

    async fn count(&self) -> u64 {
        let events = self.events.lock().await;
        events.len() as u64
    }

    async fn latest_seq(&self) -> u64 {
        self.seq.load(std::sync::atomic::Ordering::SeqCst)
    }

    async fn list_types(&self) -> Vec<String> {
        let events = self.events.lock().await;
        let mut types: Vec<String> = events.iter().map(|e| e.event_type.clone()).collect();
        types.sort();
        types.dedup();
        types
    }
}

/// Bridge that automatically persists signals to the event store.
///
/// Subscribes to all signal types on the EventBus and stores each signal
/// as a CloudEvent in the configured EventStore.
pub struct EventBridge {
    store: tokio::sync::Mutex<Option<std::sync::Arc<dyn EventStore>>>,
}

impl EventBridge {
    pub fn new() -> Self {
        Self {
            store: tokio::sync::Mutex::new(None),
        }
    }

    /// Set the event store and start persisting signals.
    pub async fn set_store(&self, store: std::sync::Arc<dyn EventStore>) {
        let mut s = self.store.lock().await;
        *s = Some(store);
    }

    /// Persist a signal as a CloudEvent.
    pub async fn persist_signal(&self, signal: &SignalArc) {
        let store = self.store.lock().await;
        if let Some(ref store) = *store {
            let event_type = format!("noesis.signal.{}", signal.signal_type());
            let ce = CloudEvent::new(
                &event_type,
                &signal.meta().source,
                serde_json::json!({
                    "depth": signal.meta().depth,
                    "id": signal.meta().id.to_string(),
                }),
                "system",
                "noesis",
                serde_json::json!({"level": "system"}),
            );
            store.append(ce).await;
        }
    }
}

impl Default for EventBridge {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eventbus::cloud_event::CloudEvent;

    fn make_test_event() -> CloudEvent {
        CloudEvent::new(
            "memory.fact.stored",
            "test-subject",
            serde_json::json!({"key": "value"}),
            "test",
            "noesis",
            serde_json::json!({"level": "user", "user_id": "test"}),
        )
    }

    #[tokio::test]
    async fn test_append_and_get() {
        let store = MemoryEventStore::new();
        let event = make_test_event();
        let seq = store.append(event.clone()).await;
        assert_eq!(seq, 1);

        let retrieved = store.get(seq).await.unwrap();
        assert_eq!(retrieved.id, event.id);
        assert_eq!(retrieved.subject, "test-subject");
    }

    #[tokio::test]
    async fn test_list_with_filter() {
        let store = MemoryEventStore::new();

        let e1 = CloudEvent::new(
            "memory.fact.stored", "subj1",
            serde_json::json!({}), "test", "noesis",
            serde_json::json!({}),
        );
        let e2 = CloudEvent::new(
            "safety.kill.triggered", "subj2",
            serde_json::json!({}), "test", "noesis",
            serde_json::json!({}),
        );

        store.append(e1).await;
        store.append(e2).await;

        let all = store.list(1, 10, None).await;
        assert_eq!(all.len(), 2);

        let memory_events = store.list(1, 10, Some("memory")).await;
        assert_eq!(memory_events.len(), 1);
    }

    #[tokio::test]
    async fn test_count() {
        let store = MemoryEventStore::new();
        assert_eq!(store.count().await, 0);

        store.append(make_test_event()).await;
        store.append(make_test_event()).await;
        store.append(make_test_event()).await;

        assert_eq!(store.count().await, 3);
    }
}
