use async_trait::async_trait;
use std::time::Duration;
use tracing;

use super::client::LLMClient;
use super::types::{
    CompletionRequest, CompletionResponse, LLMError, Usage,
};

/// OpenRouter-compatible LLM provider.
///
/// Connects to https://openrouter.ai/api (or a custom base URL) using the
/// OpenAI-compatible chat completions API. Supports model failover via
/// the ModelChain abstraction.
pub struct OpenRouterProvider {
    base_url: String,
    api_key: String,
    model: String,
    client: reqwest::Client,
    #[allow(dead_code)]
    timeout_seconds: u64,
}

impl OpenRouterProvider {
    /// Create a new OpenRouter provider.
    ///
    /// # Arguments
    /// * `api_key` - OpenRouter API key
    /// * `model` - Model identifier (e.g., "openrouter/auto", "anthropic/claude-sonnet")
    /// * `base_url` - Custom base URL (default: "https://openrouter.ai/api/v1")
    /// * `timeout_seconds` - Request timeout
    pub fn new(
        api_key: &str,
        model: &str,
        base_url: Option<&str>,
        timeout_seconds: u64,
    ) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_seconds))
            .user_agent("noesis/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self {
            base_url: base_url
                .unwrap_or("https://openrouter.ai/api/v1")
                .trim_end_matches('/')
                .to_string(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            client,
            timeout_seconds,
        }
    }

    /// Build the request body for the OpenAI-compatible API.
    fn build_body(&self, request: &CompletionRequest) -> serde_json::Value {
        serde_json::json!({
            "model": request.model.as_str(),
            "messages": request.messages.iter().map(|m| {
                serde_json::json!({
                    "role": serde_json::to_value(&m.role).unwrap_or_default(),
                    "content": m.content,
                })
            }).collect::<Vec<_>>(),
            "temperature": request.temperature.unwrap_or(0.7),
            "max_tokens": request.max_tokens.unwrap_or(4096),
            "top_p": request.top_p.unwrap_or(1.0),
            "stop": request.stop.as_deref(),
            "stream": request.stream,
        })
    }
}

#[async_trait]
impl LLMClient for OpenRouterProvider {
    fn provider_name(&self) -> &str {
        "openrouter"
    }

    fn model(&self) -> &str {
        &self.model
    }

    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, LLMError> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = self.build_body(&request);

        tracing::debug!(
            "[OpenRouter] requesting {} ({} messages, max_tokens={:?})",
            request.model,
            request.messages.len(),
            request.max_tokens,
        );

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if status.as_u16() == 429 {
            let retry_after = resp
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(30);
            return Err(LLMError::RateLimited { retry_after });
        }

        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            return Err(LLMError::Provider {
                status: status.as_u16(),
                message: text.chars().take(500).collect(),
            });
        }

        let data: serde_json::Value = resp.json().await?;

        // Extract content from OpenAI-compatible response format
        let content = data["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        let finish_reason = data["choices"][0]["finish_reason"]
            .as_str()
            .map(|s| s.to_string());

        let usage = data["usage"].as_object().map(|u| Usage {
            prompt_tokens: u.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
            completion_tokens: u
                .get("completion_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0),
            total_tokens: u.get("total_tokens").and_then(|v| v.as_u64()).unwrap_or(0),
        });

        if content.is_empty() && finish_reason.as_deref() != Some("length") {
            return Err(LLMError::EmptyResponse);
        }

        tracing::debug!(
            "[OpenRouter] response: {} tokens, finish={:?}",
            usage.as_ref().map(|u| u.total_tokens).unwrap_or(0),
            finish_reason,
        );

        Ok(CompletionResponse {
            content,
            finish_reason,
            usage,
            model: request.model,
        })
    }
}
