//! Pattern processor — detects recurring topics via simple n-gram frequency.
//!
//! On EPISODE_RECORDED, extracts significant n-gram frequencies across
//! episodes. When a term's frequency crosses the detection threshold,
//! emits PatternDetected with the topic, occurrence count, and confidence.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, PatternDetected};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Minimum term length to track.
const MIN_TERM_LEN: usize = 4;

/// Minimum occurrences before a pattern is emitted.
const PATTERN_THRESHOLD: usize = 3;

/// Common stop words (reuse the same set as indexing).
const STOP_WORDS: &[&str] = &[
    "the", "this", "that", "with", "from", "which", "their", "about",
    "there", "would", "could", "should", "because", "really", "after",
    "still", "well", "just", "also", "very", "more", "some", "than",
    "then", "been", "have", "what", "when", "where", "what", "were",
    "being", "into", "over", "such", "only", "other", "another",
];

/// Tracks token frequencies across episodes for pattern detection.
pub struct PatternProcessor {
    /// term → episode count
    term_counts: std::collections::HashMap<String, usize>,
    /// Total episodes processed
    episode_count: usize,
}

impl PatternProcessor {
    pub fn new() -> Self {
        Self {
            term_counts: std::collections::HashMap::new(),
            episode_count: 0,
        }
    }

    /// Extract significant tokens from text.
    fn extract_tokens(text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| {
                w.len() >= MIN_TERM_LEN
                    && !STOP_WORDS.contains(w)
                    && !w.contains(|c: char| c.is_ascii_digit())
            })
            .map(|w| w.to_string())
            .collect()
    }

    /// Check if any term exceeds the pattern threshold.
    fn detect_pattern(&mut self) -> Option<(String, usize, f32)> {
        for (term, count) in self.term_counts.iter() {
            if *count >= PATTERN_THRESHOLD && *count <= PATTERN_THRESHOLD + 1 {
                // Only emit the first time a term crosses the threshold
                let confidence = ((*count as f32) / 20.0).min(1.0);
                return Some((term.clone(), *count, confidence));
            }
        }
        None
    }
}

#[async_trait]
impl Processor for PatternProcessor {
    fn name(&self) -> &str {
        "pattern"
    }

    fn priority(&self) -> u8 {
        130
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::PATTERN_DETECTED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::EPISODE_RECORDED {
            if let Some(ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
                self.episode_count += 1;
                let tokens = Self::extract_tokens(&ep.content);

                for token in tokens {
                    *self.term_counts.entry(token).or_insert(0) += 1;
                }

                if let Some((term, count, confidence)) = self.detect_pattern() {
                    tracing::info!(
                        "[PatternProcessor] detected pattern: '{}' (occurrences: {}, confidence: {:.2})",
                        term, count, confidence,
                    );
                    return Ok(vec![Arc::new(PatternDetected {
                        meta: crate::kernel::signal::SignalMeta::new(
                            types::PATTERN_DETECTED,
                            "awareness::pattern",
                        ),
                        pattern_id: uuid::Uuid::new_v4(),
                        description: format!("Recurring topic: '{}'", term),
                        occurrences: count,
                        confidence,
                    })]);
                }
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!(
            "[PatternProcessor] shutting down with {} tracked terms across {} episodes",
            self.term_counts.len(),
            self.episode_count,
        );
        Ok(())
    }
}

impl Default for PatternProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::kernel::bus::EventBus;
    use crate::kernel::signal::Signal;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_pattern_processor_name() {
        let p = PatternProcessor::new();
        assert_eq!(p.name(), "pattern");
    }

    #[test]
    fn test_extract_tokens() {
        let tokens = PatternProcessor::extract_tokens("Working on the Rust compiler project");
        assert!(tokens.contains(&"working".to_string()));
        assert!(tokens.contains(&"rust".to_string()));
        assert!(tokens.contains(&"compiler".to_string()));
        assert!(tokens.contains(&"project".to_string()));
        // Stop word "the" should be filtered
        assert!(!tokens.contains(&"the".to_string()));
    }

    #[tokio::test]
    async fn test_pattern_detects_recurring_topic() {
        let mut p = PatternProcessor::new();
        let ctx = test_context();

        for _ in 0..4 {
            let ep = EpisodeRecorded::new(
                "Working on the Rust compiler and type system in Noesis",
                "test", vec![],
            );
            p.process(&ctx, Arc::new(ep)).await.unwrap();
        }

        // Check tracking
        assert!(p.term_counts.contains_key("rust"));
        assert_eq!(*p.term_counts.get("rust").unwrap(), 4);
    }

    #[tokio::test]
    async fn test_pattern_emits_at_threshold() {
        let mut p = PatternProcessor::new();
        let ctx = test_context();

        // Three episodes mentioning the same unique topic term
        for i in 0..3 {
            let ep = EpisodeRecorded::new(
                &format!("zebraquilting episode number {}", i + 1),
                "test", vec![],
            );
            let result = p.process(&ctx, Arc::new(ep)).await.unwrap();
            if !result.is_empty() {
                let sig = result[0].as_any().downcast_ref::<PatternDetected>().unwrap();
                assert!(sig.occurrences >= 3, "should have 3+ occurrences");
                return;
            }
        }
        panic!("Pattern was not emitted after 3 episodes");
    }

    #[tokio::test]
    async fn test_pattern_no_false_positive_on_unique_topics() {
        let mut p = PatternProcessor::new();
        let ctx = test_context();

        let topics = [
            "Cooking Italian pasta recipes",
            "Running in the morning park",
            "Reading a sci-fi novel",
            "Learning about quantum computing",
        ];
        for topic in &topics {
            let ep = EpisodeRecorded::new(topic, "test", vec![]);
            let result = p.process(&ctx, Arc::new(ep)).await.unwrap();
            assert!(result.is_empty(), "unique episodes should not emit pattern");
        }
    }
}
