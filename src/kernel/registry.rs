use dashmap::DashMap;
use tracing;

use crate::kernel::signal::SignalType;
use crate::field_runtime::field::Field;
use crate::processor::processor::Processor;

/// Thread-safe registry for field factories, processor factories, and signal metadata.
pub struct Registry {
    field_factories: DashMap<String, Box<dyn Fn() -> Box<dyn Field> + Send + Sync>>,
    field_instances: DashMap<String, Box<dyn Field>>,
    processor_factories: DashMap<String, Box<dyn Fn() -> Box<dyn Processor> + Send + Sync>>,
    signal_metadata: DashMap<SignalType, String>,
}

impl Registry {
    pub fn new() -> Self {
        Self {
            field_factories: DashMap::new(),
            field_instances: DashMap::new(),
            processor_factories: DashMap::new(),
            signal_metadata: DashMap::new(),
        }
    }

    pub fn register_field(&self, name: &str, factory: Box<dyn Fn() -> Box<dyn Field> + Send + Sync>) {
        tracing::info!("[Registry] registering field: {}", name);
        self.field_factories.insert(name.to_string(), factory);
    }

    pub fn register_processor(&self, name: &str, factory: Box<dyn Fn() -> Box<dyn Processor> + Send + Sync>) {
        tracing::info!("[Registry] registering processor: {}", name);
        self.processor_factories.insert(name.to_string(), factory);
    }

    pub fn register_signal(&self, signal_type: SignalType, description: &str) {
        self.signal_metadata
            .insert(signal_type, description.to_string());
    }

    pub fn create_field(&self, name: &str) -> Option<Box<dyn Field>> {
        self.field_factories
            .get(name)
            .map(|factory| (factory.value())())
    }

    pub fn create_processor(&self, name: &str) -> Option<Box<dyn Processor>> {
        self.processor_factories
            .get(name)
            .map(|factory| (factory.value())())
    }

    pub fn store_field_instance(&self, name: String, field: Box<dyn Field>) {
        self.field_instances.insert(name, field);
    }

    pub fn get_field_instance(&self, name: &str) -> Option<Box<dyn Field>> {
        self.field_instances
            .remove(name)
            .map(|(_, field)| field)
    }

    pub fn list_fields(&self) -> Vec<String> {
        self.field_factories.iter().map(|e| e.key().clone()).collect()
    }

    pub fn list_processors(&self) -> Vec<String> {
        self.processor_factories
            .iter()
            .map(|e| e.key().clone())
            .collect()
    }

    pub fn list_signals(&self) -> Vec<(SignalType, String)> {
        self.signal_metadata
            .iter()
            .map(|e| (e.key().clone(), e.value().clone()))
            .collect()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel::signal::SignalType;

    #[test]
    fn test_register_and_list_signals() {
        let registry = Registry::new();
        registry.register_signal(SignalType::new("test.signal"), "A test signal");
        let signals = registry.list_signals();
        assert_eq!(signals.len(), 1);
        assert!(signals.iter().any(|(t, _)| t.0 == "test.signal"));
    }

    #[test]
    fn test_register_and_list_fields() {
        let registry = Registry::new();
        registry.register_field("test", Box::new(|| Box::new(crate::fields::memory::MemoryField::new())));
        let fields = registry.list_fields();
        assert_eq!(fields.len(), 1);
    }

    #[test]
    fn test_create_field_by_name() {
        let registry = Registry::new();
        registry.register_field("memory", Box::new(|| Box::new(crate::fields::memory::MemoryField::new())));

        let field = registry.create_field("memory");
        assert!(field.is_some());
        assert_eq!(field.unwrap().name(), "memory");

        let missing = registry.create_field("nonexistent");
        assert!(missing.is_none());
    }
}
