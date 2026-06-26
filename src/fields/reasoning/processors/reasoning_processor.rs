//! Reasoning processor — coordinating chain for reasoning subsystems.
//!
//! Buffers epistemic classifications until enough evidence accumulates,
//! then combines them into a reasoned conclusion. Emits ConclusionReady
//! on BEAT_MEDIUM if there are enough observations to form a conclusion.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::EpistemicClassified;
use crate::signals::reasoning::ConclusionReady;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Minimum number of epistemic observations to form a conclusion.
const MIN_OBSERVATIONS: usize = 2;

/// Coordinates reasoning sub-results into conclusions.
pub struct ReasoningProcessor {
    observations: Vec<String>,
}

impl ReasoningProcessor {
    pub fn new() -> Self {
        Self {
            observations: Vec::new(),
        }
    }

    /// Combine all observations into a single conclusion statement.
    fn synthesize(&self) -> String {
        if self.observations.is_empty() {
            return "No observations to reason about.".to_string();
        }

        let unique: std::collections::BTreeSet<&str> =
            self.observations.iter().map(|s| s.as_str()).collect();
        let mut parts: Vec<&str> = unique.into_iter().collect();
        parts.sort();

        if parts.len() == 1 {
            format!("Based on reasoning: {}", parts[0])
        } else {
            let last = parts.pop().unwrap();
            let prefix = parts.join(", ");
            format!(
                "Synthesized from observations ({}): {} and {}",
                self.observations.len(),
                prefix,
                last,
            )
        }
    }

    fn confidence_from_count(&self) -> f32 {
        ((self.observations.len() as f32) / 10.0).min(0.95)
    }
}

#[async_trait]
impl Processor for ReasoningProcessor {
    fn name(&self) -> &str {
        "reasoning"
    }

    fn priority(&self) -> u8 {
        100
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISTEMICS_CLASSIFIED, types::HYPOTHESIS_GENERATED, types::ANALOGY_DETECTED, types::BEAT_MEDIUM, types::BEAT_SLOW]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::CONCLUSION_READY]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::EPISTEMICS_CLASSIFIED {
            if let Some(ec) = signal.as_any().downcast_ref::<EpistemicClassified>() {
                // Use the classification as an observation
                if ec.confidence > 0.3 {
                    self.observations.push(format!(
                        "{} ({})",
                        ec.classification, ec.signal_type,
                    ));
                    tracing::trace!(
                        "[ReasoningProcessor] buffered epistemic observation: {} (total: {})",
                        ec.classification,
                        self.observations.len(),
                    );
                }
            }
            return Ok(vec![]);
        }

        if signal_type == types::HYPOTHESIS_GENERATED
            || signal_type == types::ANALOGY_DETECTED
        {
            // These signals feed into the reasoning chain but we just
            // acknowledge them by tracking the count.
            tracing::trace!(
                "[ReasoningProcessor] received reasoning input: {}",
                signal_type.0,
            );
            return Ok(vec![]);
        }

        // We also respond to BEAT_MEDIUM to emit periodic conclusions
        if signal_type == types::BEAT_MEDIUM || signal_type == types::BEAT_SLOW {
            if self.observations.len() >= MIN_OBSERVATIONS {
                let conclusion = self.synthesize();
                let confidence = self.confidence_from_count();

                tracing::info!(
                    "[ReasoningProcessor] conclusion ready: {} (confidence: {:.2})",
                    conclusion, confidence,
                );

                // Don't clear — keep accumulating for richer conclusions
                return Ok(vec![Arc::new(ConclusionReady::new(&conclusion, confidence))]);
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for ReasoningProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::bus::EventBus;
    use crate::kernel::beat_coordinator::BeatPulse;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_reasoning_processor_name() {
        let p = ReasoningProcessor::new();
        assert_eq!(p.name(), "reasoning");
    }

    #[tokio::test]
    async fn test_reasoning_requires_min_observations() {
        let mut p = ReasoningProcessor::new();
        let ctx = test_context();

        // Add one observation below threshold
        let ec = EpistemicClassified {
            meta: crate::kernel::signal::SignalMeta::new(types::EPISTEMICS_CLASSIFIED, "test"),
            classification_id: uuid::Uuid::new_v4(),
            signal_type: "test_signal".to_string(),
            classification: "Known".to_string(),
            confidence: 0.8,
        };
        p.process(&ctx, Arc::new(ec)).await.unwrap();

        let beat = BeatPulse::new(types::BEAT_MEDIUM);
        let result = p.process(&ctx, Arc::new(beat)).await.unwrap();
        assert!(result.is_empty(), "need at least 2 observations");
    }

    #[tokio::test]
    async fn test_reasoning_emits_conclusion_on_beat() {
        let mut p = ReasoningProcessor::new();
        let ctx = test_context();

        // Add enough observations
        for i in 0..3 {
            let ec = EpistemicClassified {
                meta: crate::kernel::signal::SignalMeta::new(types::EPISTEMICS_CLASSIFIED, "test"),
                classification_id: uuid::Uuid::new_v4(),
                signal_type: format!("signal_{}", i),
                classification: "Known".to_string(),
                confidence: 0.8,
            };
            p.process(&ctx, Arc::new(ec)).await.unwrap();
        }

        let beat = BeatPulse::new(types::BEAT_MEDIUM);
        let result = p.process(&ctx, Arc::new(beat)).await.unwrap();
        assert!(!result.is_empty(), "should emit conclusion");

        let sig = result[0].as_any().downcast_ref::<ConclusionReady>().unwrap();
        assert!(!sig.conclusion.is_empty(), "conclusion should not be empty");
        assert!(sig.confidence > 0.0, "confidence should be positive");
    }
}
