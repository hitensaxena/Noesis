//! Agency field integration tests.
//! Tests: Goal → Priority → Strategy → Opportunity.

use std::sync::Arc;
use noesis::kernel::bus::EventBus;
use noesis::kernel::beat_coordinator::BeatPulse;
use noesis::kernel::signal::SignalMeta;
use noesis::field_runtime::context::FieldContext;
use noesis::signals::types;
use noesis::signals::IdentityUpdated;
use noesis::storage::memory_store::MemoryStore;
use noesis::Processor;

fn setup_env() -> (Arc<EventBus>, Arc<MemoryStore>, FieldContext) {
    let event_bus = Arc::new(EventBus::new());
    let storage = Arc::new(MemoryStore::new());
    let ctx = FieldContext::new(event_bus.clone(), storage.clone());
    (event_bus, storage, ctx)
}

#[tokio::test]
async fn test_agency_goal_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::processors::goal::GoalProcessor::new();
    let signal = IdentityUpdated {
        meta: SignalMeta::new(types::IDENTITY_UPDATED, "test"),
        identity_version: 1,
        beliefs_count: 3,
        traits_count: 2,
        summary: "Rust developer identity".to_string(),
    };
    let result = processor.process(&ctx, Arc::new(signal)).await;
    assert!(result.is_ok(), "GoalProcessor should process");
    let emitted = result.unwrap();
    assert!(!emitted.is_empty(), "should emit GoalCreated");
}

#[tokio::test]
async fn test_agency_priority_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::agency::processors::priority_processor::PriorityProcessor::new();
    for i in 0..3 {
        let goal = noesis::signals::GoalCreated::new(&format!("Goal {}", i), i as u8);
        let _ = processor.process(&ctx, Arc::new(goal)).await;
    }
    let beat = BeatPulse::new(types::BEAT_FAST);
    let result = processor.process(&ctx, Arc::new(beat)).await;
    assert!(result.is_ok(), "PriorityProcessor should process BEAT_FAST");
}

#[tokio::test]
async fn test_agency_strategy_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::agency::processors::strategy_processor::StrategyProcessor::new();
    let beat = BeatPulse::new(types::BEAT_MEDIUM);
    let result = processor.process(&ctx, Arc::new(beat)).await;
    assert!(result.is_ok(), "StrategyProcessor should process BEAT_MEDIUM");
}

#[tokio::test]
async fn test_agency_opportunity_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::agency::processors::opportunity_processor::OpportunityProcessor::new();
    let curiosity = noesis::signals::awareness::CuriosityDetected::new("Rust async", "How does tokio schedule tasks?", 0.7);
    let result = processor.process(&ctx, Arc::new(curiosity)).await;
    assert!(result.is_ok(), "OpportunityProcessor should process");
}

#[tokio::test]
async fn test_agency_goal_lifecycle() {
    let (_, _, ctx) = setup_env();
    let mut goal = noesis::processors::goal::GoalProcessor::new();
    let signal = IdentityUpdated {
        meta: SignalMeta::new(types::IDENTITY_UPDATED, "test"),
        identity_version: 1,
        beliefs_count: 3,
        traits_count: 2,
        summary: "Developer identity".to_string(),
    };
    let emitted = goal.process(&ctx, Arc::new(signal)).await.unwrap();
    assert!(!emitted.is_empty(), "goal processor should emit");
    assert_eq!(emitted[0].signal_type(), types::GOAL_CREATED);
}
