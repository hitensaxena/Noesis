//! Retrieval processor — searches stored episodes on curiosity triggers.
//!
//! Builds an in-memory term→episode index by subscribing to
//! EPISODE_RECORDED signals. On CURIOSITY_DETECTED, scores stored
//! episodes by keyword overlap against the curiosity topic and emits
//! EpisodesRetrieved with the top matches.
//!
//! This enables the cognitive system to "remember" relevant past
//! experiences when curiosity identifies an information gap.

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, EpisodesRetrieved};
use crate::signals::CuriosityDetected;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;

/// Maximum number of episodes to return in a retrieval.
const MAX_RESULTS: usize = 5;

/// Common stop words (same as IndexingProcessor for consistent indexing).
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
];

const MIN_TERM_LEN: usize = 4;

/// Simple in-memory episode store with keyword indexing.
struct EpisodeIndex {
    /// episode_id → (content, timestamp)
    episodes: Vec<(uuid::Uuid, String)>,
    /// term → set of episode_ids containing it
    term_index: HashMap<String, Vec<uuid::Uuid>>,
}

impl EpisodeIndex {
    fn new() -> Self {
        Self {
            episodes: Vec::new(),
            term_index: HashMap::new(),
        }
    }

    fn insert(&mut self, id: uuid::Uuid, content: &str) {
        self.episodes.push((id, content.to_string()));

        let terms = extract_terms_for_index(content);
        for term in terms {
            self.term_index
                .entry(term)
                .or_default()
                .push(id);
        }
    }

    /// Score episodes by keyword overlap with a query.
    fn search(&self, query: &str) -> Vec<(uuid::Uuid, String, usize)> {
        let query_terms: Vec<String> = query
            .to_lowercase()
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() >= MIN_TERM_LEN && !STOP_WORDS.contains(w))
            .map(|w| w.to_string())
            .collect();

        if query_terms.is_empty() || self.episodes.is_empty() {
            return vec![];
        }

        // Score each episode by how many query terms it matches
        let mut scores: Vec<(uuid::Uuid, usize)> = self
            .episodes
            .iter()
            .map(|(id, content)| {
                let lower = content.to_lowercase();
                let score = query_terms
                    .iter()
                    .filter(|qt| lower.contains(qt.as_str()))
                    .count();
                (*id, score)
            })
            .filter(|(_, s)| *s > 0)
            .collect();

        // Sort by score descending, then by most recent first (by position)
        scores.sort_by(|a, b| b.1.cmp(&a.1));

        // Map back to content
        scores
            .into_iter()
            .take(MAX_RESULTS)
            .filter_map(|(id, score)| {
                self.episodes
                    .iter()
                    .find(|(eid, _)| *eid == id)
                    .map(|(_, content)| (id, content.clone(), score))
            })
            .collect()
    }

    fn len(&self) -> usize {
        self.episodes.len()
    }
}

/// Extract significant terms from content for indexing.
fn extract_terms_for_index(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|w| w.len() >= MIN_TERM_LEN && !STOP_WORDS.contains(w))
        .map(|w| w.to_string())
        .collect()
}

/// Retrieves relevant episodes triggered by curiosity signals.
pub struct RetrievalProcessor {
    index: EpisodeIndex,
}

impl RetrievalProcessor {
    pub fn new() -> Self {
        Self {
            index: EpisodeIndex::new(),
        }
    }
}

#[async_trait]
impl Processor for RetrievalProcessor {
    fn name(&self) -> &str {
        "retrieval"
    }

    fn priority(&self) -> u8 {
        20 // Early — runs just after indexing
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED, types::CURIOSITY_DETECTED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::EPISODES_RETRIEVED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        let signal_type = signal.signal_type();

        if signal_type == types::EPISODE_RECORDED {
            // Build the searchable index
            if let Some(ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
                self.index.insert(ep.episode_id, &ep.content);
                tracing::trace!(
                    "[RetrievalProcessor] indexed episode {} (total: {})",
                    ep.episode_id,
                    self.index.len(),
                );
            }
            return Ok(vec![]);
        }

        if signal_type == types::CURIOSITY_DETECTED {
            if let Some(curiosity) = signal.as_any().downcast_ref::<CuriosityDetected>() {
                let query = &curiosity.topic;
                let results = self.index.search(query);

                if results.is_empty() {
                    tracing::debug!(
                        "[RetrievalProcessor] no results for curiosity query: {}",
                        query,
                    );
                    return Ok(vec![]);
                }

                let episode_ids: Vec<uuid::Uuid> = results.iter().map(|(id, _, _)| *id).collect();
                let matches: Vec<String> = results
                    .iter()
                    .map(|(_, content, score)| format!("[score={}] {}", score, content))
                    .collect();

                tracing::info!(
                    "[RetrievalProcessor] retrieved {} episodes for query: {}",
                    results.len(),
                    query,
                );

                return Ok(vec![Arc::new(EpisodesRetrieved::new(
                    query,
                    episode_ids,
                    matches,
                ))]);
            }
        }

        Ok(vec![])
    }

    async fn shutdown(&mut self) -> Result<()> {
        tracing::info!(
            "[RetrievalProcessor] shutting down with {} indexed episodes",
            self.index.len(),
        );
        Ok(())
    }
}

impl Default for RetrievalProcessor {
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
    use crate::signals::CuriosityDetected;

    fn test_context() -> FieldContext {
        let bus = Arc::new(EventBus::new());
        let storage = Arc::new(MemoryStore::new());
        FieldContext::new(bus, storage)
    }

    #[test]
    fn test_retrieval_processor_name() {
        let p = RetrievalProcessor::new();
        assert_eq!(p.name(), "retrieval");
    }

    #[test]
    fn test_retrieval_subscriptions() {
        let p = RetrievalProcessor::new();
        let subs = p.subscribed_signals();
        assert!(subs.contains(&types::EPISODE_RECORDED));
        assert!(subs.contains(&types::CURIOSITY_DETECTED));
    }

    #[tokio::test]
    async fn test_retrieval_indexes_episode() {
        let mut p = RetrievalProcessor::new();
        let ctx = test_context();

        let ep = EpisodeRecorded::new("Worked on the Rust compiler today.", "test", vec![]);
        let result = p.process(&ctx, Arc::new(ep)).await.unwrap();
        assert!(result.is_empty(), "indexing should be silent");
        assert_eq!(p.index.len(), 1, "should have 1 indexed episode");
    }

    #[tokio::test]
    async fn test_retrieval_returns_results_on_curiosity() {
        let mut p = RetrievalProcessor::new();
        let ctx = test_context();

        // Index some episodes
        p.process(&ctx, Arc::new(EpisodeRecorded::new(
            "I worked on the Rust compiler for the Noesis project.",
            "test", vec![],
        ))).await.unwrap();
        p.process(&ctx, Arc::new(EpisodeRecorded::new(
            "Cooked pasta for dinner with garlic bread.",
            "test", vec![],
        ))).await.unwrap();
        p.process(&ctx, Arc::new(EpisodeRecorded::new(
            "Debugged the memory decay processor in the cognitive system.",
            "test", vec![],
        ))).await.unwrap();

        // Curiosity about Rust should match the first episode
        let curiosity = CuriosityDetected::new("Rust", "What Rust projects have I worked on?", 0.8);
        let result = p.process(&ctx, Arc::new(curiosity)).await.unwrap();
        assert!(!result.is_empty(), "should retrieve results");

        let retrieved = result[0].as_any().downcast_ref::<EpisodesRetrieved>().unwrap();
        assert!(!retrieved.episode_ids.is_empty(), "should have episode IDs");
        assert!(retrieved.matches[0].contains("Rust"), "results should mention Rust");
    }

    #[tokio::test]
    async fn test_retrieval_no_match_returns_empty() {
        let mut p = RetrievalProcessor::new();
        let ctx = test_context();

        p.process(&ctx, Arc::new(EpisodeRecorded::new(
            "Only cooking and recipes today.",
            "test", vec![],
        ))).await.unwrap();

        // Curiosity about something unrelated
        let curiosity = CuriosityDetected::new("space exploration", "Tell me about space exploration.", 0.7);
        let result = p.process(&ctx, Arc::new(curiosity)).await.unwrap();
        assert!(result.is_empty(), "no match should return empty");
    }

    #[test]
    fn test_episode_index_search() {
        let mut index = EpisodeIndex::new();
        let id1 = uuid::Uuid::new_v4();
        let id2 = uuid::Uuid::new_v4();
        let id3 = uuid::Uuid::new_v4();

        index.insert(id1, "Rust programming and compiler design");
        index.insert(id2, "Cooking Italian pasta recipes");
        index.insert(id3, "Debugging the Rust type checker");

        let results = index.search("Rust compiler");
        assert_eq!(results.len(), 2, "should find 2 Rust-related episodes");
        // Both should have Rust
        assert!(results.iter().all(|(_, c, _)| c.to_lowercase().contains("rust")));
    }

    #[test]
    fn test_episode_index_empty_on_no_match() {
        let mut index = EpisodeIndex::new();
        index.insert(uuid::Uuid::new_v4(), "Just cooking content.");
        let results = index.search("quantum physics");
        assert!(results.is_empty(), "no match should return empty");
    }
}
