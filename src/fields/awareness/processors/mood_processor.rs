//! MoodProcessor — estimates cognitive mood from signal patterns.
//!
//! Subscribes to ObserverTransitionDetected and tracks ratios of different
//! signal types to estimate the system's cognitive "mood": focused, curious,
//! uncertain, engaged, or reflective. Emits MoodEstimated periodically.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalType, SignalArc};
use crate::signals::types;
use crate::signals::awareness::{ObserverTransitionDetected, MoodEstimated};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Tracks categories of signals over a sliding window.
struct MoodTracker {
    /// Categorized signal counts over the window
    exploration_signals: usize,  // curiosity, ingestion
    focus_signals: usize,        // attention, episode processing
    reflection_signals: usize,   // consolidation, narrative
    uncertainty_signals: usize,  // low-activation signals
    total_window: usize,
    window_size: usize,
}

impl MoodTracker {
    fn new() -> Self {
        Self {
            exploration_signals: 0,
            focus_signals: 0,
            reflection_signals: 0,
            uncertainty_signals: 0,
            total_window: 0,
            window_size: 30,
        }
    }

    fn record(&mut self, signal_type: &str, activation: f32) {
        if signal_type.contains("curiosity") || signal_type.contains("ingest") {
            self.exploration_signals += 1;
        } else if signal_type.contains("attention") || signal_type.contains("episode") {
            self.focus_signals += 1;
        } else if signal_type.contains("consolidation") || signal_type.contains("narrative") {
            self.reflection_signals += 1;
        }
        if activation < 0.3 {
            self.uncertainty_signals += 1;
        }
        self.total_window += 1;

        // Decay older counts when window exceeded
        if self.total_window > self.window_size * 2 {
            self.exploration_signals /= 2;
            self.focus_signals /= 2;
            self.reflection_signals /= 2;
            self.uncertainty_signals /= 2;
            self.total_window = self.window_size;
        }
    }

    fn estimate_mood(&self) -> (&'static str, f32, f32) {
        if self.total_window == 0 {
            return ("reflective", 0.0, 0.0);
        }

        let total = self.total_window as f32;
        let explore_ratio = self.exploration_signals as f32 / total;
        let focus_ratio = self.focus_signals as f32 / total;
        let reflect_ratio = self.reflection_signals as f32 / total;
        let uncertain_ratio = self.uncertainty_signals as f32 / total;

        // Determine dominant mood
        if uncertain_ratio > 0.4 {
            ("uncertain", uncertain_ratio, 0.4)
        } else if explore_ratio > 0.4 {
            ("curious", explore_ratio, 0.6)
        } else if focus_ratio > 0.5 {
            ("focused", focus_ratio, 0.7)
        } else if reflect_ratio > 0.3 {
            ("reflective", reflect_ratio, 0.5)
        } else {
            ("engaged", 0.5, 0.5)
        }
    }
}

/// Estimates cognitive mood from observed signal patterns.
pub struct MoodProcessor {
    tracker: MoodTracker,
    cycle_count: usize,
}

impl MoodProcessor {
    pub fn new() -> Self {
        Self {
            tracker: MoodTracker::new(),
            cycle_count: 0,
        }
    }
}

#[async_trait]
impl Processor for MoodProcessor {
    fn name(&self) -> &str { "mood" }
    fn version(&self) -> &str { "0.1.0" }
    fn priority(&self) -> u8 { 100 }
    fn activation_threshold(&self) -> f32 { 0.1 }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::OBSERVER_TRANSITION_DETECTED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::MOOD_ESTIMATED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        if let Some(obs) = signal.as_any().downcast_ref::<ObserverTransitionDetected>() {
            self.tracker.record(&obs.signal_type, obs.activation);
            self.cycle_count += 1;

            // Estimate mood every 20 transitions
            if self.cycle_count % 20 == 0 {
                let (mood, intensity, confidence) = self.tracker.estimate_mood();
                tracing::debug!("[MoodProcessor] estimated mood: {} ({:.2})", mood, intensity);

                let estimated = MoodEstimated::new(
                    mood, intensity, self.cycle_count, confidence,
                );
                return Ok(vec![Arc::new(estimated)]);
            }
        }
        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> { Ok(()) }
}

impl Default for MoodProcessor {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::signal::SignalType;
    use crate::signals::awareness::{ObserverTransitionDetected, MoodEstimated};
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_mood_name() {
        let p = MoodProcessor::new();
        assert_eq!(p.name(), "mood");
    }

    #[test]
    fn test_mood_subscriptions() {
        let p = MoodProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::OBSERVER_TRANSITION_DETECTED));
    }

    #[tokio::test]
    async fn test_mood_emits_every_20() {
        let mut p = MoodProcessor::new();
        let ctx = test_context();

        for _ in 0..19 {
            let sig = ObserverTransitionDetected::new("episode.recorded", "test", 1, 0.6, 0.5);
            let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
            assert!(result.is_empty(), "no emission before 20th");
        }

        let sig = ObserverTransitionDetected::new("episode.recorded", "test", 1, 0.6, 0.5);
        let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
        assert_eq!(result.len(), 1, "should emit MoodEstimated on 20th");
    }

    #[tokio::test]
    async fn test_mood_estimate_is_curious_when_exploring() {
        let mut p = MoodProcessor::new();
        let ctx = test_context();

        for _ in 0..20 {
            let sig = ObserverTransitionDetected::new("curiosity.detected", "test", 1, 0.6, 0.5);
            let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
            if result.len() == 1 {
                let mood = result[0].as_any().downcast_ref::<MoodEstimated>().unwrap();
                assert!(mood.mood == "curious" || mood.mood == "focused" || mood.mood == "engaged",
                    "mood should be one of: curious, focused, engaged");
            }
        }
    }
}
