# Phase 2: Monolithic File Splitting Plan

## Overview
Split `reader.rs` (4,168 lines) into 5 focused modules to improve maintainability and testability.

## Current Structure Analysis

### First impl Reader block (lines 214-3974): 3,760 lines
Contains most of the functional code organized roughly by feature:
- **Lines 214-280**: Constructor (new, from_html)
- **Lines 281-430**: Annotation and selection management
- **Lines 431-1066**: Font, alignment, contrast settings menus
- **Lines 1067-1230**: Page and search menus
- **Lines 1231-1831**: Font settings, cropping, text extraction
- **Lines 1832-3760**: Core reader operations (navigation, rendering, gestures, events)

### Second impl Reader block (lines 3974-4168): 194 lines
Contains stub methods (marked #[allow(dead_code)])

## Proposed Module Structure

### 1. **reader_core.rs** (~800 lines)
**Responsibility:** State management, navigation, page tracking

**Move from reader.rs:**
- Struct definition and fields
- `new()` and `from_html()` constructors
- Page navigation methods
- State machine (idle, rendering, searching)
- Document and cache management

**Key Methods:**
```rust
pub fn new(...)
pub fn from_html(...)
fn handle_page_change(&mut self, ...)
fn current_page(&self) -> usize
fn pages_count(&self) -> usize
fn toc() -> Option<Vec<TocEntry>>
fn find_page_by_name() -> Option<usize>
```

### 2. **reader_rendering.rs** (~1,000 lines)
**Responsibility:** Rendering, animation, visual updates

**Move from reader.rs:**
- All rendering methods
- Animation rendering
- Text extraction and display
- Result highlighting
- Frame buffer updates

**Key Methods:**
```rust
fn render_animation(...)
fn render_results(...)
fn selection_rect() -> Option<Rectangle>
fn text_excerpt() -> Option<String>
fn text_rect() -> Option<Rectangle>
```

### 3. **reader_gestures.rs** (~600 lines)
**Responsibility:** Touch and gesture handling, input processing

**Move from reader.rs:**
- Gesture event handlers
- Touch/pen interaction
- Button press handlers
- Coordinate transformation
- Swipe, tap, long-press logic

**Key Methods:**
```rust
fn handle_gesture_event(...)
fn handle_touch_event(...)
fn handle_button_event(...)
```

### 4. **reader_annotations.rs** (~600 lines)
**Responsibility:** Annotation management, highlighting, notes

**Move from reader.rs:**
- Annotation creation/modification
- Note editing
- Selection to annotation conversion
- Annotation menu operations
- Bookmark management

**Key Methods:**
```rust
fn toggle_edit_note(...)
fn toggle_annotation_menu(...)
fn toggle_selection_menu(...)
fn toggle_bookmark(...)
pub fn toggle_annotation_menu(...)
pub fn toggle_selection_menu(...)
fn find_annotation_ref(...) -> Option<&Annotation>
```

### 5. **reader_settings.rs** (~700 lines)
**Responsibility:** Reader settings menus and configuration

**Move from reader.rs:**
- All font/contrast/zoom settings menus
- Margin cropping
- Text alignment settings
- Line height settings
- Zoom and scroll mode settings

**Key Methods:**
```rust
fn toggle_font_family_menu(...)
fn toggle_font_size_menu(...)
fn toggle_text_align_menu(...)
fn toggle_line_height_menu(...)
fn toggle_contrast_exponent_menu(...)
fn toggle_contrast_gray_menu(...)
fn toggle_margin_width_menu(...)
fn toggle_page_menu(...)
fn toggle_margin_cropper_menu(...)
fn toggle_zoom_mode_menu(...)
fn set_font_size(...)
fn set_text_align(...)
fn set_font_family(...)
fn set_line_height(...)
fn set_margin_width(...)
fn set_contrast_exponent(...)
fn set_contrast_gray(...)
fn set_zoom_mode(...)
fn set_scroll_mode(...)
```

### 6. **reader_search.rs** (~200 lines)
**Responsibility:** Search functionality and result management

**Move from reader.rs:**
- Search execution
- Result navigation
- Search state tracking
- Search menu handling

**Key Methods:**
```rust
fn toggle_search_menu(...)
fn search(&mut self, ...)
fn search_next(&mut self, ...)
fn search_previous(&mut self, ...)
```

## Implementation Strategy

### Step 1: Create Module Files (1-2 hours)
1. Create `reader/mod.rs` to define modules
2. Create empty module files with stub imports
3. Update `reader.rs` to reference modules

### Step 2: Move Core State (2-3 hours)
1. Move struct definition to `reader_core.rs`
2. Move constructors to `reader_core.rs`
3. Move state management methods
4. Verify struct still compiles

### Step 3: Move Feature Groups (4-6 hours)
1. Move rendering methods to `reader_rendering.rs`
2. Move annotation methods to `reader_annotations.rs`
3. Move settings methods to `reader_settings.rs`
4. Move gesture methods to `reader_gestures.rs`
5. Move search methods to `reader_search.rs`

### Step 4: Create Public APIs (2-3 hours)
1. Define clear boundaries between modules
2. Create pub(crate) functions where needed
3. Update imports in each module
4. Update event handlers to use module APIs

### Step 5: Testing & Verification (1-2 hours)
1. Verify compilation at each step
2. Run formatting check
3. Commit each significant step
4. Create integration tests

## Key Imports Management

**Imports to share across all modules:**
```rust
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::document::Document;
use crate::framebuffer::Framebuffer;
use crate::geom::{Rectangle, Point};
use crate::view::{View, Hub, Bus, Event, RenderQueue, RenderData};
```

**Module-specific imports:**
- `reader_rendering.rs` - Framebuffer, Pixmap, UpdateMode
- `reader_gestures.rs` - GestureEvent, ButtonCode, FingerStatus
- `reader_annotations.rs` - Annotation, TextLocation, toc_as_html
- `reader_settings.rs` - Menu, MenuKind, EntryKind

## Risk Mitigation

1. **Incremental Commits:** Commit after each module group moves
2. **Frequent Builds:** Verify compilation after each step
3. **Preserve Signatures:** Keep public method signatures identical
4. **Backwards Compatibility:** Maintain event handling interface
5. **Integration Tests:** Add tests for cross-module interactions

## Success Criteria

- [ ] No file > 1,200 lines
- [ ] Clear module responsibilities
- [ ] All imports properly scoped
- [ ] Builds successfully with zero warnings
- [ ] All functionality preserved
- [ ] Tests pass
- [ ] Code is formatted correctly
- [ ] Clear git history with logical commits

## Timeline Estimate

- Step 1: 1-2 hours
- Step 2: 2-3 hours
- Step 3: 4-6 hours
- Step 4: 2-3 hours
- Step 5: 1-2 hours
- **Total: 10-17 hours** (compressed to 2-3 intensive days)
