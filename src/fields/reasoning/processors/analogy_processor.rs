//! Analogy processor — detects structural analogies between domains.
//!
//! On CONCLUSION_READY, attempts to match the conclusion's domain against
//! previously stored domains. If a structural parallel is found, emits
//! AnalogyDetected with the mapping.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::reasoning::{AnalogyDetected, ConclusionReady};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Simple structural analogy detector.
pub struct AnalogyProcessor {
    recent_conclusions: Vec<(String, String)>,
}

impl AnalogyProcessor {
    pub fn new() -> Self {
        Self {
            recent_conclusions: Vec::new(),
        }
    }

    /// Extract a domain hint from a conclusion string.
    fn extract_domain(text: &str) -> String {
        let lower = text.to_lowercase();
        let known_domains = [
            "programming", "rust", "coding", "software", "code",
            "cook", "cooking", "food", "recipe", "kitchen",
            "learning", "study", "reading", "research",
            "health", "exercise", "running", "fitness",
            "finance", "trading", "money", "investment",
        ];
        for domain in &known_domains {
            if lower.contains(domain) {
                return domain.to_string();
            }
        }
        "general".to_string()
    }

    /// Try to find an analogy between a new conclusion and stored ones.
    fn find_analogy(&self, domain: &str) -> Option<(String, String, String)> {
        for (_, prev_domain) in &self.recent_conclusions {
            if prev_domain != domain
                && !prev_domain.is_empty()
                && !domain.is_empty()
                && prev_domain != "general"
                && domain != "general"
            {
                let mapping = format!("{} is like {}", domain, prev_domain);
                return Some((prev_domain.clone(), domain.to_string(), mapping));
            }
        }
        None
    }
}

#[async_trait]
impl Processor for AnalogyProcessor {
    fn name(&self) -> &str {
        "analogy"
    }

    fn priority(&self) -> u8 {
        140
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::CONCLUSION_READY]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::ANALOGY_DETECTED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::CONCLUSION_READY {
            if let Some(conc) = signal.as_any().downcast_ref::<ConclusionReady>() {
                let domain = Self::extract_domain(&conc.conclusion);

                if let Some((source, target, mapping)) = self.find_analogy(&domain) {
                    tracing::info!(
                        "[AnalogyProcessor] analogy: {} → {} ({})",
                        source, target, mapping,
                    );
                    return Ok(vec![Arc::new(AnalogyDetected::new(
                        &source, &target, &mapping,
                    ))]);
                }

                self.recent_conclusions.push((conc.conclusion.clone(), domain));
                if self.recent_conclusions.len() > 20 {
                    self.recent_conclusions.remove(0);
                }
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for AnalogyProcessor {
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
    fn test_analogy_processor_name() {
        let p = AnalogyProcessor::new();
        assert_eq!(p.name(), "analogy");
    }

    #[test]
    fn test_extract_domain() {
        assert_eq!(AnalogyProcessor::extract_domain("Rust is fast"), "rust");
        assert_eq!(AnalogyProcessor::extract_domain("I cooked dinner"), "cook");
        assert_eq!(AnalogyProcessor::extract_domain("The sky is blue"), "general");
    }

    #[tokio::test]
    async fn test_analogy_detected_between_domains() {
        let mut p = AnalogyProcessor::new();
        let ctx = test_context();

        let c1 = ConclusionReady::new("Cooking requires timing and precision", 0.9);
        p.process(&ctx, Arc::new(c1)).await.unwrap();

        let c2 = ConclusionReady::new("Rust programming requires careful type management", 0.8);
        let result = p.process(&ctx, Arc::new(c2)).await.unwrap();
        assert!(!result.is_empty(), "should detect analogy between domains");

        let sig = result[0].as_any().downcast_ref::<AnalogyDetected>().unwrap();
        assert!(sig.mapping.contains("like"), "analogy should use 'is like' mapping");
    }

    #[tokio::test]
    async fn test_analogy_no_match_on_same_domain() {
        let mut p = AnalogyProcessor::new();
        let ctx = test_context();

        let c1 = ConclusionReady::new("Rust has zero-cost abstractions", 0.8);
        p.process(&ctx, Arc::new(c1)).await.unwrap();

        let c2 = ConclusionReady::new("Rust's borrow checker ensures memory safety", 0.9);
        let result = p.process(&ctx, Arc::new(c2)).await.unwrap();
        assert!(result.is_empty(), "same domain should not produce analogy");
    }
}
