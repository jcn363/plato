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

- [ ] Migrate existing DPI usage to new helpers (40+ instances)
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

**Remaining unwrap() instances (mostly test code and static regex):**
- `metadata/constants.rs`: 2 instances (static Regex::new - acceptable for compile-time patterns)
- `view/reader/reader_impl/reader.rs`: 3 instances (LazyLock regex - acceptable for static regex)
- `thumbnail/cache.rs`: 7 instances (test code - acceptable)
- `thumbnail/worker.rs`: 3 instances (test code - acceptable)

- [ ] Replace remaining production code unwrap() (minimal remaining)
- [ ] Add `.with_context()` to all I/O operations

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
