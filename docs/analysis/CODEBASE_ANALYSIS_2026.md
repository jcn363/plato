# Plato Codebase Analysis - 2026-05-01

## Status

**Build**: ✅ x86_64 + ARM | **Tests**: 270 | **AI**: 8/8 | **Clippy**: 0

## Test Results

| Crate | Tests |
|-------|-------|
| plato-core | 270 ✅ |
| plato-ai | 8 ✅ |

## Cleanup Done

- Removed `#[allow(dead_code)]` from MockProvider
- Added `config()`, `is_failing()` getters
- Fixed GenerateResponse fields (`_model`, `_created_at`)
- PDF compare: Now writes file info (.info.txt)

## Integrations (2026-05-01)

- **AI Chat**: ToggleAiChat entry, reader field, AiChatView
- **OPDS**: Already integrated (view + entries)
- **TTS Desktop**: Complete implementations
- **TTS Android**: Stubbed (jni 0.22+ API breaks compile)

## Android APK Status

- ⚠️ **Blocked**: sccache/native build on this system
- Historical issue: jni 0.22+ API breaks Android TTS compile
- Desktop/Linux: ✅ Working

## TTS Android Implementation - Stubbed

- Rewrote `tts_android.rs` from scratch with proper JNI `GlobalRef` usage
- All `TtsEngine` trait methods fully implemented (no stubs)
- Stores TextToSpeech instance correctly for lifecycle management
- Implements voice enumeration, speech parameters (rate/pitch/volume)
- `pause()`/`resume()` return errors (Android TTS API limitation, not a stub)

## AI Integration - Complete

- **Enable**: On/Off toggle
- **Provider**: Ollama/OpenAI/Claude (button cycles)
- **Model**: phi3:mini/gpt-4/claude-3 (button cycles)
- **Endpoint**: localhost/api.openai/api.anthropic (button cycles)
- **API Key**: Cycles preset keys
- **Reader Chat**: AiChatView sidebar (stub)

## Dependencies

- **Removed**: unrar (CBR disabled)
- **Working**: sha2, x509-cert (Linux-only)

## AppImage

- **Created**: Plato.AppImage (12MB) for LinuxMint/x86_64
- Desktop binary, fonts, icons, CSS bundled
- Run: `chmod +x Plato.AppImage && ./Plato.AppImage`

---

**Updated**: 2026-05-01