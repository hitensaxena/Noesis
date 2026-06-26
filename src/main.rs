//! Noesis daemon entrypoint.
//!
//! Wires the kernel, registers all fields and processors,
//! starts the event bus, and manages the signal propagation loop.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use tracing;

use noesis::kernel::beat_coordinator::BeatCoordinator;
use noesis::kernel::kernel::Kernel;
use noesis::kernel::state::{new_field_cache, SystemState};
use noesis::kernel::signal::SignalArc;
use noesis::field_runtime::context::FieldContext;
use noesis::field_runtime::runtime::FieldRuntime;
use noesis::interfaces::cli::{Cli, Commands};
use noesis::signals::types;
use noesis::signals::IngestRequest;
use noesis::storage::memory_store::MemoryStore;
use noesis::storage::event_store::{EventStore, MemoryEventStore};
use noesis::kernel::metrics::MetricsCollector;
use noesis::interfaces::rest::{ApiState, KernelSnapshot};

/// All registered signal types that processors subscribe to.
const ALL_SIGNAL_TYPES: &[noesis::kernel::signal::SignalType] = &[
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
    types::OBSERVER_TRANSITION_DETECTED,
    types::METACOGNITION_INSIGHT,
    types::MOOD_ESTIMATED,
    types::CONTEXT_ASSEMBLED,
    types::EPISTEMICS_CLASSIFIED,
    types::HEALTH_STATUS_CHANGED,
    types::VALUES_REFINED,
    types::PRINCIPLES_DERIVED,
    types::PLANNING_PLAN_READY,
    types::PROJECT_CREATED,
    types::TASK_CREATED,
    types::EXECUTION_STARTED,
    types::EVALUATION_COMPLETED,
    types::RISK_ASSESSED,
    types::RECOVERY_STARTED,
    types::WORLD_MODEL_UPDATED,
    types::ASSUMPTION_TESTED,
    types::SCENARIO_READY,
    types::COUNTERFACTUAL_READY,
    types::FORECAST_READY,
    types::SIMULATION_RISK_ASSESSED,
    types::HYPOTHESIS_GENERATED,
    types::CONCLUSION_READY,
    types::MENTAL_MODEL_UPDATED,
    types::ANALOGY_DETECTED,
    types::SYNTHESIS_READY,
    types::CONCEPT_FORMED,
    types::PRIORITY_REORDERED,
    types::STRATEGY_UPDATED,
    types::OPPORTUNITY_DETECTED,
    // Kernel scheduler beats
    types::BEAT_IMMEDIATE,
    types::BEAT_FAST,
    types::BEAT_MEDIUM,
    types::BEAT_SLOW,
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
        Commands::Start { rest, port, mcp, mcp_port, storage, database_url, redis_url } => {
            run_daemon(rest, port, mcp, mcp_port, &storage, database_url.as_deref(), redis_url.as_deref()).await
        },
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
async fn run_daemon(
    enable_rest: bool,
    #[allow(unused_variables)] port: u16,
    enable_mcp: bool,
    #[allow(unused_variables)] mcp_port: u16,
    storage_backend: &str,
    database_url: Option<&str>,
    redis_url: Option<&str>,
) -> Result<()> {
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

    // Try connecting to Postgres
    #[cfg(feature = "postgres-redis")]
    {
        let pg_env = std::env::var("NOESIS_DATABASE_URL").ok();
        let pg_url = database_url.or(pg_env.as_deref());
        let config: tokio_postgres::Config = match pg_url {
            Some(url) => url.parse().unwrap_or_else(|_| {
                let mut c = tokio_postgres::Config::new();
                c.host("127.0.0.1").port(5432).dbname("noesis");
                c.user("noesis").password("noesis");
                c
            }),
            None => {
                let mut c = tokio_postgres::Config::new();
                c.host("127.0.0.1").port(54321).dbname("curlyos");
                c.user("curlyos").password(std::env::var("CURLYOS_PG_PASSWORD").unwrap_or_else(|_| "curlyos".to_string()));
                c
            }
        };
        match noesis::storage::backends::postgres_backend::PostgresBackend::connect(&config).await {
            Ok(pg) => {
                composite.postgres = Some(Arc::new(pg));
                tracing::info!("[main] connected to Postgres");
            }
            Err(e) => {
                if storage_backend == "postgres" {
                    anyhow::bail!("[main] --storage postgres requested but Postgres unavailable: {}. Set --database-url or ensure Postgres is running.", e);
                }
                tracing::warn!("[main] Postgres unavailable — data lost on restart. Set --database-url or start Postgres. Error: {}", e);
            }
        }
    }

    // Try connecting to Redis
    #[cfg(feature = "postgres-redis")]
    {
        let r_url = redis_url.map(|s| s.to_string()).or_else(|| {
            Some(std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()))
        }).unwrap_or_else(|| "redis://127.0.0.1:6379".to_string());
        match noesis::storage::backends::redis_backend::RedisBackend::connect(&r_url, "noesis:").await {
            Ok(redis) => {
                composite.redis = Some(Arc::new(redis));
                tracing::info!("[main] connected to Redis");
            }
            Err(e) => tracing::debug!("[main] Redis unavailable (in-memory only): {}", e),
        }
    }

    let storage: Arc<dyn noesis::storage::store::Storage> = Arc::new(composite);
    let _field_ctx = FieldContext::new(kernel.event_bus.clone(), storage.clone());

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
    kernel.registry.register_signal(types::BEAT_FAST, "Kernel scheduler fast beat (~1s)");
    kernel.registry.register_signal(types::BEAT_MEDIUM, "Kernel scheduler medium beat (~60s)");
    kernel.registry.register_signal(types::BEAT_SLOW, "Kernel scheduler slow beat (~15min)");
    kernel.registry.register_signal(types::BEAT_IMMEDIATE, "Kernel scheduler immediate beat");
    tracing::info!("[main] {} signal types registered", kernel.registry.list_signals().len());

    // -----------------------------------------------------------------------
    // 3. Create PluginRegistry and register built-in Noesis plugin
    // -----------------------------------------------------------------------
    let plugin_registry = noesis::kernel::plugin::PluginRegistry::new();
    plugin_registry.register(Box::new(noesis::kernel::plugin::noesis_plugin::NoesisPlugin::new()));
    tracing::info!("[main] plugin registered: {} ({} capabilities, {} processors, {} signals)",
        plugin_registry.plugin_names().join(", "),
        plugin_registry.capability_ids().len(),
        plugin_registry.all_processors().len(),
        plugin_registry.all_signals().len(),
    );

    // Populate CapabilityRegistry from plugin capabilities
    let capability_registry = Arc::new(noesis::kernel::capabilities::CapabilityRegistry::new());
    for cap in plugin_registry.all_capabilities() {
        capability_registry.register(cap);
    }

    // -----------------------------------------------------------------------
    // 4. Create FieldRuntime — register fields and processors
    // -----------------------------------------------------------------------
    let metrics = Arc::new(MetricsCollector::new());
    let mut field_runtime = FieldRuntime::new();
    let field_ctx = noesis::field_runtime::context::FieldContext::new_with(
        kernel.event_bus.clone(),
        storage.clone(),
        metrics.clone(),
        capability_registry.clone(),
        "",
    );

    tracing::info!("[main] registering and initializing fields...");
    for (name, field) in [
        ("memory", Box::new(noesis::fields::memory::MemoryField::new()) as Box<dyn noesis::field_runtime::field::Field + Send>),
        ("identity", Box::new(noesis::fields::identity::IdentityField::new())),
        ("agency", Box::new(noesis::fields::agency::AgencyField::new())),
        ("action", Box::new(noesis::fields::action::ActionField::new())),
        ("awareness", Box::new(noesis::fields::awareness::AwarenessField::new())),
        ("reasoning", Box::new(noesis::fields::reasoning::ReasoningField::new())),
        ("simulation", Box::new(noesis::fields::simulation::SimulationField::new())),
        ("knowledge_graph", Box::new(noesis::fields::graph::GraphField::new())),
    ] {
        field_runtime.register_and_init_field(name, field, &field_ctx).await?;
    }
    tracing::info!("[main] {} fields initialized", field_runtime.field_names().len());

    // Register all processors from the built-in plugin
    tracing::info!("[main] registering processors from built-in plugin...");
    for proc in plugin_registry.all_processors() {
        let name = proc.name().to_string();
        field_runtime.register_processor(proc);
        tracing::info!("[main]   registered processor: {}", name);
    }
    tracing::info!("[main] {} processors registered", field_runtime.processor_names().len());

    // -----------------------------------------------------------------------
    // 4. Wire API state, metrics, and field snapshot cache
    // -----------------------------------------------------------------------
    let kernel_snapshot = KernelSnapshot {
        fields: kernel.registry.list_fields(),
        processors: field_runtime.processor_names(),
        signal_types: kernel.registry.list_signals(),
    };

    let api_state = ApiState::new(
        kernel.event_bus.clone(),
        metrics.clone(),
        kernel_snapshot,
        system_state.clone(),
        field_cache.clone(),
        capability_registry.clone(),
        Arc::new(plugin_registry),
    );

    // -----------------------------------------------------------------------
    // 4a. Wire SSE event stream for real-time cascade monitoring
    // -----------------------------------------------------------------------
    let event_tx = noesis::interfaces::rest::handlers::events::create_event_stream_channel();

    // Spawn one background task per signal type to forward signals to the SSE channel.
    // This mirrors the event-bridge pattern but for the web dashboard.
    for st in ALL_SIGNAL_TYPES {
        let mut rx = kernel.event_bus.subscribe_receiver(st.clone());
        let tx = event_tx.clone();
        tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(sig) => {
                        let msg = noesis::interfaces::rest::handlers::events::format_signal_event(
                            &sig.signal_type().to_string(),
                            sig.meta().depth,
                            sig.meta().activation,
                            &sig.meta().source,
                        );
                        let _ = tx.send(msg);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!("[event-stream] receiver lagged by {}", n);
                    }
                }
            }
        });
    }
    let api_state = api_state.with_event_stream(event_tx);
    // -----------------------------------------------------------------------
    // Get the CancellationToken for graceful shutdown coordination
    let shutdown_token = kernel.runtime.shutdown_token();

    tracing::info!("[main] starting signal processing cascade...");

    // Forward EventBus broadcast receivers into a single mpsc channel
    // so the cascade loop can await on one receiver instead of N.
    let (signal_tx, signal_rx) = tokio::sync::mpsc::channel::<SignalArc>(1024);

    for signal_type in ALL_SIGNAL_TYPES {
        let mut rx = kernel.event_bus.subscribe_receiver(signal_type.clone());
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
    // 6. Cascade loop — delegates signal dispatch to FieldRuntime
    // -----------------------------------------------------------------------
    tracing::info!("[main] entering cascade loop — delegating to FieldRuntime...");
    tracing::info!("[main] system ready — waiting for signals");

    let cascade_token = shutdown_token.clone();
    let cascade_handle = tokio::spawn(async move {
        let mut field_runtime = field_runtime;
        let mut signal_rx = signal_rx;
        let field_ctx = field_ctx;
        let cascade_metrics = metrics;
        let f_cache = field_cache;
        let token = cascade_token;

        loop {
            // Shutdown check — if the token is cancelled, drain current work and exit
            if token.is_cancelled() {
                tracing::info!("[Cascade] shutdown signal received — cascades drained, exiting");
                break;
            }

            // Wait for the next external signal (or a forwarded published signal)
            match signal_rx.recv().await {
                Some(signal) => {
                    let signal_type = signal.signal_type().to_string();

                    cascade_metrics.record_signal(&signal_type);

                    tracing::info!(
                        "[Cascade] processing root signal: {} (depth={})",
                        signal_type,
                        signal.meta().depth,
                    );

                    let start = std::time::Instant::now();
                    let result = field_runtime.process_signal_cascade(&field_ctx, signal).await;
                    let elapsed = start.elapsed();

                    cascade_metrics.record_processor_latency("cascade.dispatch", elapsed.as_nanos() as u64);

                    if result.total_signals > 1 {
                        tracing::info!(
                            "[Cascade] cascade complete — {} total signals processed from {} (took {:?})",
                            result.total_signals,
                            signal_type,
                            elapsed,
                        );
                    } else {
                        tracing::trace!(
                            "[Cascade] no cascade from {} — signal absorbed without emission",
                            signal_type,
                        );
                    }

                    // Snapshot field state to cache after the full cascade settles
                    for (name, state) in field_runtime.snapshot_states() {
                        if let Some(val) = try_serialize_field_state(&name, &*state) {
                            f_cache.insert(name, val);
                        }
                    }
                }
                None => {
                    tracing::warn!("[Cascade] signal channel closed — exiting cascade loop");
                    break;
                }
            }
        }
    });

    // -----------------------------------------------------------------------
    // 6a. Start BeatCoordinator — emits cognitive beats for processor scheduling
    // -----------------------------------------------------------------------
    {
        let mut beat_coordinator = BeatCoordinator::new();
        let token = kernel.runtime.shutdown_token();
        beat_coordinator.spawn(
            kernel.event_bus.clone(),
            token,
            types::BEAT_FAST,
            types::BEAT_MEDIUM,
            types::BEAT_SLOW,
        );
        tracing::info!("[main] beat coordinator started (fast=1s, medium=60s, slow=15min)");
    }

    // -----------------------------------------------------------------------
    // 6b. Inject a demo signal (remove in production)
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
    // 7b. Start MCP server (AI agent protocol)
    // -----------------------------------------------------------------------
    if enable_mcp {
        let mcp_app = noesis::interfaces::mcp_handler::mcp_router().with_state(api_state.clone());
        let mcp_addr = format!("127.0.0.1:{}", mcp_port);
        let mcp_handle = tokio::spawn(async move {
            tracing::info!("[MCP] server listening on {}", mcp_addr);
            let listener = tokio::net::TcpListener::bind(&mcp_addr).await.unwrap();
            axum::serve(listener, mcp_app).await.unwrap();
        });
        kernel.runtime.spawn("mcp-server", async {
            mcp_handle.await.ok();
        });
        tracing::info!("[main] MCP server started on port {}", mcp_port);
    }

    // -----------------------------------------------------------------------
    // 8. Wait for shutdown — 3-phase graceful termination
    // -----------------------------------------------------------------------
    tracing::info!("[main] Noesis is running. Press Ctrl-C to shut down.");

    // Phase 0: Background task waits for Ctrl-C and initiates shutdown
    let init_token = shutdown_token.clone();
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.ok();
        tracing::info!("[main] received Ctrl-C — initiating graceful shutdown");
        tracing::info!("[main] Phase 1: stopping new request acceptance...");
        init_token.cancel();
    });

    // Wait for the cascade loop to drain (it checks `token.is_cancelled()` after each cascade)
    let _ = cascade_handle.await;

    // Phase 2: cascade has drained, now shut down the kernel
    tracing::info!("[main] Phase 2: cascade drained — shutting down kernel...");
    kernel.shutdown().await?;

    // Phase 3: goodbye
    tracing::info!("[main] shutdown complete. goodbye.");
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
        "agency" => state
            .downcast_ref::<noesis::fields::agency::AgencyFieldState>()
            .and_then(|s| serde_json::to_value(s).ok()),
        "action" => state
            .downcast_ref::<noesis::fields::action::ActionFieldState>()
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
        "reasoning" => state
            .downcast_ref::<noesis::fields::reasoning::state::ReasoningFieldState>()
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
            println!("  agency      — Goals, priorities, and what to pursue");
            println!("  action      — Projects, tasks, and execution");
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
            println!("  agency      — Goals, priorities, and what to pursue");
            println!("  action      — Projects, tasks, and execution");
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
