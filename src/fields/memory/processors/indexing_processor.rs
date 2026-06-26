//! Indexing processor — extracts key terms from episode content.
//!
//! On each EPISODE_RECORDED, performs simple TF (term frequency) scoring
//! to identify the most significant words in the episode content, filters
//! out stop words and very short terms, and emits an IndexUpdated signal
//! with the extracted terms.
//!
//! The extracted terms can be used by downstream processors (retrieval,
//! pattern detection, curiosity) to understand what this episode is about
//! without re-parsing the full content.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, IndexUpdated};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Common English stop words to filter out.
const STOP_WORDS: &[&str] = &[
    "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for",
    "of", "with", "by", "from", "as", "is", "was", "were", "be", "been",
    "are", "has", "had", "have", "do", "does", "did", "will", "would",
    "could", "should", "may", "might", "can", "shall", "its", "it's",
    "this", "that", "these", "those", "i", "me", "my", "we", "our",
    "you", "your", "he", "she", "it", "they", "them", "their",
    "not", "no", "nor", "so", "if", "than", "then", "just", "about",
    "up", "out", "also", "very", "too", "really", "much", "more",
    "some", "any", "each", "every", "all", "both", "few", "many",
    "into", "over", "such", "only", "own", "same", "other", "another",
];

/// Minimum word length to consider as a potential term.
const MIN_TERM_LEN: usize = 4;

/// Maximum number of terms to emit per episode.
const MAX_TERMS: usize = 15;

/// Compute simple term frequency scores for a piece of text.
fn extract_terms(text: &str) -> Vec<String> {
    let lower = text.to_lowercase();

    // Split into words, clean punctuation
    let words: Vec<&str> = lower
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .filter(|w| w.len() >= MIN_TERM_LEN && !STOP_WORDS.contains(w))
        .collect();

    if words.is_empty() {
        return vec![];
    }

    // Count frequencies
    let mut freqs: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for w in &words {
        *freqs.entry(w).or_insert(0) += 1;
    }

    // Sort by frequency (descending), then alphabetically for ties
    let mut sorted: Vec<(&str, usize)> = freqs.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(b.0)));

    // Take top N
    sorted
        .into_iter()
        .take(MAX_TERMS)
        .map(|(w, _)| w.to_string())
        .collect()
}

/// Extracts key indexing terms from episode content.
pub struct IndexingProcessor;

impl IndexingProcessor {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Processor for IndexingProcessor {
    fn name(&self) -> &str {
        "indexing"
    }

    fn priority(&self) -> u8 {
        40 // After dedup, before extraction
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::INDEX_UPDATED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::EPISODE_RECORDED {
            if let Some(ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
                let terms = extract_terms(&ep.content);

                if !terms.is_empty() {
                    tracing::debug!(
                        "[IndexingProcessor] extracted {} terms from episode {}",
                        terms.len(),
                        ep.episode_id,
                    );
                    return Ok(vec![Arc::new(IndexUpdated::new(ep.episode_id, terms))]);
                }
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

impl Default for IndexingProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use crate::kernel::signal::Signal;
    use crate::kernel::bus::EventBus;
    use crate::storage::memory_store::MemoryStore;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_indexing_processor_name() {
        let p = IndexingProcessor::new();
        assert_eq!(p.name(), "indexing");
    }

    #[test]
    fn test_indexing_subscriptions() {
        let p = IndexingProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::EPISODE_RECORDED));
    }

    #[test]
    fn test_extract_terms_basic() {
        let terms = extract_terms("The programmer worked on the Rust compiler and the type system");
        assert!(!terms.is_empty(), "should extract terms");
        assert!(terms.contains(&"programmer".to_string()), "should contain 'programmer'");
        assert!(terms.contains(&"compiler".to_string()), "should contain 'compiler'");
        assert!(terms.contains(&"system".to_string()), "should contain 'system'");
        assert!(terms.contains(&"rust".to_string()), "should contain 'rust'");
        assert!(terms.contains(&"type".to_string()), "should contain 'type'");
    }

    #[test]
    fn test_extract_terms_filters_stop_words() {
        let terms = extract_terms("the and for with this that");
        assert!(terms.is_empty(), "stop words should be filtered out");
    }

    #[test]
    fn test_extract_terms_filters_short_words() {
        let terms = extract_terms("a an to at it is");
        assert!(terms.is_empty(), "short words should be filtered out");
    }

    #[test]
    fn test_extract_terms_returns_top_n() {
        let text = "apple banana cherry apple banana apple date elderberry fig grape";
        let terms = extract_terms(text);
        assert!(terms.len() <= MAX_TERMS, "should not exceed MAX_TERMS");
        // Apple appears 3x, banana 2x — apple should be first
        assert_eq!(terms[0], "apple", "highest frequency term should be first");
        assert_eq!(terms[1], "banana", "second highest should be second");
    }

    #[tokio::test]
    async fn test_indexing_emits_on_episode() {
        let mut p = IndexingProcessor::new();
        let ctx = test_context();

        let ep = EpisodeRecorded::new(
            "I worked on the memory decay processor today in Rust.",
            "test",
            vec![],
        );
        let result = p.process(&ctx, Arc::new(ep)).await.unwrap();
        assert!(!result.is_empty(), "should emit IndexUpdated");

        let sig = result[0].as_any().downcast_ref::<IndexUpdated>().unwrap();
        assert!(!sig.terms.is_empty(), "should have extracted terms");
        assert!(sig.terms.contains(&"processor".to_string()), "should contain 'processor'");
    }

    #[tokio::test]
    async fn test_indexing_empty_content_no_emission() {
        let mut p = IndexingProcessor::new();
        let ctx = test_context();

        let ep = EpisodeRecorded::new("a an the", "test", vec![]);
        let result = p.process(&ctx, Arc::new(ep)).await.unwrap();
        assert!(result.is_empty(), "stop-words-only content should not emit");
    }
}
