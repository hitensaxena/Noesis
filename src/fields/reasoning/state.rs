use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// A single reasoning chain — a sequence of logical steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningChain {
    pub id: Uuid,
    pub premises: Vec<String>,
    pub conclusion: String,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
}

/// A mental model — a simplified representation of a domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentalModel {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub last_updated: DateTime<Utc>,
}

/// A metacognitive insight — awareness of own cognitive state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetacognitiveInsight {
    pub id: Uuid,
    pub insight: String,
    pub confidence: f32,
    pub category: String, // "gap", "bias", "pattern", "uncertainty"
    pub created_at: DateTime<Utc>,
}

/// A formal decision with supporting evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: Uuid,
    pub choice: String,
    pub alternatives: Vec<String>,
    pub reasoning: String,
    pub outcome: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// A hypothesis — an untested proposition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub id: Uuid,
    pub proposition: String,
    pub supporting_evidence: Vec<String>,
    pub contradicting_evidence: Vec<String>,
    pub status: HypothesisStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HypothesisStatus {
    Proposed,
    Testing,
    Supported,
    Refuted,
}

/// An analogy — mapping between two domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analogy {
    pub id: Uuid,
    pub source_domain: String,
    pub target_domain: String,
    pub mapping: String,
    pub strength: f32,
}

/// Epistemic classification of a statement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicClassification {
    pub id: Uuid,
    pub statement: String,
    pub classification: EpistemicClass,
    pub confidence: f32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EpistemicClass {
    Known,      // Directly observed or verified
    Inferred,   // Derived from reasoning
    Believed,   // Accepted but not verified
    Uncertain,  // Insufficient evidence
    Speculative, // Conjecture or hypothesis
}

/// Synthesized knowledge — combinations of related facts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Synthesis {
    pub id: Uuid,
    pub topic: String,
    pub synthesis: String,
    pub source_facts: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// A concept — a recurring abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Concept {
    pub id: Uuid,
    pub name: String,
    pub definition: String,
    pub instances: Vec<String>,
    pub relations: Vec<String>,
}

/// State of the Reasoning Field.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReasoningFieldState {
    pub reasoning_chains: Vec<ReasoningChain>,
    pub mental_models: Vec<MentalModel>,
    pub metacognitive_insights: Vec<MetacognitiveInsight>,
    pub decisions: Vec<Decision>,
    pub hypotheses: Vec<Hypothesis>,
    pub analogies: Vec<Analogy>,
    pub epistemic_classifications: Vec<EpistemicClassification>,
    pub syntheses: Vec<Synthesis>,
    pub concepts: Vec<Concept>,
    pub chain_count: usize,
    pub insight_count: usize,
    pub decision_count: usize,
}
