use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::eventbus::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, MemoryConsolidated, CuriosityDetected};
use crate::processor::processor::Processor;
use crate::field::context::FieldContext;

/// Detects knowledge gaps and triggers curiosity signals.
pub struct CuriosityProcessor {
    episode_count: usize,
}

impl CuriosityProcessor {
    pub fn new() -> Self {
        Self { episode_count: 0 }
    }
}

#[async_trait]
impl Processor for CuriosityProcessor {
    fn name(&self) -> &str {
        "curiosity"
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED, types::MEMORY_CONSOLIDATED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::CURIOSITY_DETECTED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        if let Some(_ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
            self.episode_count += 1;

            // Every 5 episodes, generate a curiosity signal
            if self.episode_count % 5 == 0 {
                let curiosity = CuriosityDetected::new(
                    "unknown patterns",
                    &format!(
                        "After {} episodes, there may be unrecognized patterns in the data",
                        self.episode_count
                    ),
                    0.6,
                );

                tracing::info!(
                    "[CuriosityProcessor] knowledge gap detected: {}",
                    curiosity.gap_description
                );

                return Ok(vec![Arc::new(curiosity)]);
            }
        }

        if let Some(mc) = signal.as_any().downcast_ref::<MemoryConsolidated>() {
            tracing::debug!(
                "[CuriosityProcessor] examining {} consolidated memories for gaps",
                mc.memory_count
            );
        }

        Ok(vec![])
    }
}
