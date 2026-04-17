//! Centralized configuration management with validation
//!
//! This module provides centralized configuration loading, validation, and saving
//! following AGENTS.md rules:
//! - Centralize configuration management
//! - Validate all configuration values at load time
//! - Use typed configuration over raw strings or magic numbers
//! - Provide clear, actionable error messages for invalid configuration

use crate::helpers::{load_toml, save_toml};
use crate::log_error;
use crate::settings::{Settings, SETTINGS_PATH};
use crate::validation::validate_path;
use anyhow::{Context, Error};
use std::path::{Path, PathBuf};

/// Manages application configuration with validation
///
/// Provides centralized loading, validation, and saving of application settings
/// with comprehensive error handling and validation.
pub struct ConfigManager {
    settings_path: PathBuf,
}

impl ConfigManager {
    /// Creates a new ConfigManager with the default settings path
    pub fn new() -> Self {
        Self {
            settings_path: PathBuf::from(SETTINGS_PATH),
        }
    }

    /// Creates a new ConfigManager with a custom settings path
    pub fn with_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            settings_path: path.as_ref().to_path_buf(),
        }
    }

    /// Loads settings from file with comprehensive validation
    ///
    /// # Returns
    /// - `Ok(Settings)` - Validated settings on success
    /// - `Err(Error)` - Detailed error if loading or validation fails
    ///
    /// # Errors
    /// Returns error if:
    /// - Settings file cannot be read
    /// - Settings file contains invalid TOML
    /// - Settings values fail validation
    pub fn load(&self) -> Result<Settings, Error> {
        // Validate settings path before loading
        validate_path(&self.settings_path, "settings file path")?;

        if !self.settings_path.exists() {
            // Return default settings if file doesn't exist
            let settings = Settings::default();
            // Validate default settings to ensure they're valid
            settings
                .validate()
                .context("default settings failed validation")?;
            return Ok(settings);
        }

        // Load settings from file
        let settings: Settings = load_toml(&self.settings_path).with_context(|| {
            format!(
                "failed to load settings from {}",
                self.settings_path.display()
            )
        })?;

        // Validate loaded settings
        settings.validate().with_context(|| {
            format!(
                "settings loaded from {} failed validation",
                self.settings_path.display()
            )
        })?;

        Ok(settings)
    }

    /// Saves settings to file
    ///
    /// # Arguments
    /// * `settings` - Settings to save (must pass validation)
    ///
    /// # Errors
    /// Returns error if:
    /// - Settings fail validation
    /// - File cannot be written
    pub fn save(&self, settings: &Settings) -> Result<(), Error> {
        // Validate before saving
        settings
            .validate()
            .context("cannot save invalid settings")?;

        save_toml(settings, &self.settings_path).with_context(|| {
            format!(
                "failed to save settings to {}",
                self.settings_path.display()
            )
        })?;

        Ok(())
    }

    /// Loads settings with fallback to defaults on error
    ///
    /// If loading or validation fails, returns default settings and logs the error.
    /// Use this when you want the application to start even with corrupted settings.
    ///
    /// # Returns
    /// Settings (either loaded and validated, or defaults on failure)
    pub fn load_or_default(&self) -> Settings {
        match self.load() {
            Ok(settings) => settings,
            Err(e) => {
                log_error!(
                    "Failed to load settings from {}: {:#}. Using defaults.",
                    self.settings_path.display(),
                    e
                );
                Settings::default()
            }
        }
    }

    /// Gets the current settings path
    pub fn settings_path(&self) -> &Path {
        &self.settings_path
    }

    /// Sets a new settings path
    pub fn set_settings_path<P: AsRef<Path>>(&mut self, path: P) {
        self.settings_path = path.as_ref().to_path_buf();
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience function to load settings with the default path
///
/// This is the recommended way to load settings in most cases.
/// It validates all settings at load time and returns detailed errors.
pub fn load_settings() -> Result<Settings, Error> {
    ConfigManager::new().load()
}

/// Convenience function to save settings with the default path
///
/// Validates settings before saving to ensure data integrity.
pub fn save_settings(settings: &Settings) -> Result<(), Error> {
    ConfigManager::new().save(settings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_config_manager_load_default() {
        let manager = ConfigManager::new();
        // When file doesn't exist, should return defaults
        let settings = manager.load().unwrap();
        // Basic sanity check that defaults are reasonable
        assert!(settings.reader.font_size > 0.0);
    }

    #[test]
    fn test_config_manager_save_and_load() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let manager = ConfigManager::with_path(temp_file.path());

        let settings = Settings::default();
        manager.save(&settings).unwrap();

        let loaded = manager.load().unwrap();
        assert_eq!(loaded.reader.font_size, settings.reader.font_size);
    }

    #[test]
    fn test_config_manager_invalid_settings() {
        let mut temp_file = NamedTempFile::new().unwrap();
        // Write invalid TOML directly - use a field that exists but with wrong type
        writeln!(temp_file, "frontlight = \"not_a_bool\"").unwrap();

        let manager = ConfigManager::with_path(temp_file.path());
        // Should fail to parse invalid TOML (wrong type for boolean field)
        assert!(manager.load().is_err());
    }
}
