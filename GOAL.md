## Goal

The user is working on improving the Plato codebase by addressing structural issues, completing incomplete features, and migrating to safer implementations. The overarching goal is to make the codebase compliant with AGENTS.md mandates (particularly the 1000-line file limit and safe wrapper usage) while completing partially implemented features like PDF tools UI.

## Instructions

- Create comprehensive plans for specific tasks when requested (TODO.md for Home/Reader refactoring, PDF-UI_PLAN.md for PDF tools completion, FONT-WRAPPERS.md for font module migration)
- Analyze code structure to understand what needs to be done
- Prioritize work based on violations of AGENTS.md mandates
- Provide detailed implementation approaches and acceptance criteria

## Discoveries

1. **Structural violations identified**: Multiple files exceed 1000-line limit (Home view: 2,769 lines, ReaderImpl: 3,403 lines, font module: 2,400 lines)
2. **Partial implementations**: PDF tools UI has backend capabilities but missing UI components (interactive redaction regions, file selection for merge, full user input integration)
3. **Migration opportunities**: Font module already has safe wrapper implementations (freetype.rs, harfbuzz.rs) but isn't using them
4. **In-progress refactoring**: Reader refactoring was started but stalled with stub methods and partial module migration
5. **Test segregation**: Unit tests need to be moved to sibling test files per AGENTS.md

## Accomplished

- ✅ **Complete Theme System**: Implemented Light, Dark, Sepia, Auto (light sensor), and Scheduled (time-based) modes with persistence and gesture support.
- ✅ **PDF Tools UI Completion**: surrendering all backend capabilities (Delete, Rotate, Extract, Merge, Redact) through interactive UI menus and mode transitions.
- ✅ **Cover Editor UI Completion**: Full suite of interactive controls (Rotate, Grayscale, Brightness, Contrast, Crop) wired into the document management flows.
- ✅ **Structural Refactoring**:
  - Extracted **Gesture Module** (`reader_gestures.rs`) and **GestureProcessor** trait.
  - Extracted **Rendering**, **Settings**, and **Annotation** modules from `ReaderImpl`.
  - Migrated **13 lazy_static!** instances to `std::sync::LazyLock`.
  - Moved **unit tests** to sibling `_tests.rs` files for multiple core modules.
- ✅ **Documentation Audit**: Completed comprehensive audit (April 12, 2026) of all codebase `.md` documents, ensuring alignment with current source tree.

## Relevant files / directories

**Home View Refactoring:**
- `crates/core/src/view/home/mod.rs` (main file to split, 2,769 lines)
- Related view components: `book.rs`, `shelf.rs`, `address_bar.rs`, `navigation_bar.rs`, `bottom_bar.rs`, `home_utils.rs`

**ReaderImpl Refactoring:**
- `crates/core/src/view/reader/reader_impl/reader.rs` (main file, 3,403 lines)
- Existing helper modules: `reader_settings.rs`, `reader_rendering.rs`, `reader_search.rs`, `reader_annotations.rs`, `reader_gestures.rs`, `reader_core.rs`

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
