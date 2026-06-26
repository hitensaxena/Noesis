//! Dedup processor — detects and skips duplicate episode ingestions.
//!
//! Maintains a content-hash set of recently seen episode texts. On each
//! EPISODE_RECORDED signal, computes a normalized hash of the content and
//! checks against previously seen hashes. If a match is found, emits
//! DedupSkipped with the original episode ID so downstream processors can
//! avoid redundant work.
//!
//! The hash set is bounded to prevent unbounded memory growth.
//! Older entries beyond MAX_TRACKED are evicted (oldest first).

use std::collections::VecDeque;
use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, DedupSkipped};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Maximum number of content hashes to track. Beyond this, oldest entries
/// are evicted (FIFO). This bounds memory and means a truly ancient duplicate
/// won't be caught, but that's acceptable for cognitive dedup.
const MAX_TRACKED: usize = 10_000;

/// Simple content normalization: lowercase, collapse whitespace, trim.
fn normalize_content(text: &str) -> String {
    text.to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// A content-hash cache with FIFO eviction.
struct HashCache {
    hashes: VecDeque<u64>,
    set: std::collections::HashSet<u64>,
}

impl HashCache {
    fn new() -> Self {
        Self {
            hashes: VecDeque::with_capacity(MAX_TRACKED),
            set: std::collections::HashSet::new(),
        }
    }

    /// Returns true if the hash was already present.
    fn contains(&self, hash: u64) -> bool {
        self.set.contains(&hash)
    }

    /// Insert a hash. Evicts oldest if at capacity.
    fn insert(&mut self, hash: u64) {
        if self.set.len() >= MAX_TRACKED {
            if let Some(oldest) = self.hashes.pop_front() {
                self.set.remove(&oldest);
            }
        }
        self.hashes.push_back(hash);
        self.set.insert(hash);
    }

    fn len(&self) -> usize {
        self.set.len()
    }
}

/// Detects duplicate episodes via content hashing.
pub struct DedupProcessor {
    hashes: HashCache,
    /// episode_id → normalized-content-hash for recently seen episodes.
    episode_hashes: std::collections::HashMap<uuid::Uuid, u64>,
}

impl DedupProcessor {
    pub fn new() -> Self {
        Self {
            hashes: HashCache::new(),
            episode_hashes: std::collections::HashMap::new(),
        }
    }
}

#[async_trait]
impl Processor for DedupProcessor {
    fn name(&self) -> &str {
        "dedup"
    }

    fn priority(&self) -> u8 {
        30 // Early — runs right after EpisodeProcessor creates the episode
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::DEDUP_SKIPPED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::EPISODE_RECORDED {
            if let Some(ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
                let normalized = normalize_content(&ep.content);
                let hash = fxhash(&normalized);

                if self.hashes.contains(hash) {
                    // Find the original episode ID for this hash
                    let original_id = self.episode_hashes
                        .iter()
                        .find(|(_, h)| **h == hash)
                        .map(|(id, _)| *id)
                        .unwrap_or(ep.episode_id);

                    tracing::info!(
                        "[DedupProcessor] duplicate episode detected: {} (original: {})",
                        ep.episode_id,
                        original_id,
                    );

                    return Ok(vec![Arc::new(DedupSkipped::new(
                        ep.episode_id,
                        original_id,
                        &format!("{:x}", hash),
                    ))]);
                }

                // New content — track it
                self.hashes.insert(hash);
                self.episode_hashes.insert(ep.episode_id, hash);
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!(
            "[DedupProcessor] shutting down with {} unique hashes",
            self.hashes.len(),
        );
        Ok(())
    }
}

impl Default for DedupProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Fast, non-cryptographic hash for content comparison.
fn fxhash(data: &str) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::kernel::signal::Signal;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_dedup_processor_name() {
        let p = DedupProcessor::new();
        assert_eq!(p.name(), "dedup");
    }

    #[test]
    fn test_dedup_subscriptions() {
        let p = DedupProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::EPISODE_RECORDED));
    }

    #[test]
    fn test_normalize_content() {
        assert_eq!(normalize_content("Hello World"), "hello world");
        assert_eq!(normalize_content("  Hello   World  "), "hello world");
        assert_eq!(normalize_content(""), "");
    }

    #[tokio::test]
    async fn test_dedup_new_episode_pass_through() {
        let mut p = DedupProcessor::new();
        let ctx = test_context();

        let ep = EpisodeRecorded::new("A unique experience.", "test", vec![]);
        let result = p.process(&ctx, Arc::new(ep)).await.unwrap();
        assert!(result.is_empty(), "unique episode should pass through");
    }

    #[tokio::test]
    async fn test_dedup_detects_duplicate() {
        let mut p = DedupProcessor::new();
        let ctx = test_context();

        let content = "Same experience, repeated twice.";
        let ep1 = EpisodeRecorded::new(content, "test", vec![]);
        let ep1_id = ep1.episode_id;
        p.process(&ctx, Arc::new(ep1)).await.unwrap();

        let ep2 = EpisodeRecorded::new(content, "test", vec![]);
        let result = p.process(&ctx, Arc::new(ep2)).await.unwrap();
        assert!(!result.is_empty(), "duplicate should emit DedupSkipped");

        let dedup = result[0].as_any().downcast_ref::<DedupSkipped>().unwrap();
        assert_eq!(dedup.original_episode_id, ep1_id, "should reference original");
    }

    #[tokio::test]
    async fn test_dedup_case_insensitive() {
        let mut p = DedupProcessor::new();
        let ctx = test_context();

        let ep1 = EpisodeRecorded::new("Hello World", "test", vec![]);
        p.process(&ctx, Arc::new(ep1)).await.unwrap();

        let ep2 = EpisodeRecorded::new("hello world", "test", vec![]);
        let result = p.process(&ctx, Arc::new(ep2)).await.unwrap();
        assert!(!result.is_empty(), "case-insensitive duplicate should be caught");
    }

    #[tokio::test]
    async fn test_dedup_different_content_no_dedup() {
        let mut p = DedupProcessor::new();
        let ctx = test_context();

        let ep1 = EpisodeRecorded::new("First experience.", "test", vec![]);
        p.process(&ctx, Arc::new(ep1)).await.unwrap();

        let ep2 = EpisodeRecorded::new("Completely different content.", "test", vec![]);
        let result = p.process(&ctx, Arc::new(ep2)).await.unwrap();
        assert!(result.is_empty(), "different content should not dedup");
    }
}
