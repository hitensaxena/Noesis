use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::eventbus::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{BeliefChanged, IdentityUpdated};
use crate::processor::processor::Processor;
use crate::field::context::FieldContext;

/// Integrates beliefs into the self-model, emitting IdentityUpdated.
pub struct IdentityProcessor {
    belief_count: usize,
}

impl IdentityProcessor {
    pub fn new() -> Self {
        Self { belief_count: 0 }
    }
}

#[async_trait]
impl Processor for IdentityProcessor {
    fn name(&self) -> &str {
        "identity"
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::BELIEF_CHANGED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::IDENTITY_UPDATED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        if let Some(bc) = signal.as_any().downcast_ref::<BeliefChanged>() {
            self.belief_count += 1;
            tracing::info!(
                "[IdentityProcessor] integrating belief: {} (total: {})",
                bc.belief,
                self.belief_count
            );

            let updated = IdentityUpdated {
                meta: signal.meta().child(types::IDENTITY_UPDATED, "identity::processor"),
                identity_version: self.belief_count as u32,
                beliefs_count: self.belief_count,
                traits_count: 0,
                summary: format!("Integrated belief: {}", bc.belief),
            };

            return Ok(vec![Arc::new(updated)]);
        }

        Ok(vec![])
    }
}
