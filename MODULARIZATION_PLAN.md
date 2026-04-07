# Plato Codebase Modularization Plan

> Following DRY (Don't Repeat Yourself) Principle
> Last Updated: April 7, 2026

## Executive Summary

This plan identifies opportunities to modularize the Plato codebase by extracting duplicated patterns, splitting monolithic files, and creating reusable components. Following analysis of 195 source files, the highest impact opportunities are:

1. **Reader Module** (3,410 lines) - Extract nested structs, split into focused modules
2. **Home Module** (2,787 lines) - Split into library management components  
3. **Font Module** (2,783 lines) - Separate font operations from UI
4. **HTML Engine** (2,672 lines) - Isolate parsing/rendering concerns
5. **Common Patterns** - Extract duplicated code (~1,350 lines savable)

## Phase 1: Immediate Wins (2-4 hours each)

### 1.1 Extract Helper Macros

**Location:** `crates/core/src/view/common.rs`
**Action:** Create and utilize helper macros to eliminate boilerplate

```rust
// Already implemented: with_child! macro
// Usage: with_child!(view, ViewId::SomeView, |index| { /* body */ })
```

**Opportunities:**
- Replace 35+ `locate_by_id` patterns in home/mod.rs (~200 lines saved)
- Standardize child view manipulation

### 1.2 Extract Common Menu Patterns

**Location:** Create `menu_helpers.rs`
**Action:** Extract generic menu toggle/create patterns

```rust
pub fn toggle_menu<T: View>(
    view: &mut dyn View,
    id: ViewId,
    create_fn: impl FnOnce() -> T,
    rq: &mut RenderQueue,
    enable: Option<bool>,
) -> Option<bool> {
    // Generic implementation
}
```

**Savings:** ~400 lines across 6+ toggle methods

### 1.3 Render Queue Abstraction

**Location:** Extend `View` trait
**Action:** Add helper methods to reduce `rq.add(RenderData::new(...))` boilerplate

```rust
trait View {
    fn queue_render(&self, rq: &mut RenderQueue, mode: UpdateMode) {
        rq.add(RenderData::new(self.id(), self.rect(), mode));
    }
    
    fn queue_child_render(&self, index: usize, rq: &mut RenderQueue, mode: UpdateMode) {
        if let Some(child) = self.children().get(index) {
            rq.add(RenderData::new(child.id(), *child.rect(), mode));
        }
    }
}
```

**Savings:** ~300 lines (830+ scattered uses)

## Phase 2: Module Splitting (3-5 days each)

### 2.1 Reader Module Refactor

**Current:** `crates/core/src/view/reader/reader_impl/reader.rs` (3,410 lines)
**Target:** Split into focused modules

**Proposed Structure:**
```
reader/
├── mod.rs                 # Public re-exports
├── reader_core.rs         # Shared types (already started)
├── reader_state.rs        # PageState, DisplaySettings, InteractionState
├── reader_rendering.rs    # Rendering, animation, text extraction
├── reader_input.rs        # Gesture, touch, input handling
├── reader_annotations.rs  # Annotations, notes, highlights
├── reader_dialogs.rs      # Input dialogs, text entry
├── reader_settings.rs     # Settings menus, configuration
└── reader_search.rs       # Search functionality
```

**Benefits:**
- Reduce cognitive load (~700 lines per file)
- Improve testability
- Enable parallel development

### 2.2 Home Module Refactor

**Current:** `crates/core/src/view/home/mod.rs` (2,787 lines)
**Target:** Split into library management components

**Proposed Structure:**
```
home/
├── mod.rs                 # Public re-exports
├── home_state.rs          # Library state, settings, sorting
├── home_ui.rs             # Layout, rendering, view composition
├── home_input.rs          # Event handling, gestures
├── home_library.rs        # Document operations (add, remove, move)
├── home_thumbnails.rs     # Thumbnail generation, caching
└── home_search.rs         # Search/filter functionality
```

**Benefits:**
- Separate UI from data model
- Isolate filesystem operations
- Improve maintainability

### 2.3 Font Module Refactor

**Current:** `crates/core/src/font/mod.rs` (~2,800 lines)
**Target:** Separate concerns

**Proposed Structure:**
```
font/
├── mod.rs                 # Public re-exports
├── font_operations.rs     # Low-level font ops (HarfBuzz/FreeType)
├── font_layout.rs         # Text layout, shaping, measurement
├── font_cache.rs          # Glyph caching, font management
└── font_ui.rs             # Font selection UI, preview
```

**Benefits:**
- Isolate platform-specific font ops
- Enable font caching strategies
- Separate UI from layout logic

## Phase 3: Pattern Extraction (2-3 days)

### 3.1 Settings System Improvements

**Issue:** Settings scattered across modules, inconsistent access patterns
**Solution:** Create centralized settings registry

```rust
pub struct SettingsRegistry {
    reader: ReaderSettings,
    home: HomeSettings,
    // ... others
}

impl SettingsRegistry {
    fn get<T: SettingsTrait>(&self) -> &T { /* ... */ }
    fn update<T: SettingsTrait>(&mut self, update_fn: impl FnOnce(&mut T)) { /* ... */ }
}
```

### 3.2 Error Handling Consistency

**Issue:** Mixed use of `unwrap()`, `expect()`, `?`, and manual error handling
**Solution:** Standardize on `Result` propagation with context

**Already Completed:**
- All constructors now return `Result` instead of panicking
- Consistent use of `.context()` for better error messages

### 3.3 Resource Management

**Issue:** Scattered resource allocation/deallocation patterns
**Solution:** RAII wrappers for common resources

**Already Completed:**
- MuPdfContext uses Rc for shared ownership
- Pixmap::new() returns Result for allocation failures
- Consistent Drop implementations for wrappers

## Phase 4: Performance Optimizations (Ongoing)

### 4.1 Caching Layers

**Issues Identified:**
- No metadata caching (401 lines impacted)
- No font glyph cache (203 lines impacted)  
- No search result cache (153 lines impacted)
- No I/O batching (62 unnecessary clones)

**Solutions:**
- Implement filesystem metadata cache with TTL
- Add LRU font glyph cache for frequently used characters
- Create search result cache with query normalization
- Batch filesystem operations where possible

### 4.2 Rendering Optimizations

**Issues:**
- 830+ scattered render queue operations
- Inefficient pixmap allocations in hot paths
- Redundant layout calculations

**Solutions:**
- Already implemented View trait render helpers
- Consider object pools for frequently allocated objects
- Cache layout results where applicable

## Implementation Guidelines

### DRY Principles Applied

1. **Single Source of Truth**: Each concept defined in exactly one place
2. **Prefer Composition**: Build complex types from smaller, focused ones
3. **Extract Early**: When duplication is spotted, extract immediately
4. **Favor Traits**: Use traits for polymorphic behavior over inheritance
5. **Keep Functions Small**: Functions should do one thing well (<50 lines)

### Modularity Principles

1. **High Cohesion**: Modules should have related, focused responsibilities
2. **Low Coupling**: Modules should interact through well-defined interfaces
3. **Encapsulation**: Hide implementation details behind public APIs
4. **Testability**: Each module should be independently testable

### Migration Strategy

1. **Start Small**: Begin with helper functions/macros
2. **Vertical Slices**: Extract complete features, not just utilities
3. **Maintain Compatibility**: Keep public APIs stable during refactoring
4. **Test Frequently**: Run tests after each extraction
5. **Document Changes**: Update module documentation as you go

## Success Metrics

| Metric | Current | Target | Improvement |
|--------|---------|--------|-------------|
| Largest File Size | 3,410 lines | <1,000 lines | 70% reduction |
| Duplicate Lines | ~1,350 lines | 0 lines | 100% elimination |
| Module Count | ~15 modules | ~25-30 modules | 60-100% increase |
| Build Time | ~3 minutes | <2 minutes | 30%+ improvement |

## Files Already Improved

**Completed (April 7, 2026):**
- ✅ Error handling - Constructors return Result
- ✅ Type deduplication - ViewPort, PageAnimKind from reader_core.rs  
- ✅ Dead code removal - Unused constants removed
- ✅ Macro creation - with_child! for locate_by_id patterns
- ✅ AGENTS.md - Added dead code investigation section

## Next Immediate Actions

1. Commit MODULARIZATION_PLAN.md
2. Implement one quick win (e.g., menu toggle extraction)
3. Measure impact before/after
4. Proceed to next highest impact item

---
*This plan enables incremental improvement while maintaining system stability and test coverage throughout the modularization process.*