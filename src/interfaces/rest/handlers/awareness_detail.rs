//! GET /api/awareness/detail — deep awareness observability.
//!
//! Returns a structured view of the awareness field:
//! observer, attention, analytics, replay, health, curiosity.

use axum::{Json, extract::State};
use serde_json::{json, Value};

use crate::interfaces::rest::ApiState;

/// GET /api/awareness/detail — full awareness system breakdown.
pub async fn awareness_detail(
    State(state): State<ApiState>,
) -> Json<Value> {
    let aware_state = state.field_cache.get("awareness");
    let system = state.system_state;
    let total_signals = system.signal_count();

    // Extract available fields from awareness cache
    let (focus_stack, salience_map) = if let Some(cached) = &aware_state {
        let v = cached.value();
        (
            v.get("focus_stack").cloned().unwrap_or(json!([])),
            v.get("salience_map").cloned().unwrap_or(json!({})),
        )
    } else {
        (json!([]), json!({}))
    };

    let metrics = state.metrics.snapshot();
    let signals_processed = metrics.get("signals")
        .and_then(|s| s.as_u64())
        .unwrap_or(0);

    Json(json!({
        "observer": {
            "status": "active",
            "perspective": "first-person",
            "field_of_view": focus_stack.as_array().map(|a| a.len()).unwrap_or(0),
            "note": "Observer is the system's self-awareness layer. Currently focuses on attention tracking.",
        },
        "attention": {
            "current_focus": focus_stack.as_array()
                .and_then(|a| a.first())
                .and_then(|f| f.get("topic").and_then(|t| t.as_str().map(|s| s.to_string())))
                .unwrap_or_else(|| "none".to_string()),
            "focus_stack": {
                "depth": focus_stack.as_array().map(|a| a.len()).unwrap_or(0),
                "items": focus_stack,
            },
            "salience_map": {
                "entries": salience_map.as_object().map(|o| o.len()).unwrap_or(0),
                "items": salience_map,
            },
            "note": "Attention managed by the attention processor. Tracks shifting focus based on new episodes and curiosity signals.",
        },
        "analytics": {
            "signals_observed": signals_processed,
            "attention_shifts": 0,
            "focus_duration_avg_secs": null,
            "note": "Awareness analytics not yet active — will track attention patterns, focus duration, and switching costs.",
        },
        "replay": {
            "count": 0,
            "items": [],
            "note": "Experience replay not yet active — will replay recent episodes during idle cycles for offline consolidation.",
        },
        "health": {
            "status": "nominal",
            "signal_throughput": signals_processed,
            "cascade_depth": null,
            "processor_latency_p95_us": null,
            "note": "Health monitoring active at system level. Per-processor health metrics will be tracked as awareness evolves.",
        },
        "curiosity": {
            "count": 0,
            "items": [],
            "note": "Curiosity engine runs periodically (every 5 episodes) via the curiosity processor. Detects knowledge gaps from episode patterns.",
        },
        "_meta": {
            "domain": "awareness",
            "cached": aware_state.is_some(),
            "signals_processed": total_signals,
            "data_available": focus_stack.as_array().map(|a| !a.is_empty()).unwrap_or(false),
        }
    }))
}
