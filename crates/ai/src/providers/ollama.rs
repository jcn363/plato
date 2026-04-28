//! Ollama provider for local LLM inference
#![allow(clippy::cast_possible_wrap, clippy::unnecessary_literal_bound)]

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

use crate::traits::{LLMProvider, ProviderConfig};
use crate::AiContext;
use crate::AiResponse;

#[derive(Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
    options: GenerateOptions,
}

#[derive(Serialize)]
struct GenerateOptions {
    temperature: f32,
    num_predict: usize,
}

/// Ollama local LLM provider
#[derive(Debug, Clone)]
pub struct OllamaProvider {
    config: ProviderConfig,
    client: Client,
    endpoint: String,
}

impl OllamaProvider {
    /// Create a new Ollama provider
    #[must_use]
    pub fn new(config: ProviderConfig) -> Self {
        let endpoint = config
            .endpoint
            .clone()
            .unwrap_or_else(|| "http://localhost:11434".into());
        Self {
            config,
            client: Client::new(),
            endpoint,
        }
    }

    /// Check if Ollama is running
    #[must_use]
    pub fn check_connection(&self) -> bool {
        let url = format!("{}/api/tags", self.endpoint);
        self.client.get(&url).send().is_ok()
    }
}

impl LLMProvider for OllamaProvider {
    fn generate(&self, prompt: &str, _context: &AiContext) -> crate::AiResult<AiResponse> {
        let url = format!("{}/api/generate", self.endpoint);

        let request = GenerateRequest {
            model: &self.config.model,
            prompt,
            stream: false,
            options: GenerateOptions {
                temperature: self.config.temperature,
                num_predict: self.config.max_tokens,
            },
        };

        let response: GenerateResponse = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .with_context(|| format!("Failed to connect to Ollama at {}", self.endpoint))?
            .json()
            .with_context(|| "Failed to parse Ollama response")?;

        let timestamp_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let timestamp = timestamp_sec as i64;

        Ok(AiResponse {
            content: response.response,
            model: self.config.model.clone(),
            provider: "ollama".into(),
            timestamp,
            cached: false,
        })
    }

    fn is_available(&self) -> bool {
        self.check_connection()
    }

    fn name(&self) -> &str {
        "Ollama"
    }

    fn model(&self) -> &str {
        &self.config.model
    }
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
    #[allow(dead_code)]
    model: String,
    #[allow(dead_code)]
    created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::ProviderConfig;

    #[test]
    fn test_ollama_provider_creation() {
        let config = ProviderConfig {
            provider_type: crate::traits::ProviderType::Ollama,
            endpoint: Some("http://localhost:11434".into()),
            model: "phi3:mini".into(),
            ..Default::default()
        };
        let provider = OllamaProvider::new(config);
        assert_eq!(provider.name(), "Ollama");
        assert_eq!(provider.model(), "phi3:mini");
    }
}
