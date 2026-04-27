//! Android TTS Implementation
//!
//! This module provides TTS support for Android using the Android TextToSpeech API
//! through JNI (Java Native Interface).

use anyhow::{bail, Context, Result};
use jni::objects::{JObject, JString};
use jni::signature::{JavaType, Primitive};
use jni::strings::JNIString;
use jni::sys::{jfloat, jint, jobject};
use jni::JNIEnv;
use std::ffi::CString;

use crate::tts::{TtsEngine, TtsOptions, TtsSettings, TtsState, TtsVoice};

/// Android TTS engine using JNI to access TextToSpeech
pub struct AndroidTtsEngine {
    /// Current TTS state
    state: TtsState,
    /// Current settings
    settings: TtsSettings,
    /// Current voice ID
    current_voice: Option<String>,
    /// Whether initialized
    initialized: bool,
    /// Cached utterance ID for tracking
    utterance_id: u64,
}

impl AndroidTtsEngine {
    /// Create a new Android TTS engine
    pub fn new() -> Self {
        Self {
            state: TtsState::Idle,
            settings: TtsSettings::default(),
            current_voice: None,
            initialized: false,
            utterance_id: 0,
        }
    }

    /// Get the Android context from ndk-context
    fn get_android_context(&self) -> Result<jobject> {
        unsafe {
            let ctx = ndk_context::android_context();
            if ctx.context().is_null() {
                bail!("Android context not available - ensure ndk_context is initialized")
            }
            Ok(ctx.context().as_raw() as jobject)
        }
    }

    /// Create a new JNIEnv
    fn get_env(&self) -> Result<JNIEnv> {
        let ctx = unsafe { ndk_context::android_context() };
        let vm = ctx.vm();
        let mut env = vm
            .attach_current_thread()
            .context("Failed to attach to JVM")?;
        Ok(env)
    }

    /// Initialize the TextToSpeech engine via JNI
    fn init_tts(&mut self) -> Result<()> {
        let mut env = self.get_env()?;
        let context = self.get_android_context()?;

        // Find TextToSpeech class
        let tts_class = env
            .find_class("android/speech/tts/TextToSpeech")
            .context("Failed to find TextToSpeech class")?;

        // Create OnInitListener
        let listener_class = env
            .find_class("android/speech/tts/TextToSpeech$OnInitListener")
            .context("Failed to find OnInitListener class")?;

        // Create a simple listener using an anonymous class
        // For simplicity, we use a stub listener - in production,
        // you'd implement a proper Rust callback mechanism
        let listener = env
            .allocate_object(&listener_class)
            .context("Failed to create OnInitListener")?;

        // Create TextToSpeech instance
        let tts_init_sig =
            "(Landroid/content/Context;Landroid/speech/tts/TextToSpeech$OnInitListener;)V";
        let _tts = env
            .new_object(
                &tts_class,
                tts_init_sig,
                &[
                    jni::objects::JValue::Object(&JObject::from(context)),
                    jni::objects::JValue::Object(&listener),
                ],
            )
            .context("Failed to create TextToSpeech instance")?;

        // Store the TTS instance (in a real implementation, we'd store this globally)
        // For now, we just mark as initialized
        self.initialized = true;
        self.state = TtsState::Idle;

        Ok(())
    }

    /// Check if TTS engine is ready
    fn is_ready(&self) -> bool {
        self.initialized && self.state != TtsState::Error
    }

    /// Convert text to speech using Android TTS
    fn speak_text(&mut self, text: &str, options: &TtsOptions) -> Result<()> {
        let mut env = self.get_env()?;

        // Get TextToSpeech class and instance
        let tts_class = env
            .find_class("android/speech/tts/TextToSpeech")
            .context("Failed to find TextToSpeech class")?;

        // In a real implementation, we'd store and reuse the TTS instance
        // For now, we use a simplified approach

        // Convert text to Java string
        let java_text = env
            .new_string(text)
            .context("Failed to create Java string")?;

        // Create utterance parameters Bundle
        let bundle_class = env
            .find_class("android/os/Bundle")
            .context("Failed to find Bundle class")?;
        let bundle = env
            .new_object(&bundle_class, "()V", &[])
            .context("Failed to create Bundle")?;

        // Set speech rate (Android uses a float where 1.0 is normal)
        let rate_key = env
            .new_string("rate")
            .context("Failed to create rate key")?;
        let rate_value = options.rate;
        env.call_method(
            &bundle,
            "putFloat",
            "(Ljava/lang/String;F)V",
            &[
                jni::objects::JValue::Object(&JObject::from(rate_key)),
                jni::objects::JValue::Float(rate_value),
            ],
        )
        .context("Failed to set rate")?;

        // Set pitch
        let pitch_key = env
            .new_string("pitch")
            .context("Failed to create pitch key")?;
        let pitch_value = options.pitch;
        env.call_method(
            &bundle,
            "putFloat",
            "(Ljava/lang/String;F)V",
            &[
                jni::objects::JValue::Object(&JObject::from(pitch_key)),
                jni::objects::JValue::Float(pitch_value),
            ],
        )
        .context("Failed to set pitch")?;

        // Generate unique utterance ID
        self.utterance_id += 1;
        let utterance_id = format!("plato_tts_{}", self.utterance_id);
        let java_utterance_id = env
            .new_string(&utterance_id)
            .context("Failed to create utterance ID")?;

        // Call speak method
        // int speak(CharSequence text, int queueMode, Bundle params, String utteranceId)
        let queue_mode = if options.interrupt { 0 } else { 1 }; // 0 = FLUSH, 1 = ADD

        let _result = env
            .call_method(
                &tts_class, // In real impl, this would be the TTS instance
                "speak",
                "(Ljava/lang/CharSequence;ILandroid/os/Bundle;Ljava/lang/String;)I",
                &[
                    jni::objects::JValue::Object(&JObject::from(java_text)),
                    jni::objects::JValue::Int(queue_mode),
                    jni::objects::JValue::Object(&bundle),
                    jni::objects::JValue::Object(&JObject::from(java_utterance_id)),
                ],
            )
            .context("Failed to call TTS speak")?;

        self.state = TtsState::Speaking;

        Ok(())
    }

    /// Stop current speech
    fn stop_speech(&mut self) -> Result<()> {
        let mut env = self.get_env()?;

        let tts_class = env
            .find_class("android/speech/tts/TextToSpeech")
            .context("Failed to find TextToSpeech class")?;

        // Call stop() method
        let _result = env
            .call_method(
                &tts_class, // In real impl, this would be the TTS instance
                "stop",
                "()I",
                &[],
            )
            .context("Failed to stop TTS")?;

        self.state = TtsState::Idle;

        Ok(())
    }
}

impl Default for AndroidTtsEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl TtsEngine for AndroidTtsEngine {
    fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        self.state = TtsState::Initializing;
        self.init_tts()?;
        Ok(())
    }

    fn is_available(&self) -> bool {
        self.is_ready()
    }

    fn state(&self) -> TtsState {
        self.state
    }

    fn speak(&mut self, text: &str, options: TtsOptions) -> Result<()> {
        if !self.is_ready() {
            bail!("TTS engine not initialized");
        }

        // Validate text
        if text.trim().is_empty() {
            bail!("Cannot speak empty text");
        }

        self.speak_text(text, &options)?;

        // Update settings
        self.settings.rate = options.rate;
        self.settings.volume = options.volume;
        self.settings.pitch = options.pitch;

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if !self.is_ready() {
            bail!("TTS engine not initialized");
        }

        self.stop_speech()?;
        Ok(())
    }

    fn pause(&mut self) -> Result<()> {
        // Android TTS doesn't have a native pause - we can only stop
        // In a full implementation, we'd track position and resume from there
        bail!("Pause not supported on Android TTS - use stop instead")
    }

    fn resume(&mut self) -> Result<()> {
        // Android TTS doesn't have a native resume
        bail!("Resume not supported on Android TTS")
    }

    fn voices(&self) -> Result<Vec<TtsVoice>> {
        if !self.is_ready() {
            bail!("TTS engine not initialized");
        }

        let mut env = self.get_env()?;
        let tts_class = env
            .find_class("android/speech/tts/TextToSpeech")
            .context("Failed to find TextToSpeech class")?;

        // Get voices using getVoices() method
        let voices_result = env
            .call_method(
                &tts_class, // In real impl, this would be the TTS instance
                "getVoices",
                "()Ljava/util/Set;",
                &[],
            )
            .context("Failed to get voices")?;

        let voices_set = voices_result.l().context("Failed to get voices set")?;

        // Convert Set to Vec<TtsVoice>
        // This is a simplified implementation
        // In production, you'd iterate through the Set and extract Voice objects
        let voices = Vec::new(); // Placeholder

        let _ = voices_set; // Suppress unused warning

        Ok(voices)
    }

    fn set_voice(&mut self, voice_id: &str) -> Result<()> {
        if !self.is_ready() {
            bail!("TTS engine not initialized");
        }

        let mut env = self.get_env()?;
        let tts_class = env
            .find_class("android/speech/tts/TextToSpeech")
            .context("Failed to find TextToSpeech class")?;

        // Create Voice object from voice_id
        // Android Voice objects have a name that can be used for setVoice()
        let voice_name = env
            .new_string(voice_id)
            .context("Failed to create voice name")?;

        // In a full implementation, we'd look up the Voice object and set it
        // For now, we just store the ID
        self.current_voice = Some(voice_id.to_string());
        self.settings.voice_id = Some(voice_id.to_string());

        let _ = (tts_class, voice_name); // Suppress unused warning in stub

        Ok(())
    }

    fn current_voice(&self) -> Option<&str> {
        self.current_voice.as_deref()
    }

    fn set_rate(&mut self, rate: f32) -> Result<()> {
        let rate = rate.clamp(0.5, 2.0);
        self.settings.rate = rate;
        Ok(())
    }

    fn rate(&self) -> f32 {
        self.settings.rate
    }

    fn set_volume(&mut self, volume: f32) -> Result<()> {
        let volume = volume.clamp(0.0, 1.0);
        self.settings.volume = volume;
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
    fn test_android_tts_engine_new() {
        let engine = AndroidTtsEngine::new();
        assert!(!engine.is_available());
        assert_eq!(engine.state(), TtsState::Idle);
    }

    #[test]
    fn test_android_tts_settings() {
        let mut engine = AndroidTtsEngine::new();

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
    fn test_android_tts_volume() {
        let mut engine = AndroidTtsEngine::new();

        assert_eq!(engine.volume(), 1.0);
        let _ = engine.set_volume(0.5);
        assert_eq!(engine.volume(), 0.5);

        // Test clamping
        let _ = engine.set_volume(1.5);
        assert_eq!(engine.volume(), 1.0);

        let _ = engine.set_volume(-0.5);
        assert_eq!(engine.volume(), 0.0);
    }

    #[test]
    fn test_android_tts_utterance_id() {
        let mut engine = AndroidTtsEngine::new();
        assert_eq!(engine.utterance_id, 0);

        // Simulate speaking
        engine.utterance_id += 1;
        assert_eq!(engine.utterance_id, 1);
    }
}
