//! GET /api/agency/detail — deep agency & action observability.
//!
//! Returns a structured view of the agency field:
//! goals, priorities, strategy, plus action stubs for projects/tasks/plans.

use axum::{Json, extract::State};
use serde_json::{json, Value};

use crate::interfaces::rest::ApiState;

/// GET /api/agency/detail — full agency system breakdown.
pub async fn agency_detail(
    State(state): State<ApiState>,
) -> Json<Value> {
    let agency_state = state.field_cache.get("agency");
    let system = state.system_state;
    let total_signals = system.signal_count();

    // Extract available fields from agency cache
    let (goals, pursuits) = if let Some(cached) = &agency_state {
        let v = cached.value();
        (
            v.get("goals").cloned().unwrap_or(json!([])),
            v.get("active_pursuits").cloned().unwrap_or(json!([])),
        )
    } else {
        (json!([]), json!([]))
    };

    let active_goals = goals.as_array()
        .map(|a| a.iter().filter(|g| g.get("status").and_then(|s| s.as_str()) == Some("Active")).count())
        .unwrap_or(0);
    let completed_goals = goals.as_array()
        .map(|a| a.iter().filter(|g| g.get("status").and_then(|s| s.as_str()) == Some("Completed")).count())
        .unwrap_or(0);

    Json(json!({
        "goals": {
            "total": goals.as_array().map(|a| a.len()).unwrap_or(0),
            "active": active_goals,
            "completed": completed_goals,
            "abandoned": goals.as_array()
                .map(|a| a.iter().filter(|g| g.get("status").and_then(|s| s.as_str()) == Some("Abandoned")).count())
                .unwrap_or(0),
            "items": goals,
            "note": "Goals created by the goal processor when identity is updated. Each goal has a description, priority, and status lifecycle.",
        },
        "projects": {
            "count": 0,
            "active": 0,
            "items": [],
            "note": "Project tracking not yet active — will group related goals into project structures with milestones.",
        },
        "tasks": {
            "count": 0,
            "pending": 0,
            "in_progress": 0,
            "completed": 0,
            "items": [],
            "note": "Task decomposition not yet active — planned for the orchestrator engine. Will break goals into executable tasks.",
        },
        "plans": {
            "count": 0,
            "active": 0,
            "items": [],
            "note": "Planning not yet active — will generate execution plans from goals with sequenced steps and contingencies.",
        },
        "active_pursuits": {
            "count": pursuits.as_array().map(|a| a.len()).unwrap_or(0),
            "items": pursuits,
            "note": "Active pursuits tracked by the agency field. Represent immediate commitments the system is acting upon.",
        },
        "opportunities": {
            "count": 0,
            "items": [],
            "note": "Opportunity detection not yet active — will identify favorable conditions for goal advancement from the curiosity engine.",
        },
        "strategy": {
            "count": 0,
            "items": [],
            "note": "Strategic planning not yet active — will evaluate long-term goal alignments and resource allocation.",
        },
        "evaluation": {
            "count": 0,
            "items": [],
            "note": "Goal evaluation not yet active — will assess goal progress, obstacles, and recommend adjustments.",
        },
        "priorities": {
            "count": 0,
            "items": [],
            "note": "Priority management not yet active — will dynamically rank goals and pursuits by urgency, importance, and dependencies.",
        },
        "_meta": {
            "domain": "agency",
            "cached": agency_state.is_some(),
            "signals_processed": total_signals,
            "data_available": goals.as_array().map(|a| !a.is_empty()).unwrap_or(false),
        }
    }))
}
