//! Text-to-Speech (TTS) Module
//!
//! This module provides a cross-platform Text-to-Speech interface for Plato.
//! It supports:
//! - **Desktop (Linux/macOS/Windows)**: Via the `tts` crate using native system TTS
//! - **Android**: Via Android's TextToSpeech API through JNI
//!
//! TTS is disabled on Kobo e-readers due to lack of audio hardware.
//!
//! ## Usage
//!
//! ```
//! use plato_core::tts::{TtsEngine, TtsOptions};
//!
//! let mut tts = TtsEngine::new()?;
//! tts.speak("Hello, world!", TtsOptions::default())?;
//! ```

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

/// TTS playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TtsState {
    /// TTS is idle (not speaking)
    Idle,
    /// TTS is currently speaking
    Speaking,
    /// TTS is paused
    Paused,
    /// TTS engine is initializing
    Initializing,
    /// TTS engine encountered an error
    Error,
}

impl Default for TtsState {
    fn default() -> Self {
        TtsState::Idle
    }
}

/// Options for TTS utterance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsOptions {
    /// Speech rate multiplier (0.5 = half speed, 2.0 = double speed)
    pub rate: f32,
    /// Volume level (0.0 = silent, 1.0 = maximum)
    pub volume: f32,
    /// Pitch multiplier (0.5 = lower pitch, 2.0 = higher pitch)
    pub pitch: f32,
    /// Language/locale code (e.g., "en-US", "fr-FR")
    pub language: Option<String>,
    /// Whether to interrupt current speech
    pub interrupt: bool,
}

impl Default for TtsOptions {
    fn default() -> Self {
        Self {
            rate: 1.0,
            volume: 1.0,
            pitch: 1.0,
            language: None,
            interrupt: true,
        }
    }
}

impl TtsOptions {
    /// Create default options with specified rate
    pub fn with_rate(rate: f32) -> Self {
        Self {
            rate,
            ..Default::default()
        }
    }

    /// Create default options with specified language
    pub fn with_language(lang: impl Into<String>) -> Self {
        Self {
            language: Some(lang.into()),
            ..Default::default()
        }
    }
}

/// Information about available TTS voice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsVoice {
    /// Voice identifier
    pub id: String,
    /// Human-readable voice name
    pub name: String,
    /// Language code (e.g., "en-US")
    pub language: String,
    /// Whether this is a male voice
    pub is_male: Option<bool>,
    /// Quality rating (if available)
    pub quality: Option<u8>,
}

/// TTS engine trait for cross-platform support
///
/// This trait defines the interface that all TTS implementations must provide.
/// Platform-specific implementations are in separate modules:
/// - `tts_desktop.rs` for Linux/macOS/Windows
/// - `tts_android.rs` for Android
pub trait TtsEngine: Send {
    /// Initialize the TTS engine
    ///
    /// This should be called before any other TTS operations.
    /// Returns an error if TTS is not available on this platform.
    fn initialize(&mut self) -> Result<()>;

    /// Check if TTS is available and initialized
    fn is_available(&self) -> bool;

    /// Get current TTS state
    fn state(&self) -> TtsState;

    /// Speak the given text
    ///
    /// # Arguments
    /// * `text` - The text to speak
    /// * `options` - Speech options (rate, volume, pitch, etc.)
    fn speak(&mut self, text: &str, options: TtsOptions) -> Result<()>;

    /// Stop current speech
    fn stop(&mut self) -> Result<()>;

    /// Pause current speech (if supported)
    fn pause(&mut self) -> Result<()>;

    /// Resume paused speech (if supported)
    fn resume(&mut self) -> Result<()>;

    /// Get available voices
    fn voices(&self) -> Result<Vec<TtsVoice>>;

    /// Set the voice to use
    fn set_voice(&mut self, voice_id: &str) -> Result<()>;

    /// Get current voice ID (if any)
    fn current_voice(&self) -> Option<&str>;

    /// Set speech rate (0.5 to 2.0)
    fn set_rate(&mut self, rate: f32) -> Result<()>;

    /// Get current speech rate
    fn rate(&self) -> f32;

    /// Set volume (0.0 to 1.0)
    fn set_volume(&mut self, volume: f32) -> Result<()>;

    /// Get current volume
    fn volume(&self) -> f32;
}

/// Factory function to create the appropriate TTS engine for the current platform
///
/// Returns `None` if TTS is not available on the current platform (e.g., Kobo devices).
#[cfg(feature = "tts")]
pub fn create_tts_engine() -> Result<Box<dyn TtsEngine>> {
    #[cfg(target_os = "android")]
    {
        use crate::tts_android::AndroidTtsEngine;
        let mut engine = AndroidTtsEngine::new();
        engine.initialize()?;
        Ok(Box::new(engine))
    }

    #[cfg(not(target_os = "android"))]
    {
        use crate::tts_desktop::DesktopTtsEngine;
        let mut engine = DesktopTtsEngine::new();
        engine.initialize()?;
        Ok(Box::new(engine))
    }
}

/// Stub implementation when TTS feature is disabled
#[cfg(not(feature = "tts"))]
pub fn create_tts_engine() -> Result<Box<dyn TtsEngine>> {
    Err(Error::msg("TTS feature not enabled"))
}

/// Check if TTS is supported on the current platform
pub fn is_tts_supported() -> bool {
    #[cfg(feature = "tts")]
    {
        #[cfg(target_os = "android")]
        return true;

        #[cfg(target_os = "linux")]
        return true;

        #[cfg(target_os = "macos")]
        return true;

        #[cfg(target_os = "windows")]
        return true;

        #[cfg(not(any(
            target_os = "android",
            target_os = "linux",
            target_os = "macos",
            target_os = "windows"
        )))]
        return false;
    }

    #[cfg(not(feature = "tts"))]
    {
        false
    }
}

/// TTS settings for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtsSettings {
    /// Whether TTS is enabled
    pub enabled: bool,
    /// Current speech rate
    pub rate: f32,
    /// Current volume
    pub volume: f32,
    /// Current pitch
    pub pitch: f32,
    /// Selected voice ID
    pub voice_id: Option<String>,
    /// Auto-scroll while speaking
    pub auto_scroll: bool,
    /// Highlight current word/sentence
    pub highlight_text: bool,
}

impl Default for TtsSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            rate: 1.0,
            volume: 1.0,
            pitch: 1.0,
            voice_id: None,
            auto_scroll: true,
            highlight_text: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts_options_default() {
        let opts = TtsOptions::default();
        assert_eq!(opts.rate, 1.0);
        assert_eq!(opts.volume, 1.0);
        assert_eq!(opts.pitch, 1.0);
        assert!(opts.language.is_none());
        assert!(opts.interrupt);
    }

    #[test]
    fn test_tts_options_with_rate() {
        let opts = TtsOptions::with_rate(1.5);
        assert_eq!(opts.rate, 1.5);
        assert_eq!(opts.volume, 1.0);
    }

    #[test]
    fn test_tts_settings_default() {
        let settings = TtsSettings::default();
        assert!(!settings.enabled);
        assert_eq!(settings.rate, 1.0);
        assert_eq!(settings.volume, 1.0);
        assert!(settings.auto_scroll);
        assert!(settings.highlight_text);
    }

    #[test]
    fn test_tts_state_transitions() {
        let state = TtsState::Idle;
        assert_eq!(state, TtsState::Idle);

        let state = TtsState::Speaking;
        assert_eq!(state, TtsState::Speaking);
    }
}
