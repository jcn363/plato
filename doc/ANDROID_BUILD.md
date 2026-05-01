# Android Build Instructions

## Prerequisites

### NDK Setup
```bash
export ANDROID_NDK=/home/user/Android/sdk/android-ndk-r26b
export PATH=$ANDROID_NDK/toolchains/llvm/prebuilt/linux-x86_64/bin:$PATH
```

### Rust Targets
```bash
rustup target add aarch64-linux-android armv7-linux-androideabi
```

### Dependencies
- **jni = "0.22.4"** (or later 0.22.x) - JNI bindings with improved safety
- **ndk-context = "0.1.1"** - Android context access
- **tts = "0.26.3"** - Desktop TTS (optional, for comparison)

**Note:** jni 0.22+ has breaking changes:
- `JObject::from_raw()` now requires an `Env` parameter
- `GlobalRef` renamed to `Global<T>` (type alias preserved)
- `AttachGuard` replaces direct `JNIEnv` returns
- `JObject` no longer implements `Copy`

## Building for Android

### Disable sccache (if not available in NDK environment)
```bash
# Unset sccache wrapper
unset RUSTC_WRAPPER

# Or set to empty
export RUSTC_WRAPPER=""
```

### Build Commands
```bash
# ARM64 (newer Kobo devices: Libra 2, Sage, Clara 2E, etc.)
cargo build --target aarch64-linux-android --profile release-arm64 -p plato

# ARM32 (original Kobo devices)
cargo build --target armv7-linux-androideabi --profile release-arm -p plato
```

## TTS Support

### Requirements
- Android API Level 21+ (for `synthesizeToFile()` with ParcelFileDescriptor)
- `TextToSpeech` system service available
- `MediaPlayer` for pause/resume support

### Implementation
- `crates/core/src/tts_android.rs` - JNI-based TTS with pause/resume
- Uses `synthesizeToFile()` + `MediaPlayer` pattern
- jni 0.22+ compatible (see `doc/OCR_TTS.md` for details)

## Common Issues

### Issue: sccache not available in NDK environment
**Error:** `sccache: command not found`

**Solution:** Unset RUSTC_WRAPPER:
```bash
unset RUSTC_WRAPPER
export RUSTC_WRAPPER=""
```

This is an environment issue, not a code issue. The NDK environment doesn't have sccache installed.

### Issue: Linker fails
**Error:** `linker 'aarch64-linux-android21-clang' not found`

**Solution:** Verify NDK path:
```bash
ls $ANDROID_NDK/toolchains/llvm/prebuilt/linux-x86_64/bin/
```

## File Locations
- ARM64 binary: `target/aarch64-linux-android/release/plato`
- ARM32 binary: `target/armv7-linux-androideabi/release/plato`
- Build config: `.cargo/config.toml`
