use std::sync::Arc;
use dashmap::DashMap;

/// A capability that a processor provides.
#[derive(Debug, Clone)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub description: String,
    pub confidence: f32,
    pub processor: String,
}

/// Registry of capabilities — what cognitive operations the system can perform.
///
/// Capabilities enable dynamic processor discovery. Each processor declares
/// what it can do, and other processors or interfaces can query by capability.
#[derive(Clone)]
pub struct CapabilityRegistry {
    capabilities: Arc<DashMap<String, Vec<Capability>>>,
}

impl CapabilityRegistry {
    pub fn new() -> Self {
        Self {
            capabilities: Arc::new(DashMap::new()),
        }
    }

    /// Register a capability provided by a processor.
    pub fn register(&self, capability: Capability) {
        self.capabilities
            .entry(capability.id.clone())
            .or_insert_with(Vec::new)
            .push(capability);
    }

    /// Find processors that provide a given capability.
    pub fn find_providers(&self, capability_id: &str) -> Vec<Capability> {
        self.capabilities
            .get(capability_id)
            .map(|e| e.value().clone())
            .unwrap_or_default()
    }

    /// List all registered capability IDs.
    pub fn list(&self) -> Vec<String> {
        self.capabilities.iter().map(|e| e.key().clone()).collect()
    }

    /// Check if a capability is available.
    pub fn has(&self, capability_id: &str) -> bool {
        self.capabilities.contains_key(capability_id)
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}
