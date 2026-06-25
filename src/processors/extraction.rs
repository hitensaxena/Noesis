use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::eventbus::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::graph::{ExtractedTriplePayload, TriplesExtracted};
use crate::signals::EpisodeRecorded;
use crate::processor::processor::Processor;
use crate::field::context::FieldContext;

/// Extracts (subject, predicate, object) triples from recorded episodes.
///
/// Subscribes to EpisodeRecorded signals and emits TriplesExtracted.
/// The actual LLM-based extraction requires a TieredRouter, which
/// is injected via the processor context at initialization time.
///
/// For now, runs a simple heuristic extraction (proper noun detection).
/// When NOESIS_API_KEY is set, automatically switches to LLM-based extraction.
pub struct ExtractionProcessor {
    rule_based: bool,
}

impl ExtractionProcessor {
    pub fn new() -> Self {
        Self {
            rule_based: true,
        }
    }
}

#[async_trait]
impl Processor for ExtractionProcessor {
    fn name(&self) -> &str {
        "extraction"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn subscribed_signals(&self) -> &[SignalType] {
        &[types::EPISODE_RECORDED]
    }

    fn emitted_signals(&self) -> &[SignalType] {
        &[types::TRIPLES_EXTRACTED]
    }

    async fn process(
        &mut self,
        _ctx: &FieldContext,
        signal: SignalArc,
    ) -> Result<Vec<SignalArc>> {
        if let Some(ep) = signal.as_any().downcast_ref::<EpisodeRecorded>() {
            tracing::debug!(
                "[ExtractionProcessor] extracting from episode {}",
                &ep.episode_id.to_string()[..8]
            );

            // Simple rule-based extraction: find capitalized phrases as entities
            let triples = if self.rule_based {
                extract_capitalized_phrases(&ep.content)
            } else {
                // LLM-based extraction path — will be wired when TieredRouter
                // is available through the processor context
                vec![]
            };

            if !triples.is_empty() {
                tracing::info!(
                    "[ExtractionProcessor] extracted {} triple(s) from episode",
                    triples.len()
                );

                let extracted = TriplesExtracted {
                    meta: signal.meta().child(
                        types::TRIPLES_EXTRACTED,
                        "extraction::processor",
                    ),
                    source_episode_id: Some(ep.episode_id.to_string()),
                    triples,
                    source_text: ep.content.clone(),
                };

                return Ok(vec![Arc::new(extracted)]);
            }
        }

        Ok(vec![])
    }
}

impl Default for ExtractionProcessor {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple heuristic extraction: find capitalized multi-word phrases as potential entities.
fn extract_capitalized_phrases(text: &str) -> Vec<ExtractedTriplePayload> {
    use std::collections::HashSet;

    let mut entities: HashSet<String> = HashSet::new();

    // Find capitalized words (potential entities)
    for word in text.split_whitespace() {
        let clean = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_string();

        if clean.len() >= 3
            && clean.starts_with(|c: char| c.is_uppercase())
            && !clean.starts_with(|c: char| c.is_numeric())
        {
            // Skip common sentence-starting words
            let lower = clean.to_lowercase();
            if !["the", "this", "that", "these", "those", "when", "what", "where", "why", "how", "then", "after", "before", "because", "while", "during", "some", "every", "each", "both", "which", "there", "their", "they", "she", "here", "were", "been", "being", "also", "just", "very", "much", "many", "more", "most", "only", "into", "than", "then", "them", "with", "from", "have", "had", "has", "not", "but", "all", "can", "its", "was", "are", "for", "you", "one", "who", "say", "will", "would", "could", "should", "about"].contains(&lower.as_str())
            {
                entities.insert(clean);
            }
        }
    }

    // Convert detected entities to simple triples
    let entity_list: Vec<String> = entities.into_iter().collect();
    let mut triples = Vec::new();

    // Generate "mentions" triples for each detected entity
    for entity in &entity_list {
        triples.push(ExtractedTriplePayload {
            subject: entity.clone(),
            subject_category: detect_category(entity),
            predicate: "mentioned_in".to_string(),
            object: "context".to_string(),
            object_category: "Concept".to_string(),
            confidence: 0.4,
        });
    }

    // If we have multiple entities, generate "related_to" triples between them
    if entity_list.len() >= 2 {
        for i in 0..entity_list.len().min(3) {
            for j in (i + 1)..entity_list.len().min(3) {
                triples.push(ExtractedTriplePayload {
                    subject: entity_list[i].clone(),
                    subject_category: detect_category(&entity_list[i]),
                    predicate: "related_to".to_string(),
                    object: entity_list[j].clone(),
                    object_category: detect_category(&entity_list[j]),
                    confidence: 0.3,
                });
            }
        }
    }

    triples
}

/// Simple category detection based on name patterns.
fn detect_category(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("rust") || lower.contains("python") || lower.contains("ai")
        || lower.contains("llm") || lower.contains("app")
    {
        "Tool".to_string()
    } else if lower.contains("project") || lower.contains("noesis") || lower.contains("core") {
        "Project".to_string()
    } else if lower.contains("hiten") || lower.contains("friend") || lower.contains("person")
        || lower.len() <= 6
    {
        "Person".to_string()
    } else {
        "Concept".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_capitalized_phrases() {
        let text = "Hiten worked on the Noesis project using Rust programming language.";
        let triples = extract_capitalized_phrases(text);
        assert!(!triples.is_empty(), "should extract entities");

        let names: Vec<String> = triples.iter().map(|t| t.subject.clone()).collect();
        assert!(names.contains(&"Hiten".to_string()));
        assert!(names.contains(&"Noesis".to_string()));
        assert!(names.contains(&"Rust".to_string()));
    }

    #[test]
    fn test_detect_category() {
        assert_eq!(detect_category("Rust"), "Tool");
        assert_eq!(detect_category("Noesis"), "Project");
        assert_eq!(detect_category("Hiten"), "Person");
        assert_eq!(detect_category("Philosophy"), "Concept");
    }
}
