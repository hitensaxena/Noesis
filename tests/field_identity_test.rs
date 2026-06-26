//! Identity field integration tests.
//!
//! Tests the identity processor chain: MemoryConsolidated → BeliefProcessor →
//! BeliefChanged → IdentityProcessor → IdentityUpdated → GoalProcessor → GoalCreated.
//! Also tests values and principles processors.

use std::sync::Arc;

use noesis::kernel::bus::EventBus;
use noesis::kernel::signal::SignalMeta;
use noesis::field_runtime::context::FieldContext;
use noesis::signals::types;
use noesis::signals::{MemoryConsolidated, BeliefChanged, BeliefChangeType, IdentityUpdated};
use noesis::storage::memory_store::MemoryStore;
use noesis::Processor;

fn setup_env() -> (Arc<EventBus>, Arc<MemoryStore>, FieldContext) {
    let event_bus = Arc::new(EventBus::new());
    let storage = Arc::new(MemoryStore::new());
    let ctx = FieldContext::new(event_bus.clone(), storage.clone());
    (event_bus, storage, ctx)
}

fn consolidated(summary: &str, count: usize) -> MemoryConsolidated {
    MemoryConsolidated {
        meta: SignalMeta::new(types::MEMORY_CONSOLIDATED, "test"),
        episode_ids: vec![uuid::Uuid::new_v4()],
        summary: summary.to_string(),
        memory_count: count,
    }
}

/// Test: BeliefProcessor transforms MemoryConsolidated into BeliefChanged.
#[tokio::test]
async fn test_identity_belief_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::processors::belief::BeliefProcessor::new();
    let signal = consolidated("Hiten enjoys programming in Rust", 3);
    let result = processor.process(&ctx, Arc::new(signal)).await;
    assert!(result.is_ok(), "BeliefProcessor should process");
    let emitted = result.unwrap();
    assert!(!emitted.is_empty(), "should emit BeliefChanged");
    assert_eq!(emitted[0].signal_type(), types::BELIEF_CHANGED);
}

/// Test: IdentityProcessor transforms BeliefChanged into IdentityUpdated.
#[tokio::test]
async fn test_identity_identity_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::processors::identity::IdentityProcessor::new();
    let signal = BeliefChanged::new("Enjoys Rust programming", BeliefChangeType::Created, 0.85);
    let result = processor.process(&ctx, Arc::new(signal)).await;
    assert!(result.is_ok(), "IdentityProcessor should process");
    let emitted = result.unwrap();
    assert!(!emitted.is_empty(), "should emit IdentityUpdated");
    assert_eq!(emitted[0].signal_type(), types::IDENTITY_UPDATED);
}

/// Test: GoalProcessor transforms IdentityUpdated into GoalCreated.
#[tokio::test]
async fn test_identity_goal_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::processors::goal::GoalProcessor::new();
    let signal = IdentityUpdated {
        meta: SignalMeta::new(types::IDENTITY_UPDATED, "test"),
        identity_version: 2,
        beliefs_count: 5,
        traits_count: 3,
        summary: "Updated identity".to_string(),
    };
    let result = processor.process(&ctx, Arc::new(signal)).await;
    assert!(result.is_ok(), "GoalProcessor should process");
    let emitted = result.unwrap();
    assert!(!emitted.is_empty(), "should emit GoalCreated");
    assert_eq!(emitted[0].signal_type(), types::GOAL_CREATED);
}

/// Test: full identity cascade.
#[tokio::test]
async fn test_identity_full_cascade() {
    let (_, _, ctx) = setup_env();
    let mut belief = noesis::processors::belief::BeliefProcessor::new();
    let mut identity = noesis::processors::identity::IdentityProcessor::new();
    let mut goal = noesis::processors::goal::GoalProcessor::new();

    let signal = consolidated("Hiten has been writing Rust for 3 years", 5);
    let emitted = belief.process(&ctx, Arc::new(signal)).await.unwrap();
    assert!(!emitted.is_empty(), "belief should emit");
    let belief_sig = emitted.into_iter().next().unwrap();

    let emitted = identity.process(&ctx, belief_sig).await.unwrap();
    assert!(!emitted.is_empty(), "identity should emit");
    let identity_sig = emitted.into_iter().next().unwrap();

    let emitted = goal.process(&ctx, identity_sig).await.unwrap();
    assert!(!emitted.is_empty(), "goal should emit");
    assert_eq!(emitted[0].signal_type(), types::GOAL_CREATED);
}

/// Test: ValueProcessor processes observer transitions.
#[tokio::test]
async fn test_identity_value_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::identity::processors::value_processor::ValueExtractor::new();
    let ot = noesis::signals::awareness::ObserverTransitionDetected::new(
        "memory.capture.recorded", "value-test", 1, 0.5, 0.3,
    );
    let result = processor.process(&ctx, Arc::new(ot)).await;
    assert!(result.is_ok(), "ValueProcessor should process");
}

/// Test: PrincipleProcessor processes observer transitions.
#[tokio::test]
async fn test_identity_principle_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::identity::processors::principle_processor::PrincipleDistiller::new();
    let ot = noesis::signals::awareness::ObserverTransitionDetected::new(
        "memory.capture.recorded", "principle-test", 1, 0.5, 0.3,
    );
    let result = processor.process(&ctx, Arc::new(ot)).await;
    assert!(result.is_ok(), "PrincipleProcessor should process");
}
