//! Synthesis processor — merges related information into higher-level syntheses.
//!
//! On EPISTEMICS_CLASSIFIED and ANALOGY_DETECTED, buffers observations.
//! On BEAT_MEDIUM, clusters related observations and emits SynthesisReady
//! with a consolidated topic and content.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::EpistemicClassified;
use crate::signals::reasoning::{AnalogyDetected, SynthesisReady};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Minimum observations to form a synthesis.
const MIN_OBSERVATIONS: usize = 2;

/// Synthesizes related epistemic and analogy information.
pub struct SynthesisProcessor {
    observations: Vec<String>,
}

impl SynthesisProcessor {
    pub fn new() -> Self {
        Self {
            observations: Vec::new(),
        }
    }

    fn synthesize(&self) -> Option<(String, String)> {
        if self.observations.len() < MIN_OBSERVATIONS {
            return None;
        }

        let topic = self.infer_topic();
        let content = format!(
            "Synthesis of {} observations: {}",
            self.observations.len(),
            self.observations.join("; "),
        );

        Some((topic, content))
    }

    fn infer_topic(&self) -> String {
        let all = self.observations.join(" ").to_lowercase();
        if all.contains("rust") || all.contains("programming") || all.contains("code") {
            "Software Development".to_string()
        } else if all.contains("learn") || all.contains("study") || all.contains("research") {
            "Learning & Research".to_string()
        } else if all.contains("cook") || all.contains("food") || all.contains("recipe") {
            "Culinary".to_string()
        } else if all.contains("health") || all.contains("fitness") || all.contains("exercise") {
            "Health & Fitness".to_string()
        } else {
            format!("General Synthesis ({})", self.observations.len())
        }
    }
}

#[async_trait]
impl Processor for SynthesisProcessor {
    fn name(&self) -> &str {
        "synthesis"
    }

    fn priority(&self) -> u8 {
        150
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISTEMICS_CLASSIFIED, types::ANALOGY_DETECTED, types::BEAT_MEDIUM]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::SYNTHESIS_READY]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::EPISTEMICS_CLASSIFIED {
            if let Some(ec) = signal.as_any().downcast_ref::<EpistemicClassified>() {
                self.observations.push(format!(
                    "{} ({})",
                    ec.classification, ec.signal_type,
                ));
            }
            return Ok(vec![]);
        }

        if signal_type == types::ANALOGY_DETECTED {
            if let Some(an) = signal.as_any().downcast_ref::<AnalogyDetected>() {
                self.observations.push(format!(
                    "analogy: {} → {} ({})",
                    an.source, an.target, an.mapping,
                ));
            }
            return Ok(vec![]);
        }

        if signal_type == types::BEAT_MEDIUM {
            if let Some((topic, content)) = self.synthesize() {
                tracing::info!(
                    "[SynthesisProcessor] synthesis ready: {} — {}",
                    topic, content,
                );
                self.observations.clear();
                return Ok(vec![Arc::new(SynthesisReady::new(&topic, &content))]);
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for SynthesisProcessor {
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
    fn test_synthesis_processor_name() {
        let p = SynthesisProcessor::new();
        assert_eq!(p.name(), "synthesis");
    }

    #[tokio::test]
    async fn test_synthesis_requires_min_observations() {
        let mut p = SynthesisProcessor::new();
        let ctx = test_context();

        let beat = BeatPulse::new(types::BEAT_MEDIUM);
        let result = p.process(&ctx, Arc::new(beat)).await.unwrap();
        assert!(result.is_empty(), "need at least 2 observations");
    }

    #[tokio::test]
    async fn test_synthesis_emits_on_beat() {
        let mut p = SynthesisProcessor::new();
        let ctx = test_context();

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
        assert!(!result.is_empty(), "should emit synthesis");

        let sig = result[0].as_any().downcast_ref::<SynthesisReady>().unwrap();
        assert!(!sig.topic.is_empty(), "synthesis should have a topic");
        assert!(!sig.content.is_empty(), "synthesis should have content");
    }
}
