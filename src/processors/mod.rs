// Architecture-aligned layout: processors live in their respective field directories.
// Each field's processors/mod.rs self-declares its modules; this file re-exports
// them under short aliases so existing imports (noesis::processors::*) work unchanged.

// --- Memory-processors ---
pub use crate::fields::memory::processors::episode_processor as episode;
pub use crate::fields::memory::processors::extraction_processor as extraction;
pub use crate::fields::memory::processors::consolidation_processor as consolidation;
pub use crate::fields::memory::processors::resolution_processor as resolution;
pub use crate::fields::memory::processors::context_processor as context;
pub use crate::fields::memory::processors::retrieval_processor as retrieval;
pub use crate::fields::memory::processors::indexing_processor as indexing;
pub use crate::fields::memory::processors::decay_processor as decay;
pub use crate::fields::memory::processors::dedup_processor as dedup;

// --- Identity processors ---
pub use crate::fields::identity::processors::belief_processor as belief;
pub use crate::fields::identity::processors::identity_processor as identity;
pub use crate::fields::identity::processors::value_processor as values;
pub use crate::fields::identity::processors::principle_processor as principles;

// --- Agency processors ---
pub use crate::fields::agency::processors::goal_processor as goal;
pub use crate::fields::agency::processors::priority_processor as priority;
pub use crate::fields::agency::processors::strategy_processor as strategy;
pub use crate::fields::agency::processors::opportunity_processor as opportunity;

// --- Simulation processors ---
pub use crate::fields::simulation::processors::world_model_processor as world_model;
pub use crate::fields::simulation::processors::assumption_processor as assumption;
pub use crate::fields::simulation::processors::scenario_processor as scenario;
pub use crate::fields::simulation::processors::counterfactual_processor as counterfactual;
pub use crate::fields::simulation::processors::forecasting_processor as forecast;
pub use crate::fields::simulation::processors::simulation_risk_processor as simulation_risk;

// --- Action processors ---
pub use crate::fields::action::processors::plan_processor as planning;
pub use crate::fields::action::processors::project_processor as project;
pub use crate::fields::action::processors::task_processor as task;
pub use crate::fields::action::processors::execution_processor as execution;
pub use crate::fields::action::processors::evaluation_processor as evaluation;
pub use crate::fields::action::processors::risk_processor as risk;
pub use crate::fields::action::processors::recovery_processor as recovery;

// --- Awareness processors ---
pub use crate::fields::awareness::processors::attention_processor as attention;
pub use crate::fields::awareness::processors::curiosity_processor as curiosity;
pub use crate::fields::awareness::processors::narrative_processor as narrative;
pub use crate::fields::awareness::processors::reflection_processor as reflection;
pub use crate::fields::awareness::processors::observer_processor as observer;
pub use crate::fields::awareness::processors::mood_processor as mood;
pub use crate::fields::awareness::processors::health_processor as health;
pub use crate::fields::awareness::processors::pattern_processor as pattern;
pub use crate::fields::awareness::processors::open_loops_processor as open_loops;

// --- Reasoning processors ---
pub use crate::fields::reasoning::processors::metacognition_processor as metacognition;
pub use crate::fields::reasoning::processors::epistemic_processor as epistemic;
pub use crate::fields::reasoning::processors::confidence_processor as confidence;
pub use crate::fields::reasoning::processors::reasoning_processor as reasoning;
pub use crate::fields::reasoning::processors::mental_model_processor as mental_model;
pub use crate::fields::reasoning::processors::decision_processor as decision;
pub use crate::fields::reasoning::processors::hypothesis_processor as hypothesis;
pub use crate::fields::reasoning::processors::analogy_processor as analogy;
pub use crate::fields::reasoning::processors::synthesis_processor as synthesis;
pub use crate::fields::reasoning::processors::concept_processor as concept;
