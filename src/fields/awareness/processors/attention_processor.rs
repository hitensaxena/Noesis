//! Attention processor — LLM-powered salience computation.
//!
//! Subscribes to EpisodeRecorded + CuriosityDetected and emits AttentionShifted.
//! Uses Fast LLM tier to compute salience (0-1), falls back to heuristic.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, CuriosityDetected, AttentionShifted};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
use crate::engines::llm::{ModelTier, TieredRouter, CompletionRequest, Message};
use crate::engines::llm::types::LLMError;
use crate::engines::llm::extract::first_json;

/// System prompt for LLM salience computation.
const ATTENTION_SYSTEM: &str = r#"You are an attention engine. Rate the salience/importance of a new experience.

Return a JSON object with:
- topic: A short (1-3 word) label identifying the core topic of this experience
- salience: 0.0-1.0 importance rating (1.0 = extremely important, 0.0 = trivial)
- reason: A brief phrase explaining why this matters

Guidelines for salience:
- 0.9+: Directly relates to core identity, major new insight, or critical change
- 0.6-0.8: Relevant to ongoing work or learning
- 0.3-0.5: Background information, minor update
- 0.0-0.2: Trivial or routine with no new information

Reply ONLY with the JSON object, no other text.
"#;

/// LLM-powered salience computation with heuristic fallback.
pub struct AttentionProcessor {
    last_focus: Option<String>,
    llm: Option<TieredRouter>,
}

impl AttentionProcessor {
    pub fn new() -> Self {
        let llm = if TieredRouter::has_api_key() {
            match TieredRouter::from_env() {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::warn!("[Attention] LLM unavailable: {}", e);
                    None
                }
            }
        } else {
            None
        };
        Self {
            last_focus: None,
            llm,
        }
    }

    /// LLM-powered salience computation.
    async fn llm_compute_salience(&mut self, content: &str) -> Option<(String, f32, String)> {
        let router = self.llm.as_mut()?;

        let request = CompletionRequest::new(
            "attention",
            vec![
                Message::system(ATTENTION_SYSTEM),
                Message::user(&format!(
                    "New experience:\n\"{}\"\n\nCurrent focus: {}",
                    content,
                    self.last_focus.as_deref().unwrap_or("none"),
                )),
            ],
        )
        .with_temperature(0.2)
        .with_max_tokens(256);

        match router.complete(ModelTier::Fast, request).await {
            Ok(resp) => {
                if let Some(val) = first_json(&resp.content) {
                    let topic = val.get("topic").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let salience = val.get("salience").and_then(|s| s.as_f64()).unwrap_or(0.5) as f32;
                    let reason = val.get("reason").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    if !topic.is_empty() {
                        return Some((topic, salience, reason));
                    }
                }
                None
            }
            Err(LLMError::RateLimited { retry_after }) => {
                tracing::warn!("[Attention] rate limited ({}s), using heuristic", retry_after);
                None
            }
            Err(e) => {
                tracing::warn!("[Attention] LLM error: {}, using heuristic", e);
                None
            }
        }
    }
}

#[async_trait]
impl Processor for AttentionProcessor {
    fn name(&self) -> &str {
        "attention"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn priority(&self) -> u8 {
        90 // Runs early in cascade
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
        let (content, is_curiosity) = if let Some(ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
            (ep.content.clone(), false)
        } else if let Some(cd) = signal.as_any().downcast_ref::<CuriosityDetected>() {
            (cd.topic.clone(), true)
        } else {
            return Ok(vec![]);
        };

        // Try LLM salience computation, fall back to heuristic
        let (new_focus, salience, reason) = if self.llm.is_some() && !is_curiosity {
            self.llm_compute_salience(&content).await
                .unwrap_or_else(|| {
                    let topic = content.split_whitespace()
                        .find(|w| w.len() > 3)
                        .map(|w| w.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    (topic, 0.8, "New salient signal arrived".to_string())
                })
        } else if is_curiosity {
            (content.clone(), 0.7, "Curiosity-driven attention shift".to_string())
        } else {
            let topic = content.split_whitespace()
                .find(|w| w.len() > 3)
                .map(|w| w.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            (topic, 0.8, "New salient signal arrived".to_string())
        };

        tracing::debug!(
            "[AttentionProcessor] shifting focus to: {} (salience: {:.2})",
            new_focus, salience
        );

        let shifted = AttentionShifted {
            meta: signal.meta().child(types::ATTENTION_SHIFTED, "attention::processor"),
            previous_focus: self.last_focus.clone(),
            new_focus: new_focus.clone(),
            salience,
            reason,
        };

        self.last_focus = Some(new_focus);
        Ok(vec![Arc::new(shifted)])
    }
}

impl Default for AttentionProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::signal::SignalType;
    use crate::signals::EpisodeRecorded;
    use crate::signals::CuriosityDetected;
    use crate::signals::AttentionShifted;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_attention_name() {
        let p = AttentionProcessor::new();
        assert_eq!(p.name(), "attention");
    }

    #[test]
    fn test_attention_subscriptions() {
        let p = AttentionProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::EPISODE_RECORDED));
        assert!(subs.contains(&types::CURIOSITY_DETECTED));
    }

    #[tokio::test]
    async fn test_attention_processes_episode() {
        let mut p = AttentionProcessor::new();
        let ctx = test_context();

        let ep = EpisodeRecorded::new("A significant insight about Rust patterns", "test", vec![]);
        let result = p.process(&ctx, Arc::new(ep)).await.unwrap();
        // Without LLM, falls back to heuristic — should still emit
        assert_eq!(result.len(), 1, "heuristic should produce attention shift");

        let shift = result[0].as_any().downcast_ref::<AttentionShifted>().unwrap();
        assert_eq!(shift.new_focus, "significant", "should pick first long word as focus");
    }

    #[tokio::test]
    async fn test_attention_curiosity_shift() {
        let mut p = AttentionProcessor::new();
        let ctx = test_context();

        let curiosity = CuriosityDetected::new("machine learning", "Exploring ML concepts", 0.8);
        let result = p.process(&ctx, Arc::new(curiosity)).await.unwrap();
        assert_eq!(result.len(), 1, "curiosity should trigger attention shift");

        let shift = result[0].as_any().downcast_ref::<AttentionShifted>().unwrap();
        assert_eq!(shift.new_focus, "machine learning", "focus should be curiosity topic");
    }
}
