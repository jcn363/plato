# Android App Implementation Plan (APK_PLAN.md)

## Overview
This document provides a detailed, step‑by‑step plan to build a modern Android application from scratch, following current best practices and the Android App Architecture guidelines. The plan assumes the use of Kotlin, Jetpack Compose, and a unidirectional data flow (MVVM) architecture, but also notes alternatives for XML‑based UI and other architectures.

## 1. Prerequisites & Environment Setup
1. **Install Android Studio** (latest stable version) – includes SDK, emulator, and build tools.
2. **Install JDK 11** (or latest LTS) – Android Studio bundles a compatible JDK, but ensure `JAVA_HOME` is set if using command line.
3. **Configure Git** – initialize a repository for the project.
4. **(Optional) Install Docker** – for consistent CI/CD environments.
5. **Enable USB debugging** on a test device or set up an Android Virtual Device (AVD) in the AVD Manager.

## 2. Project Initialization
1. **Create a new project** in Android Studio:
   - Choose “Empty Compose Activity” (to start with Jetpack Compose) or “Empty Activity” (for XML UI).
   - Set Language: Kotlin.
   - Minimum SDK: API 21 (Android 5.0) – adjust based on target audience.
   - Package name: e.g., `com.example.myapp`.
   - Save location: choose a directory (e.g., `~/projects/MyApp`).
2. **Verify the default build works** – run on emulator/device to see the “Hello World” screen.

## 3. Project Structure & Modularization
Follow a scalable, modular structure (feature‑based or layer‑based). Example (feature‑based):

```
app/
 └── src/
     ├── main/
     │   ├── java/com/example/myapp/
     │   │   ├── di/               // Dependency injection (Hilt modules)
     │   │   ├── data/             // Data layer (repositories, sources)
     │   │   │   ├── local/        // Room database, DAOs
     │   │   │   └── remote/       // Network (Retrofit, Moshi)
     │   │   ├── domain/           // Use cases, business logic
     │   │   │   ├── model/        // Domain models
     │   │   │   └── repository/   // Repository interfaces
     │   │   ├── presentation/     // UI layer (ViewModels, Screens, Composables)
     │   │   │   ├── navigation/   // Navigation graph, routes
     │   │   │   ├── theme/        // MaterialTheme, colors, typography
     │   │   │   ├── ui/           // Screens (folders per feature)
     │   │   │   │   ├── home/
     │   │   │   │   │   ├── HomeScreen.kt
     │   │   │   │   │   └── HomeViewModel.kt
     │   │   │   │   └── detail/
     │   │   │   │       ├── DetailScreen.kt
     │   │   │   │       └── DetailViewModel.kt
     │   │   │   └── widgets/      // Reusable composables
     │   │   └── MyApp.kt          // Application class (if needed)
     │   └── res/                  // Resources (strings, images, etc.)
     │       ├── values/
     │       │   ├── strings.xml
     │       │   ├── colors.xml
     │       │   └── themes.xml
     │       ├── drawable/
     │       └── layout/           // (if using XML for specific screens)
     └── test/                     // Unit tests
     └── androidTest/              // Instrumented tests
```

Alternative: **layer‑based** (data, domain, presentation) with feature sub‑packages inside each layer.

Apply **AGENTS.md modularity rules**:
- Each module (package) has a single clear responsibility.
- Keep files under ~200–300 lines; functions under ~30 lines.
- Use `private`/`internal` visibility where possible; expose only needed public APIs.
- Avoid circular dependencies; depend on abstractions (interfaces) not concrete implementations.

## 4. Architecture Selection
### Recommended: MVI / MVVM with Jetpack Compose (Unidirectional Data Flow)
- **ViewModel** (from `androidx.lifecycle:lifecycle-viewmodel-compose`) holds UI state as immutable data classes.
- UI (Composables) observes `StateFlow` or `LiveData` and emits events via `onClick` lambdas.
- Use `remember { viewModel() }` to obtain ViewModel scoped to the composable’s lifecycle.
- For complex business logic, extract **Use Cases** (interactors) that depend on repositories.

### Alternatives
- **Classic MVVM** with XML layouts and Data Binding.
- **MVP** (less common now).
- **Clean Architecture** – same layers but with explicit boundaries (entities, use cases, interface adapters, frameworks & drivers).

Choose MVVM+Compose for brevity, strong tooling, and alignment with Android’s recommendations.

## 5. Dependency Management (Gradle)
Use the Gradle Version Catalog (`libs.versions.toml`) for centralized version control.

Example `libs.versions.toml`:
```toml
[versions]
kotlin = "1.9.0"
agp = "8.5.0"
composeBom = "2024.09.00"
coreKtx = "1.13.0"
lifecycle = "2.8.0"
activity = "1.9.0"
material = "1.12.0"
navigation = "2.8.0"
room = "2.6.1"
kotlinxCoroutines = "1.8.0"
retrofit = "2.11.0"
moshi = "1.15.0"
hilt = "2.5.1"
coil = "2.6.0"
timber = "5.0.1"
junit = "4.13.2"
truth = "1.1.5"
espresso = "3.6.1"

[libraries]
kotlin-stdlib = { module = "org.jetbrains.kotlin:kotlin-stdlib", version.ref = "kotlin" }
androidx-core-ktx = { module = "androidx.core:core-ktx", version.ref = "coreKtx" }
androidx-lifecycle-runtime-ktx = { module = "androidx.lifecycle:lifecycle-runtime-ktx", version.ref = "lifecycle" }
androidx-lifecycle-viewmodel-compose = { module = "androidx.lifecycle:lifecycle-viewmodel-compose", version.ref = "lifecycle" }
androidx-activity-compose = { module = "androidx.activity:activity-compose", version.ref = "activity" }
androidx-material3 = { module = "androidx.material3:material3", version.ref = "material" }
androidx-navigation-compose = { module = "androidx.navigation:navigation-compose", version.ref = "navigation" }
androidx-room-runtime = { module = "androidx.room:room-runtime", version.ref = "room" }
androidx-room-ktx = { module = "androidx.room:room-ktx", version.ref = "room" }
kotlinx-coroutines-core = { module = "org.jetbrains.kotlinx:kotlinx-coroutines-core", version.ref = "kotlinxCoroutines" }
kotlinx-coroutines-android = { module = "org.jetbrains.kotlinx:kotlinx-coroutines-android", version.ref = "kotlinxCoroutines" }
com-squareup-retrofit2 = { module = "com.squareup.retrofit2:retrofit", version.ref = "retrofit" }
com-squareup-retrofit2-converter-moshi = { module = "com.squareup.retrofit2:converter-moshi", version.ref = "retrofit" }
com-squareup-moshi = { module = "com.squareup.moshi:moshi", version.ref = "moshi" }
com-squareup-moshi-kotlin = { module = "com.squareup.moshi:moshi-kotlin", version.ref = "moshi" }
com.google.dagger-hilt-android = { module = "com.google.dagger:hilt-android", version.ref = "hilt" }
com.google.dagger-hilt-android-gradle = { module = "com.google.dagger:hilt-android-gradle-plugin", version.ref = "hilt" }
io-coil-coil-compose = { module = "io.coil-kt:coil-compose", version.ref = "coil" }
com-jakewharton-timber = { module = "com.jakewharton.timber:timber", version.ref = "timber" }
junit = { module = "junit:junit", version.ref = "junit" }
androidx-test-core = { module = "androidx.test:core", version.ref = "androidxTest" }
androidx-test-espresso = { module = "androidx.test.espresso:espresso-core", version.ref = "espresso" }
```

In `settings.gradle` enable version catalogs:
```groovy
dependencyResolutionManagement {
    versionCatalogs {
        create("libs") {
            from(files("gradle/libs.versions.toml"))
        }
    }
}
```

In `build.gradle (module)`:
```groovy
plugins {
    id 'com.android.application'
    id 'org.jetbrains.kotlin.android'
    id 'kotlin-kapt'
    id 'com.google.dagger.hilt.android'
}

android {
    namespace 'com.example.myapp'
    compileSdk 35

    defaultConfig {
        applicationId "com.example.myapp"
        minSdk 21
        targetSdk 35
        versionCode 1
        versionName "1.0"
        testInstrumentationRunner "androidx.test.runner.AndroidJUnitRunner"
        vectorDrawables {
            useSupportLibrary = true
        }
    }

    buildFeatures {
        compose true
    }

    composeOptions {
        kotlinCompilerExtensionVersion = "1.6.0"
    }

    packagingOptions {
        resources {
            excludes += '/META-INF/{AL2.0,LGPL2.1}'
        }
    }
}

dependencies {
    implementation libs.androidx.core.ktx
    implementation libs.androidx.lifecycle.runtime.ktx
    implementation libs.androidx.lifecycle.viewmodel.compose
    implementation libs.androidx.activity.compose
    implementation libs.androidx.material3
    implementation libs.androidx.navigation.compose
    implementation libs.androidx.room.runtime
    implementation libs.androidx.room.ktx
    implementation libs.org.jetbrains.kotlinx:kotlinx-coroutines-core
    implementation libs.org.jetbrains.kotlinx:kotlinx-coroutines-android
    implementation libs.com.squareup.retrofit2:retrofit
    implementation libs.com.squareup.retrofit2:converter-moshi
    implementation libs.com.squareup.moshi:moshi
    implementation libs.com.squareup.moshi:moshi-kotlin
    implementation libs.com.google.dagger:hilt-android
    kapt libs.com.google.dagger:hilt-compiler
    implementation libs.io.coil-kt:coil-compose
    implementation libs.com.jakewharton.timber:timber

    // Testing
    testImplementation libs.junit
    testImplementation libs.androidx.test.core
    androidTestImplementation libs.junit
    androidTestImplementation libs.androidx.test.espresso
}
```

Apply **AGENTS.md dependency rules**:
- Regularly audit with `./gradlew :app:dependencies` and `./gradlew :app:dependencyUpdates`.
- Use `cargo-audit` equivalent? Not applicable; rely on Gradle’s built‑in vulnerability checking (via Play Console or OWASP‑Dependency‑Check plugin if desired).

## 6. Dependency Injection (DI)
Use **Hilt** (recommended) for field injection in Android classes (Activity, Fragment, ViewModel) and constructor injection for plain Kotlin classes.

Steps:
1. Add `@HiltAndroidApp` to the `MyApp` class (subclass of `Application`).
2. Inject ViewModels with `@HiltViewModel` and `@Inject` constructor.
3. Provide repositories and data sources via `@Module` and `@InstallIn(SingletonComponent::class)`.
4. For Jetpack Compose, use `hiltViewModel()` delegate:
   ```kotlin
   val viewModel: HomeViewModel = hiltViewModel()
   ```

Keep DI graph lean; avoid unnecessary scopes.

## 7. Data Layer Implementation
### 7.1 Remote Data (Network)
- Use **Retrofit** + **Moshi** (or Gson) for type‑safe HTTP clients.
- Define API endpoints with Kotlin interfaces and suspend functions.
- Add interceptors for logging (HttpLoggingInterceptor), auth headers, error handling.
- Configure base URL, timeouts, retry policy via `OkHttpClient.Builder`.
- Example:
  ```kotlin
  interface ApiService {
      @GET("posts/{id}")
      suspend fun getPost(@Path("id") id: Int): Response<PostDto>
  }
  ```

### 7.2 Local Data (Persistence)
- Use **Room** for SQLite storage.
- Define `@Entity` data classes, `@Dao` interfaces with suspend functions for async queries.
- Create a `@Database` abstract class extending `RoomDatabase`.
- Provide a singleton instance via Hilt:
  ```kotlin
  @Singleton
  @Provides
  fun provideDatabase(@ApplicationContext ctx: Context): AppDatabase =
      Room.databaseBuilder(ctx, AppDatabase::class.java, "app.db")
          .fallbackToDestructiveMigration()
          .build()
  ```

### 7.3 Repository Pattern
- Create a repository interface (domain layer) that abstracts data sources.
- Implementations handle merging remote and local data, caching, error transformation.
- Expose data as `Flow` or `StateFlow` from ViewModel to UI.

### 7.4 Error Handling
- Define a sealed class `Result<T>` (or use Kotlin’s `Result`) for network/repository calls.
- Use `try/catch` and map exceptions to domain‑specific errors.
- In ViewModel, expose a `State` that includes `Error` or `Message` for UI to display via Snackbar or dialog.

## 8. Domain Layer & Use Cases
- Keep business logic pure (no Android dependencies).
- Define use case classes that take repositories via constructor and expose a single `invoke()` suspend function.
- Example:
  ```kotlin
  class GetPostsUseCase @Inject constructor(
      private val repository: PostRepository
  ) {
      suspend operator fun invoke(): Result<List<Post>> = repository.getPosts()
  }
  ```
- ViewModel calls use cases and transforms results into UI state.

## 9. Presentation Layer (UI)
### 9.1 Jetpack Compose Basics
- Set `setContent { MyAppTheme { MyAppNavigator() } }` in `MainActivity.kt`.
- Define a `MaterialTheme` (or `Material3` theme) in `theme/` folder with color, typography, shapes.
- Use `remember { mutableStateOf(...) }` for local UI state; prefer hoisting state to ViewModel.

### 9.2 Navigation
- Use **Navigation Compose** (`androidx.navigation:navigation-compose`).
- Define a sealed class `Screen` with `route` and optional arguments.
- Build `NavHost` with a `NavController`:
  ```kotlin
  val navController = rememberNavController()
  NavHost(navController, startDestination = Screen.Home.route) {
      composable(Screen.Home.route) { HomeScreen(navController) }
      composable(Screen.Detail.route) { DetailScreen(navController) }
  }
  ```
- Pass ViewModels via `hiltViewModel()` using `navController.getBackStackEntry(destination)` scope if needed.

### 9.3 State Management
- UI state should be an immutable data class:
  ```kotlin
  data class HomeUiState(
      val posts: List<Post> = emptyList(),
      val isLoading: Boolean = false,
      val error: String? = null
  )
  ```
- ViewModel exposes `StateFlow<HomeUiState>`; UI collects with `collectAsState()`.
- Events (user actions) are handled via lambda callbacks that trigger ViewModel functions.

### 9.4 Reusable Components
- Extract repeated composables (buttons, cards, loaders, error views) into `ui/widgets/`.
- Follow Material guidelines for accessibility (content descriptions, touch target size ≥48dp).
- Use `Modifier` factory functions for consistent styling (e.g., `Modifier.fillMaxWidth().padding(16.dp)`).

### 9.5 Theming & Dark Mode
- Define light and dark color schemes in `Theme.kt` using `lightColorScheme` and `darkColorScheme`.
- Use `SystemUiController` to adjust status bar icons if needed.
- Allow user to override system setting via a preference (store in DataStore or SharedPreferences).

## 10. Testing Strategy
### 10.1 Unit Tests
- Test pure Kotlin classes (use cases, repositories, data mappers) with **JUnit** and **MockK** or **Mockito**.
- Place under `src/test/java/...`.
- Aim for high coverage on domain and data layers.

### 10.2 Instrumented Tests
- Test UI interactions with **Espresso** + **Compose Test**.
- Use `createAndroidComposeRule<MyActivity>()`.
- Verify state changes, navigation, error display.
- Place under `src/androidTest/java/...`.

### 10.3 Test Rules & Fakes
- Implement fake repositories (in-memory) for deterministic UI tests.
- Use `Dispatchers.setMain` from `kotlinx-coroutines-test` to control coroutine execution.

### 10.4 Continuous Integration
- Set up GitHub Actions (or GitLab CI) to run:
  - `./gradlew test` (unit tests)
  - `./gradlew connectedAndroidTest` (instrumented tests on emulator)
  - `./gradlew lint`
  - `./gradlew :app:bundleRelease` (produce AAB)
- Cache Gradle and SDK between jobs to speed up builds.

## 11. Build Optimization & Performance
### 11.1 Enable R8 Shrinking
- In `buildTypes.release`:
  ```groovy
  minifyEnabled true
  shrinkResources true
  proguardFiles getDefaultProguardFile('proguard-android-optimize.txt'), 'proguard-rules.pro'
  ```

### 11.2 Use Baseline Profiles
- Generate via `androidx.baselineprofile` to improve startup time.

### 11.3 Enable Build Cache
- In `gradle.properties`: `org.gradle.caching=true`

### 11.4 Configure Kotlin Compiler
- Use `kotlinOptions { jvmTarget = "17"; freeCompilerArgs += ["-Xopt-in=kotlin.RequiresOptIn"] }`

### 11.5 Profile on Device
- Use Android Studio Profiler to monitor CPU, memory, network, and battery.
- Identify jank (frames >16ms) with `androidx.tracing` or `Debug` overlays.
- Optimize hot paths (e.g., expensive composables, image decoding) with `remember`, `derivedStateOf`, `LaunchedEffect`, etc.

### 11.6 Memory & Allocation
- Avoid allocating objects in recomposition (e.g., `MutableList` inside `@Composable` without `remember`).
- Use `State` hoisting and `key` parameters to prevent unnecessary recompositions.
- For large lists, use `LazyColumn`/`LazyRow` with proper `keyExtractor`.

## 12. Accessibility & Internationalization
### 12.1 Accessibility
- Add `contentDescription` to icons and non‑textual UI.
- Test with TalkBack; ensure touch targets ≥48dp.
- Use `semantics { }` modifier for custom actions.
- Follow WCAG 2.1 AA contrast ratios.

### 12.2 Internationalization (i18n)
- Store all strings in `res/values/strings.xml`.
- Use plurals and formatted strings where needed.
- Support right‑to‑left (RTL) layouts automatically by using `start`/`end` instead of `left`/`right`.
- Add `res/values-<lang>/strings.xml` for additional languages.

## 13. Privacy, Security, and Permissions
### 13.1 Data Privacy
- Store sensitive data (tokens, passwords) in **EncryptedSharedPreferences** or **Android Keystore** via Jetpack Security.
- Prefer short‑lived access tokens with refresh tokens.
- Clear sensitive data from memory when no longer needed (`Arrays.fill`).

### 13.2 Network Security
- Use HTTPS only; enforce with `network_security_config.xml` (cleartextTrafficPermitted="false").
- Implement certificate pinning if required (though often overkill for public APIs).

### 13.3 Permissions
- Declare only necessary permissions in `AndroidManifest.xml`.
- Request runtime permissions at point of use with `rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission())`.
- Explain why the permission is needed via a rationale dialog.

## 14. App Lifecycle & Background Work
### 14.1 Lifecycle-Aware Components
- Use `viewModelScope` for coroutines tied to ViewModel lifecycle.
- Observe lifecycle events with `LifecycleEventObserver` or `repeatOnLifecycle`.

### 14.2 Background Tasks
- For deferrable work, use **WorkManager** (with constraints like charging, network).
- For immediate work after UI interaction, use `CoroutineScope(Dispatchers.IO)` or `viewModelScope`.
- Avoid long‑running services; prefer foreground service with notification if needed (e.g., music playback).

### 14.3 Configuration Changes
- ViewModel survives configuration changes automatically.
- For UI‑only state, use `rememberSaveable` or `ViewModel`’s `SavedStateHandle`.

## 15. Publishing Preparation
### 15.1 Versioning
- Follow semantic versioning in `versionCode` (integer) and `versionName` (string).
- Increment `versionCode` for every upload to Play Store.

### 15.2 App Signing
- Generate an upload key via Android Studio → Build → Generate Signed Bundle/APK.
- Keep the keystore secure; backup the upload key.
- Enable Play App Signing (Google manages the signing key).

### 15.3 Store Listing
- Prepare high‑resolution icon (512x512), feature graphic, screenshots, and promo video.
- Write compelling short and full descriptions.
- Define app category, content rating, and contact details.
- Set up pricing (free/paid) and distribution countries.

### 15.4 Pre‑Launch Report
- Use internal testing track to upload an AAB and run automated pre‑launch checks (crashes, ANRs, performance).

### 15.5 Release Stages
- Start with **internal testing**, then **closed testing**, **open testing**, and finally **production**.
- Monitor crash reports via Play Console or Firebase Crashlytics.

## 16. Maintenance & Updates
### 16.1 Dependency Updates
- Schedule monthly checks with `./gradlew :app:dependencyUpdates`.
- Update libraries promptly for security fixes.

### 16.2 Feature Flags
- Consider using a remote config (Firebase Remote Config) to toggle features without releasing a new version.
- Keep flags short‑lived; remove after validation.

### 16.3 Crash Analytics
- Integrate **Firebase Crashlytics** (or equivalent) to capture non‑fatal errors.
- Add custom keys and logs for context.

### 16.4 User Feedback
- Provide a way to send feedback (email, in‑app form, or third‑party like Instabug).
- Respond to reviews on Play Store.

## 17. Compliance with AGENTS.md (Android‑Specific Adaptations)
While AGENTS.md is Rust‑focused, the underlying principles apply:
- **Modular Design**: Packages/files each have single responsibility; functions short.
- **DRY**: Extract repeated composables, utilities, and extension functions.
- **Error Handling**: Use Kotlin’s `Result` or sealed classes; avoid `throw` in production; log with Timber.
- **Performance**: Apply `inline` where appropriate (e.g., small extension functions); benchmark critical paths.
- **Imports**: Group: Kotlin stdlib, AndroidX, third‑party, project‑local.
- **Naming**: Use `snake_case` for resources, `camelCase` for Kotlin identifiers; `PascalCase` for classes.
- **Constants**: Store in `object Constants` or `const val` in companion objects; avoid magic numbers.
- **Resource Management**: Close streams, release MediaPlayer, cancel coroutines in `onCleared()` of ViewModel.
- **Configuration**: Validate values from DataStore or SharedPreferences; use defaults.
- **Test Segregation**: Unit tests in `src/test`; instrumented in `src/androidTest`; never mix production and test source sets.
- **Dead Code**: Run `./gradlew lint` and `detekt` to spot unused resources, methods, dependencies.

## 18. Estimated Timeline (Feature‑Based MVP)
| Phase | Duration | Description |
|-------|----------|-------------|
| Setup & Architecture | 2 days | Environment, project skeleton, navigation, DI, theming |
| Core Features (e.g., auth, listing) | 5 days | Repositories, ViewModels, UI screens |
| Polish & Testing | 3 days | Unit/instrumented tests, accessibility, performance tweaks |
| Release Prep | 2 days | Signing, store listing, internal testing track |
| **Total** | **~12 days** (adjustable based on scope) |

## 19. References
- Android Developer Guide: https://developer.android.com/guide
- Jetpack Compose Pathway: https://developer.android.com/courses/pathways/compose
- Official Architecture Blueprint: https://developer.android.com/architecture
- Hilt Documentation: https://developer.android.com/training/dependency-injection/hilt-android
- Coroutines Guide: https://developer.android.com/kotlin/coroutines
- Material Design 3: https://m3.material.io/
- Play Console Publishing: https://support.google.com/googleplay/android-developer/answer/9842756

--- 
*End of plan. Save this file as `APK_PLAN.md` in the project root (or `.opencode/plans/` as preferred).*