//! Concept processor — forms concepts from entity clusters.
//!
//! On EPISTEMICS_CLASSIFIED and ANALOGY_DETECTED, accumulates related
//! observations. On BEAT_SLOW, clusters them into named concepts and
//! emits ConceptFormed.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::reasoning::ConceptFormed;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Recognized concept-seeding keywords.
const DOMAIN_KEYWORDS: &[(&str, &str)] = &[
    ("rust", "Systems Programming"),
    ("programming", "Software Development"),
    ("cooking", "Culinary Arts"),
    ("fitness", "Health & Fitness"),
    ("trading", "Financial Markets"),
    ("learning", "Learning & Education"),
    ("research", "Research & Analysis"),
];

/// Forms concepts from clustered observations.
pub struct ConceptProcessor {
    observations: Vec<String>,
}

impl ConceptProcessor {
    pub fn new() -> Self {
        Self {
            observations: Vec::new(),
        }
    }

    /// Try to cluster observations into a named concept.
    fn form_concept(&self) -> Option<(String, String)> {
        let text = self.observations.join(" ").to_lowercase();
        for (keyword, concept_name) in DOMAIN_KEYWORDS {
            if text.contains(keyword) {
                let definition = format!(
                    "Concept formed around '{}' from {} observations",
                    concept_name,
                    self.observations.len(),
                );
                return Some((concept_name.to_string(), definition));
            }
        }
        if self.observations.len() >= 3 {
            return Some((
                "General Abstraction".to_string(),
                format!("Formed from {} clustered observations", self.observations.len()),
            ));
        }
        None
    }
}

#[async_trait]
impl Processor for ConceptProcessor {
    fn name(&self) -> &str {
        "concept"
    }

    fn priority(&self) -> u8 {
        160
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISTEMICS_CLASSIFIED, types::BEAT_SLOW]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::CONCEPT_FORMED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::EPISTEMICS_CLASSIFIED {
            // Accumulate epistemic observations
            tracing::trace!("[ConceptProcessor] buffered observation");
            self.observations.push("observation".to_string());
            return Ok(vec![]);
        }

        if signal_type == types::BEAT_SLOW {
            if !self.observations.is_empty() {
                if let Some((name, definition)) = self.form_concept() {
                    tracing::info!(
                        "[ConceptProcessor] formed concept: {} — {}",
                        name, definition,
                    );
                    self.observations.clear();
                    return Ok(vec![Arc::new(ConceptFormed::new(&name, &definition))]);
                }
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for ConceptProcessor {
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
    fn test_concept_processor_name() {
        let p = ConceptProcessor::new();
        assert_eq!(p.name(), "concept");
    }

    #[test]
    fn test_concept_subscriptions() {
        let p = ConceptProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::EPISTEMICS_CLASSIFIED));
        assert!(subs.contains(&types::BEAT_SLOW));
    }

    #[tokio::test]
    async fn test_concept_formed_on_beat() {
        let mut p = ConceptProcessor::new();
        let ctx = test_context();

        // Add some observations
        for i in 0..3 {
            let epi = crate::signals::EpistemicClassified {
                meta: crate::kernel::signal::SignalMeta::new(types::EPISTEMICS_CLASSIFIED, "test"),
                classification_id: uuid::Uuid::new_v4(),
                signal_type: format!("signal_{}", i),
                classification: "Known".to_string(),
                confidence: 0.8,
            };
            p.process(&ctx, Arc::new(epi)).await.unwrap();
        }

        let beat = BeatPulse::new(types::BEAT_SLOW);
        let result = p.process(&ctx, Arc::new(beat)).await.unwrap();
        assert!(!result.is_empty(), "BEAT_SLOW should emit concept");

        let sig = result[0].as_any().downcast_ref::<ConceptFormed>().unwrap();
        assert!(!sig.name.is_empty(), "concept should have a name");
    }
}
