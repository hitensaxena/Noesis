//! Action field integration tests.
//! Tests: Plan → Project → Task → Execution → Evaluation → Risk → Recovery.

use std::sync::Arc;
use noesis::kernel::bus::EventBus;
use noesis::kernel::beat_coordinator::BeatPulse;
use noesis::field_runtime::context::FieldContext;
use noesis::signals::types;
use noesis::signals::GoalCreated;
use noesis::storage::memory_store::MemoryStore;
use noesis::Processor;

fn setup_env() -> (Arc<EventBus>, Arc<MemoryStore>, FieldContext) {
    let event_bus = Arc::new(EventBus::new());
    let storage = Arc::new(MemoryStore::new());
    let ctx = FieldContext::new(event_bus.clone(), storage.clone());
    (event_bus, storage, ctx)
}

#[tokio::test]
async fn test_action_planning_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::action::processors::plan_processor::PlanDecomposer::new();
    let goal = GoalCreated::new("Build a Rust CLI tool", 1);
    let result = processor.process(&ctx, Arc::new(goal)).await;
    assert!(result.is_ok(), "PlanningProcessor should process");
    let emitted = result.unwrap();
    assert!(!emitted.is_empty(), "should emit PlanReady");
    assert_eq!(emitted[0].signal_type(), types::PLANNING_PLAN_READY);
}

#[tokio::test]
async fn test_action_project_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::action::processors::project_processor::ProjectProcessor::new();
    let goal = GoalCreated::new("Build a Rust CLI tool", 1);
    let result = processor.process(&ctx, Arc::new(goal)).await;
    assert!(result.is_ok(), "ProjectProcessor should process");
    let emitted = result.unwrap();
    assert!(!emitted.is_empty(), "should emit ProjectCreated");
    assert_eq!(emitted[0].signal_type(), types::PROJECT_CREATED);
}

#[tokio::test]
async fn test_action_task_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::action::processors::task_processor::TaskProcessor::new();
    // TaskProcessor subscribes to PROJECT_CREATED
    let project = noesis::fields::action::processors::project_processor::ProjectCreated::new(
        "Doc project", "write docs", 5,
    );
    let result = processor.process(&ctx, Arc::new(project)).await;
    assert!(result.is_ok(), "TaskProcessor should process");
    let emitted = result.unwrap();
    assert!(!emitted.is_empty(), "should emit TaskCreated");
    assert_eq!(emitted[0].signal_type(), types::TASK_CREATED);
}

#[tokio::test]
async fn test_action_execution_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::action::processors::execution_processor::ExecutionProcessor::new();
    // ExecutionProcessor subscribes to TASK_CREATED
    let task = noesis::fields::action::processors::task_processor::TaskCreated::new(
        "Execute a plan", 1,
    );
    let result = processor.process(&ctx, Arc::new(task)).await;
    assert!(result.is_ok(), "ExecutionProcessor should process");
    let emitted = result.unwrap();
    assert!(!emitted.is_empty(), "should emit ExecutionStarted");
    assert_eq!(emitted[0].signal_type(), types::EXECUTION_STARTED);
}

#[tokio::test]
async fn test_action_evaluation_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::action::processors::evaluation_processor::EvaluationProcessor::new();
    let beat = BeatPulse::new(types::BEAT_MEDIUM);
    let result = processor.process(&ctx, Arc::new(beat)).await;
    assert!(result.is_ok(), "EvaluationProcessor should process");
}

#[tokio::test]
async fn test_action_risk_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::action::processors::risk_processor::RiskProcessor::new();
    let goal = GoalCreated::new("High-stakes deployment", 1);
    let result = processor.process(&ctx, Arc::new(goal)).await;
    assert!(result.is_ok(), "RiskProcessor should process");
}

#[tokio::test]
async fn test_action_recovery_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::action::processors::recovery_processor::RecoveryProcessor::new();
    let goal = GoalCreated::new("Recover from failure", 1);
    let result = processor.process(&ctx, Arc::new(goal)).await;
    assert!(result.is_ok(), "RecoveryProcessor should process");
}

#[tokio::test]
async fn test_action_planning_cascade() {
    let (_, _, ctx) = setup_env();
    let mut project = noesis::fields::action::processors::project_processor::ProjectProcessor::new();
    let mut task = noesis::fields::action::processors::task_processor::TaskProcessor::new();

    // Branch 1: GoalCreated → ProjectCreated
    let goal = GoalCreated::new("Build Noesis action pipeline", 1);
    let emitted = project.process(&ctx, Arc::new(goal)).await.unwrap();
    assert!(!emitted.is_empty(), "project should emit ProjectCreated");
    let proj_sig = emitted.into_iter().next().unwrap();
    assert_eq!(proj_sig.signal_type(), types::PROJECT_CREATED);

    // Branch 2: ProjectCreated → TaskCreated
    let emitted = task.process(&ctx, proj_sig).await.unwrap();
    assert!(!emitted.is_empty(), "task should emit TaskCreated");
    assert_eq!(emitted[0].signal_type(), types::TASK_CREATED);
}
