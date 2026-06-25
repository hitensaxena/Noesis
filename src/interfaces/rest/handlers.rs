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
