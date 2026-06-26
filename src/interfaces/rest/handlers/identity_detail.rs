//! GET /api/identity/detail — deep identity observability.
//!
//! Returns a structured view of the identity field with all sub-domains:
//! identity, self, beliefs, values, traits, roles, preferences, principles, history, timeline.

use axum::{Json, extract::State};
use serde_json::{json, Value};

use crate::interfaces::rest::ApiState;

/// GET /api/identity/detail — full identity breakdown.
pub async fn identity_detail(
    State(state): State<ApiState>,
) -> Json<Value> {
    let identity_state = state.field_cache.get("identity");
    let system = state.system_state;
    let total_signals = system.signal_count();

    // Extract available fields from cache
    let (beliefs, traits, identity_version) = if let Some(cached) = &identity_state {
        let v = cached.value();
        (
            v.get("beliefs").cloned().unwrap_or(json!([])),
            v.get("traits").cloned().unwrap_or(json!([])),
            v.get("identity_version").and_then(|v| v.as_u64()).unwrap_or(0),
        )
    } else {
        (json!([]), json!([]), 0)
    };

    Json(json!({
        "identity": {
            "version": identity_version,
            "label": format!("Noesis v{} — Cognitive Self-Model", identity_version),
            "last_updated": null,
            "description": "Decentralized cognitive architecture identity state. Updated by the identity processor as beliefs and self-model evolve.",
        },
        "self": {
            "narrative": null,
            "coherence": null,
            "integration_level": null,
            "perspectives": [],
        },
        "beliefs": {
            "count": beliefs.as_array().map(|a| a.len()).unwrap_or(0),
            "items": beliefs,
            "note": "Beliefs are created by the belief processor when MemoryConsolidated signals arrive. Each belief has a confidence score and active flag.",
        },
        "values": {
            "count": 0,
            "items": [],
            "note": "Value extraction not yet active. Planned for the reflection processor — extracts core values from episode patterns.",
        },
        "traits": {
            "count": traits.as_array().map(|a| a.len()).unwrap_or(0),
            "items": traits,
            "note": "Traits are detected by the identity processor from accumulated beliefs and behavioral patterns.",
        },
        "roles": {
            "count": 0,
            "items": [],
            "note": "Role detection not yet active. Will be derived from graph entities and relationship patterns.",
        },
        "preferences": {
            "count": 0,
            "items": [],
            "note": "Preference learning not yet active. Planned — inferred from repeated choices in episodes.",
        },
        "principles": {
            "count": 0,
            "items": [],
            "note": "Principle distillation not yet active. Will be extracted by the meta processor from decisions and evaluations.",
        },
        "history": {
            "count": 0,
            "entries": [],
            "note": "Identity history tracking not yet active. Will record significant identity state transitions.",
        },
        "timeline": {
            "count": 0,
            "events": [],
            "note": "Identity timeline not yet active. Will show identity version changes, belief shifts, and trait discoveries over time.",
        },
        "_meta": {
            "domain": "identity",
            "cached": identity_state.is_some(),
            "signals_processed": total_signals,
            "data_available": beliefs.as_array().map(|a| !a.is_empty()).unwrap_or(false),
        }
    }))
}
