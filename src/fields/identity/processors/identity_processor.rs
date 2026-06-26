//! Identity processor — LLM-powered identity self-model integration.
//!
//! Subscribes to BeliefChanged and emits IdentityUpdated.
//! Uses Agentic LLM tier when available, falls back to template identity updates.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{BeliefChanged, IdentityUpdated};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
use crate::engines::llm::{ModelTier, TieredRouter, CompletionRequest, Message};
use crate::engines::llm::types::LLMError;

/// System prompt for LLM identity summary generation.
const IDENTITY_SYSTEM: &str = r#"You are an identity integration engine. Given a new belief being integrated into the system's self-model, generate a brief identity update.

Return a plain text summary (1-2 sentences) describing how this belief shapes the system's identity. Focus on what this belief means for the system's understanding of itself.

No JSON, no labels — just the summary text.
"#;

/// LLM-powered identity integration with template fallback.
pub struct IdentityProcessor {
    belief_count: usize,
    beliefs: Vec<String>,
    llm: Option<TieredRouter>,
}

impl IdentityProcessor {
    pub fn new() -> Self {
        let llm = if TieredRouter::has_api_key() {
            match TieredRouter::from_env() {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::warn!("[Identity] LLM unavailable: {}", e);
                    None
                }
            }
        } else {
            None
        };
        Self {
            belief_count: 0,
            beliefs: Vec::new(),
            llm,
        }
    }

    /// LLM-powered identity summary.
    async fn llm_identity_summary(&mut self, belief: &str) -> Option<String> {
        let router = self.llm.as_mut()?;

        let request = CompletionRequest::new(
            "identity",
            vec![
                Message::system(IDENTITY_SYSTEM),
                Message::user(&format!(
                    "New belief (total beliefs: {}):\n\"{}\"\n\nAll beliefs so far:\n{}",
                    self.belief_count,
                    belief,
                    self.beliefs.join("\n"),
                )),
            ],
        )
        .with_temperature(0.3)
        .with_max_tokens(256);

        match router.complete(ModelTier::Agentic, request).await {
            Ok(resp) => {
                let text = resp.content.trim().to_string();
                if text.is_empty() { None } else { Some(text) }
            }
            Err(LLMError::RateLimited { retry_after }) => {
                tracing::warn!("[Identity] rate limited ({}s), using template", retry_after);
                None
            }
            Err(e) => {
                tracing::warn!("[Identity] LLM error: {}, using template", e);
                None
            }
        }
    }
}

#[async_trait]
impl Processor for IdentityProcessor {
    fn name(&self) -> &str {
        "identity"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn priority(&self) -> u8 {
        140 // After belief processor
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
            self.beliefs.push(bc.belief.clone());

            tracing::info!(
                "[IdentityProcessor] integrating belief: {} (total: {})",
                bc.belief,
                self.belief_count
            );

            let summary = if self.llm.is_some() {
                self.llm_identity_summary(&bc.belief).await
                    .unwrap_or_else(|| format!("Integrated belief: {}", bc.belief))
            } else {
                format!("Integrated belief: {}", bc.belief)
            };

            let updated = IdentityUpdated {
                meta: signal.meta().child(types::IDENTITY_UPDATED, "identity::processor"),
                identity_version: self.belief_count as u32,
                beliefs_count: self.belief_count,
                traits_count: 0,
                summary,
            };

            return Ok(vec![Arc::new(updated)]);
        }

        Ok(vec![])
    }
}

impl Default for IdentityProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::signal::{Signal, SignalMeta, SignalType};
    use crate::signals::{BeliefChanged, BeliefChangeType, IdentityUpdated};
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_identity_name() {
        let p = IdentityProcessor::new();
        assert_eq!(p.name(), "identity");
    }

    #[test]
    fn test_identity_subscriptions() {
        let p = IdentityProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::BELIEF_CHANGED));
    }

    #[tokio::test]
    async fn test_identity_processes_belief() {
        let mut p = IdentityProcessor::new();
        let ctx = test_context();

        // Without LLM, uses template summary
        let bc = BeliefChanged::new("I am capable of complex reasoning", BeliefChangeType::Created, 0.8);
        let result = p.process(&ctx, Arc::new(bc)).await.unwrap();
        assert!(!result.is_empty(), "template fallback should produce identity update");

        let identity = result[0].as_any().downcast_ref::<IdentityUpdated>().unwrap();
        assert_eq!(identity.identity_version, 1, "first belief should be version 1");
        assert!(identity.summary.contains("Integrated belief"), "should contain template prefix");
    }

    #[tokio::test]
    async fn test_identity_tracks_belief_count() {
        let mut p = IdentityProcessor::new();
        let ctx = test_context();

        for i in 0..3 {
            let bc = BeliefChanged::new(
                &format!("Belief number {}", i + 1),
                BeliefChangeType::Created,
                0.7,
            );
            let result = p.process(&ctx, Arc::new(bc)).await.unwrap();
            assert_eq!(result.len(), 1);

            let identity = result[0].as_any().downcast_ref::<IdentityUpdated>().unwrap();
            assert_eq!(identity.beliefs_count, i + 1, "belief count should increment");
        }
    }
}
