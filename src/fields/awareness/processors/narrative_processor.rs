//! Narrative processor — LLM-powered narrative chapter generation.
//!
//! Subscribes to EpisodeRecorded and emits NarrativeGenerated.
//! Uses Agentic LLM tier when available, falls back to template narratives.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, NarrativeGenerated};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
use crate::engines::llm::{ModelTier, TieredRouter, CompletionRequest, Message};
use crate::engines::llm::types::LLMError;
use crate::engines::llm::extract::first_json;

/// System prompt for LLM narrative generation.
const NARRATIVE_SYSTEM: &str = r#"You are a narrative engine composing life chapters. Given the last 3 experiences, compose a narrative chapter.

Return a JSON object with:
- title: A short chapter title (max 5 words, thematic)
- summary: A 2-sentence narrative summarizing the thread connecting these experiences
- themes: An array of 2-3 theme keywords

Reply ONLY with the JSON object, no other text.
"#;

/// LLM-powered narrative generation with template fallback.
pub struct NarrativeProcessor {
    episode_count: usize,
    episode_buffer: Vec<String>,
    llm: Option<TieredRouter>,
}

impl NarrativeProcessor {
    pub fn new() -> Self {
        let llm = if TieredRouter::has_api_key() {
            match TieredRouter::from_env() {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::warn!("[Narrative] LLM unavailable: {}", e);
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

    /// LLM-powered narrative generation.
    async fn llm_generate_narrative(&mut self, episodes: &[String]) -> Option<(String, String, Vec<String>)> {
        let router = self.llm.as_mut()?;
        let text = episodes.iter().map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n");

        let request = CompletionRequest::new(
            "narrative",
            vec![
                Message::system(NARRATIVE_SYSTEM),
                Message::user(&format!("Last 3 experiences:\n{}", text)),
            ],
        )
        .with_temperature(0.4)
        .with_max_tokens(512);

        match router.complete(ModelTier::Agentic, request).await {
            Ok(resp) => {
                if let Some(val) = first_json(&resp.content) {
                    let title = val.get("title").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let summary = val.get("summary").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let themes: Vec<String> = val.get("themes")
                        .and_then(|a| a.as_array())
                        .map(|a| a.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                        .unwrap_or_default();
                    if !title.is_empty() && !summary.is_empty() {
                        return Some((title, summary, themes));
                    }
                }
                None
            }
            Err(LLMError::RateLimited { retry_after }) => {
                tracing::warn!("[Narrative] rate limited ({}s), using template", retry_after);
                None
            }
            Err(e) => {
                tracing::warn!("[Narrative] LLM error: {}, using template", e);
                None
            }
        }
    }
}

#[async_trait]
impl Processor for NarrativeProcessor {
    fn name(&self) -> &str {
        "narrative"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn priority(&self) -> u8 {
        120 // After attention, before consolidation
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED, types::BEAT_SLOW]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::NARRATIVE_GENERATED]
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
            tracing::debug!(
                "[NarrativeProcessor] processing episode {} (total: {})",
                &ep.episode_id.to_string()[..8],
                self.episode_count
            );
            return Ok(vec![]);
        }

        // On slow beat: generate narrative from buffered episodes
        if signal.signal_type() == types::BEAT_SLOW && self.episode_count >= 3 {
            let recent = self.episode_buffer.clone();

            let (title, summary, themes) = if self.llm.is_some() {
                self.llm_generate_narrative(&recent).await
                    .unwrap_or_else(|| {
                        let t = format!("Chapter {}", self.episode_count / 3 + 1);
                        let s = format!(
                            "A sequence of {} episodes has formed a coherent narrative thread",
                            self.episode_count
                        );
                        (t, s, vec!["experience".to_string(), "continuity".to_string()])
                    })
            } else {
                (
                    format!("Chapter {}", self.episode_count / 3 + 1),
                    format!(
                        "A sequence of {} episodes has formed a coherent narrative thread",
                        self.episode_count
                    ),
                    vec!["experience".to_string(), "continuity".to_string()],
                )
            };

            tracing::info!("[NarrativeProcessor] generated narrative: {}", title);

            let narrative = NarrativeGenerated {
                meta: signal.meta().child(types::NARRATIVE_GENERATED, "narrative::processor"),
                narrative_id: uuid::Uuid::new_v4(),
                title,
                summary,
                episode_count: self.episode_count,
                themes,
            };

            return Ok(vec![Arc::new(narrative)]);
        }

        Ok(vec![])
    }
}

impl Default for NarrativeProcessor {
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
    use crate::signals::NarrativeGenerated;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_narrative_name() {
        let p = NarrativeProcessor::new();
        assert_eq!(p.name(), "narrative");
    }

    #[test]
    fn test_narrative_subscriptions() {
        let p = NarrativeProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::EPISODE_RECORDED));
        assert!(subs.contains(&types::BEAT_SLOW));
    }

    #[tokio::test]
    async fn test_narrative_requires_3_episodes() {
        let mut p = NarrativeProcessor::new();
        let ctx = test_context();

        // Send 2 episodes, then beat — should NOT emit
        for _ in 0..2 {
            let ep = EpisodeRecorded::new("A meaningful experience", "test", vec![]);
            let result = p.process(&ctx, Arc::new(ep)).await.unwrap();
            assert!(result.is_empty(), "episodes should buffer");
        }

        let beat = BeatPulse::new(types::BEAT_SLOW);
        let result = p.process(&ctx, Arc::new(beat)).await.unwrap();
        assert!(result.is_empty(), "need >= 3 episodes for narrative");
    }

    #[tokio::test]
    async fn test_narrative_emits_on_3_episodes() {
        let mut p = NarrativeProcessor::new();
        let ctx = test_context();

        for _ in 0..3 {
            let ep = EpisodeRecorded::new("Explored new ideas and concepts today", "test", vec![]);
            let _ = p.process(&ctx, Arc::new(ep)).await.unwrap();
        }

        let beat = BeatPulse::new(types::BEAT_SLOW);
        let result = p.process(&ctx, Arc::new(beat)).await.unwrap();
        assert_eq!(result.len(), 1, "should emit NarrativeGenerated");

        let narrative = result[0].as_any().downcast_ref::<NarrativeGenerated>().unwrap();
        assert!(narrative.title.contains("Chapter"), "template title should be chapter");
        assert_eq!(narrative.episode_count, 3);
    }
}
