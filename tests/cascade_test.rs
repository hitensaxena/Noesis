//! End-to-end integration tests for the Noesis signal cascade.
//!
//! These tests verify the full recursive signal propagation:
//! - Inject signals → cascade through processors → equilibrium
//! - Multi-episode cascade triggers narrative, consolidation, curiosity
//! - All 9 processors are registered and dispatched correctly
//! - Field states are updated in response to signals

use std::sync::Arc;
use std::collections::VecDeque;

use noesis::kernel::beat_coordinator::BeatPulse;
use noesis::kernel::kernel::Kernel;
use noesis::kernel::bus::EventBus;
use noesis::kernel::signal::{SignalArc, SignalType};
use noesis::field_runtime::context::FieldContext;
use noesis::field_runtime::processor_registry::ProcessorRegistry;
use noesis::signals::types;
use noesis::signals::IngestRequest;
use noesis::storage::memory_store::MemoryStore;

/// All known signal types in the system.
#[allow(dead_code)]
const ALL_SIGNALS: &[SignalType] = &[
    types::INGEST_REQUEST,
    types::EPISODE_RECORDED,
    types::MEMORY_CONSOLIDATED,
    types::PATTERN_DETECTED,
    types::BELIEF_CHANGED,
    types::IDENTITY_UPDATED,
    types::GOAL_CREATED,
    types::GOAL_COMPLETED,
    types::ATTENTION_SHIFTED,
    types::CURIOSITY_DETECTED,
    types::NARRATIVE_GENERATED,
    types::DECISION_EVALUATED,
    types::ENTITY_CREATED,
    types::EDGE_CREATED,
    types::TRIPLES_EXTRACTED,
    types::BEAT_FAST,
    types::BEAT_MEDIUM,
    types::BEAT_SLOW,
    types::BEAT_IMMEDIATE,
];

/// Helper: set up a full kernel with all fields and processors registered.
async fn setup_full_kernel() -> (Kernel, ProcessorRegistry, FieldContext) {
    let kernel = Kernel::new();
    let storage = Arc::new(MemoryStore::new());
    let field_ctx = FieldContext::new(kernel.event_bus.clone(), storage);

    // Register all signal types
    kernel.registry.register_signal(types::INGEST_REQUEST, "Ingestion request");
    kernel.registry.register_signal(types::EPISODE_RECORDED, "Episode recorded");
    kernel.registry.register_signal(types::MEMORY_CONSOLIDATED, "Consolidated");
    kernel.registry.register_signal(types::PATTERN_DETECTED, "Pattern");
    kernel.registry.register_signal(types::BELIEF_CHANGED, "Belief changed");
    kernel.registry.register_signal(types::IDENTITY_UPDATED, "Identity updated");
    kernel.registry.register_signal(types::GOAL_CREATED, "Goal created");
    kernel.registry.register_signal(types::GOAL_COMPLETED, "Goal completed");
    kernel.registry.register_signal(types::ATTENTION_SHIFTED, "Attention shifted");
    kernel.registry.register_signal(types::CURIOSITY_DETECTED, "Curiosity");
    kernel.registry.register_signal(types::NARRATIVE_GENERATED, "Narrative");
    kernel.registry.register_signal(types::DECISION_EVALUATED, "Decision");
    kernel.registry.register_signal(types::ENTITY_CREATED, "Entity created");
    kernel.registry.register_signal(types::EDGE_CREATED, "Edge created");
    kernel.registry.register_signal(types::TRIPLES_EXTRACTED, "Triples extracted");

    // Register fields
    kernel.registry.register_field("memory", Box::new(|| Box::new(noesis::fields::memory::MemoryField::new())));
    kernel.registry.register_field("identity", Box::new(|| Box::new(noesis::fields::identity::IdentityField::new())));
    kernel.registry.register_field("agency", Box::new(|| Box::new(noesis::fields::agency::AgencyField::new())));
    kernel.registry.register_field("action", Box::new(|| Box::new(noesis::fields::action::ActionField::new())));
    kernel.registry.register_field("awareness", Box::new(|| Box::new(noesis::fields::awareness::AwarenessField::new())));
    kernel.registry.register_field("reasoning", Box::new(|| Box::new(noesis::fields::reasoning::ReasoningField::new())));
    kernel.registry.register_field("simulation", Box::new(|| Box::new(noesis::fields::simulation::SimulationField::new())));
    kernel.registry.register_field("knowledge_graph", Box::new(|| Box::new(noesis::fields::graph::GraphField::new())));

    // Initialize fields
    for name in kernel.registry.list_fields() {
        if let Some(mut field) = kernel.registry.create_field(&name) {
            field.init(&field_ctx).await.unwrap();
        }
    }

    // Register all processors
    let mut processor_registry = ProcessorRegistry::new();
    processor_registry.register(Box::new(noesis::processors::episode::EpisodeProcessor::new()));
    processor_registry.register(Box::new(noesis::processors::belief::BeliefProcessor::new()));
    processor_registry.register(Box::new(noesis::processors::identity::IdentityProcessor::new()));
    processor_registry.register(Box::new(noesis::processors::narrative::NarrativeProcessor::new()));
    processor_registry.register(Box::new(noesis::processors::goal::GoalProcessor::new()));
    processor_registry.register(Box::new(noesis::processors::attention::AttentionProcessor::new()));
    processor_registry.register(Box::new(noesis::processors::curiosity::CuriosityProcessor::new()));
    processor_registry.register(Box::new(noesis::processors::extraction::ExtractionProcessor::new()));
    processor_registry.register(Box::new(noesis::processors::consolidation::ConsolidationProcessor::new()));

    (kernel, processor_registry, field_ctx)
}

/// Helper: run a single signal through the cascade until equilibrium.
/// Returns all (signal_type, depth) pairs that were processed.
async fn run_single_cascade(
    processor_registry: &mut ProcessorRegistry,
    field_ctx: &FieldContext,
    initial_signal: SignalArc,
) -> Vec<String> {
    let mut cascade_queue: VecDeque<SignalArc> = VecDeque::new();
    cascade_queue.push_back(initial_signal);

    let mut processed: Vec<String> = Vec::new();
    let max_iterations = 100;
    let mut iterations = 0;

    while let Some(signal) = cascade_queue.pop_front() {
        if iterations >= max_iterations {
            break;
        }
        iterations += 1;

        let st = signal.signal_type().to_string();
        let depth = signal.meta().depth;
        processed.push(format!("{} (depth={})", st, depth));

        let emitted = processor_registry.dispatch(field_ctx, signal).await;
        for sig in emitted {
            cascade_queue.push_back(sig);
        }
    }

    processed
}

/// Test: basic cascade — IngestRequest → EpisodeRecorded → AttentionShifted
#[tokio::test]
async fn test_basic_cascade() {
    let (_kernel, mut processor_registry, field_ctx) = setup_full_kernel().await;

    let signal = IngestRequest::new("Hiten worked on the Noesis project today.", "test");
    let results = run_single_cascade(
        &mut processor_registry,
        &field_ctx,
        Arc::new(signal),
    ).await;

    // Verify the cascade produced expected signals
    let result_str = results.join("\n  ");
    println!("=== Basic Cascade ===\n  {}", result_str);

    assert!(results.len() >= 1, "should produce at least 1 signal");
    assert!(
        results.iter().any(|r| r.contains("memory.capture.ingested")),
        "should process ingest.request"
    );
}

/// Test: multi-episode cascade triggers narrative at 3, curiosity at 5, consolidation at 3/10
#[tokio::test]
async fn test_multi_episode_cascade_triggers() {
    let (_kernel, mut processor_registry, field_ctx) = setup_full_kernel().await;

    let texts = vec![
        "I went for a run in the park. The birds were singing.",
        "Read a fascinating paper about neural networks and AI alignment.",
        "Had a deep conversation with a friend about consciousness and identity.",
        "Started learning the Rust programming language for systems programming.",
        "Experimented with a new Mediterranean recipe for dinner tonight.",
    ];

    let mut all_signals: Vec<String> = Vec::new();

    for (_i, text) in texts.iter().enumerate() {
        let signal = IngestRequest::new(text, "test");
        let results = run_single_cascade(
            &mut processor_registry,
            &field_ctx,
            Arc::new(signal),
        ).await;

        all_signals.extend(results);

        // Brief pause for processor state to settle
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }

    // Inject cognitive beats to trigger beat-dependent processors
    // BEAT_SLOW triggers NarrativeProcessor + ConsolidationProcessor
    let beat_slow = BeatPulse::new(types::BEAT_SLOW);
    let results = run_single_cascade(
        &mut processor_registry,
        &field_ctx,
        Arc::new(beat_slow),
    ).await;
    all_signals.extend(results);

    // BEAT_MEDIUM triggers CuriosityProcessor
    let beat_med = BeatPulse::new(types::BEAT_MEDIUM);
    let results = run_single_cascade(
        &mut processor_registry,
        &field_ctx,
        Arc::new(beat_med),
    ).await;
    all_signals.extend(results);

    // Count signal types
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for sig in &all_signals {
        let name = sig.split('(').next().unwrap_or(sig).trim();
        *counts.entry(name).or_insert(0) += 1;
    }

    println!("=== Multi-Episode Cascade ({}/{} signal types) ===",
             all_signals.len(), counts.len());
    let mut sorted: Vec<_> = counts.iter().collect();
    sorted.sort_by_key(|(_, &c)| std::cmp::Reverse(c));
    for (name, count) in &sorted {
        println!("  {}: {}x", name, count);
    }

    // Verify key signals
    assert!(
        *counts.get("memory.capture.ingested").unwrap_or(&0) >= 5,
        "should have 5+ ingest requests"
    );
    assert!(
        *counts.get("memory.capture.recorded").unwrap_or(&0) >= 1,
        "should have episode.recorded"
    );
    assert!(
        *counts.get("awareness.reflection.narrative").unwrap_or(&0) >= 1,
        "narrative should fire every 3 episodes"
    );
    assert!(
        *counts.get("awareness.curiosity.detected").unwrap_or(&0) >= 1,
        "curiosity should fire every 5 episodes"
    );
    assert!(
        *counts.get("memory.consolidation.consolidated").unwrap_or(&0) >= 1,
        "consolidation should fire every 3 episodes"
    );
    assert!(
        *counts.get("memory.knowledge.triples_extracted").unwrap_or(&0) >= 1,
        "extraction should fire on each episode"
    );
    assert!(
        *counts.get("awareness.attention.shifted").unwrap_or(&0) >= 1,
        "attention should shift on each relevant signal"
    );
}

/// Test: verify the processor registry dispatch map is correct
#[tokio::test]
async fn test_processor_registry_dispatch() {
    let (_kernel, processor_registry, _field_ctx) = setup_full_kernel().await;

    let names = processor_registry.names();
    assert_eq!(names.len(), 9, "should have 9 processors");

    let expected = [
        "episode", "belief", "identity", "narrative", "goal",
        "attention", "curiosity", "extraction", "consolidation",
    ];
    for name in &expected {
        assert!(
            names.contains(&name.to_string()),
            "should have processor: {}",
            name
        );
    }
}

/// Test: field registration
#[tokio::test]
async fn test_field_registration() {
    let (kernel, _, _) = setup_full_kernel().await;

    let fields = kernel.registry.list_fields();
    assert_eq!(fields.len(), 8, "should have 8 fields");

    let expected = [
        "memory", "identity", "agency", "action", "awareness", "reasoning", "simulation", "knowledge_graph",
    ];
    for name in &expected {
        assert!(
            fields.contains(&name.to_string()),
            "should have field: {}",
            name
        );
    }
}

/// Test: signal types registration
#[tokio::test]
async fn test_signal_type_registration() {
    let (kernel, _, _) = setup_full_kernel().await;

    let signals = kernel.registry.list_signals();
    assert_eq!(signals.len(), 15, "should have 15 signal types");

    let expected = [
        "memory.capture.ingested", "memory.capture.recorded", "memory.consolidation.consolidated",
        "memory.consolidation.pattern_detected", "identity.beliefs.changed", "identity.self.updated",
        "agency.goals.created", "agency.goals.completed", "awareness.attention.shifted",
        "awareness.curiosity.detected", "awareness.reflection.narrative", "agency.decision.evaluated",
        "memory.knowledge.entity_created", "memory.knowledge.edge_created", "memory.knowledge.triples_extracted",
    ];
    for name in &expected {
        assert!(
            signals.iter().any(|(t, _)| t.0 == *name),
            "should have signal type: {}",
            name
        );
    }
}

/// Test: the cascade reaches equilibrium (no infinite loops)
#[tokio::test]
async fn test_cascade_equilibrium() {
    let (_kernel, mut processor_registry, field_ctx) = setup_full_kernel().await;

    let signal = IngestRequest::new("A single test experience.", "test");
    let results = run_single_cascade(
        &mut processor_registry,
        &field_ctx,
        Arc::new(signal),
    ).await;

    // The cascade should terminate (not infinite loop)
    // It should produce a bounded number of signals
    assert!(results.len() < 50, "cascade should not produce 50+ signals");
    assert!(results.len() >= 1, "cascade should produce at least 1 signal");

    println!("=== Equilibrium Test: {} signals in cascade ===", results.len());
    for r in &results {
        println!("  {}", r);
    }
}

/// Test: verify each processor can be instantiated independently
#[tokio::test]
async fn test_processor_independence() {
    // Each processor should be independently constructable and runnable
    use noesis::Processor;

    let storage = Arc::new(MemoryStore::new());
    let event_bus = Arc::new(EventBus::new());
    let ctx = FieldContext::new(event_bus.clone(), storage);

    // Test EpisodeProcessor
    let mut ep = noesis::processors::episode::EpisodeProcessor::new();
    let sig = IngestRequest::new("Test experience.", "test");
    let result = ep.process(&ctx, Arc::new(sig)).await;
    assert!(result.is_ok(), "EpisodeProcessor should process without error");
    let emitted = result.unwrap();
    assert!(!emitted.is_empty(), "EpisodeProcessor should emit EpisodeRecorded");
    assert_eq!(emitted[0].signal_type(), types::EPISODE_RECORDED);

    // Test ExtractionProcessor
    let mut ex = noesis::processors::extraction::ExtractionProcessor::new();
    let ep_sig = noesis::signals::EpisodeRecorded::new(
        "Hiten worked on Noesis with Rust.", "test", vec![],
    );
    let result = ex.process(&ctx, Arc::new(ep_sig)).await;
    assert!(result.is_ok(), "ExtractionProcessor should process without error");
    let emitted = result.unwrap();
    assert!(!emitted.is_empty(), "ExtractionProcessor should emit triples");
    assert_eq!(emitted[0].signal_type(), types::TRIPLES_EXTRACTED);

    // Test NarrativeProcessor (buffers episodes, emits on BEAT_SLOW)
    let mut na = noesis::processors::narrative::NarrativeProcessor::new();
    for i in 0..3 {
        let eps = noesis::signals::EpisodeRecorded::new(
            &format!("Narrative test episode {}", i), "test", vec![],
        );
        let result = na.process(&ctx, Arc::new(eps)).await.unwrap();
        assert!(result.is_empty(), "episode should buffer, not emit yet");
    }
    // Trigger narrative with slow beat
    let beat = BeatPulse::new(types::BEAT_SLOW);
    let result = na.process(&ctx, Arc::new(beat)).await.unwrap();
    assert!(!result.is_empty(), "BEAT_SLOW should trigger narrative after 3 episodes");
    assert_eq!(result[0].signal_type(), types::NARRATIVE_GENERATED);

    // Test BeliefProcessor
    let mut bp = noesis::processors::belief::BeliefProcessor::new();
    let mc = noesis::signals::MemoryConsolidated {
        meta: noesis::kernel::signal::SignalMeta::new(types::MEMORY_CONSOLIDATED, "test"),
        episode_ids: vec![],
        summary: "test".to_string(),
        memory_count: 3,
    };
    let result = bp.process(&ctx, Arc::new(mc)).await;
    assert!(result.is_ok(), "BeliefProcessor should process");
    assert!(!result.unwrap().is_empty(), "should emit BeliefChanged");

    // Test IdentityProcessor
    let mut ip = noesis::processors::identity::IdentityProcessor::new();
    let bc = noesis::signals::BeliefChanged::new(
        "Test belief", noesis::signals::BeliefChangeType::Created, 0.8,
    );
    let result = ip.process(&ctx, Arc::new(bc)).await;
    assert!(result.is_ok(), "IdentityProcessor should process");
    assert!(!result.unwrap().is_empty(), "should emit IdentityUpdated");

    // Test GoalProcessor
    let mut gp = noesis::processors::goal::GoalProcessor::new();
    let iu = noesis::signals::IdentityUpdated {
        meta: noesis::kernel::signal::SignalMeta::new(types::IDENTITY_UPDATED, "test"),
        identity_version: 1,
        beliefs_count: 1,
        traits_count: 0,
        summary: "test".to_string(),
    };
    let result = gp.process(&ctx, Arc::new(iu)).await;
    assert!(result.is_ok(), "GoalProcessor should process");
    assert!(!result.unwrap().is_empty(), "should emit GoalCreated");

    // Test AttentionProcessor
    let mut ap = noesis::processors::attention::AttentionProcessor::new();
    let eps = noesis::signals::EpisodeRecorded::new("Important event.", "test", vec![]);
    let result = ap.process(&ctx, Arc::new(eps)).await;
    assert!(result.is_ok(), "AttentionProcessor should process");
    assert!(!result.unwrap().is_empty(), "should emit AttentionShifted");

    // Test ConsolidationProcessor (buffers episodes, emits on BEAT_SLOW)
    let mut cp = noesis::processors::consolidation::ConsolidationProcessor::new();
    for i in 0..3 {
        let eps = noesis::signals::EpisodeRecorded::new(
            &format!("Consolidation test {}", i), "test", vec![],
        );
        let result = cp.process(&ctx, Arc::new(eps)).await.unwrap();
        assert!(result.is_empty(), "episode should buffer, not emit yet");
    }
    // Trigger consolidation with slow beat
    let beat = BeatPulse::new(types::BEAT_SLOW);
    let result = cp.process(&ctx, Arc::new(beat)).await.unwrap();
    assert!(!result.is_empty(), "BEAT_SLOW should trigger consolidation after 3 episodes");

    // Test CuriosityProcessor (buffers episodes, emits on BEAT_MEDIUM)
    let mut cr = noesis::processors::curiosity::CuriosityProcessor::new();
    for i in 0..5 {
        let eps = noesis::signals::EpisodeRecorded::new(
            &format!("Curiosity test {}", i), "test", vec![],
        );
        let result = cr.process(&ctx, Arc::new(eps)).await.unwrap();
        assert!(result.is_empty(), "episode should buffer, not emit yet");
    }
    // Trigger curiosity with medium beat
    let beat = BeatPulse::new(types::BEAT_MEDIUM);
    let result = cr.process(&ctx, Arc::new(beat)).await.unwrap();
    assert!(!result.is_empty(), "BEAT_MEDIUM should trigger curiosity after 5 episodes");
    assert_eq!(result[0].signal_type(), types::CURIOSITY_DETECTED);
}
