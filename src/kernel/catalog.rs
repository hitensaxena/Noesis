//! CLOSED event catalog — single source of truth for event types.
//!
//! Mirrors curlyos-core's `shared/events/catalog.py`.
//! CLOSED means closed: every event type must be registered here.
//! Adding a type is a deliberate one-line registration, never string improv.

use std::collections::HashMap;
use std::sync::LazyLock;

/// The full event catalog: short_type → domain group.
static EVENT_CATALOG: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        // ── memory / identity / knowledge ────────────────────────────────────
        m.insert("memory.episode.recorded", "MEMORY");
        m.insert("memory.fact.stored", "MEMORY");
        m.insert("memory.fact.consolidated", "MEMORY");
        m.insert("memory.fact.invalidated", "MEMORY");
        m.insert("identity.fact.updated", "MEMORY");
        m.insert("knowledge.entity.created", "MEMORY");
        m.insert("knowledge.entity.invalidated", "MEMORY");
        m.insert("knowledge.edge.created", "MEMORY");
        m.insert("knowledge.edge.invalidated", "MEMORY");
        // ── cognition ────────────────────────────────────────────────────────
        m.insert("metacog.assumption.created", "MEMORY");
        m.insert("metacog.model.created", "MEMORY");
        m.insert("cognition.reflection.completed", "MEMORY");
        m.insert("cognition.audit.completed", "MEMORY");
        m.insert("cognition.meta.models_generated", "MEMORY");
        m.insert("memory.consolidation.fast", "MEMORY");
        m.insert("memory.consolidation.deep", "MEMORY");
        // ── creative / exploration ────────────────────────────────────────────
        m.insert("studio.created", "EVENTS");
        m.insert("studio.sketch.created", "EVENTS");
        m.insert("studio.sketch.updated", "EVENTS");
        m.insert("studio.sketch.invalidated", "EVENTS");
        m.insert("studio.sketch.graduated", "EVENTS");
        m.insert("studio.sketches.linked", "EVENTS");
        m.insert("simulation.run.created", "EVENTS");
        m.insert("simulation.run.completed", "EVENTS");
        m.insert("simulation.run.forked", "EVENTS");
        // ── goal OS ──────────────────────────────────────────────────────────
        m.insert("agency.goals.created", "EVENTS");
        m.insert("goal.updated", "EVENTS");
        m.insert("goal.invalidated", "EVENTS");
        m.insert("goal.derived", "EVENTS");
        m.insert("goal.plan.proposed", "EVENTS");
        m.insert("goal.plan.approved", "EVENTS");
        m.insert("goal.task.dispatched", "EVENTS");
        m.insert("goal.task.verified", "EVENTS");
        m.insert("goal.task.retry", "EVENTS");
        m.insert("goal.progress", "EVENTS");
        m.insert("goal.achieved", "EVENTS");
        m.insert("goal.needs_work", "EVENTS");
        m.insert("decision.recorded", "EVENTS");
        m.insert("decision.reviewed", "EVENTS");
        m.insert("opportunity.detected", "EVENTS");
        m.insert("opportunity.resolved", "EVENTS");
        // ── agent runtime ────────────────────────────────────────────────────
        m.insert("agent.run.started", "AGENTS");
        m.insert("agent.run.completed", "AGENTS");
        m.insert("agent.run.failed", "AGENTS");
        m.insert("runtime.action.executed", "AGENTS");
        m.insert("runtime.observation.recorded", "AGENTS");
        m.insert("tool.call.invoked", "AGENTS");
        // ── evolution ────────────────────────────────────────────────────────
        m.insert("evolution.candidate.proposed", "EVOLUTION");
        m.insert("evolution.eval.completed", "EVOLUTION");
        m.insert("evolution.candidate.held", "EVOLUTION");
        m.insert("evolution.prompt.activated", "EVOLUTION");
        // ── safety ───────────────────────────────────────────────────────────
        m.insert("safety.approval.requested", "SAFETY");
        m.insert("safety.approval.granted", "SAFETY");
        m.insert("safety.approval.denied", "SAFETY");
        m.insert("safety.approval.expired", "SAFETY");
        m.insert("safety.kill.triggered", "SAFETY");
        m.insert("safety.budget.exceeded", "SAFETY");
        m.insert("safety.pdp.unavailable", "SAFETY");
        // ── Noesis-native signals (new) ──────────────────────────────────────
        m.insert("noesis.signal.emitted", "EVENTS");
        m.insert("noesis.cascade.equilibrium", "EVENTS");
        m.insert("noesis.processor.fired", "EVENTS");
        m
    });

/// Resolve the domain group for a short type.
///
/// # Panics
/// Panics if the type is not in the catalog (use `validate_short_type` first).
pub fn group_for(short_type: &str) -> &'static str {
    EVENT_CATALOG
        .get(short_type)
        .copied()
        .unwrap_or_else(|| panic!("event type not in catalog: {}", short_type))
}

/// Check whether a short type is registered in the catalog.
pub fn is_known(short_type: &str) -> bool {
    EVENT_CATALOG.contains_key(short_type)
}

/// Validate a short type against the catalog.
/// Returns `Ok(())` if known, `Err(msg)` if unknown.
pub fn validate_short_type(short_type: &str) -> Result<(), String> {
    if EVENT_CATALOG.contains_key(short_type) {
        Ok(())
    } else {
        Err(format!(
            "event type not in the closed catalog: {:?} — register it in catalog.rs",
            short_type
        ))
    }
}

/// List all registered short types.
pub fn list_types() -> Vec<&'static str> {
    let mut types: Vec<&'static str> = EVENT_CATALOG.keys().copied().collect();
    types.sort();
    types
}

/// List all domain groups.
pub fn list_groups() -> Vec<&'static str> {
    let mut groups: Vec<&'static str> =
        EVENT_CATALOG.values().copied().collect();
    groups.sort();
    groups.dedup();
    groups
}

/// Count events per domain group.
pub fn count_by_group() -> HashMap<&'static str, usize> {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for group in EVENT_CATALOG.values() {
        *counts.entry(group).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_has_events() {
        let types = list_types();
        assert!(types.len() > 50, "catalog should have 50+ event types");
    }

    #[test]
    fn test_groups() {
        let groups = list_groups();
        assert!(groups.contains(&"MEMORY"));
        assert!(groups.contains(&"EVENTS"));
        assert!(groups.contains(&"AGENTS"));
        assert!(groups.contains(&"SAFETY"));
        assert!(groups.contains(&"EVOLUTION"));
    }

    #[test]
    fn test_known_types() {
        assert!(is_known("memory.fact.stored"));
        assert!(is_known("safety.kill.triggered"));
        assert!(!is_known("nonexistent.type"));
    }

    #[test]
    fn test_validate() {
        assert!(validate_short_type("memory.episode.recorded").is_ok());
        assert!(validate_short_type("fake.type").is_err());
    }

    #[test]
    fn test_group_for() {
        assert_eq!(group_for("safety.approval.granted"), "SAFETY");
        assert_eq!(group_for("memory.fact.stored"), "MEMORY");
    }

    #[test]
    fn test_count_by_group() {
        let counts = count_by_group();
        assert!(counts.get("MEMORY").unwrap_or(&0) > &10);
    }
}
