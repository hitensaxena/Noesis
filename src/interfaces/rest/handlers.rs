//! REST API handlers — grouped by domain.
//!
//! Each submodule covers a coherent set of endpoints.
//! All handlers receive ApiState via axum's State extractor.

pub mod health;
pub mod ingest;
pub mod stats;
pub(crate) mod memories;
pub(crate) mod graph;
pub(crate) mod identity;
pub(crate) mod cognition;
pub(crate) mod signals;

// Deep observability detail endpoints
pub(crate) mod identity_detail;
pub(crate) mod memory_detail;
pub(crate) mod agency_detail;
pub(crate) mod awareness_detail;
pub(crate) mod simulation_detail;
pub(crate) mod core_detail;

// SSE cascade stream — public for main.rs wiring
pub mod events;

// Plugin management API
pub mod plugins;
