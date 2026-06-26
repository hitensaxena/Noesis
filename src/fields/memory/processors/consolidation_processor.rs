//! Memory consolidation processor — LLM-powered summarization.
//!
//! Subscribes to EpisodeRecorded and emits MemoryConsolidated + PatternDetected.
//! Uses Agentic tier for fast consolidation (every 3ep), Deep for deep (every 10ep).
//! Falls back to heuristic logic when LLM unavailable or rate limited.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::{EpisodeRecorded, MemoryConsolidated, PatternDetected};
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
use crate::engines::llm::{ModelTier, TieredRouter, CompletionRequest, Message};
use crate::engines::llm::types::LLMError;

/// System prompt for fast (every 3ep) consolidation.
const FAST_CONSOLIDATION_SYSTEM: &str = r#"You are a memory consolidation engine. Given 3 recent experiences, summarize them into ONE coherent memory (1-2 sentences).

Focus on:
- The common thread or topic
- Key information worth remembering
- Remove redundant details

Reply with ONLY a single paragraph, no JSON, no labels.
"#;

/// State tracked by the consolidation processor.
#[derive(Debug, Default, Serialize, Deserialize)]
struct ConsolidationState {
    episode_count: usize,
    last_consolidation: Option<DateTime<Utc>>,
    recent_content: Vec<String>,
}

/// Memory consolidation processor with LLM-powered summarization.
pub struct ConsolidationProcessor {
    state: ConsolidationState,
    llm: Option<TieredRouter>,
}

impl ConsolidationProcessor {
    pub fn new() -> Self {
        let llm = if TieredRouter::has_api_key() {
            match TieredRouter::from_env() {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::warn!("[Consolidation] LLM unavailable: {}", e);
                    None
                }
            }
        } else {
            None
        };
        Self {
            state: ConsolidationState::default(),
            llm,
        }
    }

    /// LLM-powered fast summarization (Agentic tier).
    async fn llm_fast_summary(&mut self, content: &[String]) -> Option<String> {
        let router = self.llm.as_mut()?;
        let text = content.iter().rev().take(3).map(|s| format!("- {}", s)).collect::<Vec<_>>().join("\n");

        let request = CompletionRequest::new(
            "consolidation-fast",
            vec![
                Message::system(FAST_CONSOLIDATION_SYSTEM),
                Message::user(&format!("Recent experiences:\n{}", text)),
            ],
        )
        .with_temperature(0.3)
        .with_max_tokens(512);

        match router.complete(ModelTier::Agentic, request).await {
            Ok(resp) => {
                let summary = resp.content.trim().to_string();
                if summary.is_empty() {
                    None
                } else {
                    Some(summary)
                }
            }
            Err(LLMError::RateLimited { retry_after }) => {
                tracing::warn!("[Consolidation] rate limited ({}s), using heuristic summary", retry_after);
                None
            }
            Err(e) => {
                tracing::warn!("[Consolidation] LLM error: {}, using heuristic summary", e);
                None
            }
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
        150
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED, types::BEAT_SLOW]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::MEMORY_CONSOLIDATED, types::PATTERN_DETECTED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        // Buffer episodes as they arrive
        if let Some(ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
            self.state.episode_count += 1;
            self.state.recent_content.push(ep.content.clone());
            tracing::debug!("[Consolidation] episode #{} accumulated", self.state.episode_count);
            return Ok(vec![]);
        }

        // On slow beat: consolidate buffered episodes
        if signal.signal_type() == types::BEAT_SLOW && self.state.episode_count >= 3 {
            let mut emitted: Vec<SignalArc> = Vec::new();
            let recent = self.state.recent_content.clone();

            // Fast consolidation of all buffered episodes
            let summary = if self.llm.is_some() {
                self.llm_fast_summary(&recent).await
                    .unwrap_or_else(|| summarize_episodes(&recent))
            } else {
                summarize_episodes(&self.state.recent_content)
            };

            tracing::info!("[Consolidation] consolidated {} episodes", self.state.episode_count);

            emitted.push(Arc::new(MemoryConsolidated {
                meta: signal.meta().child(types::MEMORY_CONSOLIDATED, "consolidation::processor"),
                episode_ids: recent.iter().enumerate().map(|(_i, _)| uuid::Uuid::new_v4()).collect(),
                summary: summary.clone(),
                memory_count: self.state.episode_count,
            }));

            // Pattern detection on consolidated content
            if let Some(pattern) = detect_pattern(&self.state.recent_content) {
                emitted.push(Arc::new(PatternDetected {
                    meta: signal.meta().child(types::PATTERN_DETECTED, "consolidation::processor"),
                    pattern_id: uuid::Uuid::new_v4(),
                    description: pattern,
                    occurrences: self.state.episode_count,
                    confidence: 0.5,
                }));
            }

            self.state.recent_content.clear();
            self.state.last_consolidation = Some(Utc::now());

            return Ok(emitted);
        }

        Ok(vec![])
    }
}

impl Default for ConsolidationProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Heuristic fallback functions (unchanged from original)
// ---------------------------------------------------------------------------

#[allow(dead_code)]
fn dedup_contents(contents: &[String]) -> Vec<String> {
    let mut seen: Vec<String> = Vec::new();
    let mut result = Vec::new();

    for content in contents {
        let normalized = content.to_lowercase();
        let words: Vec<String> = normalized.split_whitespace().map(|s| s.to_string()).collect();

        let is_duplicate = seen.iter().any(|existing| {
            let existing_words: Vec<&str> = existing.split_whitespace().collect();
            let overlap = words.iter().filter(|w| existing_words.contains(&w.as_str())).count();
            let similarity = overlap as f64 / words.len().max(1) as f64;
            similarity > 0.7
        });

        if !is_duplicate {
            seen.push(normalized);
            result.push(content.clone());
        }
    }

    result
}

fn summarize_episodes(contents: &[String]) -> String {
    let recent = contents.iter().rev().take(3);
    let combined: Vec<&str> = recent.map(|s| s.as_str()).collect();

    if combined.len() == 1 {
        format!("Recent experience: {}", combined[0].chars().take(100).collect::<String>())
    } else {
        let topics: Vec<&str> = combined.iter()
            .flat_map(|s| s.split_whitespace().find(|w| w.len() > 4))
            .collect();
        format!("Consolidated {} experiences: {}", combined.len(), topics.join(", "))
    }
}

fn detect_pattern(contents: &[String]) -> Option<String> {
    let mut word_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for content in contents {
        for word in content.split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
            if clean.len() > 3 {
                *word_counts.entry(clean).or_insert(0) += 1;
            }
        }
    }

    let threshold = (contents.len() / 2).max(2);
    let patterns: Vec<String> = word_counts.into_iter()
        .filter(|(_, count)| *count >= threshold)
        .map(|(word, count)| format!("{} ({}x)", word, count))
        .collect();

    if patterns.is_empty() { None }
    else { Some(format!("Recurring themes: {}", patterns.join(", "))) }
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
    }
}
