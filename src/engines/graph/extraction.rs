//! LLM-based triple extraction from text.
//!
//! Takes raw text and extracts (subject, predicate, object) triples
//! using an LLM. Returns structured triples with confidence scores
//! and entity categories. Mirrors curlyos-core's knowledge extraction pipeline.

use serde::{Deserialize, Serialize};

use super::types::{EntityCategory, Triple};
use crate::engines::llm::{CompletionRequest, Message, ModelTier, TieredRouter};
use crate::engines::llm::types::LLMError;

/// System prompt for triple extraction.
const EXTRACTION_SYSTEM: &str = r#"You extract knowledge triples from text about Hiten.

Extract (subject, predicate, object) triples where:
- subject: a named entity (person, project, tool, concept, etc.)
- predicate: a relationship between subject and object (use present tense: works_on, uses, part_of, related_to, located_in, knows, created, owns)
- object: a named entity
- subject_category: one of Person, Project, Tool, Concept, Health, Organization, Location, Event, Resource
- object_category: same options
- confidence: 0.0-1.0 (how certain you are this triple is grounded in the text)

Rules:
- Only extract triples explicitly stated or clearly implied in the text
- Use the same name format consistently for the same entity
- Prefer specific names over pronouns
- If no triples can be extracted, return an empty array

Reply ONLY JSON, no prose:
{"triples": [{"subject": "...", "subject_category": "...", "predicate": "...", "object": "...", "object_category": "...", "confidence": 0.9}]}"#;

/// Result of a triple extraction.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ExtractionResult {
    pub triples: Vec<ExtractedTriple>,
}

/// A triple extracted by the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedTriple {
    pub subject: String,
    pub subject_category: String,
    pub predicate: String,
    pub object: String,
    pub object_category: String,
    pub confidence: f32,
}

/// Extract (subject, predicate, object) triples from text using the LLM.
pub async fn extract_triples(
    router: &mut TieredRouter,
    text: &str,
) -> Result<Vec<Triple>, LLMError> {
    let request = CompletionRequest::new(
        "extraction", // model is overridden by chain
        vec![
            Message::system(EXTRACTION_SYSTEM),
            Message::user(text),
        ],
    )
    .with_temperature(0.0)
    .with_max_tokens(2048);

    let response = router.complete(ModelTier::Fast, request).await?;

    let result: ExtractionResult = crate::engines::llm::extract::first_json(&response.content)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    let triples: Vec<Triple> = result
        .triples
        .into_iter()
        .filter(|t| t.confidence > 0.3)
        .map(|t| {
            let subject_cat = parse_category(&t.subject_category);
            let object_cat = parse_category(&t.object_category);
            Triple {
                subject: t.subject,
                subject_category: subject_cat,
                predicate: t.predicate.to_lowercase(),
                object: t.object,
                object_category: object_cat,
                confidence: t.confidence,
            }
        })
        .collect();

    Ok(triples)
}

/// Parse a category string into an EntityCategory.
fn parse_category(s: &str) -> EntityCategory {
    match s.to_lowercase().trim() {
        "person" => EntityCategory::Person,
        "project" => EntityCategory::Project,
        "tool" => EntityCategory::Tool,
        "concept" => EntityCategory::Concept,
        "health" => EntityCategory::Health,
        "organization" => EntityCategory::Organization,
        "location" => EntityCategory::Location,
        "event" => EntityCategory::Event,
        "resource" => EntityCategory::Resource,
        other => EntityCategory::Other(other.to_string()),
    }
}

/// Simple entity resolution: merge entities with similar names.
///
/// Uses lowercase normalization and alias accumulation.
/// A full implementation would use embedding similarity + co-occurrence.
pub fn resolve_entities(triples: Vec<Triple>) -> Vec<Triple> {
    // Currently a pass-through — resolution happens at the field level.
    // In a full implementation, this would:
    // 1. Normalize entity names (lowercase, strip punctuation)
    // 2. Check against known entities in the graph field
    // 3. Merge duplicates, accumulate aliases
    // 4. Return resolved triples with canonical entity IDs
    triples
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_category() {
        assert_eq!(parse_category("Person"), EntityCategory::Person);
        assert_eq!(parse_category("TOOL"), EntityCategory::Tool);
        assert_eq!(parse_category("unknown"), EntityCategory::Other("unknown".to_string()));
    }

    #[test]
    fn test_extraction_prompt_format() {
        // Verify the system prompt is well-formed
        assert!(EXTRACTION_SYSTEM.contains("triple"));
        assert!(EXTRACTION_SYSTEM.contains("subject_category"));
        assert!(EXTRACTION_SYSTEM.contains("confidence"));
    }
}
