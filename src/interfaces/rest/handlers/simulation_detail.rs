//! GET /api/simulation/detail — deep simulation observability.
//!
//! Returns a structured view of the simulation field:
//! assumptions, world-models, forecasting, risk, counterfactuals, scenarios.

use axum::{Json, extract::State};
use serde_json::{json, Value};

use crate::interfaces::rest::ApiState;

/// GET /api/simulation/detail — full simulation system breakdown.
pub async fn simulation_detail(
    State(state): State<ApiState>,
) -> Json<Value> {
    let sim_state = state.field_cache.get("simulation");
    let system = state.system_state;
    let total_signals = system.signal_count();

    let (scenarios, _metrics_used) = if let Some(cached) = &sim_state {
        let v = cached.value();
        let scenarios = v.get("scenarios").cloned().unwrap_or(json!([]));
        let metrics_value = v.get("_metrics").cloned().unwrap_or(json!({}));
        (scenarios, metrics_value)
    } else {
        (json!([]), json!({}))
    };

    Json(json!({
        "assumptions": {
            "count": 0,
            "items": [],
            "note": "Assumption tracking not yet active — will inventory assumptions underlying decisions and goals for challenge and validation.",
        },
        "world_models": {
            "count": 0,
            "items": [],
            "note": "World models not yet active — will build compact causal models from episode patterns for counterfactual reasoning.",
        },
        "forecasting": {
            "count": 0,
            "items": [],
            "note": "Forecasting not yet active — will project likely outcomes from current state and active goals using the world model.",
        },
        "risk": {
            "count": 0,
            "items": [],
            "note": "Risk assessment not yet active — will evaluate potential negative outcomes, their probabilities, and mitigation strategies.",
        },
        "counterfactuals": {
            "count": 0,
            "items": [],
            "note": "Counterfactual reasoning not yet active — will explore 'what if' alternatives to past decisions for learning.",
        },
        "scenarios": {
            "count": scenarios.as_array().map(|a| a.len()).unwrap_or(0),
            "items": scenarios,
            "note": "Scenario simulation not yet active — will run what-if simulations with varying parameters. Scenarios field initialized but awaiting simulation processor.",
        },
        "_meta": {
            "domain": "simulation",
            "cached": sim_state.is_some(),
            "signals_processed": total_signals,
            "data_available": scenarios.as_array().map(|a| !a.is_empty()).unwrap_or(false),
        }
    }))
}
