//! Opportunity processor — detects opportunities from curiosity and patterns.
//!
//! Scans CURIOSITY_DETECTED signals for potential opportunities:
//! a curiosity with high intensity and a concrete topic is scored as
//! a potential opportunity worth investigating. On PATTERN_DETECTED,
//! recurring patterns with high confidence are also evaluated as
//! opportunities.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::CuriosityDetected;
use crate::signals::{PatternDetected, OpportunityDetected};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Minimum curiosity intensity to consider as an opportunity.
const MIN_CURIOSITY_INTENSITY: f32 = 0.7;

/// Minimum pattern confidence to trigger opportunity.
const MIN_PATTERN_CONFIDENCE: f32 = 0.65;

/// Detects opportunities from curiosity and pattern signals.
pub struct OpportunityProcessor;

impl OpportunityProcessor {
    pub fn new() -> Self {
        Self
    }

    /// Score a potential opportunity from a curiosity signal.
    fn score_opportunity(topic: &str, intensity: f32, gap: &str) -> Option<(String, f32)> {
        if intensity < MIN_CURIOSITY_INTENSITY {
            return None;
        }

        // Concrete topics (not too short, not too vague) score higher
        let concrete_bonus = if topic.len() > 15 { 0.15 } else { 0.0 };
        let gap_bonus = if gap.len() > 20 { 0.1 } else { 0.0 };

        let score = (intensity + concrete_bonus + gap_bonus).min(1.0);

        let description = format!(
            "Opportunity from curiosity: '{}' — {} (intensity: {:.1})",
            topic, gap, intensity,
        );

        Some((description, score))
    }

    /// Evaluate a pattern as an opportunity.
    fn score_pattern(pattern: &PatternDetected) -> Option<(String, f32)> {
        if pattern.confidence < MIN_PATTERN_CONFIDENCE {
            return None;
        }

        // More occurrences = more significant opportunity
        let occurrence_factor = (pattern.occurrences as f32 / 10.0).min(0.3);
        let score = (pattern.confidence + occurrence_factor).min(1.0);

        let description = format!(
            "Opportunity from recurring pattern: '{}' (occurred {} times, confidence: {:.2})",
            pattern.description, pattern.occurrences, pattern.confidence,
        );

        Some((description, score))
    }
}

#[async_trait]
impl Processor for OpportunityProcessor {
    fn name(&self) -> &str {
        "opportunity"
    }

    fn priority(&self) -> u8 {
        100
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::CURIOSITY_DETECTED, types::PATTERN_DETECTED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::OPPORTUNITY_DETECTED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::CURIOSITY_DETECTED {
            if let Some(curiosity) = signal.as_any().downcast_ref::<CuriosityDetected>() {
                if let Some((description, score)) = Self::score_opportunity(
                    &curiosity.topic,
                    curiosity.intensity,
                    &curiosity.gap_description,
                ) {
                    tracing::info!(
                        "[OpportunityProcessor] detected curiosity opportunity: {} (score: {:.2})",
                        description, score,
                    );
                    return Ok(vec![Arc::new(OpportunityDetected::new(&description, score))]);
                }
            }
            return Ok(vec![]);
        }

        if signal_type == types::PATTERN_DETECTED {
            if let Some(pattern) = signal.as_any().downcast_ref::<PatternDetected>() {
                if let Some((description, score)) = Self::score_pattern(pattern) {
                    tracing::info!(
                        "[OpportunityProcessor] detected pattern opportunity: {} (score: {:.2})",
                        description, score,
                    );
                    return Ok(vec![Arc::new(OpportunityDetected::new(&description, score))]);
                }
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for OpportunityProcessor {
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
    fn test_opportunity_processor_name() {
        let p = OpportunityProcessor::new();
        assert_eq!(p.name(), "opportunity");
    }

    #[test]
    fn test_opportunity_subscriptions() {
        let p = OpportunityProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::CURIOSITY_DETECTED));
        assert!(subs.contains(&types::PATTERN_DETECTED));
    }

    #[tokio::test]
    async fn test_opportunity_detected_from_high_intensity_curiosity() {
        let mut p = OpportunityProcessor::new();
        let ctx = test_context();

        let curiosity = CuriosityDetected::new(
            "Rust compiler optimization",
            "How does the Rust compiler optimize async code?",
            0.85,
        );
        let result = p.process(&ctx, Arc::new(curiosity)).await.unwrap();
        assert!(!result.is_empty(), "high-intensity curiosity should be an opportunity");

        let sig = result[0].as_any().downcast_ref::<OpportunityDetected>().unwrap();
        assert!(sig.potential >= 0.7, "should have high potential score");
    }

    #[tokio::test]
    async fn test_opportunity_ignores_low_intensity_curiosity() {
        let mut p = OpportunityProcessor::new();
        let ctx = test_context();

        let curiosity = CuriosityDetected::new(
            "trivial",
            "something minor",
            0.3,
        );
        let result = p.process(&ctx, Arc::new(curiosity)).await.unwrap();
        assert!(result.is_empty(), "low-intensity curiosity should not be an opportunity");
    }

    #[tokio::test]
    async fn test_opportunity_from_high_confidence_pattern() {
        let mut p = OpportunityProcessor::new();
        let ctx = test_context();

        let pattern = PatternDetected {
            meta: crate::kernel::signal::SignalMeta::new(types::PATTERN_DETECTED, "test"),
            pattern_id: uuid::Uuid::new_v4(),
            description: "Frequent questions about Noesis architecture".to_string(),
            occurrences: 8,
            confidence: 0.8,
        };
        let result = p.process(&ctx, Arc::new(pattern)).await.unwrap();
        assert!(!result.is_empty(), "high-confidence pattern should be an opportunity");
    }

    #[tokio::test]
    async fn test_opportunity_ignores_low_confidence_pattern() {
        let mut p = OpportunityProcessor::new();
        let ctx = test_context();

        let pattern = PatternDetected {
            meta: crate::kernel::signal::SignalMeta::new(types::PATTERN_DETECTED, "test"),
            pattern_id: uuid::Uuid::new_v4(),
            description: "Minor coincidence".to_string(),
            occurrences: 1,
            confidence: 0.2,
        };
        let result = p.process(&ctx, Arc::new(pattern)).await.unwrap();
        assert!(result.is_empty(), "low-confidence pattern should not be an opportunity");
    }
}
