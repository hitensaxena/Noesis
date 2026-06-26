//! Knowledge graph domain — entities, relations, and their embeddings.
//!
//! This is the in-MemoryField home for the knowledge graph. Types and
//! logic are defined in `engines/graph/types.rs`; this module re-exports
//! them and adds MemoryField-specific knowledge operations.

pub use crate::engines::graph::types::{
    Entity, EntityCategory, GraphSnapshot, Relation, RelationType, Triple,
};
