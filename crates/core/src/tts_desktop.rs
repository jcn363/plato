//! Desktop TTS Implementation
//!
//! This module provides TTS support for desktop platforms (Linux, macOS, Windows)
//! using the `tts` crate, which interfaces with native system TTS:
//! - **Linux**: speech-dispatcher
//! - **macOS**: AVSpeechSynthesizer
//! - **Windows**: SAPI5

use anyhow::{Context, Result};

use crate::tts::{TtsEngine, TtsOptions, TtsSettings, TtsState, TtsVoice};

/// Desktop TTS engine using the `tts` crate
pub struct DesktopTtsEngine {
    /// The underlying TTS backend
    backend: Option<tts::Tts>,
    /// Current TTS state
    state: TtsState,
    /// Current settings
    settings: TtsSettings,
    /// Current voice ID
    current_voice: Option<String>,
}

impl DesktopTtsEngine {
    /// Create a new Desktop TTS engine
    pub fn new() -> Self {
        Self {
            backend: None,
            state: TtsState::Idle,
            settings: TtsSettings::default(),
            current_voice: None,
        }
    }

    /// Get the underlying backend (if initialized)
    fn backend(&self) -> Result<&tts::Tts> {
        self.backend.as_ref().context("TTS engine not initialized")
    }

    /// Get mutable backend (if initialized)
    fn backend_mut(&mut self) -> Result<&mut tts::Tts> {
        self.backend.as_mut().context("TTS engine not initialized")
    }
}

impl Default for DesktopTtsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TtsEngine for DesktopTtsEngine {
    fn initialize(&mut self) -> Result<()> {
        if self.backend.is_some() {
            return Ok(());
        }

        self.state = TtsState::Initializing;

        let backend = tts::Tts::default().context("Failed to initialize system TTS")?;

        self.backend = Some(backend);
        self.state = TtsState::Idle;

        Ok(())
    }

    fn is_available(&self) -> bool {
        self.backend.is_some()
    }

    fn state(&self) -> TtsState {
        self.state
    }

    fn speak(&mut self, text: &str, options: TtsOptions) -> Result<()> {
        let backend = self.backend_mut()?;

        // Stop current speech if interrupt is requested
        if options.interrupt {
            let _ = backend.stop();
        }

        // Convert our options to tts crate utterance
        let utterance = tts::Utterance::new(text)
            .with_rate(options.rate)
            .with_volume(options.volume)
            .with_pitch(options.pitch);

        // Set language if specified
        let utterance = if let Some(ref lang) = options.language {
            utterance.with_language(lang)
        } else {
            utterance
        };

        backend.speak(utterance).context("Failed to speak text")?;

        self.state = TtsState::Speaking;
        self.settings.rate = options.rate;
        self.settings.volume = options.volume;
        self.settings.pitch = options.pitch;

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        let backend = self.backend_mut()?;
        backend.stop().context("Failed to stop speech")?;
        self.state = TtsState::Idle;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        let backend = self.backend_mut()?;
        backend.pause().context("Failed to pause speech")?;
        self.state = TtsState::Paused;
        Ok(())
    }

    fn resume(&mut self) -> Result<()> {
        let backend = self.backend_mut()?;
        // Note: not all backends support resume
        let _ = backend.resume();
        self.state = TtsState::Speaking;
        Ok(())
    }

    fn voices(&self) -> Result<Vec<TtsVoice>> {
        let backend = self.backend()?;
        let voices = backend.voices().context("Failed to get voices")?;

        let result: Vec<TtsVoice> = voices
            .into_iter()
            .map(|v| TtsVoice {
                id: v.id().to_string(),
                name: v.name().to_string(),
                language: v.language().to_string(),
                is_male: None, // tts crate doesn't expose gender
                quality: None,
            })
            .collect();

        Ok(result)
    }

    fn set_voice(&mut self, voice_id: &str) -> Result<()> {
        let backend = self.backend_mut()?;

        // Find voice by ID
        let voices = backend.voices().context("Failed to get voices")?;
        let voice = voices
            .into_iter()
            .find(|v| v.id() == voice_id)
            .context("Voice not found")?;

        backend.set_voice(&voice).context("Failed to set voice")?;
        self.current_voice = Some(voice_id.to_string());
        self.settings.voice_id = Some(voice_id.to_string());

        Ok(())
    }

    fn current_voice(&self) -> Option<&str> {
        self.current_voice.as_deref()
    }

    fn set_rate(&mut self, rate: f32) -> Result<()> {
        // Clamp rate to valid range
        let rate = rate.clamp(0.5, 2.0);
        self.settings.rate = rate;
        // Note: rate is set per-utterance in the tts crate
        Ok(())
    }

    fn rate(&self) -> f32 {
        self.settings.rate
    }

    fn set_volume(&mut self, volume: f32) -> Result<()> {
        // Clamp volume to valid range
        let volume = volume.clamp(0.0, 1.0);
        self.settings.volume = volume;
        // Note: volume is set per-utterance in the tts crate
        Ok(())
    }

    fn volume(&self) -> f32 {
        self.settings.volume
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_tts_engine_new() {
        let engine = DesktopTtsEngine::new();
        assert!(!engine.is_available());
        assert_eq!(engine.state(), TtsState::Idle);
    }

    #[test]
    fn test_desktop_tts_settings() {
        let mut engine = DesktopTtsEngine::new();

        // Test rate
        assert_eq!(engine.rate(), 1.0);
        let _ = engine.set_rate(1.5);
        assert_eq!(engine.rate(), 1.5);

        // Test clamping
        let _ = engine.set_rate(3.0);
        assert_eq!(engine.rate(), 2.0);

        let _ = engine.set_rate(0.1);
        assert_eq!(engine.rate(), 0.5);
    }

    #[test]
    fn test_desktop_tts_volume() {
        let mut engine = DesktopTtsEngine::new();

        assert_eq!(engine.volume(), 1.0);
        let _ = engine.set_volume(0.5);
        assert_eq!(engine.volume(), 0.5);

        // Test clamping
        let _ = engine.set_volume(1.5);
        assert_eq!(engine.volume(), 1.0);

        let _ = engine.set_volume(-0.5);
        assert_eq!(engine.volume(), 0.0);
    }
}
