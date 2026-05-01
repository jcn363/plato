//! AI settings for Plato

use serde::{Deserialize, Serialize};

use super::traits::ProviderType;

/// AI settings stored in Plato's configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(crate = "serde")]
#[serde(default, rename_all = "camelCase")]
pub struct AiSettings {
    /// Enable AI features (disabled by default)
    pub enabled: bool,

    /// Selected provider type
    pub provider_type: ProviderType,

    /// Ollama endpoint (for local LLM)
    pub ollama_endpoint: String,

    /// Cloud API endpoint (optional)
    pub cloud_endpoint: Option<String>,

    /// API key for cloud providers (stored securely)
    pub api_key: Option<String>,

    /// Model to use (e.g., "phi3:mini", "gpt-4o-mini")
    pub model: String,

    /// Maximum tokens in response
    pub max_tokens: usize,

    /// Temperature for generation (0.0 = deterministic, 1.0 = creative)
    pub temperature: f32,

    /// Enable spoiler protection (limit AI context to current reading position)
    pub spoiler_protection: bool,

    /// Cache AI responses (avoid re-computation)
    pub enable_cache: bool,

    /// Cache duration in seconds (default: 1 hour)
    pub cache_duration_seconds: i64,

    /// Enable on resource-constrained devices (256MB Kobo: false)
    pub allow_on_low_memory: bool,
}

impl Default for AiSettings {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default
            provider_type: ProviderType::Ollama,
            ollama_endpoint: "http://localhost:11434".into(),
            cloud_endpoint: None,
            api_key: None,
            model: "phi3:mini".into(),
            max_tokens: 2000,
            temperature: 0.7,
            spoiler_protection: true,
            enable_cache: true,
            cache_duration_seconds: 3600, // 1 hour
            allow_on_low_memory: false,   // Disable on 256MB devices
        }
    }
}

impl AiSettings {
    /// Check if AI can run on the current device
    #[must_use]
    pub fn can_run_on_device(&self, total_ram_mb: usize) -> bool {
        if !self.enabled {
            return false;
        }
        if !self.allow_on_low_memory && total_ram_mb < 1024 {
            // Disable on devices with <1GB RAM (e.g., 256MB Kobo)
            return false;
        }
        true
    }

    /// Get cache path for the device
    #[must_use]
    pub fn cache_path(&self) -> String {
        "/mnt/onboard/.plato/ai_cache.db".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_default() {
        let settings = AiSettings::default();
        assert!(!settings.enabled);
        assert_eq!(settings.model, "phi3:mini");
        assert!(settings.spoiler_protection);
        assert!(!settings.allow_on_low_memory);
    }

    #[test]
    fn test_can_run_on_device() {
        let mut settings = AiSettings {
            enabled: true,
            ..AiSettings::default()
        };

        // Should not run on 256MB device
        assert!(!settings.can_run_on_device(256));

        // Should run on 1GB+ device
        assert!(settings.can_run_on_device(1024));

        // Should run on 256MB if allow_on_low_memory is true
        settings.allow_on_low_memory = true;
        assert!(settings.can_run_on_device(256));
    }
}
