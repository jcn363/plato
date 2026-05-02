# Text‑to‑Speech (TTS) in Plato

Plato includes an optional TTS feature that can read aloud the current document. The implementation is split into two parts: a desktop back‑end (using `tts` crate) and an Android back‑end (via JNI and the Android `TextToSpeech` API). On Kobo devices the feature is stubbed out because there is no audio hardware.

## Feature flag

The TTS modules are gated behind the `tts` feature. In `plato‑core/Cargo.toml`:

```toml
[features]
tts = []
```

When the feature is enabled, the `plato_core::tts` module becomes available and the UI shows the TTS controls.

## Desktop implementation (`tts_desktop.rs`)

- Uses the `tts` crate that wraps system TTS engines (e.g., `speech‑dispatcher` on Linux, `say` on macOS).
- Exposes `TtsEngine` trait implementation `DesktopTts`.
- Supports:
  - `speak(&self, text: &str)`
  - `stop(&self)`
  - `set_rate(&mut self, rate: f32)`
  - `set_pitch(&mut self, pitch: f32)`
  - `set_volume(&mut self, volume: f32)`
- `pause()` and `resume()` are not supported by the underlying engine and return `PlatoError::Unknown`.

## Android implementation (`tts_android.rs`)

- Uses JNI to call the Android `TextToSpeech` class.
- Stores the TTS instance in a `GlobalRef` for lifecycle management.
- Implements all `TtsEngine` methods, but:
  - `pause()` and `resume()` are **unimplemented** – Android TTS does not expose pause/resume; the workaround is to stop and restart (not yet implemented).
- The feature is currently **blocked** on newer `jni` crate versions (0.22+) that break the compile. The code is written but disabled in `Cargo.toml`.

## UI integration

- The TTS toggle button appears in the reader view when the `tts` feature is enabled.
- The user can start/pause/stop speech, and adjust rate/pitch/volume via sliders.
- The current state is tracked in `TtsOptions` (part of `Settings`).

## Testing

Because TTS requires audio hardware (or a running system service), unit tests for the trait are done with `MockTtsEngine` (defined in `test_mocks.rs`). The desktop and Android implementations are tested manually on their respective platforms.

## Known limitations

- No bookmark‑aware playback – the TTS always starts from the current page.
- Android pause/resume not implemented.
- Kobo: feature completely disabled (no audio).
- The `tts` crate may not be available on all Linux distributions; install `speech‑dispatcher` if needed.
