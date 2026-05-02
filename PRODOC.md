# **Plato codebase – documentation update report**

*Last updated: 2026-05-02*
*Status: ✅ All documentation tasks completed*

---

## Executive Summary

This document analyzed the Plato codebase and produced a comprehensive documentation update plan. **All planned documentation tasks have been completed** as of 2026-05-02.

---

## Completed Tasks

- [x] Architecture overview extended with AI/Thumbnail/TTS/Plugin nodes (commit 5502399)
- [x] Service Layer updated to include `ai` (commit b2edbb3)
- [x] Crate-level READMEs created for: core, ai, thumbnail, plato-android, plato-view
- [x] API_OVERVIEW.md created (commit c474fe9) - Developer reference with traits, errors, helpers
- [x] TESTING.md created - Testing guide with mock usage and CI lint steps
- [x] ROADMAP.md created - Consolidated roadmap of active and planned items
- [x] AI integration doc created - Provider abstraction, embeddings, cache details
- [x] Thumbnail system doc created - Worker pool, LRU cache, sizing logic
- [x] TTS doc created - Desktop & Android implementations, limitations
- [x] CI doc generation workflow created (.github/workflows/doc.yml)
- [x] docs/README.md updated with links to all new documentation

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

*Note: Additional wrapper crates exist for modularization (battery, color, config, consts, device, doc, font, frontlight, geom, gesture, input, metadata, network, opds, reading_time, rtc, search, settings, sync, theme, tts, ui, utils, validation). These are internal to the workspace structure.*

*All crates share the same workspace `Cargo.toml` and inherit common dev‑dependencies (`anyhow`, `serde`, `rand`, etc.).*

---

### 2. Core architectural pillars (as reflected in source)

| Layer | Modules / Traits | Key responsibilities |
|------|------------------|----------------------|
| **Hardware‑Abstraction Layer** | `framebuffer/*`, `device`, `battery`, `frontlight`, `lightsensor` | Low‑level ops, expose `Framebuffer`, `Device`, `Battery` traits |
| **Service Layer** | `input`, `sync`, `opds`, `tts`, `thumbnail`, `ai` | Input handling, network sync, OPDS catalog, text‑to‑speech, background thumbnail generation, AI integration |
| **Business‑Logic Layer** | `library`, `document`, `settings`, `metadata` | Book/library database, document parsing, user configuration, metadata extraction |
| **Application / UI Layer** | `view/*`, `gesture`, `theme`, `mobile_theme` | View‑tree, event bubbling, theming, gesture processing |
| **AI / Extension Layer** | `ai/*`, `plugin` | Embedding generation, LLM provider abstraction, plug‑in entry points |

**Traits that enable testability & extensibility**

* `Device`, `Framebuffer`, `Battery`, `LightSensor` – hardware abstractions (implemented for Kobo, Mock, Android)
* `Document`, `Page` – polymorphic handling of PDF/EPUB/HTML
* `TtsEngine` – desktop / Android implementations, fully stubbed for Kobo
* `ThumbnailGenerator` (internal) – worker pool & cache abstraction

---

### 3. Error‑handling strategy (source evidence)

* `crates/error/src/error.rs` defines `PlatoError` (using `thiserror`).
* `core/src/error.rs` re‑exports `into_plato_err` and `PlatoError`.
* Application‑level code uses `anyhow::Result` (`anyhow` imported in `core/lib.rs`).
* All public APIs in `core` return `PlatoResult<T>` (alias for `Result<T, PlatoError>`).
* No mixed usage of `anyhow` and `thiserror` within the same module – complies with **AGENTS.md** rule.

---

### 4. Documentation coverage – CURRENT STATE ✅

| Area | Status | Documentation |
|------|--------|---------------|
| **User‑facing docs** (installation, UI usage) | ✅ Complete | `doc/` folder with `GUIDE.md`, `MANUAL.md`, `BUILD.md`, etc. |
| **Architecture** | ✅ Complete | `docs/architecture/OVERVIEW.md` extended with AI/Thumbnail/TTS/Plugin nodes and mermaid diagram |
| **Module‑level docs** | ✅ Complete | Crate-level READMEs created for core, ai, thumbnail, plato-android, plato-view |
| **Public API reference** | ✅ Complete | `docs/API_OVERVIEW.md` provides trait list, error variants, helper functions with examples |
| **Change logs** | ⚠️ Manual | `CHANGES.md` exists but not auto‑generated (consider `cargo-release` or `git-cliff`) |
| **Testing & Mocking** | ✅ Complete | `docs/TESTING.md` covers mock usage, performance expectations, CI lint steps |
| **Build scripts** | 🔜 Partial | `build.sh`, `dist.sh` partially documented; expand cross‑compilation steps |
| **Roadmap / Future work** | ✅ Complete | `docs/ROADMAP.md` consolidates active plans with status badges |

---

### 5. Feature‑specific documentation – COMPLETED ✅

All feature‑specific deep‑dive documents have been created:

1. **AI Integration** → `docs/AI_INTEGRATION.md`
   - Provider abstraction (`ai::Provider`), embedding cache, UI toggle
   - How to add a new LLM provider

2. **Thumbnail System** → `docs/THUMBNAIL_SYSTEM.md`
   - Worker‑pool sizing logic (`optimal_worker_count`), cache eviction policy
   - Configuration via `Settings.thumbnail`

3. **Text‑to‑Speech** → `docs/TTS.md`
   - Desktop vs Android implementation differences
   - Current limitations (pause/resume unimplemented on Android)
   - CI workflow for TTS builds

---

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

---

### 7. Next steps for the team

| Priority | Action | Status |
|----------|--------|--------|
| **High** | ~~Update `docs/architecture/OVERVIEW.md` with new diagram and AI/Thumbnail sections.~~ | ✅ Done |
| **High** | ~~Add crate‑specific READMEs for `ai`, `thumbnail`, `plato-android`.~~ | ✅ Done |
| **Medium** | ~~Draft `docs/API_OVERVIEW.md` (trait list + example usage).~~ | ✅ Done |
| **Medium** | ~~Write `docs/TESTING.md` covering mock usage and CI lint steps.~~ | ✅ Done |
| **Low** | ~~Set up CI job to publish `cargo doc` output.~~ | ✅ Done |
| **Future** | Expand build‑script documentation (cross‑compilation steps, library directory mapping) | 🔜 Planned |
| **Future** | Automate changelog generation (`cargo-release` or `git-cliff`) | 🔜 Planned |
| **Future** | Add feature‑specific docs for: Plugin system, Cloud sync, Annotation system | 🔜 Planned |

---

## Summary

The Plato codebase is well‑structured into clearly‑named crates, each with a single responsibility. Core abstractions (`Device`, `Document`, `Framebuffer`) follow trait‑based design, enabling mock‑based testing.

**All documentation tasks outlined in the original analysis have been completed:**
- Architecture overview extended with modern subsystems (AI, Thumbnail, TTS, Plugin)
- Crate‑level READMEs created for all major crates
- Developer reference (API_OVERVIEW.md) published
- Testing guide (TESTING.md) written
- Roadmap (ROADMAP.md) consolidated
- Feature‑specific deep‑dives created (AI, Thumbnail, TTS)
- CI workflow for auto‑generating docs deployed

The documentation is now comprehensive, consistent, and aligned with the project's modular architecture and the **AGENTS.md** standards. New contributors can onboard faster and safer.

---

## Appendices: Original Analysis (for historical reference)

<details>
<summary>Click to expand original analysis sections 1-3</summary>

### Original Section 1: High‑level repository layout
(See updated table above)

### Original Section 2: Core architectural pillars
(See updated table above)

### Original Section 3: Error‑handling strategy
(See updated content above)

</details>
