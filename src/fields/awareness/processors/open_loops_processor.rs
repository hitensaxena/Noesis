//! Open loops processor — tracks unresolved cognitive items.
//!
//! Monitors GOAL_CREATED, CURIOSITY_DETECTED, and GOAL_COMPLETED signals
//! to track open items. On BEAT_MEDIUM, emits OpenLoopsReport with the
//! count of unresolved items and a summary description.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::CuriosityDetected;
use crate::signals::agency::{GoalCreated, GoalCompleted};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// An open loop item tracked by the processor.
#[derive(Debug, Clone)]
struct OpenItem {
    description: String,
    #[allow(dead_code)]
    created_at: usize,   // sequence number
    item_type: String,    // "goal" or "curiosity"
}

/// Global sequence counter for ordering.
static NEXT_SEQ: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn next_seq() -> usize {
    NEXT_SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
}

/// A report of currently open loops.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenLoopsReport {
    pub meta: crate::kernel::signal::SignalMeta,
    pub open_goal_count: usize,
    pub open_curiosity_count: usize,
    pub total_open: usize,
    pub summary: String,
}

impl OpenLoopsReport {
    pub fn new(goals: usize, curiosities: usize, total: usize, summary: &str) -> Self {
        Self {
            meta: crate::kernel::signal::SignalMeta::new(types::CURIOSITY_DETECTED, "awareness::open_loops"),
            open_goal_count: goals,
            open_curiosity_count: curiosities,
            total_open: total,
            summary: summary.to_string(),
        }
    }
}

crate::signals::signal_impl!(OpenLoopsReport, CURIOSITY_DETECTED, "awareness::open_loops");

/// Tracks unresolved cognitive items and reports summaries.
pub struct OpenLoopsProcessor {
    items: Vec<OpenItem>,
}

impl OpenLoopsProcessor {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
        }
    }
}

#[async_trait]
impl Processor for OpenLoopsProcessor {
    fn name(&self) -> &str {
        "open_loops"
    }

    fn priority(&self) -> u8 {
        140
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::CURIOSITY_DETECTED, types::GOAL_CREATED, types::GOAL_COMPLETED, types::BEAT_MEDIUM]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::CURIOSITY_DETECTED] // Reuses curiosity type for open loops report
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::GOAL_CREATED {
            if let Some(gc) = signal.as_any().downcast_ref::<GoalCreated>() {
                self.items.push(OpenItem {
                    description: gc.description.clone(),
                    created_at: next_seq(),
                    item_type: "goal".to_string(),
                });
                tracing::trace!(
                    "[OpenLoops] tracked new goal: {} (total: {})",
                    gc.description, self.items.len(),
                );
            }
            return Ok(vec![]);
        }

        if signal_type == types::GOAL_COMPLETED {
            if let Some(gc) = signal.as_any().downcast_ref::<GoalCompleted>() {
                // Remove matching goal from open items
                let before = self.items.len();
                self.items.retain(|item| {
                    !(item.item_type == "goal" && item.description == gc.description)
                });
                let removed = before - self.items.len();
                if removed > 0 {
                    tracing::trace!(
                        "[OpenLoops] closed goal: {} (open items: {})",
                        gc.description, self.items.len(),
                    );
                }
            }
            return Ok(vec![]);
        }

        if signal_type == types::CURIOSITY_DETECTED {
            // Also track curiosities as open items
            if let Some(_) = signal.as_any().downcast_ref::<CuriosityDetected>() {
                // Skips tracking curiosity loops to avoid double-counting
                // (CuriosityDetected is also the emission type)
            }
            return Ok(vec![]);
        }

        if signal_type == types::BEAT_MEDIUM {
            let goal_count = self.items.iter().filter(|i| i.item_type == "goal").count();
            let curiosity_count = self.items.iter().filter(|i| i.item_type == "curiosity").count();
            let total = self.items.len();

            let summary = if total == 0 {
                "No open loops — all clear.".to_string()
            } else if goal_count > 0 && curiosity_count > 0 {
                format!(
                    "{} open loops: {} active goals, {} unresolved curiosities",
                    total, goal_count, curiosity_count,
                )
            } else if goal_count > 0 {
                format!("{} open loops: {} active goals", total, goal_count)
            } else {
                format!("{} open loops: {} unresolved curiosities", total, curiosity_count)
            };

            tracing::info!("[OpenLoops] report: {}", summary);
            return Ok(vec![Arc::new(OpenLoopsReport::new(
                goal_count, curiosity_count, total, &summary,
            ))]);
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for OpenLoopsProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::beat_coordinator::BeatPulse;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_open_loops_processor_name() {
        let p = OpenLoopsProcessor::new();
        assert_eq!(p.name(), "open_loops");
    }

    #[tokio::test]
    async fn test_open_loops_tracks_goals() {
        let mut p = OpenLoopsProcessor::new();
        let ctx = test_context();

        let g1 = GoalCreated::new("Finish the project", 5);
        let g2 = GoalCreated::new("Go for a run", 3);
        p.process(&ctx, Arc::new(g1)).await.unwrap();
        p.process(&ctx, Arc::new(g2)).await.unwrap();

        assert_eq!(p.items.len(), 2, "should track 2 open items");
    }

    #[tokio::test]
    async fn test_open_loops_emits_report_on_beat() {
        let mut p = OpenLoopsProcessor::new();
        let ctx = test_context();

        let g1 = GoalCreated::new("Finish the project", 5);
        p.process(&ctx, Arc::new(g1)).await.unwrap();

        let beat = BeatPulse::new(types::BEAT_MEDIUM);
        let result = p.process(&ctx, Arc::new(beat)).await.unwrap();
        assert!(!result.is_empty(), "should emit open loops report");

        let report = result[0].as_any().downcast_ref::<OpenLoopsReport>().unwrap();
        assert_eq!(report.open_goal_count, 1, "should report 1 open goal");
        assert!(report.summary.contains("1"), "summary should contain count");
    }

    #[tokio::test]
    async fn test_open_loops_closes_on_completion() {
        let mut p = OpenLoopsProcessor::new();
        let ctx = test_context();

        let g = GoalCreated::new("Temporary goal", 5);
        p.process(&ctx, Arc::new(g)).await.unwrap();

        let completed = GoalCompleted {
            meta: crate::kernel::signal::SignalMeta::new(types::GOAL_COMPLETED, "test"),
            goal_id: uuid::Uuid::new_v4(),
            description: "Temporary goal".to_string(),
            success: true,
            outcome: "Done".to_string(),
        };
        p.process(&ctx, Arc::new(completed)).await.unwrap();

        assert_eq!(p.items.len(), 0, "goal should be removed from open items");
    }
}
