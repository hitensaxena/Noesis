use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, IngestRequest};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::signal::SignalMeta;
    use crate::kernel::signal::SignalType;
    use crate::signals::IngestRequest;
    use crate::signals::EpisodeRecorded;
    use crate::signals::awareness::ObserverTransitionDetected;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_episode_name() {
        let p = EpisodeProcessor::new();
        assert_eq!(p.name(), "episode");
    }

    #[test]
    fn test_episode_subscriptions() {
        let p = EpisodeProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::INGEST_REQUEST));
    }

    #[tokio::test]
    async fn test_episode_processes_ingest() {
        let mut p = EpisodeProcessor::new();
        let ctx = test_context();

        let ingest = IngestRequest::new("A Walk in the Park this Morning", "test");
        let result = p.process(&ctx, Arc::new(ingest)).await.unwrap();
        assert_eq!(result.len(), 1, "should emit EpisodeRecorded");

        let ep = result[0].as_any().downcast_ref::<EpisodeRecorded>().unwrap();
        assert_eq!(ep.content, "A Walk in the Park this Morning");
    }

    #[tokio::test]
    async fn test_episode_ignores_other_signals() {
        let mut p = EpisodeProcessor::new();
        let ctx = test_context();

        let sig = ObserverTransitionDetected::new("test.signal", "test", 1, 0.5, 0.5);
        let result = p.process(&ctx, Arc::new(sig)).await.unwrap();
        assert!(result.is_empty(), "should ignore non-ingest signals");
    }
}
