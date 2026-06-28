//! GET /api/memory/detail — deep memory observability.
//!
//! Returns a structured view of the memory field across all memory systems:
//! working, episodic, semantic, procedural, graph, retrieval, consolidation, indexing.

use axum::{Json, extract::State};
use serde_json::{json, Value};

use crate::interfaces::rest::ApiState;

/// GET /api/memory/detail — full memory system breakdown.
pub async fn memory_detail(
    State(state): State<ApiState>,
) -> Json<Value> {
    let memory_state = state.field_cache.get("memory");
    let graph_state = state.field_cache.get("knowledge_graph");
    let system = state.system_state;
    let total_signals = system.signal_count();

    // Extract available fields from memory cache
    let (episodes, memories, episode_count, memory_count) = if let Some(cached) = &memory_state {
        let v = cached.value();
        (
            v.get("episodes").cloned().unwrap_or(json!([])),
            v.get("memories").cloned().unwrap_or(json!([])),
            v.get("episode_count").and_then(|c| c.as_u64()).unwrap_or(0),
            v.get("memory_count").and_then(|c| c.as_u64()).unwrap_or(0),
        )
    } else {
        (json!([]), json!([]), 0, 0)
    };

    // Extract graph info from memory field state (stored via MemoryField::handle_signal)
    let (graph_entities, graph_relations) = if let Some(cached) = &memory_state {
        let v = cached.value();
        (
            v.get("knowledge_entities").cloned().unwrap_or_else(|| {
                // Fallback: try the graph field cache
                graph_state.as_ref().and_then(|c| c.value().get("entities").cloned()).unwrap_or(json!([]))
            }),
            v.get("knowledge_relations").cloned().unwrap_or_else(|| {
                graph_state.as_ref().and_then(|c| c.value().get("relations").cloned()).unwrap_or(json!([]))
            }),
        )
    } else if let Some(cached) = &graph_state {
        let v = cached.value();
        (
            v.get("entities").cloned().unwrap_or(json!([])),
            v.get("relations").cloned().unwrap_or(json!([])),
        )
    } else {
        (json!([]), json!([]))
    };

    Json(json!({
        "working": {
            "count": 0,
            "items": [],
            "capacity": 7,
            "note": "Working memory not yet active — will store active context chunks during signal processing.",
        },
        "episodic": {
            "count": episode_count,
            "items": episodes,
            "note": "Episodes recorded by the episode processor from external ingest. Each episode has content, source, timestamp, and tags.",
        },
        "semantic": {
            "count": memory_count,
            "items": memories,
            "note": "Semantic memories created by the consolidation processor from episode clusters. Represents general knowledge extracted from experiences.",
        },
        "procedural": {
            "count": 0,
            "items": [],
            "note": "Procedural memory not yet active — will store learned sequences and how-to knowledge from repeated patterns.",
        },
        "graph": {
            "entities": graph_entities.as_array().map(|a| a.len()).unwrap_or(0),
            "relations": graph_relations.as_array().map(|a| a.len()).unwrap_or(0),
            "items": json!({
                "entities": graph_entities,
                "relations": graph_relations,
            }),
            "note": "Graph memory from the knowledge graph field. Entities extracted by the extraction/resolution processors.",
        },
        "retrieval": {
            "last_query": null,
            "last_results": null,
            "mode": "simple-content-match",
            "k_default": 6,
            "note": "Basic content-match retrieval active via /api/memories. Hybrid (BM25 + vector + graph) retrieval planned.",
        },
        "consolidation": {
            "status": "active",
            "last_run": null,
            "episodes_processed": episode_count,
            "memories_generated": memory_count,
            "pattern_count": 0,
            "note": "Consolidation processor runs every 3 episodes (fast) and every 10 episodes (deep). Creates MemConsolidated and PatternDetected signals.",
        },
        "indexing": {
            "status": "basic",
            "by_source": {},
            "by_tag": {},
            "note": "Basic indexing by source and tag available. Full inverted index and vector embeddings planned.",
        },
        "_meta": {
            "domain": "memory",
            "cached": memory_state.is_some(),
            "signals_processed": total_signals,
            "data_available": episode_count > 0,
        }
    }))
}
