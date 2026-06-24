use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::eventbus::signal::SignalMeta;
use crate::signals::types;
use crate::signals::signal_impl;

/// A raw experience was ingested into the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodeRecorded {
    pub meta: SignalMeta,
    pub episode_id: Uuid,
    pub content: String,
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub tags: Vec<String>,
}

impl EpisodeRecorded {
    pub fn new(content: &str, source: &str, tags: Vec<String>) -> Self {
        Self {
            meta: SignalMeta::new(types::EPISODE_RECORDED, source),
            episode_id: Uuid::new_v4(),
            content: content.to_string(),
            source: source.to_string(),
            timestamp: Utc::now(),
            tags,
        }
    }
}

signal_impl!(EpisodeRecorded, EPISODE_RECORDED, "noesis::signals");

/// A fact was extracted from an episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactExtracted {
    pub meta: SignalMeta,
    pub fact_id: Uuid,
    pub episode_id: Uuid,
    pub fact: String,
    pub confidence: f32,
}

signal_impl!(FactExtracted, FACT_EXTRACTED, "noesis::signals");

/// Memory consolidation completed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConsolidated {
    pub meta: SignalMeta,
    pub episode_ids: Vec<Uuid>,
    pub summary: String,
    pub memory_count: usize,
}

signal_impl!(MemoryConsolidated, MEMORY_CONSOLIDATED, "noesis::signals");

/// A recurring pattern was detected across memories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternDetected {
    pub meta: SignalMeta,
    pub pattern_id: Uuid,
    pub description: String,
    pub occurrences: usize,
    pub confidence: f32,
}

signal_impl!(PatternDetected, PATTERN_DETECTED, "noesis::signals");
