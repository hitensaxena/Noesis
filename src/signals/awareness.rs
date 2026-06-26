use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::kernel::signal::SignalMeta;
use crate::signals::types;
use crate::signals::signal_impl;

/// The system's attention shifted to a new focus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttentionShifted {
    pub meta: SignalMeta,
    pub previous_focus: Option<String>,
    pub new_focus: String,
    pub salience: f32,
    pub reason: String,
}

impl AttentionShifted {
    pub fn new(new_focus: &str, salience: f32, reason: &str) -> Self {
        Self {
            meta: SignalMeta::new(types::ATTENTION_SHIFTED, "noesis::signals"),
            previous_focus: None,
            new_focus: new_focus.to_string(),
            salience,
            reason: reason.to_string(),
        }
    }
}

signal_impl!(AttentionShifted, ATTENTION_SHIFTED, "noesis::signals");

/// A knowledge gap was detected (curiosity trigger).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuriosityDetected {
    pub meta: SignalMeta,
    pub curiosity_id: Uuid,
    pub topic: String,
    pub gap_description: String,
    pub intensity: f32,
}

impl CuriosityDetected {
    pub fn new(topic: &str, gap: &str, intensity: f32) -> Self {
        Self {
            meta: SignalMeta::new(types::CURIOSITY_DETECTED, "noesis::signals"),
            curiosity_id: Uuid::new_v4(),
            topic: topic.to_string(),
            gap_description: gap.to_string(),
            intensity,
        }
    }
}

signal_impl!(CuriosityDetected, CURIOSITY_DETECTED, "noesis::signals");

/// A coherent narrative was generated from a sequence of episodes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeGenerated {
    pub meta: SignalMeta,
    pub narrative_id: Uuid,
    pub title: String,
    pub summary: String,
    pub episode_count: usize,
    pub themes: Vec<String>,
}

signal_impl!(NarrativeGenerated, NARRATIVE_GENERATED, "noesis::signals");

/// A raw text ingested from an external source.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestRequest {
    pub meta: SignalMeta,
    pub text: String,
    pub source: String,
    pub tags: Vec<String>,
}

impl IngestRequest {
    pub fn new(text: &str, source: &str) -> Self {
        Self {
            meta: SignalMeta::new(types::INGEST_REQUEST, "noesis::signals"),
            text: text.to_string(),
            source: source.to_string(),
            tags: Vec::new(),
        }
    }
}

signal_impl!(IngestRequest, INGEST_REQUEST, "noesis::signals");

/// A state transition was observed by the observer processor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObserverTransitionDetected {
    pub meta: SignalMeta,
    pub transition_id: Uuid,
    pub signal_type: String,
    pub source: String,
    pub depth: u32,
    pub activation: f32,
    pub salience: f32,
}

impl ObserverTransitionDetected {
    pub fn new(signal_type: &str, source: &str, depth: u32, activation: f32, salience: f32) -> Self {
        Self {
            meta: SignalMeta::new(types::OBSERVER_TRANSITION_DETECTED, "noesis::signals"),
            transition_id: Uuid::new_v4(),
            signal_type: signal_type.to_string(),
            source: source.to_string(),
            depth,
            activation,
            salience,
        }
    }
}

signal_impl!(ObserverTransitionDetected, OBSERVER_TRANSITION_DETECTED, "noesis::signals");

/// A mood estimate derived from recent signal patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodEstimated {
    pub meta: SignalMeta,
    pub mood_id: Uuid,
    pub mood: String,           // "focused", "curious", "uncertain", "engaged", "reflective"
    pub intensity: f32,         // 0.0–1.0
    pub signal_volume: usize,   // signals in the observation window
    pub confidence: f32,
}

impl MoodEstimated {
    pub fn new(mood: &str, intensity: f32, signal_volume: usize, confidence: f32) -> Self {
        Self {
            meta: SignalMeta::new(types::MOOD_ESTIMATED, "noesis::signals"),
            mood_id: Uuid::new_v4(),
            mood: mood.to_string(),
            intensity,
            signal_volume,
            confidence,
        }
    }
}

signal_impl!(MoodEstimated, MOOD_ESTIMATED, "noesis::signals");
