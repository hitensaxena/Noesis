use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::eventbus::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, NarrativeGenerated};
use crate::processor::processor::Processor;
use crate::field::context::FieldContext;

/// Builds coherent narratives from sequences of episodes.
pub struct NarrativeProcessor {
    episode_count: usize,
}

impl NarrativeProcessor {
    pub fn new() -> Self {
        Self { episode_count: 0 }
    }
}

#[async_trait]
impl Processor for NarrativeProcessor {
    fn name(&self) -> &str {
        "narrative"
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::NARRATIVE_GENERATED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        if let Some(ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
            self.episode_count += 1;
            tracing::debug!(
                "[NarrativeProcessor] processing episode {} (total: {})",
                &ep.episode_id.to_string()[..8],
                self.episode_count
            );

            // Every 3 episodes, generate a narrative
            if self.episode_count % 3 == 0 {
                let narrative = NarrativeGenerated {
                    meta: signal.meta().child(types::NARRATIVE_GENERATED, "narrative::processor"),
                    narrative_id: uuid::Uuid::new_v4(),
                    title: format!("Chapter {}", self.episode_count / 3),
                    summary: format!(
                        "A sequence of {} episodes has formed a coherent narrative thread",
                        self.episode_count
                    ),
                    episode_count: self.episode_count,
                    themes: vec!["experience".to_string(), "continuity".to_string()],
                };

                tracing::info!("[NarrativeProcessor] generated narrative: {}", narrative.title);
                return Ok(vec![Arc::new(narrative)]);
            }
        }

        Ok(vec![])
    }
}
