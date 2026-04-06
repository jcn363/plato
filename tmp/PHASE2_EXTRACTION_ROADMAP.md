# Phase 2 Extraction Guide - Incremental Reader.rs Split

## Objective
Split the 4,168-line `reader.rs` file into 6 focused modules with clear responsibilities.

## Modules

### reader_core.rs ✅ COMPLETE
- **Status**: Shared types defined
- **Types Defined**:
  - `State` - Reader state machine
  - `Selection` - Text selection with anchor
  - `Contrast` - Contrast parameters
  - `PageAnimKind`, `AnimState`, `PageAnimation` - Animation types
  - `RenderChunk` - Page rendering unit
  - `Search` - Search state
  - `Resource` - Cached rendered resource
  - `ViewPort` - Viewport configuration

**Future**: Will add constructors (`new()`, `from_html()`) and basic navigation methods.

### reader_annotations.rs 🔨 IN PROGRESS
- **Status**: 1 function extracted, ready for more
- **Location**: Lines ~427-640 (annotations and bookmarks menu code)
- **Extracted**: `toggle_bookmark()` → `toggle_bookmark_at_page()`

**Next to Extract** (in order of complexity):
1. `toggle_bookmark()` ✅ DONE (~15 lines)
2. `find_annotation_ref()` (~7 lines) - Lookup annotation by location
3. `find_annotation_mut()` (~7 lines) - Mutable lookup
4. `toggle_edit_note()` (~43 lines) - Edit note UI
5. `toggle_annotation_menu()` (~69 lines) - Annotation context menu

**Extraction Strategy**:
- These are mostly UI menu code and state lookups
- Can be extracted as helper functions that take Reader fields as parameters
- Model after `toggle_bookmark_at_page()` pattern

**Dependencies**: Reader's `info`, `focus`, `children`, `id`, `rect`

### reader_settings.rs 🔨 NOT STARTED
- **Status**: Ready for extraction
- **Location**: Lines ~876-1700
- **Estimated Size**: ~700 lines

**Methods to Extract** (in order):
1. `toggle_font_family_menu()` (~59 lines) - Font selection menu
2. `toggle_font_size_menu()` (~61 lines) - Font size menu
3. `toggle_text_align_menu()` (~61 lines) - Alignment menu
4. `toggle_line_height_menu()` (~55 lines) - Line height menu
5. `toggle_contrast_exponent_menu()` (~49 lines) - Contrast exponent menu
6. `toggle_contrast_gray_menu()` (~49 lines) - Contrast gray menu
7. `toggle_margin_width_menu()` (~69 lines) - Margin width menu
8. `toggle_page_menu()` (~61 lines) - Page settings menu
9. `toggle_margin_cropper_menu()` (~78 lines) - Margin cropper UI
10. `toggle_title_menu()` and `toggle_selection_menu()` - Additional menus

**Setter Methods** (can be extracted together):
1. `set_font_size()` (~41 lines)
2. `set_text_align()` (~40 lines)
3. `set_font_family()` (~46 lines)
4. `set_line_height()` (~40 lines)
5. `set_margin_width()` (~58 lines)
6. `set_contrast_exponent()` (~15 lines)
7. `set_contrast_gray()` (~15 lines)
8. `set_zoom_mode()` (~27 lines)
9. `set_scroll_mode()` (~17 lines)

**Extraction Strategy**:
- These are highly repetitive UI menu creation code
- Each toggle_*_menu() follows same pattern:
  1. Check if menu exists (locate_by_id)
  2. If yes and enable!=false: return or remove
  3. If no and enable!=true: create menu and push to children
- Can extract pattern into helper macro or function
- Setter methods are heavier - modify document, cache, etc.

**Dependencies**: `Menu`, `MenuEntry`, heavy Reader dependencies

### reader_rendering.rs 🔨 NOT STARTED
- **Status**: Ready for extraction
- **Location**: Lines ~349-450, 2010-2130, 3574-3763
- **Estimated Size**: ~1,000 lines
- **Complexity**: MODERATE-HIGH (interconnected rendering logic)

**Methods to Extract** (in order):
1. `scaling_factor()` (~20 lines) ⭐ EASY - Pure function!
   - Calculate page scaling based on zoom mode
   - Input: rect, margin, zoom_mode
   - Output: f32 scale factor
2. `selection_rect()` (~7 lines) ⭐ EASY - Pure calculation
3. `text_rect()` (~23 lines) - Calculate text bounding box
4. `text_excerpt()` (~33 lines) - Extract text from selection
5. `selected_text()` (~7 lines) - Get current selection text
6. `render_results()` (~13 lines) - Draw search highlights
7. `render_animation()` (~78 lines) - Page transition animation
8. `render()` (~187 lines) - Main rendering method - HARDEST

**Extraction Strategy**:
- Start with pure calculation functions (scaling_factor, selection_rect)
- Move text extraction helpers next
- Then move render_results() and render_animation()
- Leave main `render()` for last (most complex)

**Dependencies**: Document, cache, chunks, text, annotations

### reader_gestures.rs ⚠️ NOT STARTED - LARGE!
- **Status**: Not started
- **Location**: Lines ~2168-3573
- **Estimated Size**: ~1,400 lines
- **Complexity**: VERY HIGH (massive match statement, many branches)

**The Gorilla: `handle_event()` (~1,400 lines)**
- This is the largest method in the file
- Contains event routing for:
  - Gesture events (swipe, pinch, spread, rotate)
  - Button events (clicks, holds, releases)
  - Touch/stylus input
  - Menu selections
  - Search/annotation interactions

**Extraction Strategy for handle_event()**:
1. Break into sub-functions based on event type:
   - `handle_gesture_event()` - All GestureEvent variants
   - `handle_button_event()` - All ButtonStatus variants
   - `handle_finger_event()` - FingerStatus events
   - `handle_menu_selection()` - Menu callbacks
2. Create dispatch table or helper methods
3. Extract incrementally - e.g., each gesture type separately

**Example Extraction Sequence**:
1. Extract swipe handling (~80 lines)
2. Extract pinch/spread handling (~60 lines)
3. Extract button handling (~200 lines)
4. Extract menu selection routing (~100 lines)
5. Etc.

**Dependencies**: ALL OTHER MODULES (this is the glue)

### reader_search.rs 🔨 NOT STARTED
- **Status**: Ready for extraction
- **Location**: Lines ~1418-1471, 4036-4054, 4121-4157
- **Estimated Size**: ~200 lines
- **Complexity**: LOW (mostly stub methods)

**Methods to Extract**:
1. `toggle_search_menu()` (~49 lines) - Search direction menu
2. `toggle_search_bar()` (~10 lines) - Search input UI (stub)
3. `toggle_results_bar()` (~9 lines) - Results display (stub)
4. `search()` (~9 lines) - Execute search (stub)
5. `go_to_results_neighbor()` (~7 lines) - Navigate results (stub)
6. `go_to_results_page()` (~8 lines) - Jump to result (stub)
7. `update_results_bar()` (~4 lines) - Update results UI (stub)

**Extraction Strategy**:
- Most methods are stubs - just extract as-is
- `toggle_search_menu()` creates menu - follows standard pattern
- This is the smallest and easiest module to complete

**Dependencies**: Minimal - mostly Reader's search field and children

## Extraction Workflow

### Step 1: Choose a Method
1. Start with smallest/simplest
2. Or follow priority: core types → helpers → UI → gestures

### Step 2: Analyze Dependencies
```rust
// List what from Reader the method uses:
// - self.field (what fields accessed?)
// - Other methods called (which ones?)
// - External types needed?
```

### Step 3: Extract as Helper Function
```rust
// In target module, create pub(crate) function
pub(crate) fn my_helper(
    // Direct parameters for external state
    current_page: usize,
    rect: Rectangle,
    // ... other fields needed
    rq: &mut RenderQueue,
) {
    // Method body
}
```

### Step 4: Update Reader to Call Helper
```rust
// In reader.rs, replace method with:
fn my_method(&mut self, rq: &mut RenderQueue, context: &mut Context) {
    // Call helper from new module
    my_module::my_helper(
        self.current_page,
        self.rect,
        // ... pass what helper needs
        rq,
    );
}
```

### Step 5: Test & Commit
```bash
cargo check --target x86_64-unknown-linux-gnu
cargo fmt --all
git add -A && git commit -m "Phase 2: Extract method_name to module_name"
```

## Recommended Extraction Order

### Batch 1: Pure Utility Functions (EASY)
1. `scaling_factor()` → reader_rendering
2. `selection_rect()` → reader_rendering
3. `find_page_by_name()` → reader (or new nav module)

**Effort**: 30 minutes total
**Lines**: ~50
**Complexity**: Very Low

### Batch 2: Search Module (EASY)
1. `toggle_search_menu()` → reader_search
2. `toggle_search_bar()` → reader_search
3. `toggle_results_bar()` → reader_search
4. Stub methods

**Effort**: 1 hour
**Lines**: ~90
**Complexity**: Low

### Batch 3: Annotation Helpers (EASY)
1. `find_annotation_ref()` → reader_annotations
2. `find_annotation_mut()` → reader_annotations
3. `toggle_edit_note()` → reader_annotations
4. `toggle_annotation_menu()` → reader_annotations

**Effort**: 1-2 hours
**Lines**: ~130
**Complexity**: Low-Medium

### Batch 4: Rendering Helpers (MEDIUM)
1. `text_excerpt()` → reader_rendering
2. `text_rect()` → reader_rendering
3. `render_results()` → reader_rendering
4. `render_animation()` → reader_rendering

**Effort**: 2-3 hours
**Lines**: ~150
**Complexity**: Medium

### Batch 5: Settings Menus (LARGE, TEDIOUS)
1. All `toggle_*_menu()` functions → reader_settings
2. All `set_*()` setter functions → reader_settings
3. Extract pattern/macro if heavily duplicated

**Effort**: 3-4 hours
**Lines**: ~600+
**Complexity**: Medium (repetitive)

### Batch 6: Event Handling (VERY LARGE, COMPLEX)
1. Break `handle_event()` into smaller functions
2. Create dispatch methods
3. Extract gesture handlers incrementally
4. Extract button handlers
5. Extract menu selection routing

**Effort**: 4-6 hours
**Lines**: ~1,400
**Complexity**: Very High

## Total Estimation
- **Easy Batches (1-3)**: 3-4 hours, ~280 lines
- **Medium Batches (4-5)**: 5-7 hours, ~750 lines
- **Hard Batch (6)**: 4-6 hours, ~1,400 lines

**Total**: 12-17 hours for full split, or ~2-3 iterations

## Useful Patterns

### Menu Creation Helper
Many methods follow this pattern:
```rust
fn toggle_*_menu(&mut self, enable: Option<bool>, rq: &mut RenderQueue) {
    if let Some(index) = locate_by_id(self, view_id) {
        if Some(true) == enable { return; }
        rq.add(RenderData::expose(...));
        self.children.remove(index);
    } else {
        if Some(false) == enable { return; }
        // Create menu
        let menu = Menu::new(...);
        rq.add(RenderData::new(...));
        self.children.push(Box::new(menu));
    }
}
```

Could extract into:
```rust
pub(crate) fn toggle_menu(
    children: &mut Vec<Box<dyn View>>,
    id: ViewId,
    create_menu: impl Fn() -> Box<dyn View>,
    enable: Option<bool>,
    rq: &mut RenderQueue,
) {
    // ... generic implementation
}
```

### Finder Helper
```rust
pub(crate) fn find_child_by_id(
    children: &[Box<dyn View>],
    id: ViewId,
) -> Option<usize> {
    locate_by_id(...) // Use existing helper
}
```

## Gotchas & Tips

1. **Borrowing**: Pass `&mut self.children` not `self` to avoid borrow conflicts
2. **IDs**: Always have access to `self.id` for `RenderData::new(self.id, ...)`
3. **Imports**: Remember to add new module to `mod.rs` and re-export in `pub use`
4. **Testing**: After each extraction, run `cargo check` to catch errors early
5. **Line Count**: Counts are estimates - actual may vary 10-20%

## Success Criteria

✅ All 6 modules have at least some methods
✅ Can extract ~50% of methods (2,000+ lines moved)
✅ Zero compilation warnings
✅ Public API unchanged
✅ Each module has clear responsibility
✅ Dependencies are documented

## Next Steps for Future Work

1. Continue with Batch 2-3 (search, annotations)
2. Extract rendering helpers (Batch 4)
3. Tackle settings (Batch 5)
4. Finally handle event dispatch (Batch 6)
5. Consider extracting navigation as separate module
6. Add tests for extracted modules
