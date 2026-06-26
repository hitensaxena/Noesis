//! Strategy processor — adjusts strategic direction based on goal outcomes.
//!
//! Tracks completed goals and their outcomes. On BEAT_MEDIUM, evaluates
//! the recent goal completion ratio and emits StrategyUpdated with the
//! current strategic assessment.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::agency::{GoalCompleted, StrategyUpdated};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// How many recent completions to keep in the sliding window.
const WINDOW_SIZE: usize = 20;

/// A recorded goal completion outcome.
#[derive(Debug, Clone)]
struct CompletionRecord {
    #[allow(dead_code)]
    description: String,
    success: bool,
}

/// Tracks goal completion statistics and emits strategy updates.
pub struct StrategyProcessor {
    completions: Vec<CompletionRecord>,
    strategy_version: usize,
}

impl StrategyProcessor {
    pub fn new() -> Self {
        Self {
            completions: Vec::with_capacity(WINDOW_SIZE),
            strategy_version: 0,
        }
    }

    /// Compute a summary of recent performance.
    fn summarize(&self) -> (usize, usize, f32) {
        let total = self.completions.len();
        let successes = self.completions.iter().filter(|c| c.success).count();
        let rate = if total > 0 {
            successes as f32 / total as f32
        } else {
            1.0 // No data yet — assume positive
        };
        (total, successes, rate)
    }
}

#[async_trait]
impl Processor for StrategyProcessor {
    fn name(&self) -> &str {
        "strategy"
    }

    fn priority(&self) -> u8 {
        90
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::GOAL_COMPLETED, types::BEAT_MEDIUM]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::STRATEGY_UPDATED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::GOAL_COMPLETED {
            if let Some(gc) = signal.as_any().downcast_ref::<GoalCompleted>() {
                self.completions.push(CompletionRecord {
                    description: gc.description.clone(),
                    success: gc.success,
                });

                // Keep window bounded
                if self.completions.len() > WINDOW_SIZE {
                    self.completions.remove(0);
                }

                tracing::trace!(
                    "[StrategyProcessor] recorded completion: {} (success: {})",
                    gc.description, gc.success,
                );
            }
            return Ok(vec![]);
        }

        if signal_type == types::BEAT_MEDIUM {
            self.strategy_version += 1;
            let (total, _successes, rate) = self.summarize();

            let description = if total == 0 {
                "Building strategic baseline — no goals completed yet.".to_string()
            } else if rate >= 0.8 {
                format!(
                    "Strong execution — {}% success rate across {} completions. Maintain course.",
                    (rate * 100.0) as u8, total,
                )
            } else if rate >= 0.5 {
                format!(
                    "Moderate performance — {}% success rate across {} completions. Consider adjustments.",
                    (rate * 100.0) as u8, total,
                )
            } else {
                format!(
                    "Needs improvement — {}% success rate across {} completions. Reassess strategy.",
                    (rate * 100.0) as u8, total,
                )
            };

            // Priority inversely correlates with success rate (lower success = higher urgency)
            let priority = if rate < 0.5 { 8 } else if rate < 0.8 { 5 } else { 3 };

            tracing::info!(
                "[StrategyProcessor] strategy update #{}: {} ({}% success)",
                self.strategy_version, description, (rate * 100.0) as u8,
            );

            return Ok(vec![Arc::new(StrategyUpdated::new(&description, priority))]);
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!(
            "[StrategyProcessor] shutting down with {} completed goals tracked",
            self.completions.len(),
        );
        Ok(())
    }
}

impl Default for StrategyProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::signal::Signal;
    use crate::kernel::beat_coordinator::BeatPulse;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_strategy_processor_name() {
        let p = StrategyProcessor::new();
        assert_eq!(p.name(), "strategy");
    }

    #[test]
    fn test_strategy_subscriptions() {
        let p = StrategyProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::GOAL_COMPLETED));
        assert!(subs.contains(&types::BEAT_MEDIUM));
    }

    #[tokio::test]
    async fn test_strategy_tracks_completions() {
        let mut p = StrategyProcessor::new();
        let ctx = test_context();

        let c = GoalCompleted {
            meta: crate::kernel::signal::SignalMeta::new(types::GOAL_COMPLETED, "test"),
            goal_id: uuid::Uuid::new_v4(),
            description: "Test goal".to_string(),
            success: true,
            outcome: "Done".to_string(),
        };
        p.process(&ctx, Arc::new(c)).await.unwrap();

        assert_eq!(p.completions.len(), 1, "should track 1 completion");
    }

    #[tokio::test]
    async fn test_strategy_emits_on_medium_beat() {
        let mut p = StrategyProcessor::new();
        let ctx = test_context();

        // Complete a couple of goals
        for i in 0..3 {
            let c = GoalCompleted {
                meta: crate::kernel::signal::SignalMeta::new(types::GOAL_COMPLETED, "test"),
                goal_id: uuid::Uuid::new_v4(),
                description: format!("Goal {}", i),
                success: true,
                outcome: "Done".to_string(),
            };
            p.process(&ctx, Arc::new(c)).await.unwrap();
        }

        let beat = BeatPulse::new(types::BEAT_MEDIUM);
        let result = p.process(&ctx, Arc::new(beat)).await.unwrap();
        assert!(!result.is_empty(), "BEAT_MEDIUM should emit strategy update");

        let sig = result[0].as_any().downcast_ref::<StrategyUpdated>().unwrap();
        assert!(sig.description.contains("100%"), "should report high success rate");
        assert_eq!(sig.priority, 3, "high success = low priority");
    }

    #[tokio::test]
    async fn test_strategy_reflects_low_success_rate() {
        let mut p = StrategyProcessor::new();
        let ctx = test_context();

        // Mix of successes and failures
        for i in 0..4 {
            let c = GoalCompleted {
                meta: crate::kernel::signal::SignalMeta::new(types::GOAL_COMPLETED, "test"),
                goal_id: uuid::Uuid::new_v4(),
                description: format!("Goal {}", i),
                success: i % 2 == 0, // 2 success, 2 failure = 50%
                outcome: "Done".to_string(),
            };
            p.process(&ctx, Arc::new(c)).await.unwrap();
        }

        let beat = BeatPulse::new(types::BEAT_MEDIUM);
        let result = p.process(&ctx, Arc::new(beat)).await.unwrap();
        let sig = result[0].as_any().downcast_ref::<StrategyUpdated>().unwrap();

        // 50% rate should be moderate -> priority 5
        assert_eq!(sig.priority, 5, "moderate success should have priority 5");
    }
}
