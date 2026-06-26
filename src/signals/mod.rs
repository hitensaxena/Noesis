pub mod memory;
pub mod identity;
pub mod agency;
pub mod awareness;
pub mod graph;
pub mod reasoning;

pub use memory::*;
pub use identity::*;
pub use agency::*;
pub use awareness::*;
pub use graph::*;
pub use reasoning::*;

/// All signal type constants in the Noesis system.
pub mod types {
    use crate::kernel::signal::SignalType;

    // Memory — architecture prefix: memory.*
    pub const EPISODE_RECORDED: SignalType = SignalType::new("memory.capture.recorded");
    pub const FACT_EXTRACTED: SignalType = SignalType::new("memory.extraction.fact_extracted");
    pub const MEMORY_CONSOLIDATED: SignalType = SignalType::new("memory.consolidation.consolidated");
    pub const PATTERN_DETECTED: SignalType = SignalType::new("memory.consolidation.pattern_detected");
    pub const ENTITY_CREATED: SignalType = SignalType::new("memory.knowledge.entity_created");
    pub const EDGE_CREATED: SignalType = SignalType::new("memory.knowledge.edge_created");
    pub const TRIPLES_EXTRACTED: SignalType = SignalType::new("memory.knowledge.triples_extracted");
    pub const CONTEXT_ASSEMBLED: SignalType = SignalType::new("memory.context.assembled");
    pub const INGEST_REQUEST: SignalType = SignalType::new("memory.capture.ingested");

    // Identity — architecture prefix: identity.*
    pub const BELIEF_CHANGED: SignalType = SignalType::new("identity.beliefs.changed");
    pub const TRAIT_DETECTED: SignalType = SignalType::new("identity.traits.detected");
    pub const IDENTITY_UPDATED: SignalType = SignalType::new("identity.self.updated");
    pub const VALUES_REFINED: SignalType = SignalType::new("identity.values.refined");
    pub const PRINCIPLES_DERIVED: SignalType = SignalType::new("identity.principles.derived");

    // Agency — architecture prefix: agency.*
    pub const GOAL_CREATED: SignalType = SignalType::new("agency.goals.created");
    pub const GOAL_COMPLETED: SignalType = SignalType::new("agency.goals.completed");
    pub const DECISION_EVALUATED: SignalType = SignalType::new("agency.decision.evaluated");
    pub const PRIORITY_REORDERED: SignalType = SignalType::new("agency.priorities.reordered");
    pub const STRATEGY_UPDATED: SignalType = SignalType::new("agency.strategy.updated");
    pub const OPPORTUNITY_DETECTED: SignalType = SignalType::new("agency.opportunity.detected");

    // Awareness — architecture prefix: awareness.*
    pub const ATTENTION_SHIFTED: SignalType = SignalType::new("awareness.attention.shifted");
    pub const CURIOSITY_DETECTED: SignalType = SignalType::new("awareness.curiosity.detected");
    pub const NARRATIVE_GENERATED: SignalType = SignalType::new("awareness.reflection.narrative");
    pub const OBSERVER_TRANSITION_DETECTED: SignalType = SignalType::new("awareness.observer.transition_detected");
    pub const MOOD_ESTIMATED: SignalType = SignalType::new("awareness.mood.estimated");
    pub const HEALTH_STATUS_CHANGED: SignalType = SignalType::new("awareness.health.status_changed");
    pub const HYPOTHESIS_GENERATED: SignalType = SignalType::new("reasoning.hypothesis.generated");

    // Reasoning — architecture prefix: reasoning.*
    pub const METACOGNITION_INSIGHT: SignalType = SignalType::new("reasoning.metacognition.insight");
    pub const EPISTEMICS_CLASSIFIED: SignalType = SignalType::new("reasoning.epistemics.classified");
    pub const CONCLUSION_READY: SignalType = SignalType::new("reasoning.conclusion.ready");
    pub const MENTAL_MODEL_UPDATED: SignalType = SignalType::new("reasoning.mental_models.updated");
    pub const ANALOGY_DETECTED: SignalType = SignalType::new("reasoning.analogy.detected");
    pub const SYNTHESIS_READY: SignalType = SignalType::new("reasoning.synthesis.ready");
    pub const CONCEPT_FORMED: SignalType = SignalType::new("reasoning.concept.formed");

    // Action — architecture prefix: action.*
    pub const PLANNING_PLAN_READY: SignalType = SignalType::new("action.planning.plan_ready");
    pub const PROJECT_CREATED: SignalType = SignalType::new("action.projects.created");
    pub const TASK_CREATED: SignalType = SignalType::new("action.tasks.created");
    pub const EXECUTION_STARTED: SignalType = SignalType::new("action.execution.started");
    pub const EVALUATION_COMPLETED: SignalType = SignalType::new("action.evaluation.evaluated");
    pub const RISK_ASSESSED: SignalType = SignalType::new("action.risk.assessed");
    pub const RECOVERY_STARTED: SignalType = SignalType::new("action.recovery.started");

    // Simulation — architecture prefix: simulation.*
    pub const WORLD_MODEL_UPDATED: SignalType = SignalType::new("simulation.world_model.updated");
    pub const ASSUMPTION_TESTED: SignalType = SignalType::new("simulation.assumption.tested");
    pub const SCENARIO_READY: SignalType = SignalType::new("simulation.scenario.ready");
    pub const COUNTERFACTUAL_READY: SignalType = SignalType::new("simulation.counterfactual.ready");
    pub const FORECAST_READY: SignalType = SignalType::new("simulation.forecast.ready");
    pub const SIMULATION_RISK_ASSESSED: SignalType = SignalType::new("simulation.risk.assessed");

    // Memory decay / dedup / indexing / retrieval
    pub const MEMORY_DECAYED: SignalType = SignalType::new("memory.decay.decayed");
    pub const DEDUP_SKIPPED: SignalType = SignalType::new("memory.dedup.skipped");
    pub const INDEX_UPDATED: SignalType = SignalType::new("memory.index.updated");
    pub const EPISODES_RETRIEVED: SignalType = SignalType::new("memory.retrieval.retrieved");

    // Kernel scheduler beats
    pub const BEAT_FAST: SignalType = SignalType::new("kernel.scheduler.beat.fast");
    pub const BEAT_MEDIUM: SignalType = SignalType::new("kernel.scheduler.beat.medium");
    pub const BEAT_SLOW: SignalType = SignalType::new("kernel.scheduler.beat.slow");
    pub const BEAT_IMMEDIATE: SignalType = SignalType::new("kernel.scheduler.beat.immediate");
    pub const BEAT_SLEEP: SignalType = SignalType::new("kernel.scheduler.beat.sleep");
    pub const BEAT_OFFLINE: SignalType = SignalType::new("kernel.scheduler.beat.offline");
}

/// Macro to implement the Signal trait for a struct.
macro_rules! signal_impl {
    ($name:ident, $signal_type:ident, $source:expr) => {
        impl crate::kernel::signal::Signal for $name {
            fn signal_type(&self) -> crate::kernel::signal::SignalType {
                crate::signals::types::$signal_type
            }
            fn meta(&self) -> &crate::kernel::signal::SignalMeta {
                &self.meta
            }
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }
    };
}

pub(crate) use signal_impl;
