//! Noesis daemon entrypoint.
//!
//! Wires the kernel, registers all fields and processors,
//! starts the event bus, and manages the signal propagation loop.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use tracing;

use noesis::core::kernel::Kernel;
use noesis::core::state::{new_field_cache, SystemState};
use noesis::eventbus::signal::SignalArc;
use noesis::field::context::FieldContext;
use noesis::interfaces::cli::{Cli, Commands};
use noesis::processor::lifecycle::ProcessorRegistry;
use noesis::signals::types;
use noesis::signals::IngestRequest;
use noesis::storage::memory_store::MemoryStore;
use noesis::storage::event_store::{EventStore, MemoryEventStore};
use noesis::metrics::metrics::MetricsCollector;
use noesis::interfaces::rest::{ApiState, KernelSnapshot};

/// All registered signal types that processors subscribe to.
const ALL_SIGNAL_TYPES: &[noesis::eventbus::signal::SignalType] = &[
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
];

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing with sensible defaults
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "noesis=info".into()),
        )
        .with_target(true)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Start { rest, port } => run_daemon(rest, port).await,
        Commands::Inject { text, source } => run_inject(&text, &source).await,
        Commands::Inspect { target, name } => run_inspect(&target, name.as_deref()).await,
        Commands::List { target } => run_list(&target).await,
        Commands::Plugins { action } => {
            tracing::info!("Plugin system not yet active");
            match action {
                Some(_) => tracing::info!("Plugin commands coming soon"),
                None => tracing::info!("Use `noesis plugins list` or `noesis plugins load <path>`"),
            }
            Ok(())
        }
    }
}

/// Run the Noesis daemon.
async fn run_daemon(enable_rest: bool, #[allow(unused_variables)] port: u16) -> Result<()> {
    let sep = "=".repeat(60);
    tracing::info!("{}", sep);
    tracing::info!("  Noesis — Decentralized Cognitive Architecture");
    tracing::info!("  Version 0.1.0");
    tracing::info!("{}", sep);

    // -----------------------------------------------------------------------
    // 1. Create kernel and storage
    // -----------------------------------------------------------------------
    let mut kernel = Kernel::new();
    let field_cache = new_field_cache();
    let system_state = Arc::new(SystemState::new(field_cache.clone()));

    // Configure composite storage: MemoryStore + optional Redis + Postgres
    let mut composite = noesis::storage::backends::CompositeStorage::new();

    // Try connecting to existing curlyos-core Postgres (port 54321)
    #[cfg(feature = "postgres-redis")]
    if let Ok(pg_config) = std::env::var("NOESIS_DATABASE_URL") {
        let config: tokio_postgres::Config = pg_config.parse().unwrap_or_else(|_| {
            let mut c = tokio_postgres::Config::new();
            c.host("127.0.0.1").port(54321).dbname("curlyos");
            c.user("curlyos").password(std::env::var("CURLYOS_PG_PASSWORD").unwrap_or_default());
            c
        });
        match noesis::storage::backends::postgres_backend::PostgresBackend::connect(&config).await {
            Ok(pg) => {
                composite.postgres = Some(Arc::new(pg));
                tracing::info!("[main] connected to Postgres (:54321)");
            }
            Err(e) => tracing::warn!("[main] Postgres unavailable (continuing with memory): {}", e),
        }
    } else {
        tracing::info!("[main] NOESIS_DATABASE_URL not set — using in-memory storage");
    }

    // Try connecting to existing curlyos-core Redis (port 6379)
    #[cfg(feature = "postgres-redis")]
    {
        let redis_url = std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
        match noesis::storage::backends::redis_backend::RedisBackend::connect(&redis_url, "noesis:").await {
            Ok(redis) => {
                composite.redis = Some(Arc::new(redis));
                tracing::info!("[main] connected to Redis (:6379)");
            }
            Err(e) => tracing::warn!("[main] Redis unavailable (continuing without): {}", e),
        }
    }

    let storage: Arc<dyn noesis::storage::store::Storage> = Arc::new(composite);
    let field_ctx = FieldContext::new(kernel.event_bus.clone(), storage.clone());

    // Wire the EventBridge for signal persistence
    let event_bridge = std::sync::Arc::new(noesis::storage::event_store::EventBridge::new());
    if let Some(event_store) = create_event_store(storage.clone()) {
        let _ = event_bridge.set_store(event_store).await;
    }
    kernel.runtime.spawn("event-bridge", {
        let bus = kernel.event_bus.clone();
        let bridge = event_bridge.clone();
        async move {
            // Subscribe to all signal types and persist them
            for st in ALL_SIGNAL_TYPES {
                let mut rx = bus.subscribe_receiver(st.clone());
                let bridge = bridge.clone();
                tokio::spawn(async move {
                    while let Ok(sig) = rx.recv().await {
                        bridge.persist_signal(&sig).await;
                    }
                });
            }
        }
    });

    // -----------------------------------------------------------------------
    // 2. Register signal types with descriptions
    // -----------------------------------------------------------------------
    tracing::info!("[main] registering signal types...");
    kernel.registry.register_signal(types::INGEST_REQUEST, "A raw text ingested from an external source");
    kernel.registry.register_signal(types::EPISODE_RECORDED, "A structured episode recorded from raw experience");
    kernel.registry.register_signal(types::FACT_EXTRACTED, "A fact extracted from an episode");
    kernel.registry.register_signal(types::MEMORY_CONSOLIDATED, "Memory consolidation completed");
    kernel.registry.register_signal(types::PATTERN_DETECTED, "A recurring pattern detected across memories");
    kernel.registry.register_signal(types::BELIEF_CHANGED, "A belief was created, updated, or invalidated");
    kernel.registry.register_signal(types::TRAIT_DETECTED, "A personality trait was detected");
    kernel.registry.register_signal(types::IDENTITY_UPDATED, "The identity self-model was updated");
    kernel.registry.register_signal(types::GOAL_CREATED, "A new goal was created");
    kernel.registry.register_signal(types::GOAL_COMPLETED, "A goal was completed");
    kernel.registry.register_signal(types::DECISION_EVALUATED, "A decision was evaluated");
    kernel.registry.register_signal(types::ATTENTION_SHIFTED, "Attention shifted to a new focus");
    kernel.registry.register_signal(types::CURIOSITY_DETECTED, "A knowledge gap was detected");
    kernel.registry.register_signal(types::NARRATIVE_GENERATED, "A coherent narrative was generated");
    kernel.registry.register_signal(types::ENTITY_CREATED, "A knowledge entity was created");
    kernel.registry.register_signal(types::EDGE_CREATED, "A relation was created between entities");
    kernel.registry.register_signal(types::TRIPLES_EXTRACTED, "Triples were extracted from content");
    tracing::info!("[main] {} signal types registered", kernel.registry.list_signals().len());

    // -----------------------------------------------------------------------
    // 3. Register field factories
    // -----------------------------------------------------------------------
    tracing::info!("[main] registering fields...");
    kernel.registry.register_field("memory", Box::new(|| Box::new(noesis::fields::memory::MemoryField::new())));
    kernel.registry.register_field("identity", Box::new(|| Box::new(noesis::fields::identity::IdentityField::new())));
    kernel.registry.register_field("executive", Box::new(|| Box::new(noesis::fields::executive::ExecutiveField::new())));
    kernel.registry.register_field("awareness", Box::new(|| Box::new(noesis::fields::awareness::AwarenessField::new())));
    kernel.registry.register_field("simulation", Box::new(|| Box::new(noesis::fields::simulation::SimulationField::new())));
    kernel.registry.register_field("knowledge_graph", Box::new(|| Box::new(noesis::fields::graph::GraphField::new())));
    tracing::info!("[main] {} fields registered", kernel.registry.list_fields().len());

    // Create and initialize field instances
    // Wrap in Arc<Mutex> so the background snapshot task can access them
    let field_instances: std::sync::Arc<tokio::sync::Mutex<Vec<Box<dyn noesis::field::field::Field>>>> =
        std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
    for name in kernel.registry.list_fields() {
        if let Some(mut field) = kernel.registry.create_field(&name) {
            field.init(&field_ctx).await?;
            tracing::info!("[main] field initialized: {}", field.name());
            field_instances.lock().await.push(field);
        }
    }

    // Background task: snapshot field states to the field cache every 3 seconds
    let f_cache = field_cache.clone();
    let f_instances = field_instances.clone();
    kernel.runtime.spawn("field-snapshot", async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            if let Ok(mut instances) = f_instances.try_lock() {
                for field in instances.iter_mut() {
                    let name = field.name().to_string();
                    let state = field.state();
                    // Try serializing common field states
                    let json_val = try_serialize_field_state(&name, &*state);
                    if let Some(val) = json_val {
                        f_cache.insert(name, val);
                    }
                }
            }
        }
    });

    // -----------------------------------------------------------------------
    // 4. Register and subscribe processors
    // -----------------------------------------------------------------------
    tracing::info!("[main] registering processors...");
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
    processor_registry.register(Box::new(noesis::processors::reflection::ReflectionProcessor::new()));
    tracing::info!("[main] {} processors registered", processor_registry.len());

    tracing::info!("[main] processors: {:?}", processor_registry.names());

    // -----------------------------------------------------------------------
    // 4a. Wire metrics + API state
    // -----------------------------------------------------------------------
    let metrics = Arc::new(MetricsCollector::new());

    let kernel_snapshot = KernelSnapshot {
        fields: kernel.registry.list_fields(),
        processors: processor_registry.names(),
        signal_types: kernel.registry.list_signals(),
    };

    let api_state = ApiState::new(
        kernel.event_bus.clone(),
        metrics.clone(),
        kernel_snapshot,
        system_state.clone(),
        field_cache.clone(),
    );

    // -----------------------------------------------------------------------
    // 5. Create the signal processing cascade
    // -----------------------------------------------------------------------
    tracing::info!("[main] starting signal processing cascade...");

    // Subscribe to all known signal types
    let mut signal_rxs: Vec<tokio::sync::broadcast::Receiver<SignalArc>> = Vec::new();
    for signal_type in ALL_SIGNAL_TYPES {
        let rx = kernel.event_bus.subscribe_receiver(signal_type.clone());
        signal_rxs.push(rx);
    }

    let (signal_tx, mut signal_rx) = tokio::sync::mpsc::channel::<SignalArc>(1024);

    // Forward all broadcast receivers into the single mpsc channel
    for mut rx in signal_rxs {
        let tx = signal_tx.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(signal) => {
                        if tx.send(signal).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("[main] signal bus lagged, skipped {} messages", n);
                    }
                }
            }
        });
    }
    drop(signal_tx);

    // -----------------------------------------------------------------------
    // 6. Main recursive cascade loop
    // -----------------------------------------------------------------------
    tracing::info!("[main] entering recursive cascade loop...");
    tracing::info!("[main] system ready — waiting for signals");

    let cascade_metrics = metrics.clone();
    let cascade_fields = field_instances.clone();
    let cascade_handle = tokio::spawn(async move {
        use std::collections::VecDeque;

        let mut cascade_queue: VecDeque<SignalArc> = VecDeque::new();
        let mut equilibrium_count: u64 = 0;

        loop {
            // Phase 1: Drain the external signal channel into our cascade queue
            loop {
                match signal_rx.try_recv() {
                    Ok(sig) => {
                        cascade_queue.push_back(sig);
                    }
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => {
                        tracing::warn!("[Cascade] signal channel closed");
                        return;
                    }
                }
            }

            // Phase 2: Process the cascade queue until equilibrium (empty queue)
            while let Some(signal) = cascade_queue.pop_front() {
                let signal_type = signal.signal_type();
                let depth = signal.meta().depth;

                // Record signal in metrics
                cascade_metrics.record_signal(&signal_type.to_string());

                tracing::info!(
                    "[Cascade] processing signal: {} (depth={})",
                    signal_type,
                    depth
                );

                let start = std::time::Instant::now();

                // Clone signal for field dispatching
                let signal_for_fields = signal.clone();

                // Dispatch to matching processors
                let emitted = processor_registry
                    .dispatch(&field_ctx, signal)
                    .await;

                let elapsed = start.elapsed().as_nanos() as u64;

                // Record processor latencies
                cascade_metrics.record_processor_latency("cascade.dispatch", elapsed);

                // Also notify fields so they can update their state
                if let Ok(mut fields) = cascade_fields.try_lock() {
                    for field in fields.iter_mut() {
                        if let Err(e) = field.handle_signal(&field_ctx, signal_for_fields.clone()).await {
                            tracing::trace!("[Cascade] field {} error: {}", field.name(), e);
                        }
                    }
                }

                if emitted.is_empty() {
                    tracing::trace!("[Cascade] no processors emitted from {}", signal_type);
                } else {
                    tracing::info!(
                        "[Cascade] {} signal(s) emitted from {}",
                        emitted.len(),
                        signal_type
                    );
                }

                // Queue emitted signals for recursive processing
                for sig in emitted {
                    cascade_metrics.record_signal(&sig.signal_type().to_string());
                    cascade_queue.push_back(sig);
                }

                equilibrium_count = 0;
            }

            // Phase 3: Equilibrium — no signals in queue, no external signals pending
            if equilibrium_count == 0 {
                tracing::info!("[Cascade] network reached equilibrium ✓");
            }
            equilibrium_count += 1;

            // Wait before checking for new external signals
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    });

    // -----------------------------------------------------------------------
    // 6a. Inject a demo signal (remove in production)
    // -----------------------------------------------------------------------
    {
        let event_bus = kernel.event_bus.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(2)).await;
            let signal = noesis::signals::IngestRequest::new(
                "I went for a run in the park. The birds were singing and the air was fresh.",
                "demo",
            );
            event_bus.publish(Arc::new(signal));
            tracing::info!("[main] demo signal injected — cascade should propagate");
        });
    }

    // -----------------------------------------------------------------------
    // 7. REST API — full HTTP surface
    // -----------------------------------------------------------------------
    if enable_rest {
        let app = noesis::interfaces::rest::router(api_state.clone());
        let addr = format!("127.0.0.1:{}", port);
        let listen_handle = tokio::spawn(async move {
            tracing::info!("[REST] API listening on {}", addr);
            let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            axum::serve(listener, app).await.unwrap();
        });
        kernel.runtime.spawn("rest-api", async {
            listen_handle.await.ok();
        });
    }

    // -----------------------------------------------------------------------
    // 8. Wait for shutdown
    // -----------------------------------------------------------------------
    tracing::info!("[main] Noesis is running. Press Ctrl-C to shut down.");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("[main] received Ctrl-C, shutting down...");
        }
        _ = cascade_handle => {
            tracing::info!("[main] cascade loop ended");
        }
    }

    kernel.shutdown().await?;
    tracing::info!("[main] goodbye");
    Ok(())
}

/// Inject a raw experience and observe the signal cascade.
async fn run_inject(text: &str, source: &str) -> Result<()> {
    // Create a minimal kernel to process the injection
    let mut kernel = Kernel::new();
    let storage = Arc::new(MemoryStore::new());
    let _field_ctx = FieldContext::new(kernel.event_bus.clone(), storage);

    // Subscribe to all known signal types to observe the cascade
    let signal_types = [
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
    ];

    let mut receivers: Vec<tokio::sync::broadcast::Receiver<SignalArc>> = Vec::new();
    for st in &signal_types {
        let rx = kernel.event_bus.subscribe_receiver(st.clone());
        receivers.push(rx);
    }

    // Merge all receivers into a single channel
    let (tx, mut rx) = tokio::sync::mpsc::channel::<SignalArc>(256);
    for mut recv in receivers {
        let tx = tx.clone();
        tokio::spawn(async move {
            loop {
                match recv.recv().await {
                    Ok(sig) => {
                        if tx.send(sig).await.is_err() {
                            break;
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("[inject] receiver lagged by {}", n);
                        break;
                    }
                }
            }
        });
    }
    drop(tx);

    tracing::info!("[inject] injecting: {}", &text[..30.min(text.len())]);

    // Publish the IngestRequest signal
    let signal = IngestRequest::new(text, source);
    kernel.event_bus.publish(Arc::new(signal));

    // Wait for signals to propagate
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Collect all signals received
    let mut count = 0;
    loop {
        match rx.try_recv() {
            Ok(sig) => {
                count += 1;
                tracing::info!(
                    "[inject] signal #{}: {} (depth={})",
                    count,
                    sig.signal_type(),
                    sig.meta().depth
                );
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => break,
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => break,
        }
    }

    tracing::info!("[inject] done — {} signals captured", count);
    kernel.shutdown().await?;
    Ok(())
}

/// Inspect registered components and their state.
async fn run_inspect(target: &str, name: Option<&str>) -> Result<()> {
    tracing::info!("[inspect] target={}, name={:?}", target, name);
    // Placeholder — will query field states when the kernel is running
    tracing::info!("[inspect] use `noesis list` to see registered components");
    Ok(())
}

/// Try to serialize a field's state to JSON by downcasting to known types.
fn try_serialize_field_state(name: &str, state: &dyn std::any::Any) -> Option<serde_json::Value> {
    match name {
        "memory" => state
            .downcast_ref::<noesis::fields::memory::MemoryFieldState>()
            .and_then(|s| serde_json::to_value(s).ok()),
        "identity" => state
            .downcast_ref::<noesis::fields::identity::IdentityFieldState>()
            .and_then(|s| serde_json::to_value(s).ok()),
        "executive" => state
            .downcast_ref::<noesis::fields::executive::ExecutiveFieldState>()
            .and_then(|s| serde_json::to_value(s).ok()),
        "awareness" => state
            .downcast_ref::<noesis::fields::awareness::AwarenessFieldState>()
            .and_then(|s| serde_json::to_value(s).ok()),
        "simulation" => state
            .downcast_ref::<noesis::fields::simulation::SimulationFieldState>()
            .and_then(|s| serde_json::to_value(s).ok()),
        "knowledge_graph" => state
            .downcast_ref::<noesis::engines::graph::types::GraphSnapshot>()
            .and_then(|s| serde_json::to_value(s).ok()),
        _ => None,
    }
}

/// Create an event store from the available storage backend.
fn create_event_store(_storage: Arc<dyn noesis::storage::store::Storage>) -> Option<std::sync::Arc<dyn EventStore>> {
    // For now, always create an in-memory event store
    // In the future, this could use Postgres or Redis
    Some(std::sync::Arc::new(MemoryEventStore::new()))
}

/// List registered components.
async fn run_list(target: &str) -> Result<()> {
    tracing::info!("[list] target={}", target);
    // Placeholder — shows what would be registered
    match target {
        "fields" => {
            println!("--- Fields ---");
            println!("  memory      — Episodic and semantic memory state");
            println!("  identity    — Beliefs, traits, and self-model");
            println!("  executive   — Goals and active intentions");
            println!("  awareness   — Current focus and salience map");
            println!("  simulation  — What-if scenarios");
        }
        "processors" => {
            println!("--- Processors ---");
            println!("  episode     — Raw text → EpisodeRecorded");
            println!("  belief      — Memories → BeliefChanged");
            println!("  identity    — Beliefs → IdentityUpdated");
            println!("  narrative   — Episodes → NarrativeGenerated");
            println!("  goal        — Identity → GoalCreated/Completed");
            println!("  attention   — Signals → AttentionShifted");
            println!("  curiosity   — Episodes → CuriosityDetected");
        }
        "signals" => {
            println!("--- Signals ---");
            println!("  ingest.request      — Raw text input");
            println!("  episode.recorded    — Structured episode");
            println!("  fact.extracted      — Extracted fact");
            println!("  memory.consolidated — Consolidation done");
            println!("  pattern.detected    — Recurring pattern");
            println!("  belief.changed      — Belief update");
            println!("  trait.detected      — Trait detected");
            println!("  identity.updated    — Self-model change");
            println!("  goal.created        — New goal");
            println!("  goal.completed      — Goal done");
            println!("  decision.evaluated  — Decision outcome");
            println!("  attention.shifted   — Focus change");
            println!("  curiosity.detected  — Knowledge gap");
            println!("  narrative.generated — Story built");
        }
        _ => {
            // For "all" or unknown targets, print everything with a note
            println!("--- Fields ---");
            println!("  memory      — Episodic and semantic memory state");
            println!("  identity    — Beliefs, traits, and self-model");
            println!("  executive   — Goals and active intentions");
            println!("  awareness   — Current focus and salience map");
            println!("  simulation  — What-if scenarios");
            println!();
            println!("--- Processors ---");
            println!("  episode     — Raw text → EpisodeRecorded");
            println!("  belief      — Memories → BeliefChanged");
            println!("  identity    — Beliefs → IdentityUpdated");
            println!("  narrative   — Episodes → NarrativeGenerated");
            println!("  goal        — Identity → GoalCreated/Completed");
            println!("  attention   — Signals → AttentionShifted");
            println!("  curiosity   — Episodes → CuriosityDetected");
            println!();
            println!("--- Signals ---");
            println!("  ingest.request      — Raw text input");
            println!("  episode.recorded    — Structured episode");
            println!("  fact.extracted      — Extracted fact");
            println!("  memory.consolidated — Consolidation done");
            println!("  pattern.detected    — Recurring pattern");
            println!("  belief.changed      — Belief update");
            println!("  trait.detected      — Trait detected");
            println!("  identity.updated    — Self-model change");
            println!("  goal.created        — New goal");
            println!("  goal.completed      — Goal done");
            println!("  decision.evaluated  — Decision outcome");
            println!("  attention.shifted   — Focus change");
            println!("  curiosity.detected  — Knowledge gap");
            println!("  narrative.generated — Story built");
            if target != "all" {
                tracing::warn!("[list] unknown target: {}. Showing all components.", target);
            }
        }
    }
    Ok(())
}
