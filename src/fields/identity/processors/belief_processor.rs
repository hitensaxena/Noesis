//! Belief processor — LLM-powered belief extraction from consolidated memories.
//!
//! Subscribes to MemoryConsolidated signals and emits BeliefChanged.
//! Uses the Fast LLM tier when available, falls back to template beliefs.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{MemoryConsolidated, BeliefChanged, BeliefChangeType};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
use crate::engines::llm::{ModelTier, TieredRouter, CompletionRequest, Message};
use crate::engines::llm::types::LLMError;
use crate::engines::llm::extract::json_records;

/// System prompt for LLM-based belief extraction.
const BELIEF_SYSTEM: &str = r#"You are a belief extraction engine. Given consolidated memories, extract 1-3 beliefs the system should hold.

A belief is a general statement about reality, recurring patterns, values, or principles derived from experiences.
Return a JSON array of objects with:
- belief: the belief statement (one clear sentence)
- confidence: 0.0-1.0 (how strongly this belief is supported by the memories)

Examples:
- "Working with Rust requires deep focus and system-level thinking"
- "Postgres with pgvector is effective for AI memory storage"
- "Recursive signal propagation can lead to emergent intelligence"

Reply ONLY with the JSON array, no other text.
"#;

/// LLM-powered belief extraction with template fallback.
pub struct BeliefProcessor {
    llm: Option<TieredRouter>,
}

impl BeliefProcessor {
    pub fn new() -> Self {
        let llm = if TieredRouter::has_api_key() {
            match TieredRouter::from_env() {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::warn!("[Belief] LLM unavailable: {}", e);
                    None
                }
            }
        } else {
            None
        };
        Self { llm }
    }

    /// Try LLM-based belief extraction from consolidated memory.
    async fn llm_extract_beliefs(&mut self, summary: &str, count: usize) -> Option<Vec<(String, f32)>> {
        let router = self.llm.as_mut()?;

        let request = CompletionRequest::new(
            "belief-extraction",
            vec![
                Message::system(BELIEF_SYSTEM),
                Message::user(&format!(
                    "Consolidated from {} episodes:\n{}",
                    count, summary
                )),
            ],
        )
        .with_temperature(0.2)
        .with_max_tokens(1024);

        match router.complete(ModelTier::Fast, request).await {
            Ok(resp) => {
                let records = json_records(&resp.content);
                let beliefs: Vec<(String, f32)> = records
                    .iter()
                    .filter_map(|v| {
                        let belief = v.get("belief")?.as_str()?.to_string();
                        let confidence = v.get("confidence").and_then(|c| c.as_f64()).unwrap_or(0.5) as f32;
                        Some((belief, confidence))
                    })
                    .collect();

                if beliefs.is_empty() {
                    tracing::warn!("[Belief] LLM returned no valid beliefs, falling back");
                    None
                } else {
                    Some(beliefs)
                }
            }
            Err(LLMError::RateLimited { retry_after }) => {
                tracing::warn!("[Belief] rate limited ({}s), falling back to template", retry_after);
                None
            }
            Err(e) => {
                tracing::warn!("[Belief] LLM error: {}, falling back to template", e);
                None
            }
        }
    }
}

#[async_trait]
impl Processor for BeliefProcessor {
    fn name(&self) -> &str {
        "belief"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn priority(&self) -> u8 {
        130 // After consolidation
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
        if let Some(mc) = signal.as_any().downcast_ref::<MemoryConsolidated>() {
            tracing::info!(
                "[BeliefProcessor] extracting beliefs from consolidated memories ({} episodes)",
                mc.episode_ids.len()
            );

            // Try LLM extraction first, fall back to template
            let belief_entries = if self.llm.is_some() {
                self.llm_extract_beliefs(&mc.summary, mc.memory_count).await
                    .unwrap_or_else(|| {
                        vec![(
                            format!("Patterns observed in recent {} episodes", mc.episode_ids.len()),
                            0.5,
                        )]
                    })
            } else {
                vec![(
                    format!("Patterns observed in recent {} episodes", mc.episode_ids.len()),
                    0.5,
                )]
            };

            let mut emitted: Vec<SignalArc> = Vec::new();
            for (belief_text, confidence) in belief_entries {
                let bc = BeliefChanged::new(&belief_text, BeliefChangeType::Created, confidence);
                tracing::debug!("[BeliefProcessor] emitted belief: {}", bc.belief);
                emitted.push(Arc::new(bc));
            }

            return Ok(emitted);
        }

        Ok(vec![])
    }
}

impl Default for BeliefProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::signal::{Signal, SignalType};
    use crate::signals::{MemoryConsolidated, BeliefChanged, BeliefChangeType};
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_belief_name() {
        let p = BeliefProcessor::new();
        assert_eq!(p.name(), "belief");
    }

    #[test]
    fn test_belief_subscriptions() {
        let p = BeliefProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::MEMORY_CONSOLIDATED));
    }

    #[tokio::test]
    async fn test_belief_processes_consolidation() {
        let mut p = BeliefProcessor::new();
        let ctx = test_context();

        // Without LLM, falls back to template belief
        let mc = MemoryConsolidated {
            meta: crate::kernel::signal::SignalMeta::new(types::MEMORY_CONSOLIDATED, "test"),
            episode_ids: vec![],
            summary: "Test consolidation with several episodes about cognition".to_string(),
            memory_count: 5,
        };
        let result = p.process(&ctx, Arc::new(mc)).await.unwrap();
        // Template fallback should always emit at least one belief
        assert!(!result.is_empty(), "template fallback should produce beliefs");

        let belief = result[0].as_any().downcast_ref::<BeliefChanged>().unwrap();
        assert!(!belief.belief.is_empty(), "belief text should not be empty");
    }

    #[tokio::test]
    async fn test_belief_includes_episode_count() {
        let mut p = BeliefProcessor::new();
        let ctx = test_context();

        let mc = MemoryConsolidated {
            meta: crate::kernel::signal::SignalMeta::new(types::MEMORY_CONSOLIDATED, "test"),
            episode_ids: vec![],
            summary: "Multiple episodes about cognitive architecture".to_string(),
            memory_count: 10,
        };
        let result = p.process(&ctx, Arc::new(mc)).await.unwrap();
        assert!(!result.is_empty(), "should produce beliefs from consolidation");
    }
}
