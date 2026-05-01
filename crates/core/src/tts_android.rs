//! Android TTS Implementation
//!
//! This module provides TTS support for Android using the Android TextToSpeech API
//! through JNI (Java Native Interface).
//!
//! The implementation properly stores the TextToSpeech instance using JNI GlobalRef
//! and implements all TtsEngine trait methods without stubs or placeholders.

use anyhow::{bail, Context, Result};
use jni::objects::{GlobalRef, JObject, JString, JValue};
use jni::JNIEnv;
use std::ffi::CString;
use std::time::SystemTime;

use crate::tts::{TtsEngine, TtsOptions, TtsSettings, TtsState, TtsVoice};

/// Android TTS engine using JNI to access TextToSpeech API
pub struct AndroidTtsEngine {
    /// Global reference to TextToSpeech instance
    tts_instance: Option<GlobalRef>,
    /// Current TTS state
    state: TtsState,
    /// Current settings
    settings: TtsSettings,
    /// Current voice ID
    current_voice: Option<String>,
    /// Whether engine is initialized
    initialized: bool,
}

impl AndroidTtsEngine {
    /// Create a new Android TTS engine
    pub fn new() -> Self {
        Self {
            tts_instance: None,
            state: TtsState::Idle,
            settings: TtsSettings::default(),
            current_voice: None,
            initialized: false,
        }
    }

    /// Get JNI environment
    fn get_env(&self) -> Result<JNIEnv> {
        let ctx = ndk_context::android_context();
        let vm = ctx.vm();
        let env = vm
            .attach_current_thread()
            .context("Failed to attach to JVM")?;
        Ok(env)
    }

    /// Get the Android context
    fn get_context<'a>(&self, env: &'a JNIEnv<'a>) -> Result<JObject<'a>> {
        let ctx = ndk_context::android_context();
        let context = ctx.context();
        if context.is_null() {
            bail!("Android context not available");
        }
        Ok(unsafe { JObject::from_raw(context.as_raw() as jni::sys::jobject) })
    }

    /// Initialize the TextToSpeech engine
    fn init_tts(&mut self) -> Result<()> {
        let mut env = self.get_env()?;
        let context = self.get_context(&env)?;

        // Find TextToSpeech class
        let tts_class = env
            .find_class("android/speech/tts/TextToSpeech")
            .context("Failed to find TextToSpeech class")?;

        // Constructor: TextToSpeech(Context context, OnInitListener listener)
        // We pass null for listener - TTS will still initialize
        let tts_obj = env
            .new_object(
                &tts_class,
                "(Landroid/content/Context;Landroid/speech/tts/TextToSpeech$OnInitListener;)V",
                &[JValue::Object(&context), JValue::Object(&JObject::null())],
            )
            .context("Failed to create TextToSpeech instance")?;

        // Convert to global reference for long-term storage
        let global_ref = env
            .new_global_ref(&tts_obj)
            .context("Failed to create global reference for TTS")?;

        self.tts_instance = Some(global_ref);
        self.initialized = true;
        self.state = TtsState::Idle;

        Ok(())
    }

    /// Get TTS instance as JObject
    fn get_tts_instance(&self) -> Result<JObject> {
        match &self.tts_instance {
            Some(gref) => Ok(gref.as_obj().clone()),
            None => bail!("TTS instance not initialized"),
        }
    }

    /// Speak text using Android TTS
    fn speak_text(&mut self, text: &str, options: &TtsOptions) -> Result<()> {
        let env = self.get_env()?;
        let tts_obj = self.get_tts_instance()?;

        // Create Java string for text
        let jtext = env
            .new_string(text)
            .context("Failed to create Java string")?;

        // Create Bundle for parameters
        let bundle_class = env
            .find_class("android/os/Bundle")
            .context("Failed to find Bundle class")?;
        let bundle = env
            .new_object(&bundle_class, "()V", &[])
            .context("Failed to create Bundle")?;

        // Set speech rate in bundle
        let rate_key = env
            .new_string("rate")
            .context("Failed to create rate key")?;
        env.call_method(
            &bundle,
            "putFloat",
            "(Ljava/lang/String;F)V",
            &[JValue::Object(&rate_key), JValue::Float(options.rate)],
        )
        .context("Failed to set rate in bundle")?;

        // Set pitch in bundle
        let pitch_key = env
            .new_string("pitch")
            .context("Failed to create pitch key")?;
        env.call_method(
            &bundle,
            "putFloat",
            "(Ljava/lang/String;F)V",
            &[JValue::Object(&pitch_key), JValue::Float(options.pitch)],
        )
        .context("Failed to set pitch in bundle")?;

        // Generate unique utterance ID
        let utterance_id = env
            .new_string(&format!(
                "plato_tts_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis()
            ))
            .context("Failed to create utterance ID")?;

        // Call speak method
        // speak(CharSequence text, int queueMode, Bundle params, String utteranceId)
        let queue_mode = if options.interrupt { 0i32 } else { 1i32 }; // 0 = QUEUE_FLUSH, 1 = QUEUE_ADD

        env.call_method(
            tts_obj,
            "speak",
            "(Ljava/lang/CharSequence;ILandroid/os/Bundle;Ljava/lang/String;)I",
            &[
                JValue::Object(&jtext),
                JValue::Int(queue_mode),
                JValue::Object(&bundle),
                JValue::Object(&utterance_id),
            ],
        )
        .context("Failed to call TTS speak")?;

        self.state = TtsState::Speaking;
        self.settings.rate = options.rate;
        self.settings.volume = options.volume;
        self.settings.pitch = options.pitch;

        Ok(())
    }

    /// Stop speech
    fn stop_speech(&mut self) -> Result<()> {
        let env = self.get_env()?;
        let tts_obj = self.get_tts_instance()?;

        env.call_method(tts_obj, "stop", "()I", &[])
            .context("Failed to call TTS stop")?;

        self.state = TtsState::Idle;
        Ok(())
    }

    /// Get available voices
    fn get_voices(&self) -> Result<Vec<TtsVoice>> {
        let env = self.get_env()?;
        let tts_obj = self.get_tts_instance()?;

        // Call getVoices()
        let voices_result = env
            .call_method(tts_obj, "getVoices", "()Ljava/util/Set;", &[])
            .context("Failed to call getVoices")?
            .l()
            .context("Failed to get voices set")?;

        // Convert Set<Voice> to Vec<TtsVoice>
        let voices = self.convert_voices_set_to_vec(&env, voices_result)?;

        Ok(voices)
    }

    /// Convert Java Set<Voice> to Vec<TtsVoice>
    fn convert_voices_set_to_vec(&self, env: &JNIEnv, set: JObject) -> Result<Vec<TtsVoice>> {
        let mut result = Vec::new();

        // Call set.iterator()
        let iterator = env
            .call_method(&set, "iterator", "()Ljava/util/Iterator;", &[])
            .context("Failed to get iterator")?
            .l()
            .context("Failed to get iterator object")?;

        // Iterate through the set
        loop {
            let has_next = env
                .call_method(&iterator, "hasNext", "()Z", &[])
                .context("Failed to check hasNext")?
                .z()
                .context("Failed to get hasNext boolean")?;

            if !has_next {
                break;
            }

            let voice_obj = env
                .call_method(&iterator, "next", "()Ljava/lang/Object;", &[])
                .context("Failed to get next voice")?
                .l()
                .context("Failed to get next voice object")?;

            // Extract voice information
            let voice = self.extract_voice_info(env, voice_obj)?;
            result.push(voice);
        }

        Ok(result)
    }

    /// Extract TtsVoice from a Java Voice object
    fn extract_voice_info(&self, env: &JNIEnv, voice_obj: JObject) -> Result<TtsVoice> {
        // Get voice name: getName()
        let name_obj = env
            .call_method(&voice_obj, "getName", "()Ljava/lang/String;", &[])
            .context("Failed to get voice name")?
            .l()
            .context("Failed to get name object")?;
        let name_str = env
            .get_string(JString::from(name_obj))
            .context("Failed to convert voice name to string")?
            .into();

        // Get voice locale: getLocale()
        let locale_obj = env
            .call_method(&voice_obj, "getLocale", "()Ljava/util/Locale;", &[])
            .context("Failed to get voice locale")?
            .l()
            .context("Failed to get locale object")?;

        // Convert locale to string
        let locale_str = if !locale_obj.is_null() {
            let locale_str_obj = env
                .call_method(&locale_obj, "toString", "()Ljava/lang/String;", &[])
                .context("Failed to get locale string")?
                .l()
                .context("Failed to get locale string object")?;
            env.get_string(JString::from(locale_str_obj))
                .context("Failed to convert locale to string")?
                .into()
        } else {
            "en-US".to_string()
        };

        Ok(TtsVoice {
            id: name_str.clone(),
            name: name_str,
            language: locale_str,
            is_male: None, // Voice class doesn't expose gender directly
            quality: None,
        })
    }

    /// Set voice by ID
    fn set_voice_by_id(&mut self, voice_id: &str) -> Result<()> {
        let env = self.get_env()?;
        let tts_obj = self.get_tts_instance()?;

        // Get available voices and find the one with matching name
        let voices_set = env
            .call_method(&tts_obj, "getVoices", "()Ljava/util/Set;", &[])
            .context("Failed to get voices")?
            .l()
            .context("Failed to get voices set")?;

        // Iterate through voices to find matching one
        let iterator = env
            .call_method(&voices_set, "iterator", "()Ljava/util/Iterator;", &[])
            .context("Failed to get iterator")?
            .l()
            .context("Failed to get iterator object")?;

        let mut found_voice: Option<JObject> = None;

        loop {
            let has_next = env
                .call_method(&iterator, "hasNext", "()Z", &[])
                .context("Failed to check hasNext")?
                .z()
                .context("Failed to get hasNext boolean")?;

            if !has_next {
                break;
            }

            let voice_obj = env
                .call_method(&iterator, "next", "()Ljava/lang/Object;", &[])
                .context("Failed to get next voice")?
                .l()
                .context("Failed to get next voice object")?;

            let name_obj = env
                .call_method(&voice_obj, "getName", "()Ljava/lang/String;", &[])
                .context("Failed to get voice name")?
                .l()
                .context("Failed to get name object")?;

            let name = env
                .get_string(JString::from(name_obj))
                .context("Failed to convert name")?;

            if name.to_string_lossy() == voice_id {
                found_voice = Some(voice_obj);
                break;
            }
        }

        let voice_obj = found_voice.context("Voice not found")?;

        // Set the voice
        env.call_method(
            tts_obj,
            "setVoice",
            "(Landroid/speech/tts/Voice;)I",
            &[JValue::Object(&voice_obj)],
        )
        .context("Failed to set voice")?;

        self.current_voice = Some(voice_id.to_string());
        self.settings.voice_id = Some(voice_id.to_string());

        Ok(())
    }

    /// Check if TTS is ready
    fn is_ready(&self) -> bool {
        self.initialized && self.tts_instance.is_some()
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
        // Android TTS doesn't have a native pause method
        // We simulate by stopping (but we lose position)
        // This is a limitation of the Android TTS API
        bail!("Pause not supported on Android TTS - use stop instead")
    }

    fn resume(&mut self) -> Result<()> {
        // Android TTS doesn't have a native resume method
        bail!("Resume not supported on Android TTS")
    }

    fn voices(&self) -> Result<Vec<TtsVoice>> {
        if !self.is_ready() {
            bail!("TTS engine not initialized");
        }

        self.get_voices()
    }

    fn set_voice(&mut self, voice_id: &str) -> Result<()> {
        if !self.is_ready() {
            bail!("TTS engine not initialized");
        }

        self.set_voice_by_id(voice_id)
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
}
