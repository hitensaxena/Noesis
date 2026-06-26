//! Decay processor — cognitive memory decay on slow beats.
//!
//! Tracks episode timestamps and, on BEAT_SLOW, computes which episodes
//! have decayed past a recency threshold. Emits MemoryDecayed with the
//! count of episodes that fell below the threshold since the last decay
//! check.
//!
//! Decay means episodes are not removed from the field — the processor
//! tracks a separate internal timestamp map and signals the *observation*
//! that some memories have gone cold, which can trigger consolidation,
//! re-consolidation, or pruning downstream.

use std::collections::BTreeMap;
use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, MemoryDecayed};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// How old an episode must be (in hours) to be considered decayed.
const DECAY_THRESHOLD_HOURS: i64 = 48;

/// How many old episodes must accumulate before we emit a decay signal.
/// Prevents emitting on every BEAT_SLOW when nothing has changed.
const EMIT_THRESHOLD: usize = 1;

/// Tracks episode timestamps and emits decay signals on slow beats.
pub struct DecayProcessor {
    /// episode_id → timestamp of the last BEAT_SLOW cycle that saw it
    timestamps: BTreeMap<uuid::Uuid, DateTime<Utc>>,
    /// Number of episodes that fell below threshold on the last check
    decayed_since_last_check: usize,
}

impl DecayProcessor {
    pub fn new() -> Self {
        Self {
            timestamps: BTreeMap::new(),
            decayed_since_last_check: 0,
        }
    }

    /// Run the decay check: count episodes older than DECAY_THRESHOLD_HOURS.
    fn check_decay(&mut self) -> usize {
        let cutoff = Utc::now() - Duration::hours(DECAY_THRESHOLD_HOURS);
        let _total_before = self.timestamps.len();

        // Retain only episodes newer than the cutoff.
        let old_count = self.timestamps.iter().filter(|(_, ts)| **ts < cutoff).count();

        if old_count > 0 {
            self.timestamps.retain(|_, ts| *ts >= cutoff);
        }

        let total_after = self.timestamps.len();
        tracing::debug!(
            "[DecayProcessor] decay check: {} removed ({} remaining)",
            old_count,
            total_after,
        );
        old_count
    }
}

#[async_trait]
impl Processor for DecayProcessor {
    fn name(&self) -> &str {
        "decay"
    }

    fn priority(&self) -> u8 {
        250 // Late in the chain — runs after buffered processors have fired
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED, types::BEAT_SLOW]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::MEMORY_DECAYED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::EPISODE_RECORDED {
            // Track the timestamp of each new episode.
            if let Some(ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
                self.timestamps.insert(ep.episode_id, ep.timestamp);
                tracing::trace!(
                    "[DecayProcessor] tracking episode {} at {}",
                    ep.episode_id,
                    ep.timestamp,
                );
            }
            return Ok(vec![]);
        }

        if signal_type == types::BEAT_SLOW {
            let removed = self.check_decay();
            self.decayed_since_last_check += removed;

            if self.decayed_since_last_check >= EMIT_THRESHOLD {
                let total_before = self.timestamps.len() + self.decayed_since_last_check;
                let total_after = self.timestamps.len();
                let emitted = self.decayed_since_last_check;

                self.decayed_since_last_check = 0;

                tracing::info!(
                    "[DecayProcessor] emitting MemoryDecayed: {} episodes decayed",
                    emitted,
                );

                return Ok(vec![Arc::new(MemoryDecayed::new(
                    emitted,
                    total_before,
                    total_after,
                ))]);
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!(
            "[DecayProcessor] shutting down with {} tracked episodes",
            self.timestamps.len(),
        );
        Ok(())
    }
}

impl Default for DecayProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::signals::{EpisodeRecorded, IngestRequest};
    use crate::kernel::signal::Signal;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;
    use crate::kernel::beat_coordinator::BeatPulse;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_decay_processor_name() {
        let p = DecayProcessor::new();
        assert_eq!(p.name(), "decay");
    }

    #[test]
    fn test_decay_subscriptions() {
        let p = DecayProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::EPISODE_RECORDED));
        assert!(subs.contains(&types::BEAT_SLOW));
    }

    #[tokio::test]
    async fn test_decay_tracks_episodes() {
        let mut p = DecayProcessor::new();
        let ctx = test_context();

        // Feed three episodes
        for i in 0..3 {
            let ingest = IngestRequest::new(&format!("Test episode {}", i), "test");
            let ep = EpisodeRecorded::new(&ingest.text, &ingest.source, vec![]);
            let result = p.process(&ctx, Arc::new(ep)).await.unwrap();
            assert!(result.is_empty(), "episodes should be tracked silently");
        }

        assert_eq!(p.timestamps.len(), 3, "should track 3 episodes");
    }

    #[tokio::test]
    async fn test_decay_does_not_emit_on_young_episodes() {
        let mut p = DecayProcessor::new();
        let ctx = test_context();

        // Add a very recent episode
        let ep = EpisodeRecorded::new("Fresh episode", "test", vec![]);
        p.process(&ctx, Arc::new(ep)).await.unwrap();

        // BEAT_SLOW — episode is fresh, nothing should decay
        let beat = BeatPulse::new(types::BEAT_SLOW);
        let result = p.process(&ctx, Arc::new(beat)).await.unwrap();
        assert!(result.is_empty(), "fresh episodes should not trigger decay");
    }

    #[tokio::test]
    async fn test_decay_processor_emits_on_old_episodes() {
        let mut p = DecayProcessor::new();
        let ctx = test_context();

        // Manually insert an episode with a very old timestamp
        let old_id = uuid::Uuid::new_v4();
        let old_ts = Utc::now() - Duration::hours(100); // well past 48h threshold
        p.timestamps.insert(old_id, old_ts);

        let decayed = p.check_decay();
        assert_eq!(decayed, 1, "old episode should be counted as decayed");
    }

    #[tokio::test]
    async fn test_decay_no_duplicate_emission() {
        let mut p = DecayProcessor::new();
        let ctx = test_context();

        // Insert old episode
        let old_id = uuid::Uuid::new_v4();
        p.timestamps.insert(old_id, Utc::now() - Duration::hours(100));

        // First BEAT_SLOW should emit
        let beat1 = BeatPulse::new(types::BEAT_SLOW);
        let result1 = p.process(&ctx, Arc::new(beat1)).await.unwrap();
        assert!(!result1.is_empty(), "first beat should emit");
        let sig = result1[0].signal_type();
        assert_eq!(sig, types::MEMORY_DECAYED);

        // Second BEAT_SLOW — no more old episodes, should NOT emit
        let beat2 = BeatPulse::new(types::BEAT_SLOW);
        let result2 = p.process(&ctx, Arc::new(beat2)).await.unwrap();
        assert!(result2.is_empty(), "second beat should not emit again");
    }
}
