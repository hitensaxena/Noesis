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
//!   /api/signals         — Signal types/history
//!   /api/observability/* — Metrics and pipeline tracing

pub mod handlers;
pub mod observability;

use std::sync::Arc;
use axum::{
    Router,
    routing::{get, post},
};
use tower_http::cors::CorsLayer;

use crate::eventbus::bus::EventBus;
use crate::metrics::metrics::MetricsCollector;
use crate::eventbus::signal::SignalType;
use crate::core::state::{SystemState, FieldStateCache};

// Re-export handler functions for the router builder
use handlers::health::health;
use handlers::ingest::ingest;
use handlers::stats::{get_stats, signal_stats};
use handlers::memories::{list_memories, create_memory, list_episodes};
use handlers::graph::{get_graph, graph_sources, expand_entity};
use handlers::identity::get_identity;
use handlers::cognition::{meta, reflection, narrative};
use handlers::signals::{list_signal_types, inject_signal, signal_history};
use observability::{overview, signal_metrics, processor_metrics, cascade_trace};

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
}

impl ApiState {
    pub fn new(
        event_bus: Arc<EventBus>,
        metrics: Arc<MetricsCollector>,
        kernel: KernelSnapshot,
        system_state: Arc<SystemState>,
        field_cache: FieldStateCache,
    ) -> Self {
        Self {
            event_bus,
            metrics,
            kernel: Arc::new(tokio::sync::Mutex::new(kernel)),
            system_state,
            field_cache,
        }
    }

    /// Update the kernel snapshot (called periodically or on field/processor changes).
    pub async fn update_kernel(&self, snapshot: KernelSnapshot) {
        let mut k = self.kernel.lock().await;
        *k = snapshot;
    }
}

/// Build the Noesis REST API router with all routes.
pub fn router(state: ApiState) -> Router {
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
        // Observability
        .route("/api/observability/overview", get(overview))
        .route("/api/observability/signals", get(signal_metrics))
        .route("/api/observability/processors", get(processor_metrics))
        .route("/api/observability/cascade", get(cascade_trace))
        // Layer CORS for development
        .layer(CorsLayer::permissive())
        .with_state(state)
}
