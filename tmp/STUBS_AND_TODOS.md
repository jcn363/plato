# Implementation Status: Stubs, TODOs, and Known Limitations

## Overview
This document tracks stub implementations, TODOs, and known limitations throughout the Plato codebase.
It serves as a roadmap for future development and maintenance.

---

## Critical TODOs (High Priority)

### 1. Panic Points That Need Error Handling

#### reader.rs:385 - HTML Document Opening
**Location**: `crates/core/src/view/reader/reader_impl/reader.rs:385`

```rust
let doc = crate::document::open_html(html)
    .unwrap_or_else(|_| panic!("Failed to open HTML document"));
```

**Issue**: Panics on invalid HTML input, crashing entire application
**Fix**: Return `Result<Reader, Error>` from `from_html()` method
**Status**: ✅ FIXED (April 7, 2026)
**Effort**: 2-3 hours
**Impact**: Medium (affects HTML document loading)

**Changes Made**:
- Changed `from_html()` return type from `Reader` to `Result<Reader, Error>`
- Added proper error handling with `.context()` for better diagnostics
- Updated all 4 call sites in app.rs and emulator/src/main.rs to handle `Result`

#### framebuffer/image.rs:24 - Pixmap Allocation
**Location**: `crates/core/src/framebuffer/image.rs:24`

```rust
data.try_reserve_exact(len).unwrap_or_else(|_| {
    panic!("Failed to allocate {} bytes for pixmap", len);
});
```

**Issue**: Panics on OOM, but `try_new()` method already exists
**Fix**: Use `try_new()` in callers where OOM is acceptable, document when to use each
**Status**: ✅ PARTIALLY FIXED (April 7, 2026)
**Effort**: 1-2 hours
**Impact**: Medium (affects rendering pipeline)

**Changes Made**:
- Added `try_new_result()` method that returns `Result<Pixmap, Error>` for callers needing proper error handling
- Kept `try_new()` returning `Option<Pixmap>` for backwards compatibility
- `new()` still panics (intentional for performance-critical hot paths)

---

## Type System Issues

### ViewId vs Id Duplication (FIXED ✓)

**Location**: `crates/core/src/view/home/`

**Issue**: Code was using `find_child_index_by_id(self, ViewId::X)` but function expected `Id` (u64)

**Fix Applied**: 
- Created `find_child_index_by_view_id()` helper in `home_utils.rs`
- Updated calls to use ViewId-aware implementation
- Removed unused `find_child_index_by_id()` function

**Status**: ✓ RESOLVED (Commit: 54f225e)

### Reader Type Duplication (TODO)

**Location**: `crates/core/src/view/reader/reader_impl/`

**Types Duplicated**:
- `ViewPort` (private in reader.rs, public in reader_core.rs)
- `Contrast`, `PageAnimation`, `RenderChunk`, `Search`, `Resource`

**Impact**: 
- Confusion about canonical location
- Potential for inconsistent definitions
- Extra type conversions needed

**Fix**: 
1. Identify canonical locations (prefer reader_core.rs for shared types)
2. Remove duplicates
3. Import canonical definitions

**Status**: TODO
**Effort**: 3-4 hours
**Priority**: Medium (code clarity)

---

## Feature Stubs (Intentional, Not Implemented)

### 1. Monochrome Display Mode
**Location**: `crates/core/src/framebuffer/mod.rs`

```rust
/// Enables monochrome (grayscale) display mode.
/// Not supported on Kobo e-readers (hardware limitation).
fn set_monochrome(&mut self, _enable: bool) {}
```

**Reason**: Kobo e-ink display controllers don't expose monochrome mode API
**Alternative**: Use grayscale image processing in document rendering
**Status**: Intentionally unimplemented
**Impact**: Minor (cosmetic feature)

### 2. PDF Font Family Selection
**Location**: `crates/core/src/view/reader/reader_impl/reader.rs`

```rust
fn set_font_family(&mut self, family_name: &str, ...) {
    // EPUB: Applies font family to layout
    // PDF: Stub - MuPDF doesn't support font substitution
}
```

**Reason**: MuPDF PDF viewer only supports fonts embedded in the document
**Alternative**: Support font substitution for EPUB documents only
**Status**: Intentionally limited to EPUB
**Impact**: Minor (PDFs use embedded fonts only)

---

## Architectural TODOs

### Phase 4: Reader Struct Simplification (20-30 hours)

**Goal**: Improve Reader struct maintainability without code extraction

**Tasks**:
1. **Consolidate Related Fields** (8 hours)
   ```ignore
   struct PageState {
       current_page: usize,
       pages_count: usize,
       synthetic: bool,
   }
   
   struct ViewportSettings {
       zoom_mode: ZoomMode,
       scroll_mode: ScrollMode,
       page_offset: Point,
       margin_width: i32,
   }
   ```

2. **Document Architectural Decisions** (4 hours)
   - Why certain methods stay in reader.rs
   - Why full extraction not beneficial
   - Parameter passing overhead analysis

3. **Add Module-Level Documentation** (3 hours)
   - Architecture diagrams
   - Design decision rationale
   - Performance characteristics

4. **Reduce Magic Numbers** (2 hours)
   - Extract to named constants
   - Group by related functionality

5. **Improve Method Organization** (3 hours)
   - Group methods by responsibility
   - Add section markers/comments
   - Improve method ordering

### Phase 4b: Event Handling Refactoring (6-8 hours)

**Goal**: Make handle_event() more modular without full extraction

**Approach**: Split into sub-handlers with clear boundaries
```ignore
fn handle_event(...) {
    match event {
        DeviceEvent::Gesture(ge) => self.handle_gesture_event(ge, ...),
        DeviceEvent::Button(bc, bs) => self.handle_button_event(bc, bs, ...),
        DeviceEvent::Finger(fe) => self.handle_finger_event(fe, ...),
    }
}
```

**Sub-handlers** (each 50-100 lines):
- `handle_gesture_event()` - Swipes, taps, long-press
- `handle_button_event()` - Physical buttons
- `handle_finger_event()` - Touch events
- `handle_menu_event()` - Menu callbacks

**Status**: TODO
**Effort**: 6-8 hours

### Phase 5: Home Module Split (20-30 hours)

**Goal**: Break down 2,690 line home module into focused sub-modules

**Proposed Structure**:
- `home_core.rs` - Data model, Library interface
- `home_ui.rs` - View hierarchy, layout, rendering
- `home_events.rs` - Event handling, user interaction
- `home_library.rs` - File system operations, document management
- `home_search.rs` - Search, filtering, sorting logic

**Performance Opportunities**:
- Async thumbnail generation
- Indexed search for large libraries
- Lazy loading of document metadata

**Status**: TODO
**Effort**: 20-30 hours
**Priority**: Low (current performance acceptable)

### Phase 6: Async I/O and Concurrency (30-40 hours)

**Goals**:
1. Move file I/O off main thread
2. Parallel page rendering for complex documents
3. Background thumbnail generation
4. Async document loading

**Considerations**:
- Kobo device constraints (limited RAM, low CPU)
- Battery impact of background threads
- Frame rate impact (eink refresh latency dominates)

**Status**: FUTURE
**Effort**: 30-40 hours
**Priority**: Low unless profiling shows bottlenecks

---

## Known Limitations

### Memory Constraints
- **Device**: Kobo devices have 1 GB RAM (shared with OS, apps)
- **Current usage**: 20-40 MB typical, 50-80 MB peak
- **Limitation**: Cannot hold multiple documents in memory simultaneously
- **Mitigation**: Load/unload documents as needed

### Display Performance
- **E-ink refresh**: 200-500ms per update (hardware limitation)
- **Rendering**: 100-300ms for simple pages
- **Total perceived latency**: Dominated by display, not computation
- **Implication**: Micro-optimizations have little user-visible benefit

### API Limitations
- **MuPDF**: No direct PDF annotation writing (workaround: parse PDF)
- **Kobo Display**: No direct monochrome mode control
- **Fonts**: PDF documents only support embedded fonts
- **Touch**: Limited stylus pressure sensitivity

### Testing Challenges
- **Native Libraries**: Tests require MuPDF, FreeType, HarfBuzz (ARM only)
- **Document Fixtures**: Need actual EPUB and PDF files
- **Device Simulation**: E-ink display behavior hard to mock
- **Current approach**: Integration tests with real documents

---

## Implementation Guidelines

### When Creating New Stubs
1. Add clear documentation explaining WHY it's a stub
2. Document the blocker (API limitation, hardware constraint, etc.)
3. Note any workarounds or alternatives
4. Link to related tracking issues/TODOs
5. Estimate effort to fully implement

### When Adding TODOs
1. Use consistent format: `// TODO (Phase N): Description`
2. Explain the current limitation
3. Describe desired behavior
4. Estimate effort (hours)
5. Note priority (High/Medium/Low)
6. Link to this document or related issues

### When Making Architectural Decisions
1. Document the trade-offs considered
2. Explain why the chosen approach was selected
3. Link to relevant code (e.g., Reader struct field grouping)
4. Note any alternatives and their costs
5. Consider future evolution (how might this change?)

---

## Maintenance Notes

### Code Review Checklist
- [ ] Does new code have stub documentation if incomplete?
- [ ] Are all panic!() calls documented with TODOs?
- [ ] Do new modules have architecture documentation?
- [ ] Are limitations and trade-offs documented?
- [ ] Are performance implications noted?

### Regular Tasks (Monthly)
1. Review TODOs and update effort estimates
2. Check for duplicate type definitions (especially across modules)
3. Profile actual memory and CPU usage on device
4. Verify panic! points are still handled elsewhere

### Before Release
1. Ensure no unintended panic!() calls in new code
2. Verify all stubs have proper documentation
3. Update this document with any new limitations discovered
4. Review for dead code (unfinished features that could be removed)

---

## Summary Statistics

**Current State**:
- Panic points: 2 (both documented with TODOs)
- Feature stubs: 2 (monochrome, PDF font family)
- Type duplications: 5 types duplicated (in progress)
- Architecture TODOs: 3 major phases (Phase 4-6)
- Unimplemented features: ~5 (by design)

**Code Quality**:
- ✓ Zero compilation errors
- ✓ Zero warnings (strict clippy mode)
- ✓ All public APIs documented
- ✓ Most TODOs have effort estimates

**Next Steps**:
1. Phase 4: Reader struct simplification (priority)
2. Fix type duplication (medium priority)
3. Replace panic!() calls (medium priority)
4. Phase 4b: Event handling refactoring (optional)
5. Phase 5+: Long-term improvements
