use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::kernel::signal::SignalMeta;
use crate::signals::types;
use crate::signals::signal_impl;

/// An epistemic classification — categorizing a piece of knowledge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicClassified {
    pub meta: SignalMeta,
    pub classification_id: Uuid,
    pub signal_type: String,
    pub classification: String,
    pub confidence: f32,
}

impl EpistemicClassified {
    pub fn new(signal_type: &str, classification: &str, confidence: f32) -> Self {
        Self {
            meta: SignalMeta::new(types::EPISTEMICS_CLASSIFIED, "reasoning::epistemic"),
            classification_id: Uuid::new_v4(),
            signal_type: signal_type.to_string(),
            classification: classification.to_string(),
            confidence,
        }
    }
}

signal_impl!(EpistemicClassified, EPISTEMICS_CLASSIFIED, "reasoning::epistemic");

/// A metacognitive insight — awareness of own cognitive processes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetacognitionInsight {
    pub meta: SignalMeta,
    pub insight_id: Uuid,
    pub insight: String,
    pub confidence: f32,
    pub category: String,
}

impl MetacognitionInsight {
    pub fn new(insight: &str, confidence: f32, category: &str) -> Self {
        Self {
            meta: SignalMeta::new(types::METACOGNITION_INSIGHT, "noesis::signals"),
            insight_id: Uuid::new_v4(),
            insight: insight.to_string(),
            confidence,
            category: category.to_string(),
        }
    }
}

signal_impl!(MetacognitionInsight, METACOGNITION_INSIGHT, "noesis::signals");

/// An analogy was detected between two domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalogyDetected {
    pub meta: SignalMeta,
    pub analogy_id: Uuid,
    pub source: String,
    pub target: String,
    pub mapping: String,
}

impl AnalogyDetected {
    pub fn new(source: &str, target: &str, mapping: &str) -> Self {
        Self {
            meta: SignalMeta::new(types::ANALOGY_DETECTED, "reasoning::analogy"),
            analogy_id: Uuid::new_v4(),
            source: source.to_string(),
            target: target.to_string(),
            mapping: mapping.to_string(),
        }
    }
}

signal_impl!(AnalogyDetected, ANALOGY_DETECTED, "reasoning::analogy");

/// A concept was formed from clustered entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptFormed {
    pub meta: SignalMeta,
    pub concept_id: Uuid,
    pub name: String,
    pub definition: String,
}

impl ConceptFormed {
    pub fn new(name: &str, definition: &str) -> Self {
        Self {
            meta: SignalMeta::new(types::CONCEPT_FORMED, "reasoning::concept"),
            concept_id: Uuid::new_v4(),
            name: name.to_string(),
            definition: definition.to_string(),
        }
    }
}

signal_impl!(ConceptFormed, CONCEPT_FORMED, "reasoning::concept");

/// A decision was made with a reasoned choice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionMade {
    pub meta: SignalMeta,
    pub decision_id: Uuid,
    pub choice: String,
    pub reasoning: String,
}

impl DecisionMade {
    pub fn new(choice: &str, reasoning: &str) -> Self {
        Self {
            meta: SignalMeta::new(types::DECISION_EVALUATED, "reasoning::decision"),
            decision_id: Uuid::new_v4(),
            choice: choice.to_string(),
            reasoning: reasoning.to_string(),
        }
    }
}

signal_impl!(DecisionMade, DECISION_EVALUATED, "reasoning::decision");

/// A hypothesis was generated from curiosity or belief change.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HypothesisGenerated {
    pub meta: SignalMeta,
    pub hypothesis_id: Uuid,
    pub proposition: String,
}

impl HypothesisGenerated {
    pub fn new(proposition: &str) -> Self {
        Self {
            meta: SignalMeta::new(types::HYPOTHESIS_GENERATED, "reasoning::hypothesis"),
            hypothesis_id: Uuid::new_v4(),
            proposition: proposition.to_string(),
        }
    }
}

signal_impl!(HypothesisGenerated, HYPOTHESIS_GENERATED, "reasoning::hypothesis");

/// A mental model was updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentalModelUpdated {
    pub meta: SignalMeta,
    pub model_id: Uuid,
    pub name: String,
    pub confidence: f32,
}

impl MentalModelUpdated {
    pub fn new(name: &str, confidence: f32) -> Self {
        Self {
            meta: SignalMeta::new(types::MENTAL_MODEL_UPDATED, "reasoning::mental_model"),
            model_id: Uuid::new_v4(),
            name: name.to_string(),
            confidence,
        }
    }
}

signal_impl!(MentalModelUpdated, MENTAL_MODEL_UPDATED, "reasoning::mental_model");

/// A conclusion is ready after reasoning chain completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConclusionReady {
    pub meta: SignalMeta,
    pub conclusion_id: Uuid,
    pub conclusion: String,
    pub confidence: f32,
}

impl ConclusionReady {
    pub fn new(conclusion: &str, confidence: f32) -> Self {
        Self {
            meta: SignalMeta::new(types::CONCLUSION_READY, "reasoning::reasoning"),
            conclusion_id: Uuid::new_v4(),
            conclusion: conclusion.to_string(),
            confidence,
        }
    }
}

signal_impl!(ConclusionReady, CONCLUSION_READY, "reasoning::reasoning");

/// A synthesis of related information is ready.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SynthesisReady {
    pub meta: SignalMeta,
    pub synthesis_id: Uuid,
    pub topic: String,
    pub content: String,
}

impl SynthesisReady {
    pub fn new(topic: &str, content: &str) -> Self {
        Self {
            meta: SignalMeta::new(types::SYNTHESIS_READY, "reasoning::synthesis"),
            synthesis_id: Uuid::new_v4(),
            topic: topic.to_string(),
            content: content.to_string(),
        }
    }
}

signal_impl!(SynthesisReady, SYNTHESIS_READY, "reasoning::synthesis");
