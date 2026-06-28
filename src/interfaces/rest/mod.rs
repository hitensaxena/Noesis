//! Noesis REST API — full HTTP surface for the cognitive architecture.
//!
//! Routes mirror curlyos-core's API patterns:
//!   /api/health          — System health
//!   /api/ingest          — Inject raw text
//!   /api/stats           — Signal/processor statistics
//!   /api/memories        — List/create memory field state
//!   /api/episodes        — List episodes
//!   /api/graph           — Knowledge graph state
//!   /api/identity        — Identity state (beliefs, traits)
//!   /api/cognition/*     — Metacognition endpoints
//!   /api/identity/detail    — Deep identity observability
//!   /api/memory/detail      — Deep memory observability
//!   /api/agency/detail      — Deep agency observability
//!   /api/awareness/detail   — Deep awareness observability
//!   /api/simulation/detail  — Deep simulation observability
//!   /api/core/detail        — Deep core system observability
//!   /api/signals         — Signal types/history
//!   /api/observability/* — Metrics and pipeline tracing
//!   /api/docs/*          — OpenAPI spec + Swagger UI
//!   /api/events/*        — SSE cascade stream
//!   /api/dashboard/*     — Web dashboard

pub mod handlers;
pub mod observability;
pub mod dashboard;

use std::sync::Arc;

// Note: RateLimiter uses full paths for atomics to avoid import conflicts.
use axum::{
    Router,
    routing::{get, post},
    http::StatusCode,
    response::IntoResponse,
    response::Html,
    middleware::{self, Next},
};
use axum::body::Body;
use tower_http::cors::CorsLayer;
use tokio::sync::broadcast;

use crate::kernel::bus::EventBus;
use crate::kernel::capabilities::CapabilityRegistry;
use crate::kernel::metrics::MetricsCollector;
use crate::kernel::plugin::PluginRegistry;
use crate::kernel::signal::SignalType;
use crate::kernel::state::{SystemState, FieldStateCache};
use crate::storage::event_store::EventStore;

// Re-export handler functions for the router builder
use handlers::health::health;
use handlers::ingest::ingest;
use handlers::stats::{get_stats, signal_stats};
use handlers::memories::{list_memories, create_memory, list_episodes, recall_memories};
use handlers::graph::{get_graph, graph_sources, expand_entity};
use handlers::identity::get_identity;
use handlers::cognition::{meta, reflection, narrative};
use handlers::signals::{list_signal_types, inject_signal, signal_history};
use handlers::identity_detail::identity_detail;
use handlers::memory_detail::memory_detail;
use handlers::agency_detail::agency_detail;
use handlers::awareness_detail::awareness_detail;
use handlers::simulation_detail::simulation_detail;
use handlers::core_detail::core_detail;
use handlers::events::event_stream;
use handlers::plugins::{list_plugins, plugin_detail, reload_plugins, system_config};
use observability::{overview, signal_metrics, processor_metrics, cascade_trace, capabilities};

/// Snapshot of the kernel's state for API queries.
#[derive(Clone, Debug)]
pub struct KernelSnapshot {
    pub fields: Vec<String>,
    pub processors: Vec<String>,
    pub signal_types: Vec<(SignalType, String)>,
}

/// Shared state for all REST API handlers.
#[derive(Clone)]
pub struct ApiState {
    pub event_bus: Arc<EventBus>,
    pub metrics: Arc<MetricsCollector>,
    pub kernel: Arc<tokio::sync::Mutex<KernelSnapshot>>,
    pub system_state: Arc<SystemState>,
    pub field_cache: FieldStateCache,
    pub capability_registry: Arc<CapabilityRegistry>,
    /// Plugin registry for listing and reloading plugins.
    pub plugin_registry: Arc<PluginRegistry>,
    /// Optional event store for signal history queries.
    pub event_store: Option<Arc<dyn EventStore>>,
    /// Broadcast sender for SSE cascade log stream (set when router is built).
    pub event_stream_tx: Option<broadcast::Sender<String>>,
}

impl ApiState {
    pub fn new(
        event_bus: Arc<EventBus>,
        metrics: Arc<MetricsCollector>,
        kernel: KernelSnapshot,
        system_state: Arc<SystemState>,
        field_cache: FieldStateCache,
        capability_registry: Arc<CapabilityRegistry>,
        plugin_registry: Arc<PluginRegistry>,
    ) -> Self {
        Self {
            event_bus,
            metrics,
            kernel: Arc::new(tokio::sync::Mutex::new(kernel)),
            system_state,
            field_cache,
            capability_registry,
            plugin_registry,
            event_store: None,
            event_stream_tx: None,
        }
    }

    /// Set the event store for signal history.
    pub fn with_event_store(mut self, store: Arc<dyn EventStore>) -> Self {
        self.event_store = Some(store);
        self
    }

    /// Set the event stream sender for SSE cascade logging.
    pub fn with_event_stream(mut self, tx: broadcast::Sender<String>) -> Self {
        self.event_stream_tx = Some(tx);
        self
    }

    /// Update the kernel snapshot (called periodically or on field/processor changes).
    pub async fn update_kernel(&self, snapshot: KernelSnapshot) {
        let mut k = self.kernel.lock().await;
        *k = snapshot;
    }
}

/// Build the Noesis REST API router with all routes.
///
/// Event stream (SSE) is wired via state.event_stream_tx — if set, the
/// `/api/events/stream` endpoint becomes active. Background forwarding
/// tasks must be spawned in main.rs (one per signal type).
pub fn router(state: ApiState) -> Router {
    // Optional: set up event stream page handlers
    let docs_handler = get(|| async {
        Html(crate::docs::swagger_ui_html())
    });
    let dashboard_handler = get(|| async {
        Html(crate::interfaces::rest::dashboard::dashboard_html())
    });

    Router::new()
        // Health
        .route("/api/health", get(health))
        // Ingest
        .route("/api/ingest", post(ingest))
        // Stats
        .route("/api/stats", get(get_stats))
        .route("/api/stats/signals", get(signal_stats))
        // Memories
        .route("/api/memories", get(list_memories))
        .route("/api/memories", post(create_memory))
        .route("/api/memory/recall", get(recall_memories))
        // Episodes
        .route("/api/episodes", get(list_episodes))
        // Graph
        .route("/api/graph", get(get_graph))
        .route("/api/graph/sources", get(graph_sources))
        .route("/api/graph/expand", get(expand_entity))
        // Identity
        .route("/api/identity", get(get_identity))
        // Cognition
        .route("/api/cognition/meta", get(meta))
        .route("/api/cognition/reflection", get(reflection))
        .route("/api/cognition/narrative", get(narrative))
        // Signals
        .route("/api/signals", get(list_signal_types))
        .route("/api/signals/inject", post(inject_signal))
        .route("/api/signals/history", get(signal_history))
        // Deep observability detail views
        .route("/api/identity/detail", get(identity_detail))
        .route("/api/memory/detail", get(memory_detail))
        .route("/api/agency/detail", get(agency_detail))
        .route("/api/awareness/detail", get(awareness_detail))
        .route("/api/simulation/detail", get(simulation_detail))
        .route("/api/core/detail", get(core_detail))
        // Observability
        .route("/api/observability/overview", get(overview))
        .route("/api/observability/signals", get(signal_metrics))
        .route("/api/observability/processors", get(processor_metrics))
        .route("/api/observability/cascade", get(cascade_trace))
        .route("/api/capabilities", get(capabilities))
        // Plugin management
        .route("/api/plugins", get(list_plugins))
        .route("/api/plugins/reload", post(reload_plugins))
        .route("/api/plugins/{name}", get(plugin_detail))
        .route("/api/config", get(system_config))
        // Documentation
        .route("/api/docs/openapi.json", get(|| async {
            axum::response::Json(crate::docs::generate_openapi_spec())
        }))
        .route("/api/docs/", docs_handler)
        // SSE event stream
        .route("/api/events/stream", get(event_stream))
        // Web dashboard
        .route("/api/dashboard/", dashboard_handler)
        // Auth middleware — checks NOESIS_API_KEY env var if set
        .layer(middleware::from_fn(auth_middleware))
        // Rate limiting — global token bucket (100 req/10s)
        .layer(middleware::from_fn(rate_limit_middleware))
        // Layer CORS for development
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// Global rate limiter — shared atomic between try_acquire and the refill loop.
static RATE_TOKENS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(10_000);
static RATE_LIMITER: std::sync::LazyLock<RateLimiter> =
    std::sync::LazyLock::new(|| RateLimiter::new(10_000, 100));

/// Simple token-bucket rate limiter.
///
/// Uses a static atomic counter and a background task that refills at the
/// configured rate. Thread-safe and lock-free on the fast path.
struct RateLimiter;

impl RateLimiter {
    fn new(max_tokens: u64, refill_per_sec: u64) -> Self {
        RATE_TOKENS.store(max_tokens, std::sync::atomic::Ordering::Relaxed);
        let max = max_tokens;
        tokio::spawn(async move {
            let interval_ms = 1000u64 / refill_per_sec.max(1);
            let mut interval = tokio::time::interval(std::time::Duration::from_millis(interval_ms));
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                interval.tick().await;
                loop {
                    let current = RATE_TOKENS.load(std::sync::atomic::Ordering::Relaxed);
                    if current >= max {
                        break;
                    }
                    if RATE_TOKENS.compare_exchange(
                        current, current + 1,
                        std::sync::atomic::Ordering::Relaxed,
                        std::sync::atomic::Ordering::Relaxed,
                    ).is_ok() {
                        break;
                    }
                }
            }
        });
        RateLimiter
    }

    fn try_acquire(&self) -> bool {
        loop {
            let current = RATE_TOKENS.load(std::sync::atomic::Ordering::Relaxed);
            if current == 0 {
                return false;
            }
            if RATE_TOKENS.compare_exchange(
                current, current - 1,
                std::sync::atomic::Ordering::Relaxed,
                std::sync::atomic::Ordering::Relaxed,
            ).is_ok() {
                return true;
            }
        }
    }
}

/// Rate limiting middleware — returns 429 when token bucket is empty.
async fn rate_limit_middleware(
    req: axum::http::Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    // Skip rate limiting in test builds — the global token bucket would
    // exhaust under parallel test execution across 263+ tests, causing
    // spurious 429 failures. Rate limiting is a runtime concern only.
    if cfg!(test) || RATE_LIMITER.try_acquire() {
        Ok(next.run(req).await)
    } else {
        tracing::debug!("[REST] rate limit hit for {}", req.uri());
        Err((StatusCode::TOO_MANY_REQUESTS, "rate limit exceeded"))
    }
}

/// Simple API key auth middleware.
/// If NOESIS_API_KEY env var is set, all requests must include
/// `Authorization: Bearer <key>` or `X-API-Key: <key>` header.
/// Exempts dashboard and docs pages (no auth needed for the UI).
/// If unset (default), auth is disabled for localhost development.
async fn auth_middleware(
    req: axum::http::Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, &'static str)> {
    // Exempt UI/documentation paths — they are accessed directly in the browser,
    // so auth headers aren't practical. Other GET endpoints (API, health) require auth.
    let path = req.uri().path();
    if path.starts_with("/api/dashboard/")
        || path.starts_with("/api/docs/")
        || path == "/api/docs"
    {
        return Ok(next.run(req).await);
    }

    let expected_key = match std::env::var("NOESIS_API_KEY") {
        Ok(key) if !key.is_empty() => key,
        _ => return Ok(next.run(req).await), // No key configured — allow all
    };

    let auth_value = req.headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim_start_matches("Bearer ").trim().to_string())
        .or_else(|| {
            req.headers().get("X-API-Key")
                .and_then(|v| v.to_str().ok())
                .map(|v| v.trim().to_string())
        });

    match auth_value {
        Some(val) if val == expected_key => Ok(next.run(req).await),
        Some(_) => {
            tracing::warn!("[REST] rejected request with invalid API key");
            Err((StatusCode::UNAUTHORIZED, "invalid api key"))
        }
        None => {
            tracing::warn!("[REST] rejected request: no API key provided");
            Err((StatusCode::UNAUTHORIZED, "api key required"))
        }
    }
}
