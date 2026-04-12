# Plato Codebase Integration Opportunities

## Status Vocabulary

- `Completed`
- `Open`
- `Deferred`
- `Blocked`
- `Stale/Retired`

## Completed Previously

These items were valid integration opportunities in earlier reviews, but the code now contains the corresponding helpers or abstractions:

- `with_child!` macro in `crates/core/src/view/common.rs`
- `add_menu()` helper in `crates/core/src/view/common.rs`
- Generic menu toggle helpers in `crates/core/src/view/menu_helpers.rs`

They should not remain in the active backlog.

## Open

### 1. Reader stub block and partial module migration

**Problem**

The reader refactor stopped halfway. `crates/core/src/view/reader/reader_impl/reader.rs` is still large and still ends with a stub-method section that duplicates reader behavior with placeholder render calls.

**Evidence**

- `crates/core/src/view/reader/reader_impl/reader.rs`: `3403` lines
- Stub block starts near the end of the file under `// Stub Method Declarations (Reader trait interface)`
- Extracted reader modules exist:
  - `reader_rendering.rs`
  - `reader_gestures.rs`
  - `reader_settings.rs`
  - `reader_annotations.rs`
  - `reader_search.rs`
- Many extracted functions remain `#[allow(dead_code)]`, so the split is partial rather than complete.

**Recommended action**

- Make a decision on the refactor direction:
  - either complete the reader split and move active implementations out of `reader.rs`
  - or collapse the unused extracted layer and keep a single implementation surface
- Remove the duplicate stub layer last, after the active call paths are clear.

**Implementation order**

1. Map which reader methods are active versus stubbed.
2. Promote active helper modules into the real execution path.
3. Delete the duplicate stub block.

**Acceptance condition**

- No placeholder reader interface block remains.
- Reader behavior is implemented in one active layer only.

### 2. Home view modularization after state deduplication

**Problem**

The home view is still oversized (2769 lines) and violates the `AGENTS.md` 1000-line mandate.

**Evidence**

- `crates/core/src/view/home/mod.rs`: `2769` lines

**Recommended action**

- Split the home view by behavior to reach < 1000 lines:
  - shared state / construction
  - event handling
  - batch operations
  - menu orchestration
  - rendering

**Implementation order**

1. Identify stable responsibility boundaries in the active `Home` implementation.
2. Split only once ownership boundaries are stable.
3. Keep state ownership inside the compiled module tree only.

**Acceptance condition**

- Home file is under 1000 lines.

### 3. Safe Wrapper Migration for Fonts

**Problem**

`font/mod.rs` (2400 lines) violates the `AGENTS.md` 1000-line mandate and the mandatory rule to use safe wrappers instead of direct FFI.

**Evidence**

- `crates/core/src/font/mod.rs`: `2400` lines
- Direct usage of `FtFace` and other raw FFI types.

**Recommended action**

- Replace direct FFI calls with the safe wrappers in `crates/core/src/font/`.
- Split the monolithic `font/mod.rs` into smaller modules to reach < 1000 lines.

**Acceptance condition**

- Zero direct FFI calls in `font/mod.rs`.
- All modules under 1000 lines.

### 4. Unit Test Segregation

**Problem**

Unit tests are currently embedded in production code, violating the `AGENTS.md` mandatory rule for sibling test files.

**Evidence**

- Grep for `#[test]` shows many tests inside production `.rs` files.
- No `_tests.rs` files exist in `crates/core/src`.

**Recommended action**

- Extract all unit tests into sibling files named `{module}_tests.rs`.
- Ensure test files use `mod {module};` or `use super::*;` to access production code.

**Acceptance condition**

- Zero `#[test]` blocks in production source files.

### 5. PDF tools UI workflow completion

**Status:** Partially completed. Interactive page selection and degree input for Delete, Rotate, and Extract operations are implemented. Redaction workflow now includes page selection and transitions to a mode for defining the redaction region.

**Problem:**
- Interactive redaction region definition UI is pending implementation.
- File selection for PDF merging is missing.
- Some manipulation paths still depend on hard-coded defaults rather than fully integrated user inputs.

**Evidence:**
- `crates/core/src/view/pdf_manipulator.rs` now transitions to `DefineRedactionRegion` mode after page selection for redaction.
- `process_manipulation` has placeholders for user input, but the UI for Redaction region input and Merge file selection is not yet fully implemented.

**Recommended action:**
- Implement interactive redaction region definition UI (e.g., input fields for coordinates or predefined region selection).
- Implement file selection flow for PDF merging.
- Ensure all manipulation paths fully integrate user inputs.

**Acceptance condition:**
- Every PDF tool action is reachable from the UI and uses user-selected inputs.

### 6. Cover editor UI completion

**Status:** Substantially completed. Interactive controls for Rotate, Grayscale, Save, Reset, Brightness, and Contrast are implemented. Crop functionality is initiated with UI elements and mode transitions, allowing visual selection (application is placeholder).

**Problem:**
- Interactive cropping region application is pending.
- Some helper functions might still be guarded by `#[allow(dead_code)]`.

**Evidence:**
- `crates/core/src/view/cover_editor.rs` has added UI for initiating crop and visual selection feedback.
- `apply_crop_selection` is a placeholder.

**Recommended action:**
- Implement the interactive application of the crop selection.
- Address any remaining `#[allow(dead_code)]` scaffolding.

**Acceptance condition:**
- The user-facing feature set matches the implemented code path.
- All implemented helper capabilities (including interactive Crop application) are surfaced via the UI.

### 7. Restore verification

**Status:** Completed (2026-04-12)

**Problem**

Verification claims must be tied to exact commands run and current dates.

**Evidence**

- Host verification passes (2026-04-12):
  - `cargo check --target x86_64-unknown-linux-gnu` - Pass
  - `cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings` - Pass
- ARM Kobo verification passes (2026-04-12):
  - `cd mupdf_wrapper && ./build-kobo.sh` (rebuild wrapper)
  - `cargo build --profile release-arm --target arm-unknown-linux-gnueabihf -p plato` - Pass
- Any `cargo clean` requires wrapper rebuild before ARM build.

**Recommended action**

- Document exact commands and date in verification sections.
- Include native wrapper rebuild for ARM targets in verification flow.

**Acceptance condition**

- Documentation and actual branch verification results match.

## Deferred

These are broader ideas that should remain explicitly deferred unless a concrete implementation effort pulls them forward:

- Settings registry / schema-generation layer
- Event handler unification framework
- Cross-cutting input validation framework
- Generic object pooling
- Broad performance work without a measured hotspot

They are too abstract for the current source state and should not be presented as near-term integration work.

## Stale/Retired

Remove these from future review summaries unless they regress:

- Missing `with_child!`
- Missing `add_menu()`
- Missing generic menu toggle helpers
- Generic statements that plugin settings, external storage, or cover editor settings are "not integrated" without checking current call sites.

These are now historical review notes, not current backlog items.
