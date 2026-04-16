# APK_PLAN.md – Plan to Build an Android APK from Plato Rust Codebase

## 1. Overview
Port Plato (a Rust document reader for Kobo) to Android by:
- Compiling Plato’s core logic as a Rust static/dynamic library for the Android target.
- Creating a thin Java/Kotlin UI layer that calls into the Rust library via JNI.
- Packaging everything into an APK using the Android NDK and Gradle.

## 2. Prerequisites
- Android SDK & NDK (r25+).
- Rust toolchain with `android-linux-androideabi`, `aarch64-linux-android`, `armv7-linux-androideabi`, and `i686-linux-android` targets (`rustup target add ...`).
- Android Studio or Gradle command‑line tools.
- Basic knowledge of JNI and Android app structure.

## 3. High‑Level Steps

### 3.1 Isolate the Core Library
1. Identify the pure‑Rust, non‑GUI components (document parsing, rendering, layout, input handling) in `crates/core`.
2. Extract these into a new crate, e.g., `plato-core-lib`, with a `#[no_std]`‑friendly API or at least std‑only (Android provides std).
3. Define a C‑compatible public API (using `extern "C"` and `#[repr(C)]` structs) for the functions the UI will need (open document, render page, handle touch, etc.).
4. Add `#[inline]` and `#[cfg_attr(target_os = "android", ...)]` as needed.

### 3.2 Set Up Android Cross‑Compilation
1. Create a `jni/` directory alongside the Rust crate.
2. Write a `build.rs` or use `cargo:rustc-link-lib` to link against Android log, OpenGL ES (if using GPU), and any required system libs.
3. Configure `.cargo/config.toml` for Android targets (e.g., `[target.aarch64-linux-android]` linker = `aarch64-linux-android21-clang`).
4. Test cross‑compilation: `cargo build --release --target aarch64-linux-android`.

### 3.3 Generate JNI Bindings
1. Write a small Rust file (`jni/src/lib.rs`) that:
   - Uses `#[no_mangle]` and `extern "C"` to export JNI functions (`Java_com_example_plato_PlatoLib_openDocument`, etc.).
   - Calls into the safe Rust core API.
   - Handles Java byte arrays ↔ Rust slices (e.g., for file paths, image buffers).
2. Optionally use the `jni` crate to simplify JNI boilerplate.
3. Ensure all exported functions are `unsafe` only at the FFI boundary; keep safety inside Rust.

### 3.4 Create Android Studio Project
1. In Android Studio, start a new “Native C++” project (or empty activity) with minimum API 21.
2. Replace the default `cpp/` source with the compiled Rust library:
   - Add `src/main/jniLibs/<abi>/libplato_core.so` (built via Cargo) as a prebuilt native library.
   - Or use Gradle’s `externalNativeBuild` to invoke Cargo directly (see `cargo gradle` plugin).
3. Declare the native methods in a Java/Kotlin class (`PlatoLib`) with `static { System.loadLibrary("plato_core"); }`.
4. Implement the UI (Activity/View) that:
   - Loads a document file (from assets or storage).
   - Calls `PlatoLib.openDocument(path)`.
   - On each frame, calls a Rust render function to produce a pixel buffer (or renders directly to a Surface/OpenGL texture).
   - Routes touch events to Rust via JNI.

### 3.5 Rendering Strategy Options
- **Software rendering**: Render to a Rust `Vec<u8>` (RGBA) and pass the byte array to Java to draw via `Bitmap`/`Canvas`.
- **OpenGL ES**: Render Rust output to a texture and let Java issue a draw call; avoid copying pixels.
- **SurfaceTexture**: Have Rust render via `glfw`/`sdl2`‑like context bound to a ANativeWindow (more complex).
Choose the simplest (software) for initial proof‑of‑concept; optimize later.

### 3.6 Packaging & Signing
1. Ensure the APK includes the correct `.so` files for each ABI in `lib/`.
2. Use `./gradlew assembleRelease` to build a signed APK (or `assembleDebug` for testing).
3. Install on device/emulator: `adb install app-debug.apk`.

### 3.7 Testing & Iteration
1. Verify document opening, page rendering, and touch navigation work.
2. Use Android Profiler to check CPU/memory usage; adjust hot‑paths with `#[inline]` or `Cow<str>` as needed.
3. Iterate on the Rust‑Java boundary to minimize data copying (e.g., render directly into a Java‑allocated `ByteBuffer`).

## 4. Non‑Goals (Keep Scope Minimal)
- Do **not** attempt to port the Kobo‑specific framebuffer or hardware abstraction layers; replace them with Android equivalents.
- Do **not** maintain backward compatibility with the Kobo UI; the Android app will have its own UI.
- Do **not** try to reuse the existing SDL2 emulator code unless it simplifies OpenGL setup; prefer a clean JNI layer.

## 5. References & Existing Work
- Plato’s existing `mupdf_wrapper` shows how to expose C functions from Rust for MuPDF; adapt that pattern.
- The `emulator` crate uses SDL2; can inspire OpenGL ES context creation.
- Android NDK documentation: calling Rust from Java via JNI.
- Examples: `rust-android-gradle` plugin, `cargo-apk` tool.

## 6. Success Criteria
- APK installs and runs on ARM64 Android device.
- User can open EPUB/PDF files, scroll pages, and interact with basic UI (zoom, rotate, etc.).
- No crashes or undefined behavior from Rust side.
- Build process is reproducible via Gradle/Cargo commands.

--- 
*End of plan. Save this content as `APK_PLAN.md` in the project root.*