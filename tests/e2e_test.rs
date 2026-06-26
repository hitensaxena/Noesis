//! End-to-end cascade stress test.
//! Validates full cognitive pipeline across all 8 fields.

use std::sync::Arc;
use std::collections::VecDeque;

use noesis::kernel::bus::EventBus;
use noesis::kernel::beat_coordinator::BeatPulse;
use noesis::kernel::plugin::Plugin;
use noesis::kernel::signal::SignalArc;
use noesis::field_runtime::context::FieldContext;
use noesis::field_runtime::processor_registry::ProcessorRegistry;
use noesis::signals::types;
use noesis::signals::IngestRequest;
use noesis::storage::memory_store::MemoryStore;

/// Set up the full kernel with all 49 processors.
async fn setup_full_kernel() -> (ProcessorRegistry, FieldContext) {
    let event_bus = Arc::new(EventBus::new());
    let storage = Arc::new(MemoryStore::new());
    let ctx = FieldContext::new(event_bus.clone(), storage);

    let mut registry = ProcessorRegistry::new();

    // Register all processors from the built-in Noesis plugin
    let plugin = noesis::kernel::plugin::noesis_plugin::NoesisPlugin::new();
    for proc in plugin.processors() {
        registry.register(proc);
    }

    (registry, ctx)
}

/// Process a signal through the cascade until equilibrium.
async fn run_cascade(
    registry: &mut ProcessorRegistry,
    ctx: &FieldContext,
    initial: SignalArc,
    max_depth: usize,
) -> Vec<String> {
    let mut queue: VecDeque<SignalArc> = VecDeque::new();
    queue.push_back(initial);
    let mut processed = Vec::new();
    let mut iterations = 0;

    while let Some(signal) = queue.pop_front() {
        if iterations >= max_depth { break; }
        iterations += 1;
        let st = signal.signal_type().to_string();
        let depth = signal.meta().depth;
        processed.push(format!("{} (depth={})", st, depth));
        let emitted = registry.dispatch(ctx, signal).await;
        for sig in emitted { queue.push_back(sig); }
    }
    processed
}

#[tokio::test]
async fn test_e2e_single_ingest_cascade() {
    let (mut registry, ctx) = setup_full_kernel().await;
    let signal = IngestRequest::new("Hiten debugged a Rust borrow checker issue.", "e2e");
    let results = run_cascade(&mut registry, &ctx, Arc::new(signal), 50).await;

    println!("=== E2E Single Ingest ({} signals) ===", results.len());
    for r in &results { println!("  {}", r); }

    assert!(!results.is_empty(), "should process at least root signal");
    assert!(results.iter().any(|r| r.contains("memory.capture.ingested")), "should include ingested");
    assert!(results.len() < 50, "cascade should converge");
}

#[tokio::test]
async fn test_e2e_multi_episode_cascade() {
    let (mut registry, ctx) = setup_full_kernel().await;
    let texts = vec![
        "I went for a run in the park this morning.",
        "Read a paper on transformer architectures.",
        "Had coffee with a friend learning Rust.",
        "Experimented with a new recipe for dinner.",
        "Researched tokio internals and async scheduling.",
    ];

    let mut all_signals: Vec<String> = Vec::new();
    for text in &texts {
        let signal = IngestRequest::new(text, "e2e");
        let results = run_cascade(&mut registry, &ctx, Arc::new(signal), 50).await;
        all_signals.extend(results);
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }

    let beat = BeatPulse::new(types::BEAT_SLOW);
    all_signals.extend(run_cascade(&mut registry, &ctx, Arc::new(beat), 50).await);
    let beat = BeatPulse::new(types::BEAT_MEDIUM);
    all_signals.extend(run_cascade(&mut registry, &ctx, Arc::new(beat), 50).await);

    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for sig in &all_signals {
        let name = sig.split('(').next().unwrap_or(sig).trim();
        *counts.entry(name).or_insert(0) += 1;
    }

    println!("=== E2E Multi ({}/{}) ===", all_signals.len(), counts.len());
    let mut sorted: Vec<_> = counts.iter().collect();
    sorted.sort_by_key(|(_, &c)| std::cmp::Reverse(c));
    for (name, count) in &sorted { println!("  {}: {}x", name, count); }

    assert!(*counts.get("memory.capture.ingested").unwrap_or(&0) >= 5);
    assert!(*counts.get("memory.capture.recorded").unwrap_or(&0) >= 1);
    assert!(*counts.get("awareness.attention.shifted").unwrap_or(&0) >= 1);
}

#[tokio::test]
async fn test_e2e_cascade_convergence() {
    let (mut registry, ctx) = setup_full_kernel().await;
    let signal = IngestRequest::new("A single test experience to verify convergence.", "e2e");
    let results = run_cascade(&mut registry, &ctx, Arc::new(signal), 100).await;
    assert!(results.len() < 80, "cascade should converge early");
    assert!(results.len() >= 1, "should process at least root");
    println!("=== Convergence: {} signals ===", results.len());
}

#[tokio::test]
async fn test_e2e_all_processors_registered() {
    let (registry, _) = setup_full_kernel().await;
    let names = registry.names();
    println!("=== {} processors registered ===", names.len());
    for name in &names { println!("  {}", name); }
    assert_eq!(names.len(), 49, "should have 49 processors");
}
