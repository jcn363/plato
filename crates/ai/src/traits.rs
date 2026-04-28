//! Trait definitions for LLM providers

// anyhow::Error is used via crate::AiResult
use serde::{Deserialize, Serialize};

use crate::AiContext;
use crate::AiResponse;

/// Trait for LLM providers (local or cloud)
pub trait LLMProvider: Send + Sync {
    /// Generate a response from the LLM
    fn generate(&self, prompt: &str, context: &AiContext) -> crate::AiResult<AiResponse>;

    /// Check if the provider is available (e.g., Ollama running)
    fn is_available(&self) -> bool;

    /// Get provider name
    fn name(&self) -> &str;

    /// Get model name being used
    fn model(&self) -> &str;

    /// Summarize a chapter (convenience method)
    #[allow(clippy::uninlined_format_args)]
    fn summarize(&self, text: &str, context: &AiContext) -> crate::AiResult<AiResponse> {
        let prompt = format!(
            "Please provide a concise summary of the following text (under 200 words):\n\n{}",
            text
        );
        self.generate(&prompt, context)
    }
}

/// Configuration for LLM providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub provider_type: ProviderType,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: ProviderType::Ollama,
            endpoint: Some("http://localhost:11434".into()),
            api_key: None,
            model: "phi3:mini".into(),
            max_tokens: 2000,
            temperature: 0.7,
        }
    }
}

/// Supported provider types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderType {
    /// Local Ollama instance
    Ollama,
    /// `OpenAI` API (cloud)
    #[allow(clippy::doc_markdown)]
    OpenAI,
    /// Anthropic Claude (cloud)
    Claude,
    /// Mock provider (for testing)
    Mock,
}

/// Request to LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMRequest {
    pub prompt: String,
    pub context: AiContext,
    pub max_tokens: usize,
    pub temperature: f32,
}

/// Response from LLM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMResponse {
    pub text: String,
    pub model: String,
    pub tokens_used: Option<usize>,
}
