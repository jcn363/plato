//! Android TTS Implementation
//!
#![allow(
    clippy::needless_borrows_for_generic_args,
    clippy::cmp_owned,
    clippy::redundant_pattern_matching
)]

//! This module provides TTS support for Android using the Android TextToSpeech API
//! through JNI (Java Native Interface).
//!
//! ## Pause/Resume Implementation
//!
//! Android TTS doesn't have native pause/resume. We implement it by:
//! 1. Using `synthesizeToFile()` to create an audio file
//! 2. Using MediaPlayer to play/pause/resume the audio
//!
//! ## JNI 0.22+ Compatibility
//!
//! This implementation uses jni 0.22+ API:
//! - `Global::as_obj()` returns `&JObject<'static>` (GlobalRef is now Global<JObject<'static>>)
//! - `JObject::from_raw(env, raw)` requires an Env parameter for safety
//! - `JObject` no longer implements `Copy`
//! - `JValue::Object` expects `&JObject<'a>` (use &obj instead of obj.into())

use anyhow::{bail, Context, Result};
use jni::objects::{GlobalRef, JObject, JString, JValue};
use jni::JNIEnv;
use std::time::SystemTime;

use crate::tts::{TtsEngine, TtsOptions, TtsSettings, TtsState, TtsVoice};

/// Android TTS engine using JNI to access TextToSpeech API
///
/// Implements pause/resume by synthesizing to a temp file and using MediaPlayer.
pub struct AndroidTtsEngine {
    /// JavaVM kept alive to allow AttachGuard to work
    vm: Option<jni::JavaVM>,
    /// Global reference to TextToSpeech instance
    tts_instance: Option<GlobalRef>,
    /// Global reference to MediaPlayer instance (for pause/resume)
    media_player: Option<GlobalRef>,
    /// Current TTS state
    state: TtsState,
    /// Current settings
    settings: TtsSettings,
    /// Current voice ID
    current_voice: Option<String>,
    /// Whether engine is initialized
    initialized: bool,
    /// Temp file path for synthesized audio (for pause/resume)
    temp_file_path: Option<String>,
}

impl AndroidTtsEngine {
    /// Create a new Android TTS engine
    pub fn new() -> Self {
        let vm = ndk_context::android_context();
        let java_vm = unsafe { jni::JavaVM::from_raw(vm.vm().cast()).ok() };
        Self {
            vm: java_vm,
            tts_instance: None,
            media_player: None,
            state: TtsState::Idle,
            settings: TtsSettings::default(),
            current_voice: None,
            initialized: false,
            temp_file_path: None,
        }
    }

    /// Get JNI environment (jni 0.21 + ndk-context pattern)
    fn get_env(&self) -> Result<jni::AttachGuard<'_>> {
        let vm = self.vm.as_ref().context("JavaVM not initialized")?;
        let env = vm
            .attach_current_thread()
            .context("Failed to attach to JVM")?;
        Ok(env)
    }

    /// Get the Android context as JObject
    fn get_android_context() -> Result<JObject<'static>> {
        let ctx = ndk_context::android_context();
        let context = ctx.context();
        if context.is_null() {
            bail!("Android context not available");
        }
        Ok(unsafe { JObject::from_raw(context.cast()) })
    }

    /// Initialize the TextToSpeech engine
    fn init_tts(&mut self) -> Result<()> {
        let context = Self::get_android_context()?;

        // Do JNI operations in a separate scope to release env borrow
        let global_ref = {
            let mut env = self.get_env()?;

            // Find TextToSpeech class
            let tts_class = env
                .find_class("android/speech/tts/TextToSpeech")
                .context("Failed to find TextToSpeech class")?;

            // Constructor: TextToSpeech(Context context, OnInitListener listener)
            let tts_obj = env
                .new_object(
                    &tts_class,
                    "(Landroid/content/Context;Landroid/speech/tts/TextToSpeech$OnInitListener;)V",
                    &[JValue::Object(&context), JValue::Object(&JObject::null())],
                )
                .context("Failed to create TextToSpeech instance")?;

            // Convert to global reference
            env.new_global_ref(&tts_obj)
                .context("Failed to create global reference for TTS")?
        };

        self.tts_instance = Some(global_ref);
        self.initialized = true;
        self.state = TtsState::Idle;

        Ok(())
    }

    /// Get TTS instance as JObject reference
    fn get_tts_instance(&self) -> Result<&JObject<'static>> {
        match &self.tts_instance {
            Some(gref) => Ok(gref.as_obj()),
            None => bail!("TTS instance not initialized"),
        }
    }

    /// Get MediaPlayer instance as JObject reference
    fn get_media_player(&self) -> Result<&JObject<'static>> {
        match &self.media_player {
            Some(gref) => Ok(gref.as_obj()),
            None => bail!("MediaPlayer not initialized"),
        }
    }

    /// Clean up temp file
    fn cleanup_temp_file(&mut self) {
        if let Some(path) = &self.temp_file_path {
            let _ = std::fs::remove_file(path);
        }
        self.temp_file_path = None;
    }

    /// Speak text using Android TTS with pause/resume support
    ///
    /// This method:
    /// 1. Synthesizes text to a temporary file using `synthesizeToFile()`
    /// 2. Waits for synthesis to complete (with timeout)
    /// 3. Creates a MediaPlayer to play the audio file
    /// 4. MediaPlayer supports pause/resume natively
    fn speak_text(&mut self, text: &str, options: &TtsOptions) -> Result<()> {
        // Step 1: Do all self modifications FIRST (before any JNI)
        self.cleanup_temp_file();

        // Create temp file path
        let temp_dir = std::env::temp_dir();
        let temp_file_path = temp_dir.join(format!(
            "plato_tts_{}.wav",
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        ));
        let temp_file_path_str = temp_file_path.to_str().context("Invalid temp file path")?;

        // Create the file (ensure it exists)
        let _file = std::fs::File::create(&temp_file_path).context("Failed to create temp file")?;

        // Store temp file path
        let temp_file_for_cleanup = temp_file_path_str.to_string();
        self.temp_file_path = Some(temp_file_for_cleanup.clone());

        // Step 2: Do JNI operations (create scope to release env borrow before updating self)
        let (mp_global_ref, final_rate, final_volume, final_pitch) = {
            let mut env = self.get_env()?;
            let tts_obj = self.get_tts_instance()?;

            // Convert path to Java File object
            let jfile_path = env
                .new_string(&temp_file_path_str)
                .context("Failed to create file path string")?;

            let file_class = env
                .find_class("java/io/File")
                .context("Failed to find File class")?;
            let file_obj = env
                .new_object(
                    &file_class,
                    "(Ljava/lang/String;)V",
                    &[JValue::Object(&jfile_path)],
                )
                .context("Failed to create File object")?;

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
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis()
                ))
                .context("Failed to create utterance ID")?;

            // Call synthesizeToFile
            let text_jstr = env
                .new_string(text)
                .context("Failed to create text string")?;
            env.call_method(
                tts_obj,
                "synthesizeToFile",
                "(Ljava/lang/CharSequence;Landroid/os/Bundle;Ljava/io/File;Ljava/lang/String;)I",
                &[
                    JValue::Object(&text_jstr),
                    JValue::Object(&bundle),
                    JValue::Object(&file_obj),
                    JValue::Object(&utterance_id),
                ],
            )
            .context("Failed to call synthesizeToFile")?;

            // Wait for synthesis to complete
            let mut attempts = 0;
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if let Ok(metadata) = std::fs::metadata(&temp_file_path) {
                    if metadata.len() > 44 {
                        break;
                    }
                }
                attempts += 1;
                if attempts > 100 {
                    bail!("TTS synthesis timeout");
                }
            }

            // Get Android context for MediaPlayer
            let context = Self::get_android_context()?;

            // Create Uri from file
            let uri_class = env
                .find_class("android/net/Uri")
                .context("Failed to find Uri class")?;
            let uri_obj = env
                .call_static_method(
                    &uri_class,
                    "fromFile",
                    "(Ljava/io/File;)Landroid/net/Uri;",
                    &[JValue::Object(&file_obj)],
                )
                .context("Failed to create Uri from file")?
                .l()
                .context("Failed to get Uri object")?;

            // Create MediaPlayer
            let mp_class = env
                .find_class("android/media/MediaPlayer")
                .context("Failed to find MediaPlayer class")?;
            let mp_obj = env
                .call_static_method(
                    &mp_class,
                    "create",
                    "(Landroid/content/Context;Landroid/net/Uri;)Landroid/media/MediaPlayer;",
                    &[JValue::Object(&context), JValue::Object(&uri_obj)],
                )
                .context("Failed to create MediaPlayer")?
                .l()
                .context("Failed to get MediaPlayer object")?;

            // Store MediaPlayer as global reference
            let mp_global_ref = env
                .new_global_ref(&mp_obj)
                .context("Failed to create global reference for MediaPlayer")?;

            // Start playback
            let mp_obj = self.get_media_player()?;
            env.call_method(&mp_obj, "start", "()V", &[])
                .context("Failed to start MediaPlayer")?;

            (mp_global_ref, options.rate, options.volume, options.pitch)
        }; // env borrow ends here

        // Step 3: Update self after env is released
        self.media_player = Some(mp_global_ref);
        self.state = TtsState::Speaking;
        self.settings.rate = final_rate;
        self.settings.volume = final_volume;
        self.settings.pitch = final_pitch;

        Ok(())
    }

    /// Stop speech (stops both TTS and MediaPlayer)
    fn stop_speech(&mut self) -> Result<()> {
        // Do JNI operations in separate scope
        {
            let mut env = self.get_env()?;

            // Stop and release MediaPlayer if active
            if let Some(_) = &self.media_player {
                if let Ok(mp_obj) = self.get_media_player() {
                    let _ = env.call_method(&mp_obj, "stop", "()V", &[]);
                    let _ = env.call_method(&mp_obj, "release", "()V", &[]);
                }
            }

            // Also stop TTS in case it's still synthesizing
            if let Ok(tts_obj) = self.get_tts_instance() {
                let _ = env.call_method(tts_obj, "stop", "()I", &[]);
            }
        }

        // Update state after env is released
        self.media_player = None;
        self.state = TtsState::Idle;
        self.cleanup_temp_file();

        Ok(())
    }

    /// Pause speech (uses MediaPlayer.pause())
    fn pause_speech(&mut self) -> Result<()> {
        if self.media_player.is_none() {
            bail!("No active speech to pause");
        }

        let mp_obj = self.get_media_player()?;

        {
            let mut env = self.get_env()?;
            env.call_method(&mp_obj, "pause", "()V", &[])
                .context("Failed to pause MediaPlayer")?;
        }

        self.state = TtsState::Paused;
        Ok(())
    }

    /// Resume speech (uses MediaPlayer.start())
    fn resume_speech(&mut self) -> Result<()> {
        if self.media_player.is_none() {
            bail!("No paused speech to resume");
        }

        let mp_obj = self.get_media_player()?;

        {
            let mut env = self.get_env()?;
            env.call_method(&mp_obj, "start", "()V", &[])
                .context("Failed to resume MediaPlayer")?;
        }

        self.state = TtsState::Speaking;
        Ok(())
    }

    /// Get available voices
    fn get_voices(&self) -> Result<Vec<TtsVoice>> {
        let mut env = self.get_env()?;
        let tts_obj = self.get_tts_instance()?;

        // Call getVoices()
        let voices_result = env
            .call_method(tts_obj, "getVoices", "()Ljava/util/Set;", &[])
            .context("Failed to call getVoices")?
            .l()
            .context("Failed to get voices set")?;

        // Convert Set<Voice> to Vec<TtsVoice>
        let voices = self.convert_voices_set_to_vec(&mut env, voices_result)?;

        Ok(voices)
    }

    /// Convert Java Set<Voice> to Vec<TtsVoice>
    fn convert_voices_set_to_vec(&self, env: &mut JNIEnv, set: JObject) -> Result<Vec<TtsVoice>> {
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
    fn extract_voice_info(&self, env: &mut JNIEnv, voice_obj: JObject) -> Result<TtsVoice> {
        // Get voice name: getName()
        let name_obj = env
            .call_method(&voice_obj, "getName", "()Ljava/lang/String;", &[])
            .context("Failed to get voice name")?
            .l()
            .context("Failed to get name object")?;
        let name_jstr: JString = name_obj.into();
        let name_str: String = env
            .get_string(&name_jstr)
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
            let locale_jstr: JString = locale_str_obj.into();
            let s: String = env
                .get_string(&locale_jstr)
                .context("Failed to convert locale to string")?
                .into();
            s
        } else {
            "en-US".to_string()
        };

        Ok(TtsVoice {
            id: name_str.clone(),
            name: name_str,
            language: locale_str,
            is_male: None,
            quality: None,
        })
    }

    /// Set voice by ID
    fn set_voice_by_id(&mut self, voice_id: &str) -> Result<()> {
        // Do JNI operations in a scoped block
        let voice_id_string = {
            let mut env = self.get_env()?;
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

                let name_jstr: JString = name_obj.into();
                let name: String = env
                    .get_string(&name_jstr)
                    .context("Failed to convert name")?
                    .into();

                if name.to_string() == voice_id {
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

            voice_id.to_string()
        }; // env borrow ends here

        // Update self after env is released
        self.current_voice = Some(voice_id_string.clone());
        self.settings.voice_id = Some(voice_id_string);

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
        if !self.is_ready() {
            bail!("TTS engine not initialized");
        }

        if self.state != TtsState::Speaking {
            bail!("Not currently speaking");
        }

        self.pause_speech()
    }

    fn resume(&mut self) -> Result<()> {
        if !self.is_ready() {
            bail!("TTS engine not initialized");
        }

        if self.state != TtsState::Paused {
            bail!("Not currently paused");
        }

        self.resume_speech()
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

#[cfg(target_os = "android")]
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
