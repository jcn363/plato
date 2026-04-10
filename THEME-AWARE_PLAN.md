# Plato Theme-Aware Conversion Plan

## Current Status
✅ Core theme infrastructure implemented:
- Global dark mode state in `crate::theme`
- Theme-aware color helpers in `crate::color`:
  - `background(dark)` 
  - `foreground(dark)`
  - `text_normal(dark) -> [Color; 3]`
  - `text_bump_small(dark) -> [Color; 3]`
  - `separator(dark)`
  - `keyboard_bg(dark)`
- Dark mode toggle in Settings > Display
- Updated ~40 view components to use theme functions

## Remaining Work

### Phase 1: Text and Background Colors (~30 files)
Convert hardcoded color constants to theme-aware functions:

1. **Text Colors** (TEXT_NORMAL variants)
   - `TEXT_INVERTED_HARD` - Need dark variant `DARK_TEXT_INVERTED_HARD`
   - `TEXT_INVERTED_SOFT` - Need dark variant `DARK_TEXT_INVERTED_SOFT`  
   - `TEXT_BUMP_LARGE` - Need dark variant `DARK_TEXT_BUMP_LARGE`
   - Create `text_inverted_hard(dark)`, `text_inverted_soft(dark)`, `text_bump_large(dark)` helpers

2. **Background/Fill Colors**
   - `WHITE` → `background(dark)` 
   - `BLACK` → `foreground(dark)`
   - `SEPARATOR_STRONG` → Need `separator_strong(dark)` helper
   - `KEYBOARD_BG` → Already have `keyboard_bg(dark)`
   - `READING_PROGRESS` → Need `reading_progress(dark)` helper
   - Progress bar colors (`PROGRESS_FULL`, `PROGRESS_EMPTY`, `PROGRESS_VALUE`)

3. **Special Colors**
   - `BATTERY_FILL` → Need `battery_fill(dark)` helper
   - Other domain-specific colors as needed

### Phase 2: Component Updates (~50 files)
Update remaining view components:

**High Priority (Frequently Used):**
- `view/button.rs` - Already done ✅
- `view/menu_entry.rs` - Already done ✅  
- `view/rounded_button.rs` - Already done ✅
- `view/key.rs` - Already done ✅
- `view/notification.rs` - Already done ✅
- `view/label.rs` - Already done ✅
- `view/icon.rs` - Already done ✅

**Medium Priority:**
- `view/slider.rs` - Progress bar colors
- `view/battery.rs` - Battery indicator
- `view/clock.rs` - Time display
- `view/reader/*` - Reader-specific components
- `view/home/*` - Home screen components
- `view/settings/*` - Settings screens
- `view/calculator/*` - Calculator components
- `view/dictionary/*` - Dictionary components

**Lower Priority:**
- `view/epub_editor/*` - EPUB editor (less used)
- `view/sketch/*` - Sketch functionality (Elipsa only)
- `view/touch_events/*` - Touch debugging
- `view/pdf_manipulator/*` - PDF manipulation
- `view/image/*` - Image viewer
- `view/reader/reader_impl/*` - Core reader (complex)

### Phase 3: Theme Refinements
1. **Transitions** - Add smooth crossfade when toggling theme
2. **Persistence** - Ensure dark mode setting survives reboots
3. **System Integration** - Follow Kobo system dark mode if available
4. **Testing** - Add visual regression tests for theme consistency
5. **Documentation** - Update comments and docs for theme system

## Implementation Approach

### Color Helper Functions Needed
Add to `crate::color`:

```rust
pub fn text_inverted_hard(dark: bool) -> [Color; 3] {
    if dark {
        DARK_TEXT_INVERTED_HARD
    } else {
        TEXT_INVERTED_HARD
    }
}

pub fn text_inverted_soft(dark: bool) -> [Color; 3] {
    if dark {
        DARK_TEXT_INVERTED_SOFT
    } else {
        TEXT_INVERTED_SOFT
    }
}

pub fn text_bump_large(dark: bool) -> [Color; 3] {
    if dark {
        DARK_TEXT_BUMP_LARGE
    } else {
        TEXT_BUMP_LARGE
    }
}

pub fn separator_strong(dark: bool) -> Color {
    if dark {
        DARK_SEPARATOR_STRONG  // Need to define
    } else {
        SEPARATOR_STRONG
    }
}

pub fn reading_progress(dark: bool) -> Color {
    if dark {
        DARK_READING_PROGRESS  // Need to define
    } else {
        READING_PROGRESS
    }
}

// ... similar for progress bars, battery, etc.
```

### Conversion Pattern
For each file:
1. Replace `use crate::color::{WHITE, BLACK, ...}` with theme helpers
2. Replace hardcoded colors with function calls:
   - `WHITE` → `background(theme::is_dark_mode())`
   - `BLACK` → `foreground(theme::is_dark_mode())`  
   - `TEXT_NORMAL` → `text_normal(theme::is_dark_mode())[index]`
   - `SEPARATOR_NORMAL` → `separator(theme::is_dark_mode())`
   - etc.

### Priority Order
1. Reader components (most viewed)
2. Home screen components (frequently accessed)  
3. Settings components (where toggle lives)
4. Calculator/dictionary (utility features)
5. Specialized editors (less frequently used)

## Files Needing Updates (Sample)
Based on grep analysis, key files remaining:
- `view/slider.rs` - Progress bar colors
- `view/battery.rs` - Battery UI
- `view/clock.rs` - Time display
- `view/home/directories_bar.rs` - TEXT_BUMP_SMALL
- `view/home/bottom_bar.rs` - WHITE fillers
- `view/reader/bottom_bar.rs` - WHITE fillers
- `view/reader/results_bar.rs` - WHITE fillers
- `view/dictionary/bottom_bar.rs` - WHITE fillers
- `view/epub_editor/mod.rs` - WHITE/BLACK
- `view/sketch/mod.rs` - WHITE/BLACK pen colors
- `view/touch_events/mod.rs` - BLACK/WHITE regions
- `view/pdf_manipulator.rs` - WHITE background
- `view/image.rs` - WHITE background
- `view/reader/page_animation.rs` - WHITE
- `view/presets_list.rs` - WHITE
- `view/page_label.rs` - BLACK/WHITE
- `view/named_input.rs` - BLACK/WHITE
- `view/filler.rs` - Color parameter
- `view/dialog.rs` - BLACK/WHITE background
- `view/calculator/bottom_bar.rs` - WHITE

## Estimated Effort
- Phase 1 (helpers): 2-3 hours
- Phase 2 (components): 20-30 files × 15 min = 5-7.5 hours
- Phase 3 (refinements): 3-5 hours
- Total: ~10-15 hours development time

## Success Criteria
- All builds pass (x86_64, ARM32, ARM64)
- Zero warnings from clippy
- Dark mode toggle persists across reboots
- Visual consistency in both light and dark modes
- No regression in functionality