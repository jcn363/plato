# Phase 1: Consolidation Guide

## Completed Tasks

### ✅ Task 1: with_child! Macro (Completed)
**Location:** `crates/core/src/view/common.rs:24-31`
**Impact:** ~200 lines saved when applied systematically

The `with_child!` macro eliminates boilerplate for the pattern:
```rust
if let Some(index) = locate_by_id(view, id) {
    // body code
}
```

**Usage:**
```rust
with_child!(self, ViewId::SortMenu, |index| {
    // body code uses index
});
```

**Locations to Apply (35+ uses identified):**
- `crates/core/src/view/home/mod.rs:1064, 1121, 1164, 1272, 1381, 1458+`
- `crates/core/src/view/reader/reader_impl/reader.rs` (multiple locations)
- Other view files

### ✅ Task 2: View Trait Render Methods (Completed)
**Location:** `crates/core/src/view/view_trait.rs:107-125`
**Impact:** ~300 lines saved when applied systematically

Added two methods to View trait:
```rust
fn queue_render(&self, rq: &mut RenderQueue, mode: UpdateMode)
fn queue_child_render(&self, index: usize, rq: &mut RenderQueue, mode: UpdateMode)
```

**Benefits:**
- Eliminates scattered `RenderData::new()` calls (830+ instances)
- Provides consistent interface for all render operations
- Makes intent clearer: "queue this for rendering"

**Locations to Apply:**
- `crates/core/src/view/rendering.rs:73` (RenderData::new pattern, 830+ uses)
- All view files using render queues

## In-Progress Tasks

### 🔄 Task 3: Extract toggle_menu() Helper

**Status:** DEFERRED - Complex Custom Logic

The 6 toggle methods in `home/mod.rs` (lines 700, 815, 898, 1157, 1264, 1374) are highly specialized with complex layout recalculations. Generic consolidation is risky without refactoring UI layout system first.

**Recommendation:**
1. Create integration tests for each toggle method first
2. Then systematically consolidate common patterns
3. Current estimated savings: ~500 lines IF layout logic can be abstracted

**Common Patterns Identified:**
- Pre/post-enable checks
- Layout recalculations using `scale_by_dpi()`, `halves()`, etc.
- Child insertion/removal at specific indices
- Settings updates via `context.settings`
- Keyboard toggle cascades

### 🔄 Task 4: Create add_menu() Helper

**Location:** Target - `crates/core/src/view/common.rs`

**Pattern Identified (50+ uses):**
```rust
let menu = Menu::new(...);
rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
self.children.push(Box::new(menu) as Box<dyn View>);
```

**Proposed Signature:**
```rust
fn add_menu(&mut self, menu: Menu, rq: &mut RenderQueue);
```

**Estimated Savings:** ~150 lines

**Locations (50+ identified):**
- `crates/core/src/view/home/mod.rs` (primary)
- `crates/core/src/view/settings/` (multiple files)
- Various other view files

### 🔄 Task 5: Consolidate Error Handling

**Patterns Identified:** 29+ identical match patterns
**Estimated Savings:** ~200 lines

**Current Pattern (example):**
```rust
match result {
    Ok(value) => { /* handle */ },
    Err(e) => {
        log_error!("Operation failed: {}", e);
        // render error notification
    }
}
```

**Recommendation:**
Create error handling helper in `crates/core/src/view/common.rs`:
```rust
pub fn handle_result<T, F>(
    result: Result<T, Error>,
    on_ok: F,
    context: &mut Context,
    rq: &mut RenderQueue,
) where
    F: FnOnce(T)
```

## Next Steps

1. **Immediate (Next 1-2 hours):**
   - Implement `add_menu()` helper
   - Consolidate error handling patterns
   - Verify all builds pass

2. **Short-term (Next 2-3 hours):**
   - Create integration tests for layout-sensitive toggles
   - Systematically apply `with_child!` macro to reduce locate_by_id calls
   - Apply `queue_render()` pattern in performance-critical paths

3. **Medium-term (After Phase 1):**
   - Refactor toggle methods with test coverage
   - Split monolithic files (Phase 2)
   - Create settings registry

## Success Metrics

- [ ] Boilerplate reduced by ~1,350 lines total
- [ ] All builds pass with zero warnings
- [ ] Tests pass for all changed code
- [ ] Git history shows clear incremental commits
- [ ] Code review approvals on all changes
