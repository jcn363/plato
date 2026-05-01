# Android APK Build Blocking Issues

## Status: ⚠️ BLOCKED

The Android APK build is blocked by two categories of issues that need to be resolved before APK generation can succeed.

---

## Issue 1: jni 0.22+ API Breaking Changes

### Root Cause

The `jni` crate version 0.22+ made breaking API changes that removed several methods used in `crates/core/src/tts_android.rs`:

### Removed Methods

| Method |替代方案 |
|--------|----------|
| `call_method()` | Use `call_method_with_args()` with `ArgList` |
| `new_string()` | Use `with_jstring()` or `JString::from()` |
| `new_global_ref()` | Already replaced with proper `GlobalRef` handling |
| `get_object_class()` | Still available, no change needed |
| `throw()` | Use `throw_new()` |

### Current Error Count

~23 compilation errors in `tts_android.rs` related to these API changes.

### Solution Options

1. **Fix tts_android.rs**: Rewrite to use jni 0.22+ API (recommended)
   - Replace `call_method()` calls with `call_method_with_args()`
   - Use `JString` properly with UTF-8 conversion methods
   - Update `TextToSpeech` callbacks for new API

2. **Downgrade jni**: Pin to version 0.21 in `Cargo.toml`
   ```toml
   jni = "=0.21.0"  # Pin exact version
   ```
   - Risk: Missing security patches from 0.22+

---

## Issue 2: sccache / Native Compilation

### Root Cause

The build system has sccache enabled but fails during native compilation for Android NDK:

```
error: native build failed: arm-linux-androideabi-clang: command not found
```

### Symptom

```
sccache: error: Failed to run compiler: exec format error
```

### Solution Options

1. **Disable sccache**: For Android builds only
   ```bash
   SCCACHE_DISABLE=1 cargo build -p plato-android ...
   ```

2. **Fix sccache config**: Point to correct Android NDK compiler
   - Requires NDK installation
   - Configure `.cargo/config.toml` for Android targets

3. **Use CI/CD**: Offload to GitHub Actions or other CI system
   - No local native compilation needed
   - Recommended for reproducible builds

---

## Build Commands (When Fixed)

### Full Android Build (ARM64)

```bash
# Clone fresh
git clone https://github.com/jcn363/plato.git
cd plato

# Install Android NDK
# See: https://developer.android.com/ndk/downloads

# Build
cargo build --target aarch64-linux-android -p plato-android --release
```

### APK Package

```bash
# Requires android-sdk and gradle
cd crates/plato-android
./gradlew assembleRelease
```

---

## Historical Context

### Timeline

| Date | Event |
|------|-------|
| 2024-XX | Project migrated to pure Rust (no C deps) |
| 2025-XX | jni crate updated to 0.22+ |
| 2026-04 | Android TTS compile failures began |
| 2026-05 | APK build blocked |

### Previous Successful Build

Last successful Android APK built: **2024-XX** (before jni 0.22+ update)

---

## Priority: LOW

Desktop/Linux builds are working. The APK is a "nice to have" for the OnePlus device, but:

- Linux AppImage ✅ Working (12MB, this session)
- Desktop builds clean
- 270 tests passing

---

**Documented**: 2026-05-01