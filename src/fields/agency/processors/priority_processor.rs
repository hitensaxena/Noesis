//! Priority processor — maintains goal priority queue and reorders on beats.
//!
//! Tracks all active goals and their priorities. On BEAT_FAST, re-scores
//! goals by their declared priority field and emits PriorityReordered if
//! the ordering changed since the last check.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::agency::{GoalCreated, GoalCompleted, PriorityReordered};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// A tracked goal with its current priority.
#[derive(Debug, Clone)]
struct TrackedGoal {
    id: uuid::Uuid,
    #[allow(dead_code)]
    description: String,
    priority: u8,
    active: bool,
}

/// Reorders goal priorities on fast beats.
pub struct PriorityProcessor {
    goals: Vec<TrackedGoal>,
    last_order: Vec<uuid::Uuid>,
}

impl PriorityProcessor {
    pub fn new() -> Self {
        Self {
            goals: Vec::new(),
            last_order: Vec::new(),
        }
    }

    /// Re-score all active goals by priority, return the top candidate if order changed.
    fn reorder(&mut self) -> Option<(uuid::Uuid, u8)> {
        let mut active: Vec<&TrackedGoal> = self.goals.iter().filter(|g| g.active).collect();
        if active.is_empty() {
            return None;
        }

        // Sort by priority descending (higher = more urgent), then by age (first created first)
        active.sort_by(|a, b| {
            b.priority.cmp(&a.priority)
                .then_with(|| a.id.to_string().cmp(&b.id.to_string()))
        });

        let top = active[0];
        let current_order: Vec<uuid::Uuid> = active.iter().map(|g| g.id).collect();

        // Only emit if the ordering changed (top goal differs)
        if self.last_order.is_empty() || self.last_order != current_order {
            self.last_order = current_order;
            return Some((top.id, top.priority));
        }

        None
    }

    fn find_goal_mut(&mut self, id: uuid::Uuid) -> Option<&mut TrackedGoal> {
        self.goals.iter_mut().find(|g| g.id == id)
    }
}

#[async_trait]
impl Processor for PriorityProcessor {
    fn name(&self) -> &str {
        "priority"
    }

    fn priority(&self) -> u8 {
        80
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::GOAL_CREATED, types::GOAL_COMPLETED, types::BEAT_FAST]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::PRIORITY_REORDERED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::GOAL_CREATED {
            if let Some(gc) = signal.as_any().downcast_ref::<GoalCreated>() {
                self.goals.push(TrackedGoal {
                    id: gc.goal_id,
                    description: gc.description.clone(),
                    priority: gc.priority,
                    active: true,
                });
                tracing::trace!(
                    "[PriorityProcessor] tracked goal: {} (priority: {})",
                    gc.description, gc.priority,
                );
            }
            return Ok(vec![]);
        }

        if signal_type == types::GOAL_COMPLETED {
            if let Some(gc) = signal.as_any().downcast_ref::<GoalCompleted>() {
                if let Some(goal) = self.find_goal_mut(gc.goal_id) {
                    goal.active = false;
                    tracing::trace!("[PriorityProcessor] goal completed: {}", gc.description);
                }
            }
            return Ok(vec![]);
        }

        if signal_type == types::BEAT_FAST {
            if let Some((goal_id, priority)) = self.reorder() {
                tracing::info!(
                    "[PriorityProcessor] reordered — top goal: {} (priority: {})",
                    goal_id, priority,
                );
                return Ok(vec![Arc::new(PriorityReordered::new(goal_id, priority))]);
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!(
            "[PriorityProcessor] shutting down with {} goals tracked",
            self.goals.len(),
        );
        Ok(())
    }
}

impl Default for PriorityProcessor {
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
    fn test_priority_processor_name() {
        let p = PriorityProcessor::new();
        assert_eq!(p.name(), "priority");
    }

    #[test]
    fn test_priority_subscriptions() {
        let p = PriorityProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::GOAL_CREATED));
        assert!(subs.contains(&types::GOAL_COMPLETED));
        assert!(subs.contains(&types::BEAT_FAST));
    }

    #[tokio::test]
    async fn test_priority_tracks_goals() {
        let mut p = PriorityProcessor::new();
        let ctx = test_context();

        let g1 = GoalCreated::new("Finish the Noesis project", 5);
        let g2 = GoalCreated::new("Go for a walk", 3);

        p.process(&ctx, Arc::new(g1)).await.unwrap();
        p.process(&ctx, Arc::new(g2)).await.unwrap();

        assert_eq!(p.goals.len(), 2, "should track 2 goals");
    }

    #[tokio::test]
    async fn test_priority_reorders_on_beat() {
        let mut p = PriorityProcessor::new();
        let ctx = test_context();

        let g_low = GoalCreated::new("Low priority task", 2);
        let g_high = GoalCreated::new("High priority task", 10);
        let g_high_id = g_high.goal_id;

        p.process(&ctx, Arc::new(g_low)).await.unwrap();
        p.process(&ctx, Arc::new(g_high)).await.unwrap();

        // Beat should emit with the highest priority goal
        let beat = BeatPulse::new(types::BEAT_FAST);
        let result = p.process(&ctx, Arc::new(beat)).await.unwrap();
        assert!(!result.is_empty(), "should emit PriorityReordered");

        let sig = result[0].as_any().downcast_ref::<PriorityReordered>().unwrap();
        assert_eq!(sig.goal_id, g_high_id, "top goal should be high priority");
        assert_eq!(sig.new_priority, 10, "should have priority 10");
    }

    #[tokio::test]
    async fn test_priority_no_emission_on_unchanged_order() {
        let mut p = PriorityProcessor::new();
        let ctx = test_context();

        // One goal, one reorder
        let g = GoalCreated::new("Solo goal", 5);
        p.process(&ctx, Arc::new(g)).await.unwrap();

        let beat1 = BeatPulse::new(types::BEAT_FAST);
        let r1 = p.process(&ctx, Arc::new(beat1)).await.unwrap();
        assert!(!r1.is_empty(), "first beat should emit");

        // Second beat — no change, should not emit
        let beat2 = BeatPulse::new(types::BEAT_FAST);
        let r2 = p.process(&ctx, Arc::new(beat2)).await.unwrap();
        assert!(r2.is_empty(), "second beat should not reorder unchanged order");
    }

    #[tokio::test]
    async fn test_priority_marks_completed() {
        let mut p = PriorityProcessor::new();
        let ctx = test_context();

        let g = GoalCreated::new("A goal to complete", 8);
        let g_id = g.goal_id;
        p.process(&ctx, Arc::new(g)).await.unwrap();

        let completed = GoalCompleted {
            meta: crate::kernel::signal::SignalMeta::new(types::GOAL_COMPLETED, "test"),
            goal_id: g_id,
            description: "A goal to complete".to_string(),
            success: true,
            outcome: "Done".to_string(),
        };
        p.process(&ctx, Arc::new(completed)).await.unwrap();

        assert!(!p.find_goal_mut(g_id).unwrap().active, "goal should be inactive");
    }
}
