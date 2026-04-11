# Plato Codebase Integration Opportunities

## Status Vocabulary

This backlog uses four statuses only:

- `Completed`
- `Open`
- `Deferred`
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
- Extracted modules exist:
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

The home view is still oversized and still duplicates batch-selection state.

**Evidence**

- `crates/core/src/view/home/mod.rs`: `2769` lines

**Recommended action**

- Fix state ownership first, then split the home view by behavior:
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

- Home file boundaries reflect active responsibilities.

### 3. PDF tools need a real surfaced workflow

**Problem**

The PDF tools view contains real manipulation logic, but the UI flow is incomplete and still depends on parked code.

**Evidence**

- `crates/core/src/view/pdf_manipulator.rs` still uses `#[allow(dead_code)]` on manipulation modes.
- The view can now be launched from selected-document contexts, so `show_actions()` is active.
- File-selection and action-selection flow is still not fully connected for the broader workflow.
- Some manipulations still use hard-coded defaults rather than user-driven parameters.

**Recommended action**

- Define a single supported workflow:
  - choose PDF
  - choose action
  - provide action-specific parameters
  - run operation
  - show result path or error

**Implementation order**

1. Decide whether to integrate with an existing file browser or launch from a selected document context.
2. Remove unreachable menus.
3. Replace hard-coded defaults with explicit user inputs.

**Acceptance condition**

- Every PDF tool visible in the UI is reachable and parameterized by user choice.

### 4. Cover editor needs either completion or scope reduction

**Problem**

The cover editor has real image-editing helpers but no complete user-facing editing flow.

**Evidence**

- `crates/core/src/view/cover_editor.rs` contains `apply_crop`, `apply_rotate`, `apply_brightness`, `apply_contrast`, `apply_grayscale`, and `save_cover`.
- The view can now be launched from the home book menu for selected EPUBs.
- The view and impl are still broadly marked `#[allow(dead_code)]`.
- The editing helpers are not backed by a surfaced toolbar/menu system.

**Recommended action**

- Choose one product direction:
  - full interactive editor
  - or intentionally limited cover replacement with a smaller UI

**Implementation order**

1. Define the supported user flow.
2. Remove dead scaffolding that falls outside that flow.
3. Surface only the operations that are actually supported.

**Acceptance condition**

- The cover editor’s advertised capability matches the reachable UI.

### 5. Restore verification before claiming integration progress

**Problem**

Verification claims can drift quickly on this branch and must be kept tied to rerun commands.

**Evidence**

- Host verification now passes after the theme/settings fixes.
- ARM verification also passes, but a clean ARM rebuild additionally requires rebuilding `mupdf_wrapper` after `cargo clean`.
- Earlier review snapshots were stale because they described failing verification after the issue had already been fixed.

**Recommended action**

- Do not mark any integration document as "clean build verified" without naming the exact commands rerun.
- When `cargo clean` is part of the verification flow, include the native wrapper rebuild step for Kobo targets.

**Implementation order**

1. Re-run the target-specific commands you intend to claim.
2. For Kobo release builds, rebuild `mupdf_wrapper` if the clean step removed it.
3. Only then refresh any build-health claims.

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
- Generic statements that plugin settings, external storage, or cover editor settings are "not integrated" without checking current call sites
