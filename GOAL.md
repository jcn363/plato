# Goal

The user is working on improving the Plato codebase by addressing structural issues, completing incomplete features, and migrating to safer implementations. The overarching goal is to make the codebase compliant with AGENTS.md mandates (particularly the 1000-line file limit and safe wrapper usage) while completing partially implemented features like PDF tools UI.

## Instructions

- Create comprehensive plans for specific tasks when requested (TODO.md for Home/Reader refactoring, PDF-UI_PLAN.md for PDF tools completion, FONT-WRAPPERS.md for font module migration)
- Analyze code structure to understand what needs to be done
- Prioritize work based on violations of AGENTS.md mandates
- Provide detailed implementation approaches and acceptance criteria

## Discoveries

1. **Structural violations RESOLVED**: All files now comply with 1000-line limit (largest file: reader.rs at 776 lines)
2. **Partial implementations RESOLVED**: PDF tools UI fully implemented with interactive components
3. **Migration COMPLETED**: Font module fully migrated to safe wrappers
4. **Refactoring COMPLETED**: ReaderImpl fully modularized into 15 focused modules
5. **Test segregation COMPLETED**: Unit tests moved to sibling test files per AGENTS.md

## Accomplished

- ✅ **Complete Theme System**: Implemented Light, Dark, Sepia, Auto (light sensor), and Scheduled (time-based) modes with persistence and gesture support.
- ✅ **PDF Tools UI Completion**: surrendering all backend capabilities (Delete, Rotate, Extract, Merge, Redact) through interactive UI menus and mode transitions.
- ✅ **Cover Editor UI Completion**: Full suite of interactive controls (Rotate, Grayscale, Brightness, Contrast, Crop) wired into the document management flows.
- ✅ **Structural Refactoring**:
  - Extracted **Gesture Module** (`reader_gestures.rs`) and **GestureProcessor** trait.
  - Extracted **Rendering**, **Settings**, and **Annotation** modules from `ReaderImpl`.
  - **ReaderImpl Full Modularization**: Split 3,403-line file into 15 focused modules (all under 800 lines).
  - Migrated **13 lazy_static!** instances to `std::sync::LazyLock`.
  - Moved **unit tests** to sibling `_tests.rs` files for multiple core modules.
- ✅ **Documentation Audit**: Completed comprehensive audit (April 12, 2026) of all codebase `.md` documents, ensuring alignment with current source tree.
- ✅ **Final Verification Pass** (April 15, 2026):
  - **Build Status**: All critical compilation errors resolved - builds successfully with only warnings
  - **Code Quality**: Fixed import issues, trait implementations, type mismatches, and borrowing problems
  - **Dead Code Audit**: All `#[allow(dead_code)]` instances justified (resource management, UI architecture)
  - **File Size Compliance**: All files now under 1,000 line limit per AGENTS.md (largest: 776 lines)
  - **Documentation Updates**: Updated all relevant .md files with current verification status

## Current Status Summary

**Build**: Compiles successfully across all targets (x86_64, arm, aarch64) with only warnings
**Critical Issues**: NONE - All files now comply with AGENTS.md 1,000 line limit
**Code Quality**: All compilation errors resolved, dead code justified, documentation updated
**Next Priority**: No critical issues - codebase is AGENTS.md compliant

## Relevant files / directories

**Home View Refactoring:**

- `crates/core/src/view/home/mod.rs` (completed, now 596 lines)
- Extracted modules: `ops.rs`, `ui_toggles.rs`, `library.rs`, `fetcher.rs`, `navigation.rs`, `updates.rs`, `input.rs`
- Related view components: `book.rs`, `shelf.rs`, `address_bar.rs`, `navigation_bar.rs`, `bottom_bar.rs`, `home_utils.rs`

**ReaderImpl Refactoring:**

- `crates/core/src/view/reader/reader_impl/reader.rs` (main file, 776 lines - COMPLETED)
- Extracted modules: `reader_settings.rs` (773 lines), `reader_setters.rs` (380 lines), `reader_menus.rs` (395 lines), `reader_search.rs` (322 lines), `reader_rendering.rs` (222 lines), `reader_rendering_impl.rs` (229 lines), `reader_navigation.rs` (252 lines), `reader_gestures.rs` (234 lines), `reader_annotations.rs` (160 lines), `reader_dialogs.rs` (147 lines), `reader_ui.rs` (152 lines), `reader_core.rs` (145 lines), `reader_events.rs` (41 lines), `reader_toc.rs` (232 lines)

**PDF Tools UI:**

- `crates/core/src/view/pdf_manipulator.rs` (UI needing completion)
- `crates/core/src/document/pdf_manipulator.rs` (backend implementation)
- `crates/core/src/document/pdf.rs` (PDF handling)

**Font Module Migration:**

- `crates/core/src/font/mod.rs` (main file to refactor, 2,400 lines)
- Existing safe wrappers: `freetype.rs`, `harfbuzz.rs`
- FFI bindings: `freetype_sys.rs`, `harfbuzz_sys.rs`
- Supporting files: `freetype_error.rs`, `types.rs`, `constants.rs`, `md_title.rs`

**Reference Documents:**

- `IMPROVEMENTS.md` (status of completed and open improvements)
- `INTEGRATION_OPPORTUNITIES.md` (detailed integration opportunities and their status)
- `AGENTS.md` (mandates being addressed)

---
