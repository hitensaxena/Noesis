//! Trait processor — extracts personality traits from accumulated beliefs.
//!
//! Subscribes to BeliefChanged signals and emits TraitDetected.
//! Maps recurring belief topics to Big Five personality dimensions.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::SignalMeta;
use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{BeliefChanged, TraitDetected};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
use uuid::Uuid;

/// Maps belief text keywords to trait dimensions and calculates strength.
pub struct TraitProcessor;

impl TraitProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Simple keyword-based trait scoring from belief text.
    fn score_trait(belief: &str) -> Option<(&'static str, f32)> {
        let lower = belief.to_lowercase();
        // Openness
        if lower.contains("explore") || lower.contains("curious") || lower.contains("creative")
            || lower.contains("learn") || lower.contains("discover")
        {
            return Some(("openness", 0.7));
        }
        // Conscientiousness
        if lower.contains("organize") || lower.contains("plan") || lower.contains("discipline")
            || lower.contains("systematic") || lower.contains("thorough")
        {
            return Some(("conscientiousness", 0.7));
        }
        // Extraversion
        if lower.contains("social") || lower.contains("engage") || lower.contains("collaborate")
            || lower.contains("communicate") || lower.contains("share")
        {
            return Some(("extraversion", 0.6));
        }
        // Agreeableness
        if lower.contains("help") || lower.contains("support") || lower.contains("empathy")
            || lower.contains("cooperate") || lower.contains("trust")
        {
            return Some(("agreeableness", 0.6));
        }
        // Neuroticism
        if lower.contains("anxious") || lower.contains("uncertain") || lower.contains("worry")
            || lower.contains("fear") || lower.contains("stress")
        {
            return Some(("neuroticism", 0.5));
        }
        None
    }
}

#[async_trait]
impl Processor for TraitProcessor {
    fn name(&self) -> &str {
        "trait"
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
        &[types::TRAIT_DETECTED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        if let Some(bc) = signal.as_any().downcast_ref::<BeliefChanged>() {
            if let Some((trait_name, strength)) = Self::score_trait(&bc.belief) {
                tracing::info!(
                    "[TraitProcessor] detected trait '{}' (strength: {:.2}) from belief: {}",
                    trait_name, strength, &bc.belief[..40.min(bc.belief.len())]
                );

                let trait_signal = TraitDetected {
                    meta: SignalMeta::new(crate::signals::types::TRAIT_DETECTED, "trait::processor"),
                    trait_id: Uuid::new_v4(),
                    trait_name: trait_name.to_string(),
                    evidence: bc.belief[..40.min(bc.belief.len())].to_string(),
                    strength,
                };
                return Ok(vec![Arc::new(trait_signal)]);
            }
        }

        Ok(vec![])
    }
}

impl Default for TraitProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::signal::{Signal, SignalType};
    use crate::signals::{BeliefChanged, BeliefChangeType, TraitDetected};
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_trait_name() {
        let p = TraitProcessor::new();
        assert_eq!(p.name(), "trait");
    }

    #[test]
    fn test_trait_subscriptions() {
        let p = TraitProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::BELIEF_CHANGED));
    }

    #[tokio::test]
    async fn test_trait_detects_openness() {
        let mut p = TraitProcessor::new();
        let ctx = test_context();

        let bc = BeliefChanged::new("I love to explore new ideas and learn constantly", BeliefChangeType::Created, 0.8);
        let result = p.process(&ctx, Arc::new(bc)).await.unwrap();
        assert_eq!(result.len(), 1, "should detect openness trait");

        let trait_sig = result[0].as_any().downcast_ref::<TraitDetected>().unwrap();
        assert_eq!(trait_sig.trait_name, "openness");
    }

    #[tokio::test]
    async fn test_trait_ignores_neutral_beliefs() {
        let mut p = TraitProcessor::new();
        let ctx = test_context();

        let bc = BeliefChanged::new("The sky is blue", BeliefChangeType::Created, 0.9);
        let result = p.process(&ctx, Arc::new(bc)).await.unwrap();
        assert!(result.is_empty(), "neutral beliefs should not emit traits");
    }
}
