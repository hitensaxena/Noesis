//! Curiosity processor — LLM-powered knowledge gap detection.
//!
//! Subscribes to EpisodeRecorded + MemoryConsolidated and emits CuriosityDetected.
//! Uses Agentic LLM tier when available, falls back to template curiosity.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, MemoryConsolidated, CuriosityDetected};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
use crate::engines::llm::{ModelTier, TieredRouter, CompletionRequest, Message};
use crate::engines::llm::types::LLMError;
use crate::engines::llm::extract::json_records;

/// System prompt for LLM curiosity/gap detection.
const CURIOSITY_SYSTEM: &str = r#"You are a knowledge gap analyzer. Given recent experiences, identify topics that deserve deeper exploration.

Return a JSON array of objects with:
- topic: The area or subject worth exploring further
- gap_description: What's unknown or unclear about this topic (one sentence)
- intensity: 0.0-1.0 (how interesting/important this gap is)

Focus on:
- Concepts that were mentioned but not explained
- Connections between topics that aren't fully understood
- Questions the experiences raise but don't answer
- Areas where more information would be valuable

Reply ONLY with the JSON array, no other text.
"#;

/// LLM-powered curiosity detection with template fallback.
pub struct CuriosityProcessor {
    episode_count: usize,
    episode_buffer: Vec<String>,
    llm: Option<TieredRouter>,
}

impl CuriosityProcessor {
    pub fn new() -> Self {
        let llm = if TieredRouter::has_api_key() {
            match TieredRouter::from_env() {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::warn!("[Curiosity] LLM unavailable: {}", e);
                    None
                }
            }
        } else {
            None
        };
        Self {
            episode_count: 0,
            episode_buffer: Vec::new(),
            llm,
        }
    }

    /// LLM-powered knowledge gap detection.
    async fn llm_detect_gaps(&mut self) -> Option<Vec<(String, String, f32)>> {
        let router = self.llm.as_mut()?;
        let text = self.episode_buffer.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n");

        let request = CompletionRequest::new(
            "curiosity",
            vec![
                Message::system(CURIOSITY_SYSTEM),
                Message::user(&format!(
                    "Experiences accumulated ({} total):\n{}",
                    self.episode_count, text
                )),
            ],
        )
        .with_temperature(0.4)
        .with_max_tokens(1024);

        match router.complete(ModelTier::Agentic, request).await {
            Ok(resp) => {
                let records = json_records(&resp.content);
                let gaps: Vec<(String, String, f32)> = records.iter().filter_map(|v| {
                    let topic = v.get("topic")?.as_str()?.to_string();
                    let desc = v.get("gap_description").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let intensity = v.get("intensity").and_then(|c| c.as_f64()).unwrap_or(0.5) as f32;
                    Some((topic, desc, intensity))
                }).collect();

                if gaps.is_empty() {
                    tracing::warn!("[Curiosity] LLM returned no gaps, using template");
                    None
                } else {
                    Some(gaps)
                }
            }
            Err(LLMError::RateLimited { retry_after }) => {
                tracing::warn!("[Curiosity] rate limited ({}s), using template", retry_after);
                None
            }
            Err(e) => {
                tracing::warn!("[Curiosity] LLM error: {}, using template", e);
                None
            }
        }
    }
}

#[async_trait]
impl Processor for CuriosityProcessor {
    fn name(&self) -> &str {
        "curiosity"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn priority(&self) -> u8 {
        110 // After attention
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED, types::MEMORY_CONSOLIDATED, types::BEAT_MEDIUM]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::CURIOSITY_DETECTED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        // Buffer episodes as they arrive
        if let Some(ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
            self.episode_count += 1;
            self.episode_buffer.push(ep.content.clone());
            return Ok(vec![]);
        }

        // Track consolidation events
        if let Some(mc) = signal.as_any().downcast_ref::<MemoryConsolidated>() {
            tracing::debug!(
                "[CuriosityProcessor] examining {} consolidated memories for gaps",
                mc.memory_count
            );
            return Ok(vec![]);
        }

        // On medium beat: detect knowledge gaps from buffered episodes
        if signal.signal_type() == types::BEAT_MEDIUM && !self.episode_buffer.is_empty() {
            let gaps = if self.llm.is_some() {
                self.llm_detect_gaps().await
                    .unwrap_or_else(|| {
                        vec![(
                            "unexplored patterns".to_string(),
                            format!("After {} episodes, there may be unrecognized patterns", self.episode_count),
                            0.6,
                        )]
                    })
            } else {
                vec![(
                    "unexplored areas".to_string(),
                    format!("After {} episodes, there may be unrecognized patterns", self.episode_count),
                    0.6,
                )]
            };

            let mut emitted: Vec<SignalArc> = Vec::new();
            for (topic, desc, intensity) in &gaps {
                let curiosity = CuriosityDetected::new(topic, desc, *intensity);
                tracing::info!("[CuriosityProcessor] gap detected: {} ({})", topic, desc);
                emitted.push(Arc::new(curiosity));
            }

            return Ok(emitted);
        }

        Ok(vec![])
    }
}

impl Default for CuriosityProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::beat_coordinator::BeatPulse;
    use crate::kernel::signal::SignalType;
    use crate::signals::EpisodeRecorded;
    use crate::signals::CuriosityDetected;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_curiosity_name() {
        let p = CuriosityProcessor::new();
        assert_eq!(p.name(), "curiosity");
    }

    #[test]
    fn test_curiosity_subscriptions() {
        let p = CuriosityProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::EPISODE_RECORDED));
        assert!(subs.contains(&types::BEAT_MEDIUM));
    }

    #[tokio::test]
    async fn test_curiosity_buffers_and_emits_on_beat() {
        let mut p = CuriosityProcessor::new();
        let ctx = test_context();

        // Buffer an episode
        let ep = EpisodeRecorded::new("Exploring quantum computing concepts", "test", vec![]);
        let result = p.process(&ctx, Arc::new(ep)).await.unwrap();
        assert!(result.is_empty(), "episode should be buffered, not emitted");

        // Beat with buffer — should emit
        let beat = BeatPulse::new(types::BEAT_MEDIUM);
        let result = p.process(&ctx, Arc::new(beat)).await.unwrap();
        assert!(!result.is_empty(), "should emit curiosity on beat after episode");

        let curiosity = result[0].as_any().downcast_ref::<CuriosityDetected>().unwrap();
        assert!(!curiosity.topic.is_empty(), "curiosity should have a topic");
    }

    #[tokio::test]
    async fn test_curiosity_no_beat_without_buffer() {
        let mut p = CuriosityProcessor::new();
        let ctx = test_context();

        // Beat without episodes — no emission
        let beat = BeatPulse::new(types::BEAT_MEDIUM);
        let result = p.process(&ctx, Arc::new(beat)).await.unwrap();
        assert!(result.is_empty(), "no curiosity without episodes");
    }
}
