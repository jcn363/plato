# **Plan to create an Android version of Plato**

---

### 1. High‑level strategy
- Keep the existing Rust core (`crates/core`) as a **static/dynamic library** (`libplato_core.so`) that exposes a thin C‑compatible API.
- Build a **native Android project** (Kotlin/Java) that loads this library via JNI and provides the UI, lifecycle handling, and Android‑specific services (storage, permissions, etc.).
- Render Plato’s output to a **software bitmap** (RGBA) that is passed to Java each frame, or optionally render to an OpenGL ES texture for zero‑copy display.
- Package everything with Gradle, produce an APK/AAB, and sign it for distribution.

---

### 2. Prerequisites
| Tool | Version / Notes |
|------|-----------------|
| Android Studio (latest) | Includes SDK, emulator, build tools |
| JDK 17 (or latest LTS) | `JAVA_HOME` set if using CLI |
| Rust toolchain | `stable` plus Android targets: `aarch64-linux-android`, `armv7-linux-androideabi`, `i686-linux-android`, `x86_64-linux-android` (`rustup target add …`) |
| Android NDK (r25+) | Provides clang linker & sysroots |
| Gradle (wrapper) | Comes with Android Studio |
| Git | For version control |
| (Optional) Docker | For reproducible CI builds |

---

### 3. Isolate & prepare the Rust core
1. **Create a new crate** (e.g., `plato-core-lib`) that re‑exports the public API you need from `crates/core`.
   - Keep only the **pure‑logic** parts: document loading, layout, rendering to a pixmap, input handling, settings.
   - Remove any Kobo‑specific framebuffer/hardware code; replace with platform‑agnostic interfaces.
2. **Define a C‑compatible ABI**
   - Use `#[no_mangle]` and `extern "C"` functions.
   - Example signatures:
     ```rust
     #[no_mangle]
     pub extern "C" fn plato_open(path: *const c_char, path_len: usize) -> *mut PlatoContext;
     #[no_mangle]
     pub extern "C" fn plato_render(ctx: *mut PlatoContext, width: i32, height: i32, out_buf: *mut u8) -> i32; // returns 0 on success
     #[no_mangle]
     pub extern "C" fn plato_input(ctx: *mut PlatoContext, kind: i32, x: i32, y: i32) -> void;
     #[no_mangle]
     pub extern "C" fn plato_free(ctx: *mut PlatoContext);
     ```
   - Return opaque pointers (`*mut PlatoContext`) that hold a Rust `Box<CoreContext>`.
   - Use `c_char` for UTF‑8 paths; pass length to avoid needing NUL‑termination.
3. **Error handling**
   - On failure, return `nullptr` (for object creation) or a negative error code; optionally set a global error string via another getter.
4. **Build script (`build.rs`)**
   - Link against Android system libraries (`-llog`, `-landroid`, `-lEGL`, `-lGLESv2` if using GL).
   - Set `crate-type = ["staticlib", "cdylib"]` to produce both `.a` and `.so`.
5. **Cross‑compile test**
   ```bash
   cargo build --release --target aarch64-linux-android
   cargo build --release --target armv7-linux-androideabi
   # verify .so files appear in target/*/release/libplato_core.so
   ```

---

### 4. Android project setup
1. **Create a new Android Studio project**
   - Choose **Empty Compose Activity** (Kotlin, Jetpack Compose) – gives a modern UI with less boilerplate.
   - Minimum SDK: API 21 (Android 5.0) – matches Plato’s target hardware class.
   - Package name: e.g., `org.platoreader.app`.
2. **Add the native library**
   - Under `app/src/main/jniLibs/<abi>/` place the compiled `.so` files for each ABI (`armeabi-v7a`, `arm64-v8a`, `x86`, `x86_64`).
   - Alternatively, use Gradle’s `externalNativeBuild` to invoke Cargo directly (see `cargo gradle` plugin) – this keeps the build single‑step.
3. **Declare native methods** in a Kotlin singleton/object (e.g., `PlatoLib`):
   ```kotlin
   object PlatoLib {
       init {
           System.loadLibrary("plato_core") // loads libplato_core.so
       }

       @Suppress("unused")
       @JniStatic
       external fun openDocument(path: String): Long   // returns native pointer as Long

       @Suppress("unused")
       @JniStatic
       external fun renderDocument(ctxPtr: Long, width: Int, height: Int, outBuf: ByteArray): Int

       @Suppress("unused")
       @JniStatic
       external fun inputEvent(ctxPtr: Long, type: Int, x: Int, y: Int): Unit

       @Suppress("unused")
       @JniStatic
       external fun freeContext(ctxPtr: Long): Unit
   }
   ```
   - Use `@JniStatic` from the `androidx.core:core-ktx` `jnistatic` helper or write the corresponding Java class; the signatures must match the Rust export names exactly.
4. **Permission & manifest**
   - Add `<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE"/>` (or use the newer Storage Access Framework for API 29+).
   - Request runtime permission at API 23+ via `ActivityResultContracts.RequestPermission`.
   - Enable `usesCleartextTraffic="false"` in `network_security_config.xml` if you ever load remote assets.
5. **UI layer (Jetpack Compose)**
   - **State**: hold the native context pointer (`var ctxPtr by remember { mutableStateOf(0L) }`), the bitmap (`ImageBitmap?`), and UI state (page, zoom, etc.).
   - **Lifecycle**:
     - In `LaunchedEffect(Unit)` after obtaining a file URI (via storage access picker), call `PlatoLib.openDocument(path)` and store the pointer.
     - On `onDestroy`, call `PlatoLib.freeContext(ctxPtr)`.
   - **Rendering loop**:
     - Use `remember { mutableStateOf<Bitmap?>(null) }`.
     - Launch a coroutine with `while (isActive) { delay(16); // ~60fps` that:
       1. Allocates a `ByteArray` of `width * height * 4`.
       2. Calls `PlatoLib.renderDocument(ctxPtr, w, h, buf)`.
       3. Converts the RGBA byte array to an `Bitmap` via `Bitmap.createBitmap(buf, w, h, Bitmap.Config.ARGB_8888)`.
       4. Updates the state; Compose recomposes and draws the bitmap (`Image(bitmap = ..., contentDescription = null)`).
     - Adjust `w/h` to the current composable size (use `LocalDensity` and `withResources` or `rememberUpdatedState`).
   - **Input**: wrap the composable in `pointerInput(Unit) { detectTapGestures { … } }` or `detectDragGestures` and forward events via `PlatoLib.inputEvent`.
   - **Controls**: toolbar buttons for zoom, rotation, page navigation – each calls into the Rust core to update internal state then triggers a re‑render.

---

### 5. Rendering alternatives (optional)
- **OpenGL ES texture**:
  - Allocate a GL texture in Java (`GLES20.glGenTextories`).
  - Pass a direct `ByteBuffer` (via `ByteArray` wrapped in `ByteBuffer.wrap(...).asDirectBuffer()`) to Rust, which writes RGBA directly into the buffer (no copy).
  - Then issue `glTexSubImage2D` and draw a full‑screen quad.
  - Saves one copy per frame but adds GL complexity; start with the bitmap approach for simplicity.
- **SurfaceTexture**:
  - More involved; not needed for an MVP.

---

### 6. Build & packaging
1. **Gradle configuration** (module `build.gradle.kts`):
   ```kotlin
   android {
       compileSdk = 34
       defaultConfig {
           applicationId = "org.platoreader.app"
           minSdk = 21
           targetSdk = 34
           versionCode = 1
           versionName = "1.0"
       }
       // Pack native libs from jniLibs
       sourceSets.main {
           jniLibs.srcDirs = listOf("src/main/jniLibs")
       }
   }
   ```
2. **Enable R8 shrinking** (release):
   ```kotlin
   buildTypes {
       release {
           isMinifyEnabled = true
           isShrinkResources = true
           proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
       }
   }
   ```
3. **Create a signed bundle / APK**:
   - `./gradlew bundleRelease` → upload to Play Console (or `./gradlew assembleRelease` for a local APK).
   - Test on emulator and physical devices (arm64-v8a and armeabi-v7a).

---

### 7. Testing strategy
| Level | Tool | What to verify |
|-------|------|----------------|
| Unit | JUnit + MockK | Pure Rust logic exposed via FFI (can be tested with `cargo test` on host). |
| Integration | Android instrumentation tests (`androidx.test`) | Launch app, load a sample EPUB/PDF, verify first page renders, pinch‑zoom changes bitmap size. |
| UI | Compose testing (`createAndroidComposeRule`) | Verify UI state updates (toolbar icons, page number) after simulated gestures. |
| Performance | Android Studio Profiler | CPU usage per frame (<16ms for 60 fps), memory allocations (aim < 2 MB/frame), battery drain sample. |
| Crash | Firebase Crashlytics (optional) | Capture native crashes via `Breakpad` or `google-breakpad` NDK integration. |

---

### 8. Continuously follow AGENTS.md principles (adapted)
- **Modularity**: Keep the Rust core, JNI layer, and Android UI as separate modules with clear interfaces.
- **DRY**: Extract common JNI boilerplate (pointer conversion, error handling) into Kotlin helper functions.
- **Error handling**: Propagate Rust errors via return codes; translate to Kotlin `Result` or exceptions only at the UI boundary.
- **Performance**: Mark small Rust functions with `#[inline]`; avoid allocation in the render loop (reuse `ByteArray`).
- **Imports**: Group as per Android conventions (stdlib, androidx, third‑party, project).
- **Naming**: `snake_case` for resources, `camelCase` for Kotlin, `PascalCase` for classes.
- **Constants**: Keep in `object Constants` or `top‑level const val`.
- **Resource management**: Ensure every native context is freed (`freeContext`) in `onCleared` of a ViewModel or in `onDestroy` of the Activity.
- **Testing**: Unit tests in `src/test`, instrumented in `src/androidTest`; keep them separate.
- **Dead code**: Run `./gradlew lint` and `detekt` to prune unused resources or methods.

---

### 9. Timeline (MVP)
| Phase | Duration | Main deliverables |
|-------|----------|-------------------|
| Environment & core isolation | 2 days | `plato-core-lib` with C API, cross‑compiled `.so` for at least one ABI. |
| Android project + JNI bindings | 3 days | Studio project loads library, can open a file and receive a bitmap. |
| Rendering loop + basic UI (page, zoom) | 4 days | Compose screen shows document, responds to tap/drag, toolbar controls work. |
| Input handling & lifecycle | 2 days | Proper pause/resume, context free, permission flow. |
| Polish, testing, performance tweaks | 3 days | Unit/UI tests, profiler checks, shrink enabled, APK size < 5 MB (core + UI). |
| Release preparation | 1 day | Signed AAB, Play Store internal test track upload. |
| **Total** | **≈15 days** (adjustable based on scope) |

---

### 10. References & existing work
- Plato’s `mupdf_wrapper` – demonstrates exposing C functions from Rust for MuPDF; reuse the pattern.
- Android NDK docs – *Calling native code from Java* (JNI).
- `cargo gradle` plugin – example of invoking Cargo from Gradle.
- Jetpack Compose documentation – state hoisting, lifecycle effects.
- Android Performance Tuning guide – baseline profiles, R8, memory allocation tips.

---

**End of plan.**
Save the above content as `APK_PLAN.md` in the repository root (or in `.opencode/plans/` if you prefer). You can then proceed to implement each step sequentially. Good luck building Plato for Android!

---
