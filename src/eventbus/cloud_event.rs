//! CloudEvents v1.0 envelope for Noesis event sourcing.
//!
//! Mirrors curlyos-core's `shared/events/__init__.py`:
//! - CloudEvents v1.0 envelope with typed-prefix ULID event ids
//! - Full type grammar: `art.curlybrackets.curlyos.<domain>.<verb>`
//! - Event catalog integration (closed catalog of known types)

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type prefix for all Noesis CloudEvents.
pub const FULL_TYPE_PREFIX: &str = "art.curlybrackets.curlyos.";

/// CloudEvents v1.0 envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudEvent {
    /// CloudEvents spec version (always "1.0").
    pub specversion: String,
    /// Full event type: `art.curlybrackets.curlyos.<domain>.<verb>`.
    #[serde(rename = "type")]
    pub event_type: String,
    /// Source of the event (e.g., "noesis", "curlyos-core").
    pub source: String,
    /// Unique event ID (ULID-style, prefixed with "evt").
    pub id: String,
    /// ISO-8601 timestamp of when the event occurred.
    pub time: String,
    /// Subject of the event (relevant resource identifier).
    pub subject: String,
    /// Event payload.
    pub data: serde_json::Value,
    /// Actor that triggered the event.
    pub actor: String,
    /// Scope context (e.g., `{"level": "user", "user_id": "usr_hiten"}`).
    pub scope: serde_json::Value,
}

impl CloudEvent {
    /// Build a new CloudEvent envelope with proper type prefixing and catalog validation.
    pub fn new(
        short_type: &str,
        subject: &str,
        data: serde_json::Value,
        actor: &str,
        source: &str,
        scope: serde_json::Value,
    ) -> Self {
        // Validate against the closed catalog
        crate::eventbus::catalog::validate_short_type(short_type)
            .expect("event type not in catalog");

        Self {
            specversion: "1.0".to_string(),
            event_type: full_type(short_type),
            source: source.to_string(),
            id: mint_event_id(),
            time: Utc::now().to_rfc3339(),
            subject: subject.to_string(),
            data,
            actor: actor.to_string(),
            scope,
        }
    }

    /// Get the short type (strip the prefix).
    pub fn short_type(&self) -> &str {
        short_type(&self.event_type)
    }

    /// Get the NATS subject for this event.
    pub fn subject_for(&self) -> String {
        subject_for(self.short_type())
    }

    /// Get the domain group for this event type.
    pub fn group(&self) -> &str {
        crate::eventbus::catalog::group_for(self.short_type())
    }
}

/// Build a full CloudEvents type from a short type.
pub fn full_type(short_type: &str) -> String {
    format!("{}{}", FULL_TYPE_PREFIX, short_type)
}

/// Strip the type prefix from a full type.
pub fn short_type(full: &str) -> &str {
    if let Some(rest) = full.strip_prefix(FULL_TYPE_PREFIX) {
        rest
    } else {
        full
    }
}

/// Generate a NATS subject for a short type.
pub fn subject_for(short_type: &str) -> String {
    format!("curlyos.{}", short_type)
}

/// Generate a ULID-style event ID.
/// Format: `evt_<timestamp_ms><random_hex>` to match curlyos-core's `mint("evt")`.
pub fn mint_event_id() -> String {
    let now = Utc::now();
    let ts = now.timestamp_millis();
    let suffix = Uuid::new_v4().to_string().replace('-', "");
    format!("evt{}{}", ts, &suffix[..12])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eventbus::catalog;

    #[test]
    fn test_full_type_roundtrip() {
        let short = "memory.fact.stored";
        let full = full_type(short);
        assert_eq!(full, "art.curlybrackets.curlyos.memory.fact.stored");
        assert_eq!(short_type(&full), short);
    }

    #[test]
    fn test_mint_event_id_format() {
        let id = mint_event_id();
        assert!(id.starts_with("evt"));
        assert!(id.len() > 10);
    }

    #[test]
    fn test_build_event() {
        // Use a type from the catalog
        let types = catalog::list_types();
        assert!(!types.is_empty(), "catalog should have event types");
    }
}
