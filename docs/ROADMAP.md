# Plato Roadmap

This document lists the major work items for the Plato project, their current status, and references to detailed plans where applicable.

## Status badge legend

- ✅ **Done** – implemented and shipped
- 🔄 **In progress** – active development
- 🔜 **Planned** – not started yet
- ⚠️ **Blocked** – cannot proceed due to external dependency

## Core architecture & infrastructure

| Item | Status | Notes |
|------|--------|-------|
| **AI integration** (embeddings, LLM providers) | ✅ | `plato‑ai` crate, provider abstraction (Ollama, OpenAI, Claude). |
| **Thumbnail subsystem** (worker pool, LRU cache) | ✅ | `plato‑thumbnail` crate, optimal sizing for Kobo/Android. |
| **TTS** (text‑to‑speech) | 🔄 | Desktop fully working; Android stubbed (jni API breaks compile). |
| **Theme system** (dark mode, mobile theme) | ✅ | `mobile_theme` module, `Theme` trait. |
| **Extended architecture documentation** | ✅ | `docs/architecture/OVERVIEW.md` now includes AI, Thumbnail, TTS, Plugin nodes. |
| **Crate‑level READMEs** | ✅ | Created for core, ai, thumbnail, plato‑android, plato‑view. |
| **API overview** (`docs/API_OVERVIEW.md`) | ✅ | Traits, errors, helpers summarised. |
| **Testing guide** (`docs/TESTING.md`) | ✅ | Mock usage, performance rules, CI expectations. |

## Platform support

| Item | Status | Notes |
|------|--------|-------|
| **Kobo e‑readers** (original target) | ✅ | All features implemented, default build target. |
| **Android** (plato‑android) | 🔄 | JNI glue present, APK build blocked on sccache/native toolchain. |
| **iOS / iPhone / iPad** | 🔄 | Detailed plan in `docs/APPLE-PLAN.md` (5 phases). |
| **Desktop (Linux/macOS)** | ✅ | Host build works, AppImage created. |

## Feature backlog

| Item | Status | Notes |
|------|--------|-------|
| **Plugin system** (extensible document formats) | 🔜 | Design discussed in architecture docs. |
| **Cloud sync** (background sync) | 🔜 | `sync` module exists, full architecture not yet spec‑ed. |
| **Annotation system** (enhanced storage & rendering) | 🔜 | Basic annotations done, enhancement planned. |
| **OPDS catalog** | ✅ | Fully integrated, `opds` module. |
| **Document manipulation** (PDF page ops, annotations) | ✅ | `plato‑document` crate, `pdf_manipulator`. |
| **EPUB editing** | ✅ | `epub_edit` & `epub_editor` crates. |
| **Search & metadata** (saved queries, search index) | ✅ | `metadata` module, `SearchIndex`. |
| **Reading stats & time estimation** | ✅ | `reading_time` module, `ReadingSpeed`. |

## Feature Opportunities (Priority Order)

### Tier 1 – High Impact, Strategic Alignment
| Feature | Status | Notes |
|---------|--------|-------|
| **Enhanced Accessibility Suite** | ✅ | Fonts, bionic reading, auto-pace, focus mode (May 2026) |
| **Pocket/Instapaper Integration** | ✅ | Full API clients with OAuth sync (May 2026) |
| **Advanced Typography & Reading Experience** | ✅ | Per-book CSS, chunked reading, spacing controls (May 2026) |
| **Enhanced Library Management & Discovery** | ✅ | Smart collections, duplicate detection (May 2026) |

### Tier 2 – Medium Impact, Good ROI
| Feature | Status | Notes |
|---------|--------|-------|
| **Audiobook Support** | ✅ | AudioSettings, speed/sleep timer/bluetooth (May 2026) |
| **Handwriting & Note‑taking Enhancement** | ✅ | Notebook settings, ExportFormat enum (May 2026) |
| **Cross‑Device Sync 2.0** | ✅ | WebDAV, Dropbox, Google Drive, conflict resolution (May 2026) |
| **Comic/Manga Optimization** | ✅ | ComicSettings struct, panel/RTL/color options (May 2026) |

### Tier 3 – Niche but Differentiating
| Feature | Status | Notes |
|---------|--------|-------|
| **Academic/Research Mode** | 🔜 | Citations, split‑screen, LaTeX |
| **Social Reading Features** | ✅ | Readwise/Obsidian export, quote cards (May 2026) |

## Documentation tasks

| Item | Status | Notes |
|------|--------|-------|
| **Feature‑specific deep dives** (AI, Thumbnail, TTS) | 🔜 | `docs/AI_INTEGRATION.md`, `docs/THUMBNAIL_SYSTEM.md`, `docs/TTS.md` not yet written. |
| **Build‑script documentation** (cross‑compile, library dirs) | 🔜 | Expand `build.sh`/`dist.sh` docs. |
| **Changelog automation** (cargo‑release, git‑cliff) | 🔜 | `CHANGES.md` exists but manual. |
| **CI doc generation** (cargo doc → GitHub Pages) | 🔜 | `.github/workflows/doc.yml` not yet created. |

## References

- [APPLE‑PLAN.md](APPLE-PLAN.md) – iPhone/iPad support plan (5 phases).
- [PLAN.md](PLAN.md) – Original overall project plan.
- [PLAN2.md](PLAN2.md) – Updated project plan.
- [archive/](archive/) – Completed or superseded plans.

*This roadmap is a living document; update it as items are completed or new priorities emerge.*
