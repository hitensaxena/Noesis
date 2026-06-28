use serde::{Deserialize, Serialize};

// Re-export domain types for cohesive state access
pub use super::domains::beliefs::Belief;
pub use super::domains::traits::Trait;
pub use super::domains::values::Value;
pub use super::domains::roles::Role;
pub use super::domains::principles::Principle;
pub use super::domains::self_model::SelfModel;
pub use super::domains::personality::PersonalityProfile;
pub use super::domains::timeline::TimelineEntry;
pub use super::domains::evolution::IdentityProjection;
pub use super::domains::narrative_self::NarrativeSelf;

/// State of the Identity Field.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityFieldState {
    pub beliefs: Vec<Belief>,
    pub traits: Vec<Trait>,
    pub identity_version: u32,
}
