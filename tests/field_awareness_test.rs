//! Awareness field integration tests.
//! Tests: Observer → Attention → Curiosity → Mood → Narrative → Health → Pattern → OpenLoops.

use std::sync::Arc;
use noesis::kernel::bus::EventBus;
use noesis::kernel::beat_coordinator::BeatPulse;
use noesis::kernel::signal::SignalMeta;
use noesis::field_runtime::context::FieldContext;
use noesis::signals::types;
use noesis::signals::EpisodeRecorded;
use noesis::signals::awareness::ObserverTransitionDetected;
use noesis::signals::awareness::NarrativeGenerated;
use noesis::signals::MemoryConsolidated;
use noesis::storage::memory_store::MemoryStore;
use noesis::Processor;

fn setup_env() -> (Arc<EventBus>, Arc<MemoryStore>, FieldContext) {
    let event_bus = Arc::new(EventBus::new());
    let storage = Arc::new(MemoryStore::new());
    let ctx = FieldContext::new(event_bus.clone(), storage.clone());
    (event_bus, storage, ctx)
}

#[tokio::test]
async fn test_awareness_observer_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::awareness::processors::observer_processor::ObserverProcessor::new();
    let signal = EpisodeRecorded::new("Something happened.", "test", vec![]);
    let result = processor.process(&ctx, Arc::new(signal)).await;
    assert!(result.is_ok(), "ObserverProcessor should process");
    let emitted = result.unwrap();
    assert!(!emitted.is_empty(), "should emit ObserverTransitionDetected");
    assert_eq!(emitted[0].signal_type(), types::OBSERVER_TRANSITION_DETECTED);
}

#[tokio::test]
async fn test_awareness_observer_no_self_loop() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::awareness::processors::observer_processor::ObserverProcessor::new();
    let ot = ObserverTransitionDetected::new("memory.capture.recorded", "obs-test", 1, 0.5, 0.0);
    let result = processor.process(&ctx, Arc::new(ot)).await.unwrap();
    assert!(result.is_empty(), "should NOT emit for OBSERVER_TRANSITION_DETECTED");
}

#[tokio::test]
async fn test_awareness_attention_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::processors::attention::AttentionProcessor::new();
    let signal = EpisodeRecorded::new("Important event.", "test", vec![]);
    let result = processor.process(&ctx, Arc::new(signal)).await;
    assert!(result.is_ok(), "AttentionProcessor should process");
    let emitted = result.unwrap();
    assert!(!emitted.is_empty(), "should emit AttentionShifted");
    assert_eq!(emitted[0].signal_type(), types::ATTENTION_SHIFTED);
}

#[tokio::test]
async fn test_awareness_curiosity_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::processors::curiosity::CuriosityProcessor::new();
    for i in 0..5 {
        let signal = EpisodeRecorded::new(&format!("Ep {}", i), "test", vec![]);
        let result = processor.process(&ctx, Arc::new(signal)).await.unwrap();
        assert!(result.is_empty(), "episode should buffer");
    }
    let beat = BeatPulse::new(types::BEAT_MEDIUM);
    let result = processor.process(&ctx, Arc::new(beat)).await.unwrap();
    assert!(!result.is_empty(), "BEAT_MEDIUM should trigger curiosity");
    assert_eq!(result[0].signal_type(), types::CURIOSITY_DETECTED);
}

#[tokio::test]
async fn test_awareness_narrative_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::processors::narrative::NarrativeProcessor::new();
    for i in 0..2 {
        let signal = EpisodeRecorded::new(&format!("Nar ep {}", i), "test", vec![]);
        let result = processor.process(&ctx, Arc::new(signal)).await.unwrap();
        assert!(result.is_empty(), "should buffer");
    }
    let beat = BeatPulse::new(types::BEAT_SLOW);
    let result = processor.process(&ctx, Arc::new(beat)).await.unwrap();
    assert!(result.is_empty(), "not yet with <3 episodes");
    let signal = EpisodeRecorded::new("Third episode.", "test", vec![]);
    let _ = processor.process(&ctx, Arc::new(signal)).await;
    let beat = BeatPulse::new(types::BEAT_SLOW);
    let result = processor.process(&ctx, Arc::new(beat)).await.unwrap();
    assert!(!result.is_empty(), "should emit narrative");
    assert_eq!(result[0].signal_type(), types::NARRATIVE_GENERATED);
}

#[tokio::test]
async fn test_awareness_mood_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::awareness::processors::mood_processor::MoodProcessor::new();
    let signal = EpisodeRecorded::new("Test mood.", "test", vec![]);
    let result = processor.process(&ctx, Arc::new(signal)).await;
    assert!(result.is_ok(), "MoodProcessor should not error");
}

#[tokio::test]
async fn test_awareness_pattern_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::awareness::processors::pattern_processor::PatternProcessor::new();
    for i in 0..5 {
        let signal = MemoryConsolidated {
            meta: SignalMeta::new(types::MEMORY_CONSOLIDATED, "test"),
            episode_ids: vec![],
            summary: format!("Pattern test {}", i),
            memory_count: 2,
        };
        let _ = processor.process(&ctx, Arc::new(signal)).await;
    }
}

#[tokio::test]
async fn test_awareness_open_loops_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::awareness::processors::open_loops_processor::OpenLoopsProcessor::new();
    use noesis::signals::GoalCreated;
    use noesis::signals::awareness::CuriosityDetected;
    let goal = GoalCreated::new("Build a Rust project", 1);
    let _ = processor.process(&ctx, Arc::new(goal)).await;
    let curiosity = CuriosityDetected::new("Rust async", "How?", 0.6);
    let _ = processor.process(&ctx, Arc::new(curiosity)).await;
    let beat = BeatPulse::new(types::BEAT_MEDIUM);
    let result = processor.process(&ctx, Arc::new(beat)).await;
    assert!(result.is_ok(), "OpenLoopsProcessor should process BEAT_MEDIUM");
}

#[tokio::test]
async fn test_awareness_reflection_processor() {
    let (_, _, ctx) = setup_env();
    let mut processor = noesis::fields::awareness::processors::reflection_processor::ReflectionProcessor::new();
    let narrative = noesis::signals::awareness::NarrativeGenerated {
        meta: noesis::kernel::signal::SignalMeta::new(types::NARRATIVE_GENERATED, "test"),
        narrative_id: uuid::Uuid::new_v4(),
        title: "Chapter 1".to_string(),
        summary: "The beginning".to_string(),
        episode_count: 3,
        themes: vec!["test".to_string()],
    };
    let result = processor.process(&ctx, Arc::new(narrative)).await;
    assert!(result.is_ok(), "ReflectionProcessor should process");
}
