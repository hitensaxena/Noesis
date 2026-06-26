//! Memory field integration tests.
//!
//! Tests the full memory processor chain: ingest → episode → extraction → consolidation
//! as well as context, decay, dedup, indexing, and retrieval processors.

use std::sync::Arc;

use noesis::kernel::bus::EventBus;
use noesis::kernel::beat_coordinator::BeatPulse;
use noesis::field_runtime::context::FieldContext;
use noesis::signals::types;
use noesis::signals::{IngestRequest, EpisodeRecorded};
use noesis::storage::memory_store::MemoryStore;
use noesis::Processor;

/// Helper: set up a memory-focused test environment.
fn setup_memory_env() -> (Arc<EventBus>, Arc<MemoryStore>, FieldContext) {
    let event_bus = Arc::new(EventBus::new());
    let storage = Arc::new(MemoryStore::new());
    let ctx = FieldContext::new(event_bus.clone(), storage.clone());
    (event_bus, storage, ctx)
}

/// Test: EpisodeProcessor converts IngestRequest to EpisodeRecorded.
#[tokio::test]
async fn test_memory_episode_processor() {
    let (_, _, ctx) = setup_memory_env();

    let mut processor = noesis::processors::episode::EpisodeProcessor::new();
    let signal = IngestRequest::new("Test experience for episode processing.", "test");

    let result = processor.process(&ctx, Arc::new(signal)).await;
    assert!(result.is_ok(), "EpisodeProcessor should process without error");
    let emitted = result.unwrap();
    assert!(!emitted.is_empty(), "should emit EpisodeRecorded");
    assert_eq!(emitted[0].signal_type(), types::EPISODE_RECORDED);
}

/// Test: ExtractionProcessor extracts triples from episode content.
#[tokio::test]
async fn test_memory_extraction_processor() {
    let (_, _, ctx) = setup_memory_env();

    let mut processor = noesis::processors::extraction::ExtractionProcessor::new();
    let signal = EpisodeRecorded::new(
        "Hiten worked on the Noesis project with Rust and tokio.",
        "test", vec![],
    );

    let result = processor.process(&ctx, Arc::new(signal)).await;
    assert!(result.is_ok(), "ExtractionProcessor should process without error");
    let emitted = result.unwrap();
    assert!(!emitted.is_empty(), "should emit TriplesExtracted");
    assert_eq!(emitted[0].signal_type(), types::TRIPLES_EXTRACTED);
}

/// Test: ConsolidationProcessor buffers episodes, emits on BEAT_SLOW.
#[tokio::test]
async fn test_memory_consolidation_processor() {
    let (_, _, ctx) = setup_memory_env();

    let mut processor = noesis::processors::consolidation::ConsolidationProcessor::new();

    // Buffer episodes — no emission yet
    for i in 0..3 {
        let signal = EpisodeRecorded::new(
            &format!("Consolidation episode {}", i), "test", vec![],
        );
        let result = processor.process(&ctx, Arc::new(signal)).await.unwrap();
        assert!(result.is_empty(), "episode should buffer, not emit yet");
    }

    // BEAT_SLOW triggers consolidation
    let beat = BeatPulse::new(types::BEAT_SLOW);
    let result = processor.process(&ctx, Arc::new(beat)).await.unwrap();
    assert!(!result.is_empty(), "BEAT_SLOW should trigger consolidation");
    assert_eq!(result[0].signal_type(), types::MEMORY_CONSOLIDATED);
}

/// Test: DecayProcessor tracks episode timestamps, processes BEAT_SLOW without error.
#[tokio::test]
async fn test_memory_decay_processor() {
    let (_, _, ctx) = setup_memory_env();

    let mut processor = noesis::fields::memory::processors::decay_processor::DecayProcessor::new();

    // Feed episodes
    for i in 0..5 {
        let signal = EpisodeRecorded::new(
            &format!("Decay test episode {}", i), "test", vec![],
        );
        let result = processor.process(&ctx, Arc::new(signal)).await;
        assert!(result.is_ok(), "DecayProcessor should accept episodes");
    }

    // BEAT_SLOW triggers decay check — shouldn't error
    let beat = BeatPulse::new(types::BEAT_SLOW);
    let result = processor.process(&ctx, Arc::new(beat)).await;
    assert!(result.is_ok(), "decay should not error on beat");
}

/// Test: DedupProcessor deduplicates identical episode content without error.
#[tokio::test]
async fn test_memory_dedup_processor() {
    let (_, _, ctx) = setup_memory_env();

    let mut processor = noesis::fields::memory::processors::dedup_processor::DedupProcessor::new();
    let content = "Duplicate test content for dedup verification.";

    // First occurrence
    let signal = EpisodeRecorded::new(content, "test", vec![]);
    let result1 = processor.process(&ctx, Arc::new(signal)).await;
    assert!(result1.is_ok(), "first occurrence should process");

    // Second identical content — should not error
    let signal2 = EpisodeRecorded::new(content, "test", vec![]);
    let result2 = processor.process(&ctx, Arc::new(signal2)).await;
    assert!(result2.is_ok(), "duplicate should not error");
}

/// Test: ContextConstructor emits every 30 ObserverTransitionDetected signals.
#[tokio::test]
async fn test_memory_context_processor() {
    let (_, _, ctx) = setup_memory_env();

    let mut processor = noesis::fields::memory::processors::context_processor::ContextConstructor::new();
    let mut total_emitted = 0;

    // ContextConstructor only processes ObserverTransitionDetected signals
    for i in 0..35 {
        let ot = noesis::signals::awareness::ObserverTransitionDetected::new(
            &format!("test.signal.{}", i), "ctx-test", 1, 0.5, 0.3,
        );
        let result = processor.process(&ctx, Arc::new(ot)).await.unwrap();
        total_emitted += result.len();
    }

    // Should have emitted at least once by 35 signals (threshold is 30)
    assert!(total_emitted >= 1, "ContextConstructor should emit by signal 35");
}

/// Test: IndexingProcessor extracts terms from episodes.
#[tokio::test]
async fn test_memory_indexing_processor() {
    let (_, _, ctx) = setup_memory_env();

    let mut processor = noesis::fields::memory::processors::indexing_processor::IndexingProcessor::new();

    let signal = EpisodeRecorded::new(
        "The Rust programming language enables safe systems programming.",
        "test", vec![],
    );
    let result = processor.process(&ctx, Arc::new(signal)).await;
    assert!(result.is_ok(), "IndexingProcessor should process without error");
}

/// Test: RetrievalProcessor scores episodes against query terms.
#[tokio::test]
async fn test_memory_retrieval_processor() {
    let (_, _, ctx) = setup_memory_env();

    let mut processor = noesis::fields::memory::processors::retrieval_processor::RetrievalProcessor::new();

    // Feed some episodes
    for content in &["Machine learning concepts", "Rust programming", "System architecture"] {
        let signal = EpisodeRecorded::new(content, "test", vec![]);
        let _ = processor.process(&ctx, Arc::new(signal)).await;
    }

    // BEAT_MEDIUM triggers retrieval processing
    let beat = BeatPulse::new(types::BEAT_MEDIUM);
    let result = processor.process(&ctx, Arc::new(beat)).await;
    assert!(result.is_ok(), "RetrievalProcessor should process beat without error");
}
