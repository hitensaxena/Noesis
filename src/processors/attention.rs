use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::eventbus::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, CuriosityDetected, AttentionShifted};
use crate::processor::processor::Processor;
use crate::field::context::FieldContext;

/// Computes salience and shifts attention based on incoming signals.
pub struct AttentionProcessor {
    last_focus: Option<String>,
}

impl AttentionProcessor {
    pub fn new() -> Self {
        Self { last_focus: None }
    }
}

#[async_trait]
impl Processor for AttentionProcessor {
    fn name(&self) -> &str {
        "attention"
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED, types::CURIOSITY_DETECTED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::ATTENTION_SHIFTED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let new_focus = if let Some(ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
            // Extract topic from first significant word
            ep.content
                .split_whitespace()
                .find(|w| w.len() > 3)
                .map(|w| w.to_string())
                .unwrap_or_else(|| "unknown".to_string())
        } else if let Some(cd) = signal.as_any().downcast_ref::<CuriosityDetected>() {
            cd.topic.clone()
        } else {
            return Ok(vec![]);
        };

        tracing::debug!(
            "[AttentionProcessor] shifting focus to: {}",
            new_focus
        );

        let shifted = AttentionShifted {
            meta: signal.meta().child(types::ATTENTION_SHIFTED, "attention::processor"),
            previous_focus: self.last_focus.clone(),
            new_focus: new_focus.clone(),
            salience: 0.8,
            reason: "New salient signal arrived".to_string(),
        };

        self.last_focus = Some(new_focus);
        Ok(vec![Arc::new(shifted)])
    }
}
