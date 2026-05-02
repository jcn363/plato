# **Plato codebase – analysis for documentation update**

*Last updated: 2026-05-02*

## Progress (2026-05-02)
- [x] Architecture overview extended with AI/Thumbnail/TTS/Plugin nodes (commit 5502399)
- [x] Service Layer updated to include `ai` (commit b2edbb3)
- [x] Crate-level READMEs created for: core, ai, thumbnail, plato-android, plato-view
- [x] API_OVERVIEW.md created (commit c474fe9)
- [x] TESTING.md created
- [x] ROADMAP.md created
- [x] AI integration doc created
- [x] Thumbnail system doc created
- [x] TTS doc created
- [x] CI doc generation workflow created

---

### 1. High‑level repository layout (Cargo workspace)

| Crate | Path | Primary purpose |
|------|------|-----------------|
| **core** | `crates/core` | Central library – device abstractions, rendering pipeline, library management, metadata, settings, UI view system, thumbnail cache, TTS (desktop / Android) |
| **plato** | `crates/plato` | Main binary that wires the core library to the Kobo UI, command‑line entry point, event loop |
| **importer** | `crates/importer` | Stand‑alone document import tool (EPUB, PDF, etc.) |
| **fetcher** | `crates/fetcher` | Article‑fetcher (`curl`‑like) for remote content |
| **epub_edit** | `crates/epub_edit` | Library + CLI for editing EPUB files (cover, metadata, chapters) |
| **epub_editor** | `crates/epub_editor` | Tiny wrapper binary around `epub_edit` |
| **thumbnail** | `crates/thumbnail` | Background thumbnail generation, thread‑pool, LRU cache |
| **ai** | `crates/ai` | Local LLM embedding engine, provider abstraction (Ollama, OpenAI, Claude), simple cache |
| **error** | `crates/error` | Global `PlatoError` enum and `PlatoResult<T>` alias used throughout `core` |
| **plato‑android** | `crates/plato-android` | Android‑specific glue (JNI) for the same core functionality |
| **plato‑view** | `crates/plato-view` | UI view‑tree implementation (used by `core::view`) |
| **plato‑document** | `crates/plato-document` | Document‑type abstractions (PDF, EPUB, HTML) and helpers |
| **plato‑library** | `crates/plato-library` | Public API for library management (scan, query, maintenance) |
| **plato‑reader** (implicit) | `crates/plato` binary | The running e‑reader application |
| **rar** (dependency) | `crates/rar` | RAR extraction utilities (used by importer) |

*All crates share the same workspace `Cargo.toml` and inherit common dev‑dependencies (`anyhow`, `serde`, `rand`, etc.).*


### 2. Core architectural pillars (as reflected in source)

| Layer | Modules / Traits | Key responsibilities |
|------|------------------|----------------------|
| **Hardware‑Abstraction Layer** | `framebuffer/*`, `device`, `battery`, `frontlight`, `lightsensor` | Low‑level ops, expose `Framebuffer`, `Device`, `Battery` traits |
| **Service Layer** | `input`, `sync`, `opds`, `tts`, `thumbnail`, `ai` | Input handling, network sync, OPDS catalog, text‑to‑speech, background thumbnail generation |
| **Business‑Logic Layer** | `library`, `document`, `settings`, `metadata` | Book/library database, document parsing, user configuration, metadata extraction |
| **Application / UI Layer** | `view/*`, `gesture`, `theme`, `mobile_theme` | View‑tree, event bubbling, theming, gesture processing |
| **AI / Extension Layer** | `ai/*`, `plugin` | Embedding generation, LLM provider abstraction, plug‑in entry points |

**Traits that enable testability & extensibility**

* `Device`, `Framebuffer`, `Battery`, `LightSensor` – hardware abstractions (implemented for Kobo, Mock, Android)
* `Document`, `Page` – polymorphic handling of PDF/EPUB/HTML
* `TtsEngine` – desktop / Android implementations, fully stubbed for Kobo
* `ThumbnailGenerator` (internal) – worker pool & cache abstraction


### 3. Error‑handling strategy (source evidence)

* `crates/error/src/error.rs` defines `PlatoError` (using `thiserror`).
* `core/src/error.rs` re‑exports `into_plato_err` and `PlatoError`.
* Application‑level code uses `anyhow::Result` (`anyhow` imported in `core/lib.rs`).
* All public APIs in `core` return `PlatoResult<T>` (alias for `Result<T, PlatoError>`).
* No mixed usage of `anyhow` and `thiserror` within the same module – complies with **AGENTS.md** rule.

### 4. Documentation coverage – what exists and what is missing

| Area | Existing docs | Gaps / Recommended additions |
|------|--------------|------------------------------|
| **User‑facing docs** (installation, UI usage) | `doc/` folder with `GUIDE.md`, `MANUAL.md`, `BUILD.md`, etc. | Keep up‑to‑date with recent UI changes (AI chat toggle, TTS UI, thumbnail settings). |
| **Architecture** | `docs/architecture/OVERVIEW.md` (high‑level layers) | Extend to include **AI integration**, **Thumbnail pipeline**, **Android TTS**, **Plugin system** (planned in roadmap). |
| **Module‑level docs** | Many `mod.rs` files contain `///` top‑level comments; `cargo doc` produces API docs. | Add / improve module‑level `README.md` or `MODULE.md` files for crates that lack them (e.g. `ai`, `thumbnail`, `plato‑android`). |
| **Public API reference** | No single consolidated API reference. | Generate a **developer reference** (`docs/DEVELOPER_API.md`) that pulls key `pub use` re‑exports (e.g. `plato_core::Document`, `plato_core::Device`, `plato_core::PlatoError`). |
| **Change logs** | `CHANGES.md` exists but not auto‑generated. | Consider using `cargo-release` or `git-cliff` to keep changelog in sync with commits. |
| **Testing & Mocking** | `test_mocks.rs` well documented, but no guide. | Add a **Testing Guide** (`docs/TESTING.md`) describing how to use `test_mocks` for unit tests, how to run fast host tests, CI expectations. |
| **Build scripts** | `build.sh`, `dist.sh` partially documented. | Expand documentation on cross‑compilation steps, library directory mapping (`libs/`, `libs64/`, `libs_host/`). |
| **Roadmap / Future work** | Several archived plans (`docs/archive/*`) and an active `APPLE-PLAN.md`. | Consolidate current roadmap into a single `ROADMAP.md` that references archived plans and marks active items (AI, Plugin System, Cloud Sync). |

### 5. Suggested documentation updates (actionable checklist)

1. **Architecture overview**
   * Add a *new section* titled **“Extended Architecture”** that visualizes the added layers:
     - AI Integration (LLM provider, embedding cache)
     - Thumbnail Subsystem (worker pool, LRU cache)
     - TTS (desktop & Android)
     - Plugin / Extension hook
   * Update the mermaid diagram to include these nodes and their dependencies.

2. **Crate‑level READMEs**
   * Create a `README.md` inside each top‑level crate (`core`, `ai`, `thumbnail`, `plato-android`, `plato-view`) summarizing:
     - Purpose
     - Public API surface (`pub use` re‑exports)
     - How to enable optional features (`tts`, `android`, `iai` etc.)
   * Link those READMEs from the top‑level `docs/README.md` “Project Documentation” table.

3. **Public API cheat‑sheet**
   * Generate a concise table (maybe `docs/API_OVERVIEW.md`) listing:
     - Core traits (`Device`, `Framebuffer`, `Document`, `Page`, `TtsEngine`)
     - Errors (`PlatoError` variants)
     - Key helper functions (`estimate_from_page_count`, `reading_time::format_duration`, thumbnail utilities)
   * Provide example snippets for each trait implementation (mock vs real).

4. **Testing guide**
   * Document:
     - How to run fast host tests (`cargo test --target x86_64-unknown-linux-gnu`)
     - How to run hardware‑agnostic tests using `MockDevice`
     - Performance expectations (≤ 60 s per test)
     - Lint/format checks (`cargo fmt`, `cargo clippy -D warnings`)

5. **Feature‑specific docs**
   * **AI integration** – describe the provider abstraction (`ai::Provider`), embedding cache, UI toggle, and how to add a new LLM.
   * **Thumbnail system** – explain worker‑pool sizing logic (`optimal_worker_count`), cache eviction policy, and how to adjust via `Config`.
   * **TTS** – delineate desktop implementation vs Android stub; note the current limitation (pause/resume unimplemented on Android).

6. **Roadmap consolidation**
   * Merge active items from `APPLE-PLAN.md`, `ARCHITECTURE.md`, and archived plans into a single `ROADMAP.md`.
   * Mark each item with a status badge (✅ Done, ⚠️ Blocked, 🔜 Planned).

7. **Link generation**
   * In the top‑level `README.md`, add direct links to the newly created module READMEs and the `API_OVERVIEW.md`.
   * Ensure the “Documentation Standards” section reflects the new layout.

8. **Auto‑generated docs**
   * Add a CI step (e.g., in `.github/workflows/doc.yml`) that runs `cargo doc --no-deps` and publishes the HTML to GitHub Pages or a `docs/target` folder.
   * Reference this link from `docs/README.md`.

### 6. Quick reference – module purpose map (for the documentation)

```
core/
 ├─ geom/               – geometric primitives (Point, Rect, Vec2, Region)
 ├─ framebuffer/        – hardware‑specific display drivers (kobo1, kobo2, desktop)
 ├─ device/             – KoboDevice, MockDevice, AndroidDevice
 ├─ document/           – Document trait + PDF/EPUB implementations
 ├─ library/            – Book library (scan, query, maintenance)
 ├─ metadata/           – Saved queries, search index, reading stats
 ├─ settings/           – Config manager with validation
 ├─ theme / mobile_theme – theme handling for e‑ink / mobile UI
 ├─ thumbnail/          – worker pool & LRU cache for cover thumbnails
 ├─ tts/                – TtsEngine abstraction (desktop & Android)
 └─ view/               – UI view hierarchy, event bubbling, gesture handling
```

### 7. Next steps for the team

| Priority | Action | Owner |
|----------|--------|-------|
| **High** | Update `docs/architecture/OVERVIEW.md` with new diagram and AI/Thumbnail sections. | Docs lead |
| **High** | Add crate‑specific READMEs for `ai`, `thumbnail`, `plato-android`. | Module owners |
| **Medium** | Draft `docs/API_OVERVIEW.md` (trait list + example usage). | API maintainer |
| **Medium** | Write `docs/TESTING.md` covering mock usage and CI lint steps. | QA lead |
| **Low** | Set up CI job to publish `cargo doc` output. | DevOps |

---

**Summary**

The codebase is well‑structured into clearly‑named crates, each with a single responsibility. Core abstractions (`Device`, `Document`, `Framebuffer`) follow trait‑based design, enabling mock‑based testing. Documentation currently covers user guides and a high‑level architecture diagram, but it omits several recent subsystems (AI, thumbnail cache, TTS, plugin hooks) and lacks per‑crate READMEs and a consolidated API reference. Implementing the checklist above will bring the documentation in line with the project's modular architecture and the **AGENTS.md** standards, and will make onboarding new contributors faster and safer.

---
