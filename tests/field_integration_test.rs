//! Field-level integration tests for the Noesis cognitive cascade.
//!
//! Exercises each field's processor chain and verifies cascade results.

use std::sync::Arc;
use std::collections::VecDeque;

use noesis::kernel::beat_coordinator::BeatPulse;
use noesis::kernel::kernel::Kernel;
use noesis::kernel::bus::EventBus;
use noesis::kernel::signal::{SignalArc, SignalMeta, SignalType};
use noesis::field_runtime::context::FieldContext;
use noesis::field_runtime::processor_registry::ProcessorRegistry;
use noesis::signals::types;
use noesis::signals::*;
use noesis::signals::agency::*;
use noesis::storage::memory_store::MemoryStore;

/// Helper: full cascade run.
async fn run_cascade(
    registry: &mut ProcessorRegistry,
    ctx: &FieldContext,
    initial: SignalArc,
) -> Vec<String> {
    let mut queue: VecDeque<SignalArc> = VecDeque::new();
    queue.push_back(initial);
    let mut results = Vec::new();

    while let Some(signal) = queue.pop_front() {
        if results.len() >= 200 { break; }
        results.push(signal.signal_type().to_string());
        let emitted = registry.dispatch(ctx, signal).await;
        for sig in emitted {
            queue.push_back(sig);
        }
    }
    results
}

async fn setup_ctx() -> (Kernel, ProcessorRegistry, FieldContext) {
    let kernel = Kernel::new();
    let storage = Arc::new(MemoryStore::new());
    let ctx = FieldContext::new(kernel.event_bus.clone(), storage);

    // Register signals
    for (sig, desc) in &[
        (types::INGEST_REQUEST, "ingest"),
        (types::EPISODE_RECORDED, "episode"),
        (types::MEMORY_CONSOLIDATED, "consolidation"),
        (types::PATTERN_DETECTED, "pattern"),
        (types::BELIEF_CHANGED, "belief"),
        (types::IDENTITY_UPDATED, "identity"),
        (types::GOAL_CREATED, "goal"),
        (types::GOAL_COMPLETED, "goal completed"),
        (types::ATTENTION_SHIFTED, "attention"),
        (types::CURIOSITY_DETECTED, "curiosity"),
        (types::NARRATIVE_GENERATED, "narrative"),
        (types::DECISION_EVALUATED, "decision"),
        (types::ENTITY_CREATED, "entity"),
        (types::EDGE_CREATED, "edge"),
        (types::TRIPLES_EXTRACTED, "triples"),
        (types::PLANNING_PLAN_READY, "plan"),
        (types::PROJECT_CREATED, "project"),
        (types::TASK_CREATED, "task"),
        (types::EXECUTION_STARTED, "execution"),
        (types::HYPOTHESIS_GENERATED, "hypothesis"),
        (types::CONCLUSION_READY, "conclusion"),
        (types::EPISTEMICS_CLASSIFIED, "epistemic"),
        (types::WORLD_MODEL_UPDATED, "world model"),
        (types::SCENARIO_READY, "scenario"),
        (types::FORECAST_READY, "forecast"),
    ] {
        kernel.registry.register_signal(sig.clone(), desc);
    }

    // Register fields
    kernel.registry.register_field("memory", Box::new(|| Box::new(noesis::fields::memory::MemoryField::new())));
    kernel.registry.register_field("identity", Box::new(|| Box::new(noesis::fields::identity::IdentityField::new())));
    kernel.registry.register_field("agency", Box::new(|| Box::new(noesis::fields::agency::AgencyField::new())));
    kernel.registry.register_field("action", Box::new(|| Box::new(noesis::fields::action::ActionField::new())));
    kernel.registry.register_field("awareness", Box::new(|| Box::new(noesis::fields::awareness::AwarenessField::new())));
    kernel.registry.register_field("reasoning", Box::new(|| Box::new(noesis::fields::reasoning::ReasoningField::new())));
    kernel.registry.register_field("simulation", Box::new(|| Box::new(noesis::fields::simulation::SimulationField::new())));
    kernel.registry.register_field("knowledge_graph", Box::new(|| Box::new(noesis::fields::graph::GraphField::new())));

    // Initialize fields (async, no nested runtime)
    for name in kernel.registry.list_fields() {
        if let Some(mut f) = kernel.registry.create_field(&name) {
            f.init(&ctx).await.unwrap();
        }
    }

    // Register ALL processors
    let mut registry = ProcessorRegistry::new();
    registry.register(Box::new(noesis::processors::episode::EpisodeProcessor::new()));
    registry.register(Box::new(noesis::processors::extraction::ExtractionProcessor::new()));
    registry.register(Box::new(noesis::processors::resolution::EntityResolutionProcessor::new()));
    registry.register(Box::new(noesis::processors::consolidation::ConsolidationProcessor::new()));
    registry.register(Box::new(noesis::processors::belief::BeliefProcessor::new()));
    registry.register(Box::new(noesis::processors::identity::IdentityProcessor::new()));
    registry.register(Box::new(noesis::processors::goal::GoalProcessor::new()));
    registry.register(Box::new(noesis::processors::attention::AttentionProcessor::new()));
    registry.register(Box::new(noesis::processors::curiosity::CuriosityProcessor::new()));
    registry.register(Box::new(noesis::processors::narrative::NarrativeProcessor::new()));
    registry.register(Box::new(noesis::processors::dedup::DedupProcessor::new()));
    registry.register(Box::new(noesis::processors::indexing::IndexingProcessor::new()));
    registry.register(Box::new(noesis::processors::retrieval::RetrievalProcessor::new()));
    registry.register(Box::new(noesis::processors::decay::DecayProcessor::new()));
    registry.register(Box::new(noesis::processors::priority::PriorityProcessor::new()));
    registry.register(Box::new(noesis::processors::strategy::StrategyProcessor::new()));
    registry.register(Box::new(noesis::processors::planning::PlanDecomposer::new()));
    registry.register(Box::new(noesis::processors::project::ProjectProcessor::new()));
    registry.register(Box::new(noesis::processors::task::TaskProcessor::new()));
    registry.register(Box::new(noesis::processors::execution::ExecutionProcessor::new()));
    registry.register(Box::new(noesis::processors::evaluation::EvaluationProcessor::new()));
    registry.register(Box::new(noesis::processors::risk::RiskProcessor::new()));
    registry.register(Box::new(noesis::processors::pattern::PatternProcessor::new()));
    registry.register(Box::new(noesis::processors::world_model::WorldModelProcessor::new()));
    registry.register(Box::new(noesis::processors::assumption::AssumptionProcessor::new()));
    registry.register(Box::new(noesis::processors::scenario::ScenarioProcessor::new()));
    registry.register(Box::new(noesis::processors::forecast::ForecastingProcessor::new()));
    registry.register(Box::new(noesis::processors::simulation_risk::SimulationRiskProcessor::new()));
    registry.register(Box::new(noesis::processors::analogy::AnalogyProcessor::new()));
    registry.register(Box::new(noesis::processors::concept::ConceptProcessor::new()));
    registry.register(Box::new(noesis::processors::decision::DecisionProcessor::new()));
    registry.register(Box::new(noesis::processors::hypothesis::HypothesisProcessor::new()));
    registry.register(Box::new(noesis::processors::mental_model::MentalModelProcessor::new()));
    registry.register(Box::new(noesis::processors::reasoning::ReasoningProcessor::new()));
    registry.register(Box::new(noesis::processors::synthesis::SynthesisProcessor::new()));
    registry.register(Box::new(noesis::processors::epistemic::EpistemicClassifier::new()));
    registry.register(Box::new(noesis::processors::metacognition::MetaProcessor::new()));
    registry.register(Box::new(noesis::processors::context::ContextConstructor::new()));
    registry.register(Box::new(noesis::processors::mood::MoodProcessor::new()));
    registry.register(Box::new(noesis::processors::observer::ObserverProcessor::new()));
    registry.register(Box::new(noesis::processors::health::HealthChecker::new()));
    registry.register(Box::new(noesis::processors::principles::PrincipleDistiller::new()));
    registry.register(Box::new(noesis::processors::values::ValueExtractor::new()));
    registry.register(Box::new(noesis::processors::confidence::ConfidenceEstimator::new()));
    registry.register(Box::new(noesis::processors::reflection::ReflectionProcessor::new()));

    (kernel, registry, ctx)
}

/// Memory field cascade: ingest → episode → triples → attention
#[tokio::test]
async fn test_memory_cascade() {
    let (_kernel, mut registry, ctx) = setup_ctx().await;
    let ingest = IngestRequest::new("The Noesis system uses Rust for memory management.", "test");
    let results = run_cascade(&mut registry, &ctx, Arc::new(ingest)).await;

    assert!(results.iter().any(|r| r == "memory.capture.recorded"));
    assert!(results.iter().any(|r| r == "memory.knowledge.triples_extracted"));
    assert!(results.iter().any(|r| r == "awareness.attention.shifted"));
}

/// Agency cascade: identity update → goal → plan
#[tokio::test]
async fn test_agency_cascade() {
    let (_kernel, mut registry, ctx) = setup_ctx().await;
    let iu = IdentityUpdated {
        meta: SignalMeta::new(types::IDENTITY_UPDATED, "test"),
        identity_version: 1, beliefs_count: 2, traits_count: 0,
        summary: "I build cognitive systems".to_string(),
    };
    let results = run_cascade(&mut registry, &ctx, Arc::new(iu)).await;
    assert!(results.iter().any(|r| r == "agency.goals.created"));
    assert!(results.iter().any(|r| r == "action.planning.plan_ready"));
}

/// Action cascade: goal → plan → project → tasks
#[tokio::test]
async fn test_action_cascade() {
    let (_kernel, mut registry, ctx) = setup_ctx().await;
    let goal = GoalCreated::new("Build a dashboard", 8);
    let results = run_cascade(&mut registry, &ctx, Arc::new(goal)).await;

    assert!(results.iter().any(|r| r == "action.planning.plan_ready"));
    assert!(results.iter().any(|r| r == "action.projects.created"));
    assert!(results.iter().any(|r| r == "action.tasks.created"));
}

/// Awareness cascade: episodes → narrative on BEAT_SLOW
#[tokio::test]
async fn test_awareness_cascade() {
    let (_kernel, mut registry, ctx) = setup_ctx().await;

    for i in 0..5 {
        let ingest = IngestRequest::new(
            &format!("Episode number {} about cognitive systems", i + 1), "test",
        );
        run_cascade(&mut registry, &ctx, Arc::new(ingest)).await;
    }

    let beat_slow = BeatPulse::new(types::BEAT_SLOW);
    let slow_results = run_cascade(&mut registry, &ctx, Arc::new(beat_slow)).await;

    assert!(slow_results.iter().any(|r| r == "awareness.reflection.narrative"),
        "BEAT_SLOW should trigger narrative after 3+ episodes");
}

/// Simulation cascade: decision → scenario → risk
#[tokio::test]
async fn test_simulation_cascade() {
    let (_kernel, mut registry, ctx) = setup_ctx().await;
    let de = DecisionEvaluated {
        meta: SignalMeta::new(types::DECISION_EVALUATED, "test"),
        decision_id: uuid::Uuid::new_v4(),
        decision: "Use Rust".to_string(),
        outcome: "Good performance".to_string(),
        satisfaction: 0.85,
    };
    let results = run_cascade(&mut registry, &ctx, Arc::new(de)).await;
    assert!(results.iter().any(|r| r == "simulation.scenario.ready"));
}

/// BEAT_FAST triggers priority reordering
#[tokio::test]
async fn test_beat_fast_triggers_priority() {
    let (_kernel, mut registry, ctx) = setup_ctx().await;
    run_cascade(&mut registry, &ctx, Arc::new(GoalCreated::new("Low priority", 3))).await;
    run_cascade(&mut registry, &ctx, Arc::new(GoalCreated::new("High priority", 10))).await;

    let beat = BeatPulse::new(types::BEAT_FAST);
    let results = run_cascade(&mut registry, &ctx, Arc::new(beat)).await;
    assert!(results.iter().any(|r| r == "agency.priorities.reordered"));
}

/// Processor registry count — sync test
fn setup_registry_only() -> ProcessorRegistry {
    let mut registry = ProcessorRegistry::new();
    registry.register(Box::new(noesis::processors::episode::EpisodeProcessor::new()));
    registry.register(Box::new(noesis::processors::extraction::ExtractionProcessor::new()));
    registry.register(Box::new(noesis::processors::resolution::EntityResolutionProcessor::new()));
    registry.register(Box::new(noesis::processors::consolidation::ConsolidationProcessor::new()));
    registry.register(Box::new(noesis::processors::belief::BeliefProcessor::new()));
    registry.register(Box::new(noesis::processors::identity::IdentityProcessor::new()));
    registry.register(Box::new(noesis::processors::goal::GoalProcessor::new()));
    registry.register(Box::new(noesis::processors::attention::AttentionProcessor::new()));
    registry.register(Box::new(noesis::processors::curiosity::CuriosityProcessor::new()));
    registry.register(Box::new(noesis::processors::narrative::NarrativeProcessor::new()));
    registry.register(Box::new(noesis::processors::dedup::DedupProcessor::new()));
    registry.register(Box::new(noesis::processors::indexing::IndexingProcessor::new()));
    registry.register(Box::new(noesis::processors::retrieval::RetrievalProcessor::new()));
    registry.register(Box::new(noesis::processors::decay::DecayProcessor::new()));
    registry.register(Box::new(noesis::processors::priority::PriorityProcessor::new()));
    registry.register(Box::new(noesis::processors::strategy::StrategyProcessor::new()));
    registry.register(Box::new(noesis::processors::planning::PlanDecomposer::new()));
    registry.register(Box::new(noesis::processors::project::ProjectProcessor::new()));
    registry.register(Box::new(noesis::processors::task::TaskProcessor::new()));
    registry.register(Box::new(noesis::processors::execution::ExecutionProcessor::new()));
    registry.register(Box::new(noesis::processors::evaluation::EvaluationProcessor::new()));
    registry.register(Box::new(noesis::processors::risk::RiskProcessor::new()));
    registry.register(Box::new(noesis::processors::pattern::PatternProcessor::new()));
    registry.register(Box::new(noesis::processors::world_model::WorldModelProcessor::new()));
    registry.register(Box::new(noesis::processors::assumption::AssumptionProcessor::new()));
    registry.register(Box::new(noesis::processors::scenario::ScenarioProcessor::new()));
    registry.register(Box::new(noesis::processors::forecast::ForecastingProcessor::new()));
    registry.register(Box::new(noesis::processors::simulation_risk::SimulationRiskProcessor::new()));
    registry.register(Box::new(noesis::processors::analogy::AnalogyProcessor::new()));
    registry.register(Box::new(noesis::processors::concept::ConceptProcessor::new()));
    registry.register(Box::new(noesis::processors::decision::DecisionProcessor::new()));
    registry.register(Box::new(noesis::processors::hypothesis::HypothesisProcessor::new()));
    registry.register(Box::new(noesis::processors::mental_model::MentalModelProcessor::new()));
    registry.register(Box::new(noesis::processors::reasoning::ReasoningProcessor::new()));
    registry.register(Box::new(noesis::processors::synthesis::SynthesisProcessor::new()));
    registry.register(Box::new(noesis::processors::epistemic::EpistemicClassifier::new()));
    registry.register(Box::new(noesis::processors::metacognition::MetaProcessor::new()));
    registry.register(Box::new(noesis::processors::context::ContextConstructor::new()));
    registry.register(Box::new(noesis::processors::mood::MoodProcessor::new()));
    registry.register(Box::new(noesis::processors::observer::ObserverProcessor::new()));
    registry.register(Box::new(noesis::processors::health::HealthChecker::new()));
    registry.register(Box::new(noesis::processors::principles::PrincipleDistiller::new()));
    registry.register(Box::new(noesis::processors::values::ValueExtractor::new()));
    registry.register(Box::new(noesis::processors::confidence::ConfidenceEstimator::new()));
    registry.register(Box::new(noesis::processors::reflection::ReflectionProcessor::new()));
    registry
}

#[test]
fn test_processor_count() {
    let registry = setup_registry_only();
    let names = registry.names();
    assert!(names.len() >= 40, "should have 40+ processors registered, got {}", names.len());
}
