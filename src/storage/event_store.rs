//! Event persistence — durable storage for CloudEvents.
//!
//! Provides an EventStore trait with an in-memory implementation,
//! plus a bridge that automatically persists events published on the EventBus.

use std::sync::atomic::AtomicU64;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::kernel::cloud_event::CloudEvent;
use crate::kernel::signal::SignalArc;

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

/// File-backed event store using append-only JSONL format.
///
/// Each event is stored as a single JSON line. The file is appended to
/// sequentially. On startup, the store replays the file to populate its
/// in-memory index. This provides crash-safe persistence without requiring
/// Postgres.
pub struct FileEventStore {
    /// Path to the JSONL event log.
    path: std::path::PathBuf,
    /// In-memory event index for fast lookup.
    events: tokio::sync::Mutex<Vec<StoredEvent>>,
    /// File handle for appending (opened once).
    file: tokio::sync::Mutex<std::fs::File>,
    seq: std::sync::atomic::AtomicU64,
    /// Maximum file size before rotation (default 100MB).
    max_size: u64,
}

impl FileEventStore {
    /// Create or open a file-backed event store.
    ///
    /// If the file exists, it is replayed to populate the in-memory index.
    /// If it doesn't exist, a new file is created.
    pub fn new(path: impl Into<std::path::PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.into();
        tracing::info!("[FileEventStore] opening event log: {}", path.display());

        // Create parent directories if needed
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Open file for append (create if not exists)
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .read(true)
            .open(&path)?;
        let file_size = file.metadata()?.len();

        // Parse existing events to initialize in-memory state
        let mut events = Vec::new();
        if file_size > 0 {
            let content = std::fs::read_to_string(&path)?;
            for line in content.lines() {
                if !line.trim().is_empty() {
                    if let Ok(event) = serde_json::from_str::<StoredEvent>(line) {
                        events.push(event);
                    }
                }
            }
        }

        let seq = events.len() as u64;

        Ok(Self {
            path,
            events: tokio::sync::Mutex::new(events),
            file: tokio::sync::Mutex::new(file),
            seq: std::sync::atomic::AtomicU64::new(seq),
            max_size: 100 * 1024 * 1024, // 100 MB default
        })
    }

    /// Set the maximum file size before auto-rotation.
    pub fn with_max_size(mut self, bytes: u64) -> Self {
        self.max_size = bytes;
        self
    }

    /// Check if the file needs rotation and handle it.
    fn check_rotation(&self) -> Result<(), Box<dyn std::error::Error>> {
        let metadata = std::fs::metadata(&self.path)?;
        if metadata.len() >= self.max_size {
            let rotated = self.path.with_extension("jsonl.old");
            std::fs::rename(&self.path, &rotated)?;
            let new_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&self.path)?;
            *self.file.blocking_lock() = new_file;
            tracing::info!("[FileEventStore] rotated log: {} -> {}", self.path.display(), rotated.display());
        }
        Ok(())
    }
}

#[async_trait]
impl EventStore for FileEventStore {
    async fn append(&self, event: CloudEvent) -> u64 {
        let seq = self.seq.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
        let stored = StoredEvent {
            seq,
            id: event.id,
            event_type: event.event_type,
            source: event.source,
            subject: event.subject,
            time: chrono::Utc::now(),
            data: event.data,
            actor: event.actor,
            scope: event.scope,
        };

        // Append to file
        {
            let mut file = self.file.lock().await;
            use std::io::Write;
            let _ = writeln!(file, "{}", serde_json::to_string(&stored).unwrap_or_default());
            let _ = file.flush();
        }

        // Check if file needs rotation after write
        if let Err(e) = self.check_rotation() {
            tracing::warn!("[FileEventStore] rotation check failed: {}", e);
        }

        // Index in memory
        let mut events = self.events.lock().await;
        events.push(stored);
        seq
    }

    async fn get(&self, seq: u64) -> Option<StoredEvent> {
        let events = self.events.lock().await;
        if seq == 0 || seq > events.len() as u64 {
            return None;
        }
        events.get((seq - 1) as usize).cloned()
    }

    async fn get_by_id(&self, id: &str) -> Option<StoredEvent> {
        let events = self.events.lock().await;
        events.iter().find(|e| e.id == id).cloned()
    }

    async fn list(&self, from_seq: u64, limit: u64, event_type: Option<&str>) -> Vec<StoredEvent> {
        let events = self.events.lock().await;
        events.iter()
            .skip((from_seq.saturating_sub(1)) as usize)
            .filter(|e| event_type.map_or(true, |t| e.event_type == t))
            .take(limit as usize)
            .cloned()
            .collect()
    }

    async fn list_by_subject(&self, subject: &str, limit: u64) -> Vec<StoredEvent> {
        let events = self.events.lock().await;
        events.iter()
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

#[cfg(test)]
mod file_tests {
    use super::*;
    use crate::kernel::cloud_event::CloudEvent;

    fn make_event(id: &str, _event_type: &str) -> CloudEvent {
        // Use a valid catalog event type to avoid catalog validation panic.
        // Override the generated ID with the provided one for test assertions.
        let mut evt = CloudEvent::new(
            "memory.episode.recorded", "test-subject",
            serde_json::json!({"key": "value"}),
            "test", "noesis",
            serde_json::json!({"level": "user"}),
        );
        evt.id = id.to_string();
        evt
    }

    fn make_event_no_id() -> CloudEvent {
        CloudEvent::new(
            "memory.episode.recorded", "test-subject",
            serde_json::json!({"key": "value"}),
            "test", "noesis",
            serde_json::json!({"level": "user"}),
        )
    }

    #[tokio::test]
    async fn test_file_store_append_and_read() {
        let tmp = std::env::temp_dir().join(format!("noesis_events_test_{}", std::process::id()));
        let _ = std::fs::remove_file(&tmp);

        let store = FileEventStore::new(&tmp).unwrap();
        let seq = store.append(make_event("evt-1", "test.type")).await;
        assert_eq!(seq, 1);

        let retrieved = store.get(seq).await.unwrap();
        assert_eq!(retrieved.id, "evt-1");

        let _ = std::fs::remove_file(&tmp);
    }

    #[tokio::test]
    async fn test_file_store_survives_restart() {
        let tmp = std::env::temp_dir().join(format!("noesis_events_restart_{}", std::process::id()));
        let _ = std::fs::remove_file(&tmp);

        // Write some events (scope ensures file handle is closed)
        let id_a;
        let id_b;
        {
            let store = FileEventStore::new(&tmp).unwrap();
            id_a = store.append(make_event("evt-a", "type.a")).await;
            id_b = store.append(make_event("evt-b", "type.b")).await;
        }

        // Re-open — should replay events from file
        let store2 = FileEventStore::new(&tmp).unwrap();
        assert_eq!(store2.count().await, 2, "should replay 2 events from file");

        let evt_a = store2.get(id_a).await.unwrap();
        assert_eq!(evt_a.id, "evt-a");

        let _ = std::fs::remove_file(&tmp);
    }

    #[tokio::test]
    async fn test_file_store_list_all() {
        let tmp = std::env::temp_dir().join(format!("noesis_events_list2_{}", std::process::id()));
        let _ = std::fs::remove_file(&tmp);

        let store = FileEventStore::new(&tmp).unwrap();
        store.append(make_event("a", "memory.episode.recorded")).await;
        store.append(make_event("b", "memory.episode.recorded")).await;
        store.append(make_event("c", "memory.episode.recorded")).await;

        let all = store.list(1, 10, None).await;
        assert_eq!(all.len(), 3, "should list all 3 events");
        assert_eq!(all[0].id, "a");

        let _ = std::fs::remove_file(&tmp);
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
    use crate::kernel::cloud_event::CloudEvent;

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
