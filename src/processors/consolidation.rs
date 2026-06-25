use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing;

use crate::eventbus::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, MemoryConsolidated, PatternDetected};
use crate::processor::processor::Processor;
use crate::field::context::FieldContext;

/// State tracked by the consolidation processor.
#[derive(Debug, Default, Serialize, Deserialize)]
struct ConsolidationState {
    episode_count: usize,
    last_consolidation: Option<DateTime<Utc>>,
    recent_content: Vec<String>,
}

/// Memory consolidation processor.
///
/// Mirrors curlyos-core's `memory/consolidation/scheduler.py` behavior:
/// - Tracks episode count and content
/// - Periodically runs consolidation: dedup, summarize, link
/// - Emits MemoryConsolidated and PatternDetected signals
///
/// Consolidation cadences:
/// - FAST: After every 3 episodes (lightweight dedup + pattern scan)
/// - DEEP: After every 10 episodes (full summarize + link)
pub struct ConsolidationProcessor {
    state: ConsolidationState,
}

impl ConsolidationProcessor {
    pub fn new() -> Self {
        Self {
            state: ConsolidationState::default(),
        }
    }
}

#[async_trait]
impl Processor for ConsolidationProcessor {
    fn name(&self) -> &str {
        "consolidation"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn priority(&self) -> u8 {
        150 // Runs after episode processor
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::MEMORY_CONSOLIDATED, types::PATTERN_DETECTED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        if let Some(ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
            self.state.episode_count += 1;
            self.state.recent_content.push(ep.content.clone());

            tracing::debug!(
                "[Consolidation] episode #{} accumulated",
                self.state.episode_count
            );

            let mut emitted: Vec<SignalArc> = Vec::new();

            // FAST consolidation every 3 episodes
            if self.state.episode_count % 3 == 0 {
                let summary = summarize_episodes(&self.state.recent_content);
                tracing::info!(
                    "[Consolidation] fast consolidation: {} episodes -> {}",
                    3,
                    &summary[..40.min(summary.len())]
                );

                let mem_consolidated = MemoryConsolidated {
                    meta: signal.meta().child(
                        types::MEMORY_CONSOLIDATED,
                        "consolidation::processor",
                    ),
                    episode_ids: vec![ep.episode_id],
                    summary: summary.clone(),
                    memory_count: self.state.recent_content.len(),
                };
                emitted.push(Arc::new(mem_consolidated));

                // Pattern detection on every other fast consolidation
                if self.state.episode_count % 6 == 0 {
                    if let Some(pattern) = detect_pattern(&self.state.recent_content) {
                        let pattern_detected = PatternDetected {
                            meta: signal.meta().child(
                                types::PATTERN_DETECTED,
                                "consolidation::processor",
                            ),
                            pattern_id: uuid::Uuid::new_v4(),
                            description: pattern,
                            occurrences: self.state.recent_content.len(),
                            confidence: 0.5,
                        };
                        emitted.push(Arc::new(pattern_detected));
                    }
                }
            }

            // DEEP consolidation every 10 episodes
            if self.state.episode_count % 10 == 0 {
                let deep_summary = deep_summarize(&self.state.recent_content);
                tracing::info!(
                    "[Consolidation] deep consolidation: {} episodes",
                    self.state.episode_count
                );

                let mem_consolidated = MemoryConsolidated {
                    meta: signal.meta().child(
                        types::MEMORY_CONSOLIDATED,
                        "consolidation::processor",
                    ),
                    episode_ids: vec![ep.episode_id],
                    summary: deep_summary,
                    memory_count: self.state.recent_content.len(),
                };
                emitted.push(Arc::new(mem_consolidated));

                // Reset recent content after deep consolidation
                self.state.recent_content.clear();
                self.state.last_consolidation = Some(Utc::now());
            }

            Ok(emitted)
        } else {
            Ok(vec![])
        }
    }
}

impl Default for ConsolidationProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple dedup: remove near-duplicate content strings.
fn dedup_contents(contents: &[String]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut result = Vec::new();

    for content in contents {
        let normalized = content.to_lowercase();
        let words: Vec<String> = normalized.split_whitespace().map(|s| s.to_string()).collect();

        // Check if this content is similar to something we've seen
        let is_duplicate = seen.iter().any(|existing| {
            let existing_words: Vec<&str> = existing.split_whitespace().collect();
            let overlap = words
                .iter()
                .filter(|w| existing_words.contains(&w.as_str()))
                .count();
            let similarity = overlap as f64 / words.len().max(1) as f64;
            similarity > 0.7 // 70% word overlap = duplicate
        });

        if !is_duplicate {
            seen.push(normalized);
            result.push(content.clone());
        }
    }

    result
}

/// Summarize a list of episode contents into a short summary.
fn summarize_episodes(contents: &[String]) -> String {
    let recent = contents.iter().rev().take(3);
    let combined: Vec<&str> = recent.map(|s| s.as_str()).collect();

    if combined.len() == 1 {
        format!("Recent experience: {}", combined[0].chars().take(100).collect::<String>())
    } else {
        let topics: Vec<&str> = combined
            .iter()
            .flat_map(|s| s.split_whitespace().find(|w| w.len() > 4))
            .collect();
        format!(
            "Consolidated {} experiences: {}",
            combined.len(),
            topics.join(", ")
        )
    }
}

/// Deep summarize across all accumulated content.
fn deep_summarize(contents: &[String]) -> String {
    let deduped = dedup_contents(contents);
    let total_chars: usize = deduped.iter().map(|s| s.len()).sum();
    let avg_len = if !deduped.is_empty() {
        total_chars / deduped.len()
    } else {
        0
    };

    format!(
        "Deep consolidation: {} unique episodes (avg {} chars). Total experiences: {}.",
        deduped.len(),
        avg_len,
        contents.len()
    )
}

/// Detect repeating patterns in recent content.
fn detect_pattern(contents: &[String]) -> Option<String> {
    // Count word frequencies across all content
    let mut word_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for content in contents {
        for word in content.split_whitespace() {
            let clean = word
                .trim_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            if clean.len() > 3 {
                *word_counts.entry(clean).or_insert(0) += 1;
            }
        }
    }

    // Find words that appear in multiple episodes
    let threshold = (contents.len() / 2).max(2);
    let patterns: Vec<String> = word_counts
        .into_iter()
        .filter(|(_, count)| *count >= threshold)
        .map(|(word, count)| format!("{} ({}x)", word, count))
        .collect();

    if patterns.is_empty() {
        None
    } else {
        Some(format!("Recurring themes: {}", patterns.join(", ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dedup_exact_duplicates() {
        let contents = vec![
            "I went for a run in the park".to_string(),
            "I went for a run in the park".to_string(),
            "I went for a run in the park today".to_string(),
        ];
        let deduped = dedup_contents(&contents);
        assert!(deduped.len() < contents.len(), "should remove duplicates");
    }

    #[test]
    fn test_summarize_single() {
        let contents = vec!["Went for a run".to_string()];
        let summary = summarize_episodes(&contents);
        assert!(summary.contains("Recent experience"));
    }

    #[test]
    fn test_pattern_detection() {
        let contents = vec![
            "Working on the Rust project today".to_string(),
            "The Rust project is coming along well".to_string(),
            "Fixed a bug in the Rust compiler".to_string(),
        ];
        let pattern = detect_pattern(&contents);
        assert!(pattern.is_some(), "should detect a pattern");
        let p = pattern.unwrap();
        assert!(p.contains("rust") || p.contains("Rust"), "pattern should mention rust, got: {}", p);
    }
}
