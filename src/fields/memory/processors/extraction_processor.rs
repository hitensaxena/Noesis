//! Extraction processor — LLM-powered triple extraction from episodes.
//!
//! Subscribes to EpisodeRecorded signals and emits TriplesExtracted.
//! Uses the Fast LLM tier when available, falls back to heuristic extraction.

use std::sync::Arc;
use async_trait::async_trait;
use anyhow::Result;
use tracing;

use crate::kernel::signal::{SignalArc, SignalType};
use crate::signals::types;
use crate::signals::graph::{ExtractedTriplePayload, TriplesExtracted};
use crate::signals::EpisodeRecorded;
use crate::processor::processor::Processor;
use crate::field_runtime::context::FieldContext;
use crate::engines::llm::{ModelTier, TieredRouter, CompletionRequest, Message};
use crate::engines::llm::types::LLMError;
use crate::engines::llm::extract::json_records;

/// System prompt for LLM-based triple extraction.
const EXTRACTION_SYSTEM: &str = r#"You are an entity extraction engine. Given a piece of text, extract all entities (people, projects, tools, concepts, etc.) and their relationships.

Return a JSON array of objects with these fields:
- subject: the entity name (as-is from text)
- subject_category: "Person" | "Project" | "Tool" | "Concept" | "Organization" | "Location" | "Event" | "Resource"
- predicate: the relationship type (e.g., "works_on", "uses", "part_of", "created", "related_to", "knows", "mentions")
- object: the related entity name
- object_category: same categories as subject
- confidence: 0.0-1.0

Rules:
- Only extract entities that are explicitly named in the text
- Skip common words, pronouns, and generic concepts
- Categories: Person (individuals), Project (named projects/systems), Tool (languages/frameworks/tools), Concept (abstract ideas), Organization (companies/groups), Location (places), Event (happenings), Resource (data/artifacts)
- Use "related_to" as predicate when the relationship isn't clearly one of the specific types
- Set confidence high (0.8+) when the entity is clearly named and the relationship is explicit

Reply ONLY with the JSON array, no other text.
"#;

/// LLM-powered extraction with heuristic fallback.
pub struct ExtractionProcessor {
    llm: Option<TieredRouter>,
}

impl ExtractionProcessor {
    pub fn new() -> Self {
        let llm = if TieredRouter::has_api_key() {
            match TieredRouter::from_env() {
                Ok(r) => Some(r),
                Err(e) => {
                    tracing::warn!("[Extraction] LLM unavailable: {}", e);
                    None
                }
            }
        } else {
            None
        };
        Self { llm }
    }

    /// Try LLM-based triple extraction.
    async fn llm_extract(&mut self, text: &str) -> Option<Vec<ExtractedTriplePayload>> {
        let router = self.llm.as_mut()?;

        let request = CompletionRequest::new(
            "extraction",
            vec![
                Message::system(EXTRACTION_SYSTEM),
                Message::user(&format!("Text:\n{}", text)),
            ],
        )
        .with_temperature(0.1)
        .with_max_tokens(2048);

        match router.complete(ModelTier::Fast, request).await {
            Ok(resp) => {
                let records = json_records(&resp.content);
                let triples: Vec<ExtractedTriplePayload> = records
                    .iter()
                    .filter_map(|v| {
                        let subject = v.get("subject")?.as_str()?.to_string();
                        let subject_category = v.get("subject_category")
                            .and_then(|c| c.as_str())
                            .unwrap_or("Concept")
                            .to_string();
                        let predicate = v.get("predicate")
                            .and_then(|p| p.as_str())
                            .unwrap_or("mentioned_in")
                            .to_string();
                        let object = v.get("object")?.as_str()?.to_string();
                        let object_category = v.get("object_category")
                            .and_then(|c| c.as_str())
                            .unwrap_or("Concept")
                            .to_string();
                        let confidence = v.get("confidence").and_then(|c| c.as_f64()).unwrap_or(0.5) as f32;
                        Some(ExtractedTriplePayload {
                            subject,
                            subject_category,
                            predicate,
                            object,
                            object_category,
                            confidence,
                        })
                    })
                    .collect();

                if triples.is_empty() {
                    tracing::warn!("[Extraction] LLM returned no valid triples, falling back");
                    None
                } else {
                    tracing::info!("[Extraction] LLM extracted {} triples", triples.len());
                    Some(triples)
                }
            }
            Err(LLMError::RateLimited { retry_after }) => {
                tracing::warn!("[Extraction] rate limited ({}s), falling back to heuristic", retry_after);
                None
            }
            Err(e) => {
                tracing::warn!("[Extraction] LLM error: {}, falling back to heuristic", e);
                None
            }
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

    fn priority(&self) -> u8 {
        110 // After episode processor
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

            // Try LLM extraction first, fall back to heuristic
            let triples = if self.llm.is_some() {
                self.llm_extract(&ep.content).await
                    .unwrap_or_else(|| extract_capitalized_phrases(&ep.content))
            } else {
                extract_capitalized_phrases(&ep.content)
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

// ---------------------------------------------------------------------------
// Heuristic fallback extraction (unchanged from original)
// ---------------------------------------------------------------------------

/// Simple heuristic extraction: find capitalized multi-word phrases as potential entities.
fn extract_capitalized_phrases(text: &str) -> Vec<ExtractedTriplePayload> {
    use std::collections::HashSet;

    let mut entities: HashSet<String> = HashSet::new();

    for word in text.split_whitespace() {
        let clean = word
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_string();

        if clean.len() >= 3
            && clean.starts_with(|c: char| c.is_uppercase())
            && !clean.starts_with(|c: char| c.is_numeric())
        {
            let lower = clean.to_lowercase();
            if !["the", "this", "that", "these", "those", "when", "what", "where", "why", "how",
                 "then", "after", "before", "because", "while", "during", "some", "every", "each",
                 "both", "which", "there", "their", "they", "she", "here", "were", "been", "being",
                 "also", "just", "very", "much", "many", "more", "most", "only", "into", "than",
                 "then", "them", "with", "from", "have", "had", "has", "not", "but", "all", "can",
                 "its", "was", "are", "for", "you", "one", "who", "say", "will", "would", "could",
                 "should", "about"].contains(&lower.as_str())
            {
                entities.insert(clean);
            }
        }
    }

    let entity_list: Vec<String> = entities.into_iter().collect();
    let mut triples = Vec::new();

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
