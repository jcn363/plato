# TODO List for Plato Project

## Critical Priority (AGENTS.md Mandatory Rules - No Backward Compatibility)

### 1. Zero Dead Code Without Justification
**AGENTS.md rule:** All `#[allow(dead_code)]` must have explanatory comments or be removed immediately.

**✅ COMPLETED:** All 20+ violations now have justification comments:
- `document/pdf.rs:119` - Used for PDF page-level operations
- `document/mupdf/text.rs:222` - MuPDF context reference for text character operations
- `font/face.rs:15, 21` - HarfBuzz font and space codepoint for text shaping/layout
- `view/reader/chapter_label.rs:12, 21` - Chapter navigation display and UI implementation
- `view/reader/reader_impl/reader_gestures.rs:27, 61, 106` - Selection rectangle updates for gesture handling
- `view/reader/results_label.rs:10, 19` - Search results display and UI implementation
- `view/reader/tool_bar.rs:57, 62, 67, 72, 77, 82, 87` - Toolbar update methods for reader settings
- `view/reader/margin_cropper.rs:15, 18, 27` - Margin cropper UI button sizing and implementation
- `view/reader/bottom_bar.rs:18, 27` - Reader navigation bar and implementation

### 2. File Modularization (Hard 1,000 Line Limit)
**AGENTS.md rule:** No source file exceeds 1,000 lines. No function exceeds 50 lines.

**✅ COMPLETED:** All files now under 1,000 line limit
- Largest file: `view/reader/reader_impl/reader.rs` at 921 lines (under limit)
- Second largest: `view/reader/reader_impl/reader_settings.rs` at 903 lines (under limit)
- All 17 previously identified files are now compliant

**Current file sizes (verified 2026-04-16):**
| File | Current Lines | Status |
|------|--------------|--------|
| `view/reader/reader_impl/reader.rs` | 921 | ✅ Under limit |
| `view/reader/reader_impl/reader_settings.rs` | 903 | ✅ Under limit |
| `document/pdf_manipulator.rs` | 872 | ✅ Under limit |
| `font/mod.rs` | 801 | ✅ Under limit |
| `view/pdf_manipulator.rs` | 772 | ✅ Under limit |
| `document/mod.rs` | 742 | ✅ Under limit |
| `document/html/layout.rs` | 703 | ✅ Under limit |

- [ ] Ensure all functions under 50 lines (next priority)

### 3. Function Size Compliance (50 Line Limit)
**AGENTS.md rule:** No function exceeds 50 lines. Extract helper functions for complex logic.

- [ ] Identify all functions >50 lines across codebase
- [ ] Extract helper functions for complex logic
- [ ] Each function must do one thing well
- [ ] Focus on largest files first (reader.rs, reader_settings.rs, pdf_manipulator.rs)

## High Priority

### 4. DRY (Don't Repeat Yourself)
**AGENTS.md rule:** Never duplicate code. Extract shared logic immediately.

**✅ COMPLETED:** MuPDF Context Creation
- Added `MuPdfContext::new_with_context()` helper for custom error messages
- Removed Default impl using .expect() (violates error handling)
- All uses now properly use `MuPdfContext::new()` with `?` for error handling

**✅ COMPLETED:** DPI Scaling Helper
- Added `scale_by_device_dpi()` and `scale_by_device_dpi_raw()` helpers to unit.rs
- These eliminate need for repeated `let dpi = CURRENT_DEVICE.dpi` pattern in 40+ files
- View modules can now use `scale_by_device_dpi(x)` directly

**Remaining DRY violations:**
- **Device Rotation**: `CURRENT_DEVICE.to_canonical()`, `CURRENT_DEVICE.mirroring_scheme()` patterns
- **Font Loading**: Repeated font loading patterns in multiple view modules
- **Scale by DPI**: Migrate existing `let dpi = CURRENT_DEVICE.dpi` + `scale_by_dpi()` pattern to use new helpers

**IN PROGRESS:** Function Size Compliance
- Found `build_pages()` in document/html/mod.rs (~108 lines) - Split into 4 helper functions
- Found `update_content()` in view/home/directories_bar.rs (~60 lines) - Split into 3 helper functions
- Found `make_page()` in view/home/directories_bar.rs (~140 lines) - Split into 5 helper functions
- Found `copy_to()` in library/manage.rs (~82 lines) - Split into 8 helper functions
- Found `move_to()` in library/manage.rs (~78 lines) - Split into 6 helper functions
- Found `import()` in library/scan.rs (~126 lines) - Split into 4 helper functions
- Found `clean_up()` in library/maintenance.rs (~53 lines) - Split into 3 helper functions
- Found `reload()` in library/maintenance.rs (~59 lines) - Split into 4 helper functions
- Found `new()` in document/epub/opener.rs (~60 lines) - Split into 5 helper functions
- Found `merge_pdfs()` in document/pdf_manipulator.rs (~60 lines) - Split into 6 helper functions
- Found `new()` in view/home/mod.rs (~195 lines) - Split into 6 helper functions
- Found `new()` in view/reader/reader_impl/reader.rs (~62 lines) - Split into 3 helper functions
- Found `from_html()` in view/reader/reader_impl/reader.rs (~66 lines) - Split into 3 helper functions
- Found `render_animation()` in view/reader/reader_impl/reader.rs (~73 lines) - Split into 6 helper functions
- Found `update()` in view/presets_list.rs (~60 lines) - Split into 4 helper functions
- Found `handle_event()` in view/rotation_values/mod.rs (~75 lines) - Split into 4 helper functions
- Found `new()` in view/search_replace.rs (~165 lines) - Split into 7 helper functions
- Found `new()` in view/search_bar.rs (~69 lines) - Split into 6 helper functions
- Found `parse_atom()` in opds.rs (~60 lines) - Split into 3 helper functions + AtomParserState struct
- Found `parse_nav()` in opds.rs (~55 lines) - Split into 2 helper functions + NavParserState struct
- Found `update()` in view/home/shelf.rs (~90 lines) - Split into 8 helper functions
- Found `new()` in view/reader/bottom_bar.rs (~71 lines) - Split into 5 helper functions
- Found `guess_frontlight()` in settings/preset.rs (~53 lines) - Split into 3 helper functions
- Found `parse_device_events()` in input.rs (~180 lines) - Split into 6 helper functions
- Found `new()` in context.rs (~53 lines) - Split into 3 helper functions
- Remaining: Search for and split other functions >50 lines

**✅ COMPLETED:** DPI Migration
- Found 60+ instances of `let dpi = CURRENT_DEVICE.dpi` across view modules
- Helpers added to unit.rs: `scale_by_device_dpi()`, `scale_by_device_dpi_raw()`, `get_device_dpi()`
- Migrated all 57 instances across 37 files: view/intermission.rs, view/preset.rs, view/clock.rs, view/slider.rs, view/menu.rs, view/calculator/display.rs, view/calculator/state.rs, view/calculator/code_area.rs, view/calculator/input_bar.rs, view/rotation_values.rs, view/menu_entry.rs, view/label.rs, view/button.rs, view/presets_list.rs, view/notification.rs, view/page_label.rs, view/statistics.rs, view/frontlight.rs, view/settings/mod.rs, view/reader/results_label.rs, view/reader/margin_cropper.rs, view/reader/chapter_label.rs, view/search_bar.rs, view/search_replace.rs, view/cover_editor.rs, view/key.rs, view/sketch/, view/rounded_button.rs, view/dialog.rs, view/input_field.rs, view/named_input.rs, view/battery.rs, view/reader/reader_impl/reader.rs, view/reader/reader_impl/reader_annotations.rs, view/reader/reader_impl/reader_rendering_impl.rs, view/reader/tool_bar/layout.rs, view/keyboard.rs, view/pdf_manipulator.rs, view/touch_events/mod.rs, view/epub_editor/mod.rs, view/dictionary/display.rs, view/dictionary/mod.rs, view/dictionary/events.rs, view/dictionary/lookup.rs, view/home/book.rs, view/home/address_bar.rs, view/home/directory.rs, view/home/navigation_bar.rs, view/home/shelf.rs, view/home/library_label.rs, view/home/mod.rs, view/home/directories_bar.rs, view/home/updates.rs, view/home/ui_toggles/utils.rs
- All view modules now use centralized DPI helpers per DRY rule
- [ ] Extract device rotation helpers (to_canonical, mirroring_scheme)
- [ ] Consolidate font loading patterns
- [ ] Extract common `match` arm patterns into methods
- [ ] Move repeated constants to shared `consts` module

### 5. Error Handling Standardization
**AGENTS.md rule:** Use `anyhow` for apps, `thiserror` for libraries. Never mix.

**✅ COMPLETED:** Critical unwrap() replacements
- `theme.rs`: 14 instances - All Mutex lock poisoning replaced with `.expect("lock_name poisoned")`
- `thumbnail/manager.rs`: 7 instances - Lock poisoning and test unwrap replaced with `.expect()`
- `document/html/engine_text/text_renderer.rs`: 1 instance - Replaced with `.expect("glyph_id should be in cache")`
- `thumbnail/cache.rs`: 1 instance - Added validation and `.expect("max_size > 0 validated above")`

**✅ COMPLETED:** Added .with_context() to I/O operations
- `battery/kobo.rs`: Added context to File::open calls for battery files
- `library/manage.rs`: Added context to File::open for destination file
- `lightsensor/kobo.rs`: Added context to File::open for light sensor file
- `view/sketch/mod.rs`: Added context to File::open for sketch file
- `document/epub/opener.rs`: Added context to File::open for EPUB file
- `document/html/mod.rs`: Added context to File::open calls (2 instances)
- `document/mod.rs`: Added context to File::open for file type detection
- `frontlight/natural.rs`: Added context to File::open for frontlight max value
- `cover_editor.rs`: Added context to File::open and File::create calls
- `framebuffer/image.rs`: Added context to File::open for PNG file
- `dictionary/dictreader.rs`: Added error context to File::open calls
- `rtc.rs`: Added context to File::open for RTC device file
- `dictionary/indexing.rs`: Added error context to File::open call

**Remaining unwrap() instances (mostly test code and static regex):**
- `metadata/constants.rs`: 2 instances (static Regex::new - acceptable for compile-time patterns)
- `view/reader/reader_impl/reader.rs`: 3 instances (LazyLock regex - acceptable for static regex)
- `thumbnail/cache.rs`: 7 instances (test code - acceptable)
- `thumbnail/worker.rs`: 3 instances (test code - acceptable)

### 5.1. Modern Static Initialization (LazyLock Migration)
**AGENTS.md rule:** Use `std::sync::LazyLock` for runtime initialization, not `lazy_static!`.

**✅ COMPLETED:** Migrated 4 files from `lazy_static!` to `std::sync::LazyLock`:
- `font/md_title.rs:5` - MD_TITLE now uses LazyLock
- `frontlight/natural.rs:38` - FRONTLIGHT_DIRS now uses LazyLock
- `view/icon.rs:19` - ICONS_PIXMAPS now uses LazyLock
- `font/mod.rs:91` - MD_TITLE now uses LazyLock

**Kept as `lazy_static!` (intentional - hardware config):**
- `device.rs:474` - CURRENT_DEVICE requires runtime hardware config per AGENTS.md

**Files already using `LazyLock` (12 files - good):**
- `metadata/constants.rs`, `theme.rs`, `view/keyboard.rs`, `document/html/layout.rs`, `helpers.rs`, `thumbnail/worker.rs`, `i18n/mod.rs`, `framebuffer/transform.rs`, `view/reader/reader_impl/reader.rs`

### 6. Input Validation
**AGENTS.md rule:** Validate all public API inputs. Fail fast.

- [ ] Add validation for all public functions
- [ ] Validate configuration at load time
- [ ] Use typed enums instead of raw strings
- [ ] Provide clear, actionable error messages for invalid inputs

### 7. Configuration Management
**AGENTS.md rule:** Centralize configuration management and validate all configuration values.

- [ ] Group related configuration in dedicated structs or modules
- [ ] Add validation for configuration values at load time
- [ ] Use typed configuration over raw strings or magic numbers
- [ ] Define enums for known sets of valid values
- [ ] Document all configuration options, valid ranges, and default values
- [ ] Validate configuration values against constraints (font size ranges, color values, timeout limits)

### 8. Single Source of Truth
**AGENTS.md rule:** Every piece of knowledge or logic must have one authoritative location.

- [ ] Define constants in the module that owns the concept, then pub/pub(crate) export
- [ ] Avoid shadowing or overriding the same data in multiple layers
- [ ] If a setting is in Context, don't also cache it locally without invalidation strategy
- [ ] When refactoring duplicated patterns, consolidate into canonical location and remove copies

### 9. Modular Architecture
**AGENTS.md rule:** Design for clear separation of concerns and testability.

- [ ] Add interfaces/traits for major components to improve testability
- [ ] Mock trait implementations in tests rather than relying on concrete types
- [ ] Each layer should depend only on abstractions (traits), not concrete implementations
- [ ] Group related functionality behind well-defined module boundaries with minimal public surface area

### 10. Module Hierarchy
**AGENTS.md rule:** Structure modules logically, avoid circular dependencies, and document purposes.

- [ ] Group related functionality by domain (e.g., `document/pdf`, `document/epub`, `view/reader`)
- [ ] Avoid circular dependencies between modules
- [ ] Document each module's purpose at the top of its `mod.rs` file
- [ ] Extract shared types to third module if two modules reference each other

### 11. Architecture Documentation
**AGENTS.md rule:** Add high-level architecture diagrams and document design decisions and trade-offs.

- [ ] Add architecture diagrams to docs/architecture/
- [ ] Document rationale behind major structural choices (e.g., trait-based abstraction over concrete types)
- [ ] Reference architecture docs from module-level documentation

## Medium Priority

### 7. Reader View Completion
- [ ] Complete extraction from `reader_view.rs` into focused modules
- [ ] Replace stub methods with active call paths
- [ ] Ensure safe wrapper usage (no direct FFI)

### 8. Cover Editor Crop Feature
- [ ] Implement interactive crop application
- [ ] Reference: CROP_PLAN.md

## Low Priority / Deferred

### 17. Lazy Thumbnail Implementation
- Deferred due to device memory constraints
- Reference: MODULARIZATION_PLAN.md line 373

## Completed

| Task | Status |
|------|--------|
| Home view modularization | home/mod.rs: 596 lines |
| Font module safe wrappers | font/mod.rs: 802 lines |
| PDF Tools UI | Redaction + merging implemented |
| Unit test segregation | `_tests.rs` sibling files |
| Type consolidation | ViewPort imported correctly |

## Verification Checklist

Before any PR:
- [ ] `cargo fmt` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test --target x86_64-unknown-linux-gnu` passes
- [ ] ARM target builds: `cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato`
- [ ] Zero new `#[allow(dead_code)]` without justification
- [ ] No files >1,000 lines introduced
- [ ] No functions >50 lines introduced
- [ ] No code duplication added
