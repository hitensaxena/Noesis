use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::eventbus::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{MemoryConsolidated, BeliefChanged, BeliefChangeType};
use crate::processor::processor::Processor;
use crate::field::context::FieldContext;

/// Extracts beliefs from consolidated memories.
pub struct BeliefProcessor;

impl BeliefProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Processor for BeliefProcessor {
    fn name(&self) -> &str {
        "belief"
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::MEMORY_CONSOLIDATED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::BELIEF_CHANGED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        if let Some(_mc) = signal.as_any().downcast_ref::<MemoryConsolidated>() {
            tracing::info!("[BeliefProcessor] extracting beliefs from consolidated memories");

            // Simple belief extraction based on memory summary
            let belief = BeliefChanged::new(
                "Patterns observed in recent experiences",
                BeliefChangeType::Created,
                0.5,
            );

            tracing::debug!(
                "[BeliefProcessor] emitted BeliefChanged: {}",
                belief.belief
            );

            return Ok(vec![Arc::new(belief)]);
        }

        Ok(vec![])
    }
}
