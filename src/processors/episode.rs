use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::eventbus::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, IngestRequest};
use crate::processor::processor::Processor;
use crate::field::context::FieldContext;

/// Converts raw ingest requests into structured EpisodeRecorded signals.
pub struct EpisodeProcessor;

impl EpisodeProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Processor for EpisodeProcessor {
    fn name(&self) -> &str {
        "episode"
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::INGEST_REQUEST]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        if let Some(req) = signal.as_any().downcast_ref::<IngestRequest>() {
            tracing::info!("[EpisodeProcessor] processing ingest: {}", &req.text[..30.min(req.text.len())]);

            // Extract tags from content (simple word-based)
            let tags: Vec<String> = req
                .text
                .split_whitespace()
                .filter(|w| w.len() > 5 && w.starts_with(|c: char| c.is_uppercase()))
                .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase())
                .filter(|w| !w.is_empty())
                .collect();

            let episode = EpisodeRecorded::new(&req.text, &req.source, tags);
            let child_meta = signal.meta().child(types::EPISODE_RECORDED, "episode::processor");

            let mut signal = episode;
            signal.meta = child_meta;

            tracing::debug!(
                "[EpisodeProcessor] emitted EpisodeRecorded {}",
                signal.episode_id
            );

            return Ok(vec![Arc::new(signal)]);
        }

        Ok(vec![])
    }
}
