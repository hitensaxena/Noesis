use std::env;
use tracing;

use super::chain::ModelChain;
use super::openrouter::OpenRouterProvider;
use super::types::{CompletionRequest, CompletionResponse, LLMError};

/// The three task tiers for LLM routing.
///
/// Matches curlyos-core's tiered routing pattern:
/// - **Fast**: High-volume, cheap operations (classification, extraction)
/// - **Agentic**: Orchestration, agent runs (ReAct, planning)
/// - **Deep**: Heavy cognition (reflection, narrative, meta)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    Fast,
    Agentic,
    Deep,
}

impl std::fmt::Display for ModelTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelTier::Fast => write!(f, "fast"),
            ModelTier::Agentic => write!(f, "agentic"),
            ModelTier::Deep => write!(f, "deep"),
        }
    }
}

/// Default model chains per tier.
///
/// These mirror curlyos-core's defaults from `shared/models.py`:
/// - General chain: opensource models with free tier fallbacks
mod defaults {
    pub const FAST_CHAIN: &str = "openai/gpt-4o-mini,mistralai/mistral-small-24b-instruct-2501:free";
    pub const AGENTIC_CHAIN: &str = "openai/gpt-4o-mini";
    pub const DEEP_CHAIN: &str = "openai/gpt-4o-mini";
}

/// Environment variable names for configuring tiers.
mod env_keys {
    #[allow(dead_code)]
    pub const FAST_MODEL: &str = "NOESIS_FAST_MODEL";
    pub const FAST_CHAIN: &str = "NOESIS_FAST_CHAIN";
    #[allow(dead_code)]
    pub const AGENTIC_MODEL: &str = "NOESIS_AGENTIC_MODEL";
    pub const AGENTIC_CHAIN: &str = "NOESIS_AGENTIC_CHAIN";
    #[allow(dead_code)]
    pub const DEEP_MODEL: &str = "NOESIS_DEEP_MODEL";
    pub const DEEP_CHAIN: &str = "NOESIS_DEEP_CHAIN";
    pub const API_KEY: &str = "NOESIS_API_KEY";
    pub const BASE_URL: &str = "NOESIS_API_URL";
}

/// Tiered router that manages model chains per task tier.
pub struct TieredRouter {
    fast_chain: ModelChain,
    agentic_chain: ModelChain,
    deep_chain: ModelChain,
}

impl TieredRouter {
    /// Create a tiered router from environment configuration.
    ///
    /// Reads the following env vars:
    /// - `NOESIS_API_KEY` — API key (required)
    /// - `NOESIS_API_URL` — Base URL (default: OpenRouter)
    /// - `NOESIS_FAST_CHAIN` / `NOESIS_FAST_MODEL` — Fast tier model(s)
    /// - `NOESIS_AGENTIC_CHAIN` / `NOESIS_AGENTIC_MODEL` — Agentic tier
    /// - `NOESIS_DEEP_CHAIN` / `NOESIS_DEEP_MODEL` — Deep tier
    pub fn from_env() -> Result<Self, String> {
        let api_key = env::var(env_keys::API_KEY)
            .map_err(|_| "NOESIS_API_KEY not set".to_string())?;
        let base_url = env::var(env_keys::BASE_URL).ok();

        let fast_models = parse_chain(env_keys::FAST_CHAIN, defaults::FAST_CHAIN);
        let agentic_models = parse_chain(env_keys::AGENTIC_CHAIN, defaults::AGENTIC_CHAIN);
        let deep_models = parse_chain(env_keys::DEEP_CHAIN, defaults::DEEP_CHAIN);

        tracing::info!(
            "[TieredRouter] fast={:?}, agentic={:?}, deep={:?}",
            fast_models,
            agentic_models,
            deep_models
        );

        let provider = OpenRouterProvider::new(
            &api_key,
            &fast_models[0], // placeholder, overridden by chain
            base_url.as_deref(),
            120,
        );

        Ok(Self {
            fast_chain: ModelChain::new(
                Box::new(provider),
                fast_models,
            ),
            agentic_chain: ModelChain::new(
                Box::new(OpenRouterProvider::new(
                    &api_key,
                    &agentic_models[0],
                    base_url.as_deref(),
                    180,
                )),
                agentic_models,
            ),
            deep_chain: ModelChain::new(
                Box::new(OpenRouterProvider::new(
                    &api_key,
                    &deep_models[0],
                    base_url.as_deref(),
                    300,
                )),
                deep_models,
            ),
        })
    }

    /// Send a completion request through the specified tier.
    pub async fn complete(
        &mut self,
        tier: ModelTier,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LLMError> {
        tracing::info!("[TieredRouter] routing to tier {} (model chain)", tier);

        match tier {
            ModelTier::Fast => self.fast_chain.complete_with_failover(request).await,
            ModelTier::Agentic => self.agentic_chain.complete_with_failover(request).await,
            ModelTier::Deep => self.deep_chain.complete_with_failover(request).await,
        }
    }

    /// Access the underlying chain for a tier (for direct model access).
    pub fn chain(&mut self, tier: ModelTier) -> &mut ModelChain {
        match tier {
            ModelTier::Fast => &mut self.fast_chain,
            ModelTier::Agentic => &mut self.agentic_chain,
            ModelTier::Deep => &mut self.deep_chain,
        }
    }

    pub fn has_api_key() -> bool {
        env::var(env_keys::API_KEY).is_ok()
    }
}

/// Parse a comma-separated model chain string into a Vec of model names.
fn parse_chain(env_key: &str, default: &str) -> Vec<String> {
    let raw = env::var(env_key).unwrap_or_else(|_| default.to_string());
    raw.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_tier_display() {
        assert_eq!(ModelTier::Fast.to_string(), "fast");
        assert_eq!(ModelTier::Agentic.to_string(), "agentic");
        assert_eq!(ModelTier::Deep.to_string(), "deep");
    }

    #[test]
    fn test_model_tier_equality() {
        assert_eq!(ModelTier::Fast, ModelTier::Fast);
        assert_ne!(ModelTier::Fast, ModelTier::Deep);
    }

    #[test]
    fn test_parse_chain_single() {
        let models = parse_chain("NONEXISTENT_VAR_12345", "gpt-4o");
        assert_eq!(models, vec!["gpt-4o"]);
    }

    #[test]
    fn test_parse_chain_multi() {
        let models = parse_chain("NONEXISTENT_VAR_12346", "gpt-4o,claude-3,gemini-pro");
        assert_eq!(models, vec!["gpt-4o", "claude-3", "gemini-pro"]);
    }

    #[test]
    fn test_parse_chain_trims_whitespace() {
        let models = parse_chain("NONEXISTENT_VAR_12347", " model-a , model-b ");
        assert_eq!(models, vec!["model-a", "model-b"]);
    }

    #[test]
    fn test_has_api_key_default_false() {
        // Without the api key set, has_api_key should return false
        // in a clean test environment
        assert!(!TieredRouter::has_api_key());
    }

    #[test]
    fn test_defaults_exist() {
        assert!(!defaults::FAST_CHAIN.is_empty());
        assert!(!defaults::AGENTIC_CHAIN.is_empty());
        assert!(!defaults::DEEP_CHAIN.is_empty());
    }

    #[test]
    fn test_env_key_constants() {
        assert_eq!(env_keys::FAST_CHAIN, "NOESIS_FAST_CHAIN");
        assert_eq!(env_keys::AGENTIC_CHAIN, "NOESIS_AGENTIC_CHAIN");
        assert_eq!(env_keys::DEEP_CHAIN, "NOESIS_DEEP_CHAIN");
    }
}
