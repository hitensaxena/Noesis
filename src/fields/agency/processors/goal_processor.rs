//! Goal processor — LLM-powered goal creation from identity updates.
//!
//! Subscribes to IdentityUpdated and emits GoalCreated.
//! Uses Fast LLM tier when available, falls back to template goals.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{IdentityUpdated, GoalCreated};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
use crate::engines::llm::{ModelTier, TieredRouter, CompletionRequest, Message};
use crate::engines::llm::types::LLMError;
use crate::engines::llm::extract::first_json;

/// System prompt for LLM goal generation.
const GOAL_SYSTEM: &str = r#"You are a goal-setting engine. Given an identity update describing how the system's self-model is evolving, generate a relevant goal the system should pursue.

Return a JSON object with:
- description: A clear, actionable goal (one sentence)
- priority: 1-100 (100 = highest priority)

The goal should be:
- Meaningful for the system's cognitive development
- Actionable and specific
- Aligned with the identity update

Reply ONLY with the JSON object, no other text.
"#;

/// LLM-powered goal generation with template fallback.
pub struct GoalProcessor {
    goal_count: usize,
    llm: Option<TieredRouter>,
}

impl GoalProcessor {
    pub fn new() -> Self {
        let llm = if TieredRouter::has_api_key() {
            match TieredRouter::from_env() {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::warn!("[Goal] LLM unavailable: {}", e);
                    None
                }
            }
        } else {
            None
        };
        Self {
            goal_count: 0,
            llm,
        }
    }

    /// LLM-powered goal generation.
    async fn llm_generate_goal(&mut self, summary: &str, version: u32) -> Option<(String, u8)> {
        let router = self.llm.as_mut()?;

        let request = CompletionRequest::new(
            "goal",
            vec![
                Message::system(GOAL_SYSTEM),
                Message::user(&format!(
                    "Identity update (v{}, total beliefs: unknown):\n{}",
                    version, summary
                )),
            ],
        )
        .with_temperature(0.3)
        .with_max_tokens(256);

        match router.complete(ModelTier::Fast, request).await {
            Ok(resp) => {
                if let Some(val) = first_json(&resp.content) {
                    let desc = val.get("description").and_then(|s| s.as_str()).unwrap_or("").to_string();
                    let priority = val.get("priority").and_then(|p| p.as_u64()).unwrap_or(50) as u8;
                    if !desc.is_empty() {
                        return Some((desc, priority));
                    }
                }
                None
            }
            Err(LLMError::RateLimited { retry_after }) => {
                tracing::warn!("[Goal] rate limited ({}s), using template", retry_after);
                None
            }
            Err(e) => {
                tracing::warn!("[Goal] LLM error: {}, using template", e);
                None
            }
        }
    }
}

#[async_trait]
impl Processor for GoalProcessor {
    fn name(&self) -> &str {
        "goal"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn priority(&self) -> u8 {
        160 // After identity processor
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::IDENTITY_UPDATED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::GOAL_CREATED, types::GOAL_COMPLETED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        if let Some(iu) = signal.as_any().downcast_ref::<IdentityUpdated>() {
            self.goal_count += 1;
            tracing::info!(
                "[GoalProcessor] identity updated (v{}), considering new goals",
                iu.identity_version
            );

            let (description, priority) = if self.llm.is_some() {
                self.llm_generate_goal(&iu.summary, iu.identity_version).await
                    .unwrap_or_else(|| {
                        (format!("Explore implications of identity v{}", iu.identity_version), 50)
                    })
            } else {
                (format!("Explore implications of identity v{}", iu.identity_version), 50)
            };

            let goal = GoalCreated::new(&description, priority);
            tracing::debug!("[GoalProcessor] emitted GoalCreated: {}", goal.description);
            return Ok(vec![Arc::new(goal)]);
        }

        Ok(vec![])
    }
}

impl Default for GoalProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::signal::{Signal, SignalMeta, SignalType};
    use crate::signals::{IdentityUpdated, GoalCreated};
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_goal_name() {
        let p = GoalProcessor::new();
        assert_eq!(p.name(), "goal");
    }

    #[test]
    fn test_goal_subscriptions() {
        let p = GoalProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::IDENTITY_UPDATED));
    }

    #[tokio::test]
    async fn test_goal_processes_identity_update() {
        let mut p = GoalProcessor::new();
        let ctx = test_context();

        let iu = IdentityUpdated {
            meta: SignalMeta::new(types::IDENTITY_UPDATED, "test"),
            identity_version: 1,
            beliefs_count: 3,
            traits_count: 1,
            summary: "I am becoming better at Rust development".to_string(),
        };
        let result = p.process(&ctx, Arc::new(iu)).await.unwrap();
        assert!(!result.is_empty(), "template fallback should produce goal");

        let goal = result[0].as_any().downcast_ref::<GoalCreated>().unwrap();
        assert!(!goal.description.is_empty(), "goal should have a description");
        assert!(goal.priority > 0, "goal should have a priority");
    }

    #[tokio::test]
    async fn test_goal_increments_count() {
        let mut p = GoalProcessor::new();
        let ctx = test_context();

        for v in 0..3 {
            let iu = IdentityUpdated {
                meta: SignalMeta::new(types::IDENTITY_UPDATED, "test"),
                identity_version: v + 1,
                beliefs_count: 2,
                traits_count: 0,
                summary: format!("Identity version {}", v + 1),
            };
            let result = p.process(&ctx, Arc::new(iu)).await.unwrap();
            assert_eq!(result.len(), 1, "each identity update should produce a goal");

            let goal = result[0].as_any().downcast_ref::<GoalCreated>().unwrap();
            assert!(goal.description.contains(&format!("Explore implications of identity v{}", v + 1)),
                "template description should reference version");
        }

        assert_eq!(p.goal_count, 3, "should have processed 3 updates");
    }
}
