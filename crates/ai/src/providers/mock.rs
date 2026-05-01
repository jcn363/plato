//! Mock provider for testing

#![allow(clippy::unnecessary_literal_bound)]

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::bail;

use crate::traits::{LLMProvider, ProviderConfig};
use crate::AiContext;
use crate::AiResponse;

/// Mock LLM provider for testing
#[derive(Debug, Clone)]
pub struct MockProvider {
    config: ProviderConfig,
    should_fail: bool,
}

impl MockProvider {
    /// Create a new mock provider
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            should_fail: false,
        }
    }

    /// Create a mock provider that always fails
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn new_failing(config: ProviderConfig) -> Self {
        Self {
            config,
            should_fail: true,
        }
    }

    /// Get the provider configuration
    #[must_use]
    pub fn config(&self) -> &ProviderConfig {
        &self.config
    }

    /// Check if provider is configured to fail
    #[must_use]
    pub fn is_failing(&self) -> bool {
        self.should_fail
    }
}

impl LLMProvider for MockProvider {
    #[allow(clippy::cast_possible_wrap, clippy::similar_names)]
    fn generate(&self, prompt: &str, context: &AiContext) -> crate::AiResult<AiResponse> {
        if self.should_fail {
            bail!("Mock provider configured to fail");
        }

        let timestamp_secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let timestamp = timestamp_secs as i64;

        // Generate a mock response based on the prompt
        let prompt_text = prompt.to_lowercase();
        let content = if prompt_text.contains("summarize") || prompt_text.contains("summary") {
            format!(
                "Mock summary of text from {}. This is a test response.",
                context.document_path
            )
        } else if prompt.contains("quiz") || prompt.contains("question") {
            "Mock quiz question: What is the main idea of this chapter?".into()
        } else {
            format!(
                "Mock response to: {} (at page {}/{})",
                &prompt[..prompt.len().min(50)],
                context.current_page,
                context.total_pages
            )
        };

        Ok(AiResponse {
            content,
            model: "mock-model".into(),
            provider: "mock".into(),
            timestamp,
            cached: false,
        })
    }

    fn is_available(&self) -> bool {
        true
    }

    fn name(&self) -> &str {
        "Mock"
    }

    fn model(&self) -> &str {
        "mock-model"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::ProviderType;

    #[test]
    fn test_mock_provider() {
        let config = ProviderConfig {
            provider_type: ProviderType::Mock,
            ..Default::default()
        };
        let provider = MockProvider::new(config);
        assert!(provider.is_available());
        assert_eq!(provider.name(), "Mock");
    }

    #[test]
    fn test_mock_provider_response() {
        let config = ProviderConfig {
            provider_type: ProviderType::Mock,
            ..Default::default()
        };
        let provider = MockProvider::new(config);
        let context = crate::AiContext::new("/test.epub".into(), 5, 20);
        let response = provider
            .generate("Summarize this chapter", &context)
            .expect("Test assertion failed");
        assert!(
            response.content.to_lowercase().contains("summary"),
            "Expected response to contain 'summary', got: {}",
            response.content
        );
    }
}
