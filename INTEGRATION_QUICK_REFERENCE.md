# Plato Integration Opportunities - Quick Reference

> Updated: April 8, 2026 - Device support documentation and AGENTS.md updated

## Critical Issues (Fix Immediately)

### 1. PDF Manipulator - Unreachable File Browser Code
- **File:** `crates/core/src/view/pdf_manipulator.rs`
- **Lines:** 24-26, 27, 118-192, 269-296
- **Issue:** File selection code marked dead, hard-coded paths
- **Fix:** 2-3 days

### 2. Reader Stub Methods - Duplicate Interface
- **File:** `crates/core/src/view/reader/reader_impl/reader.rs`
- **Lines:** 3970-4168
- **Issue:** 40+ stub methods that duplicate trait interface
- **Fix:** Clarify design, consolidate or remove stubs
- **Effort:** 1-2 days investigation + architecture decision

### 3. Batch Mode Duplication
- **Files:** 
  - `crates/core/src/view/home/mod.rs:70` (batch_mode)
  - `crates/core/src/view/home/home.rs:75` (duplicate)
- **Issue:** Same field defined in two places
- **Fix:** Consolidate to single location
- **Effort:** 1 day

### 4. Cover Editor - Dead Code
- **File:** `crates/core/src/view/cover_editor.rs`
- **Lines:** 18-39, 46, 59
- **Issue:** Entire UI marked dead code, icons reserved but unused
- **Fix:** Complete implementation or remove
- **Effort:** 3-5 days (if completing feature)

---

## High-Impact Quick Wins

### Quick Win 1: locate_by_id Macro
- **Pattern:** `crates/core/src/view/home/mod.rs` (35+ uses)
- **Savings:** ~200 lines
- **Effort:** 2 hours
- **File Locations:** 1064, 1121, 1164, 1272, 1381, 1458+
```rust
// Create macro
macro_rules! with_child {
    ($view:expr, $id:expr, $body:expr) => {
        if let Some(index) = locate_by_id($view, $id) { $body(index) }
    };
}
```

### Quick Win 2: View Trait Render Methods
- **Current:** 830+ scattered `rq.add(RenderData::new(...))`
- **Solution:** Add to View trait:
```rust
fn queue_render(&self, rq: &mut RenderQueue, mode: UpdateMode)
fn queue_child_render(&self, index: usize, rq: &mut RenderQueue, mode: UpdateMode)
```
- **Savings:** ~300 lines
- **Effort:** 4 hours

### Quick Win 3: Menu Toggle Helper
- **Files:** `crates/core/src/view/home/mod.rs`
- **Methods:** Lines 700, 815, 898, 1157, 1264, 1374
- **Pattern:** 6 nearly identical toggle methods
- **Solution:** Extract to `toggle_menu<T: View>()`
- **Savings:** ~500 lines
- **Effort:** 1 day

### Quick Win 4: add_menu Helper
- **Pattern:** Menu creation in 50+ places
- **Current Code:**
```rust
let menu = Menu::new(...);
rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
self.children.push(Box::new(menu) as Box<dyn View>);
```
- **Solution:** `self.add_menu(ViewId, entries, context, rq)`
- **Savings:** ~150 lines
- **Effort:** 4 hours

---

## Large Refactorings Needed

### Split reader.rs (3,410 → 5 files of ~700 lines each)

**Current Structure:**
- Lines 1-200: Structs and constants
- Lines 201-1500: Core reader implementation
- Lines 1501-2500: Event handling
- Lines 2501-3410: Rendering, gestures, and stubs

**Proposed Split:**
1. **reader_core.rs** (700 lines) - State, navigation, page management
2. **reader_rendering.rs** (800 lines) - Render methods, animation
3. **reader_gestures.rs** (600 lines) - Gesture handling, touch
4. **reader_annotations.rs** (600 lines) - Annotation operations
5. **reader_events.rs** (700 lines) - Event routing

**Effort:** 2-3 days
**Benefit:** Maintainability, clarity, testing

### Split home/mod.rs (2,787 → 5 files of ~550 lines each)

**Current Structure:**
- Lines 1-100: Modules and imports
- Lines 101-500: Home struct definition
- Lines 501-1000: Construction and initialization
- Lines 1001-1500: Menu creation/toggling (~500 dup lines)
- Lines 1501-2000: Event handling
- Lines 2001-2500: More event handling
- Lines 2501-2787: Rendering

**Proposed Split:**
1. **home_core.rs** (500 lines) - Struct definition, construction
2. **home_events.rs** (600 lines) - Event dispatching
3. **home_menus.rs** (400 lines) - Menu toggles (extracted to helpers)
4. **home_operations.rs** (600 lines) - Batch ops, file operations
5. **home_rendering.rs** (300 lines) - Render methods

**Effort:** 3-4 days
**Benefit:** Maintainability, parallel editing

---

## Module Integration Gaps - Priority Order

### P1: Settings Registry (2-3 days)
**Files:**
- `crates/core/src/settings/mod.rs` - 18 structs scattered
- `crates/core/src/settings/features.rs` - 5 feature structs
- Various settings view files - duplicated patterns

**Create:**
1. Settings trait interface
2. Registry builder pattern
3. Versioning system
4. Schema/metadata for UI generation

### P2: Event Handler Unification (3-4 days)
**Files:**
- `crates/core/src/view/event_dispatch.rs` - Central dispatcher
- `crates/core/src/view/home/mod.rs` - 1000+ lines event handling
- `crates/core/src/view/reader/reader_impl/reader.rs` - 3000+ lines

**Create:**
1. EventCategory trait
2. Centralized router with priority queue
3. Standard event handlers

### P3: Input Validation Framework (2-3 days)
**Files:**
- `crates/core/src/view/input_field.rs`
- `crates/core/src/view/search_bar.rs`
- `crates/core/src/view/search_replace.rs`
- `crates/core/src/view/calculator/input_bar.rs`

**Create:**
1. InputValidator trait
2. Standard validators (length, pattern, range)
3. Composition pattern

---

## Performance Opportunities - Quick Impact

### Caching Opportunities
1. **Filesystem Metadata Cache** (2-3 days)
   - Location: `crates/core/src/library/`
   - Impact: Faster library loading
   - Files: scan.rs (342 lines)

2. **Font Glyph Cache** (1-2 days)
   - Location: `crates/core/src/font/mod.rs` (2783 lines)
   - Impact: Faster text rendering
   - Index: (font, size, character) → glyph

3. **Search Result Cache** (1-2 days)
   - Location: Search/replace implementations
   - Impact: Faster subsequent searches

### I/O Batching (3-4 days)
- Combine multiple file operations
- Batch settings saves
- Reduce E-ink refresh calls

### Memory Optimization (2-3 days)
- Pre-allocate Vecs (62 clones in home/mod.rs)
- Use Cow<str> for conditional ownership
- Object pooling for frequently created views

---

## Dead Code to Remove/Complete

### Dead Code Summary
- `crates/core/src/view/pdf_manipulator.rs`: 27, 120
- `crates/core/src/view/cover_editor.rs`: 22-38, 46, 59
- `crates/core/src/view/reader/results_bar.rs`: 15, 24
- `crates/core/src/view/reader/reader_impl/reader.rs`: 77, 88, 128, 143, 158+

### Unused Settings/Features
- `CoverEditorSettings` - not fully wired
- `PluginSettings` - unclear integration
- `enable_duplicates_detection` - field exists but unused
- `EditLanguagesInput` - language editing unclear

---

## File Sizes Needing Attention

| File | Lines | Status |
|------|-------|--------|
| reader_impl/reader.rs | 3,410 | CRITICAL - split |
| home/mod.rs | 2,787 | CRITICAL - split |
| font/mod.rs | ~2,800 | HIGH - split |
| document/html/engine.rs | ~2,672 | HIGH - split |
| document/html/layout.rs | 718 | MEDIUM |
| document/html/parse.rs | 622 | MEDIUM |
| home/directories_bar.rs | 620 | OK |

**Total lines in view module:** ~25,000
**Files over 500 lines:** 15+
**Files over 1000 lines:** 4

---

## Duplicate Patterns Found

### Pattern 1: locate_by_id (35+ uses)
```rust
if let Some(index) = locate_by_id(self, ViewId::X) {
    // 3-4 line operation
}
```
**Consolidation:** Create `with_child!` macro

### Pattern 2: Toggle Menu (6 methods)
```rust
fn toggle_menu_x(&mut self, ...) {
    // 20+ lines of identical logic
}
```
**Consolidation:** Extract to generic `toggle_menu<T>()`

### Pattern 3: Render Queue (830+ uses)
```rust
rq.add(RenderData::new(id, rect, UpdateMode::X));
```
**Consolidation:** Add View trait methods

### Pattern 4: Menu Creation (50+ uses)
```rust
let menu = Menu::new(...);
rq.add(RenderData::new(...));
self.children.push(...);
```
**Consolidation:** Create `add_menu()` helper

### Pattern 5: Error Handling (29+ uses)
```rust
match result {
    Ok(v) => { /* handle */ },
    Err(e) => { hub.send(Event::Render(...)).ok(); }
}
```
**Consolidation:** Create error helper trait

---

## Implementation Checklist

### Phase 1: Quick Wins (Week 1)
- [ ] Create `with_child!` macro
- [ ] Add View trait render methods
- [ ] Extract `toggle_menu()` and `add_menu()` helpers
- [ ] Consolidate error handling patterns
- **Expected:** ~1,350 lines saved, improved readability

### Phase 2: Consolidation (Weeks 2-3)
- [ ] Investigate reader stub methods
- [ ] Split reader.rs into 5 modules
- [ ] Split home/mod.rs into 5 modules
- [ ] Create settings registry
- **Expected:** Monolithic files eliminated

### Phase 3: Feature Completion (Weeks 4-5)
- [ ] Complete PDF manipulator file browser
- [ ] Complete cover editor toolbar
- [ ] Wire up batch mode operations
- [ ] Complete dictionary features
- [ ] Add frontlight settings

### Phase 4: Performance (Weeks 6+)
- [ ] Implement metadata caching
- [ ] Add font glyph cache
- [ ] Batch I/O operations
- [ ] Optimize rendering
- [ ] Optimize search

---

## Testing Strategy

### Unit Tests to Add
- Render queue helpers
- Menu toggle helpers
- Settings registry
- Event routing
- Input validators

### Integration Tests to Add
- PDF manipulator with file browser
- Batch operations
- Settings persistence
- Event flow

### Performance Tests
- Memory usage before/after refactoring
- Rendering performance
- Library loading performance
- Search performance

