//! Hypothesis processor — generates testable hypotheses from curiosities.
//!
//! On CURIOSITY_DETECTED, formulates a testable hypothesis about the
//! knowledge gap the curiosity identified. On PATTERN_DETECTED, proposes
//! a hypothesis about the pattern's cause.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{CuriosityDetected, PatternDetected};
use crate::signals::reasoning::HypothesisGenerated;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Minimum curiosity intensity to generate a hypothesis.
const MIN_CURIOSITY: f32 = 0.5;

/// Generates testable hypotheses from curiosity and pattern signals.
pub struct HypothesisProcessor;

impl HypothesisProcessor {
    pub fn new() -> Self {
        Self
    }

    fn from_curiosity(topic: &str, gap: &str) -> String {
        format!(
            "Hypothesis: {} may explain '{}' — testing this could resolve the gap",
            topic, gap,
        )
    }

    fn from_pattern(description: &str, occurrences: usize) -> String {
        format!(
            "Hypothesis: '{}' (observed {} times) is not random — there is an underlying cause",
            description, occurrences,
        )
    }
}

#[async_trait]
impl Processor for HypothesisProcessor {
    fn name(&self) -> &str {
        "hypothesis"
    }

    fn priority(&self) -> u8 {
        130
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::CURIOSITY_DETECTED, types::PATTERN_DETECTED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::HYPOTHESIS_GENERATED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::CURIOSITY_DETECTED {
            if let Some(curiosity) = signal.as_any().downcast_ref::<CuriosityDetected>() {
                if curiosity.intensity >= MIN_CURIOSITY {
                    let proposition = Self::from_curiosity(&curiosity.topic, &curiosity.gap_description);
                    tracing::info!("[HypothesisProcessor] generated from curiosity: {}", proposition);
                    return Ok(vec![Arc::new(HypothesisGenerated::new(&proposition))]);
                }
            }
            return Ok(vec![]);
        }

        if signal_type == types::PATTERN_DETECTED {
            if let Some(pattern) = signal.as_any().downcast_ref::<PatternDetected>() {
                if pattern.confidence >= 0.6 {
                    let proposition = Self::from_pattern(&pattern.description, pattern.occurrences);
                    tracing::info!("[HypothesisProcessor] generated from pattern: {}", proposition);
                    return Ok(vec![Arc::new(HypothesisGenerated::new(&proposition))]);
                }
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for HypothesisProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_hypothesis_processor_name() {
        let p = HypothesisProcessor::new();
        assert_eq!(p.name(), "hypothesis");
    }

    #[tokio::test]
    async fn test_hypothesis_from_curiosity() {
        let mut p = HypothesisProcessor::new();
        let ctx = test_context();

        let curiosity = CuriosityDetected::new(
            "memory consolidation",
            "Why do some memories consolidate faster than others?",
            0.8,
        );
        let result = p.process(&ctx, Arc::new(curiosity)).await.unwrap();
        assert!(!result.is_empty(), "curiosity should generate hypothesis");

        let sig = result[0].as_any().downcast_ref::<HypothesisGenerated>().unwrap();
        assert!(sig.proposition.contains("Hypothesis"), "should be a hypothesis statement");
    }

    #[tokio::test]
    async fn test_hypothesis_skips_low_intensity_curiosity() {
        let mut p = HypothesisProcessor::new();
        let ctx = test_context();

        let curiosity = CuriosityDetected::new("trivia", "minor thing", 0.2);
        let result = p.process(&ctx, Arc::new(curiosity)).await.unwrap();
        assert!(result.is_empty(), "low intensity should not generate hypothesis");
    }

    #[tokio::test]
    async fn test_hypothesis_from_pattern() {
        let mut p = HypothesisProcessor::new();
        let ctx = test_context();

        let pattern = PatternDetected {
            meta: crate::kernel::signal::SignalMeta::new(types::PATTERN_DETECTED, "test"),
            pattern_id: uuid::Uuid::new_v4(),
            description: "Frequent questions about async Rust".to_string(),
            occurrences: 7,
            confidence: 0.75,
        };
        let result = p.process(&ctx, Arc::new(pattern)).await.unwrap();
        assert!(!result.is_empty(), "pattern should generate hypothesis");
    }
}
