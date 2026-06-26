use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::kernel::signal::SignalMeta;
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

/// Episodes were decayed (removed or marked as stale) on a beat cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryDecayed {
    pub meta: SignalMeta,
    pub decay_id: Uuid,
    pub episodes_removed: usize,
    pub episode_count_before: usize,
    pub episode_count_after: usize,
}

impl MemoryDecayed {
    pub fn new(removed: usize, before: usize, after: usize) -> Self {
        Self {
            meta: SignalMeta::new(types::MEMORY_DECAYED, "memory::decay"),
            decay_id: Uuid::new_v4(),
            episodes_removed: removed,
            episode_count_before: before,
            episode_count_after: after,
        }
    }
}

signal_impl!(MemoryDecayed, MEMORY_DECAYED, "memory::decay");

/// A duplicate episode was detected and skipped.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DedupSkipped {
    pub meta: SignalMeta,
    pub episode_id: Uuid,
    pub original_episode_id: Uuid,
    pub content_hash: String,
}

impl DedupSkipped {
    pub fn new(episode_id: Uuid, original_id: Uuid, content_hash: &str) -> Self {
        Self {
            meta: SignalMeta::new(types::DEDUP_SKIPPED, "memory::dedup"),
            episode_id,
            original_episode_id: original_id,
            content_hash: content_hash.to_string(),
        }
    }
}

signal_impl!(DedupSkipped, DEDUP_SKIPPED, "memory::dedup");

/// Index terms were extracted from a new episode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexUpdated {
    pub meta: SignalMeta,
    pub episode_id: Uuid,
    pub terms: Vec<String>,
}

impl IndexUpdated {
    pub fn new(episode_id: Uuid, terms: Vec<String>) -> Self {
        Self {
            meta: SignalMeta::new(types::INDEX_UPDATED, "memory::indexing"),
            episode_id,
            terms,
        }
    }
}

signal_impl!(IndexUpdated, INDEX_UPDATED, "memory::indexing");

/// A retrieval result from a curiosity-driven recall query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodesRetrieved {
    pub meta: SignalMeta,
    pub query: String,
    pub episode_ids: Vec<Uuid>,
    pub matches: Vec<String>,
}

impl EpisodesRetrieved {
    pub fn new(query: &str, episode_ids: Vec<Uuid>, matches: Vec<String>) -> Self {
        Self {
            meta: SignalMeta::new(types::EPISODES_RETRIEVED, "memory::retrieval"),
            query: query.to_string(),
            episode_ids,
            matches,
        }
    }
}

signal_impl!(EpisodesRetrieved, EPISODES_RETRIEVED, "memory::retrieval");
