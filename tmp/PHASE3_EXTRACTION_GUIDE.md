# Phase 3 - Settings Extraction Guide

## Overview

Phase 3 targets extracting the remaining ~9 setter methods from Reader to reader_settings.rs. These are complex methods due to their interdependencies with Reader state management.

## Challenge Analysis

### Current Architecture Issues

The setter methods (`set_font_size()`, `set_text_align()`, etc.) have these characteristics:

1. **State Management** - Each updates Reader state:
   - Info reader metadata
   - Document state
   - View port settings
   - Cache and text storage

2. **Document Manipulation** - Requires exclusive access:
   - `Arc::strong_count()` check
   - Document locking with poisoning handling
   - Layout/reflow operations

3. **Complex Updates** - Triggers cascading UI updates:
   - Calls `self.update()` - affects rendering
   - Calls `self.update_tool_bar()` - updates toolbar
   - Calls `self.update_bottom_bar()` - updates status bar
   - Clears caches

4. **Inter-dependencies** - Method calls:
   - `set_zoom_mode()` calls `set_contrast_exponent()`
   - `scale_page()` calls `set_zoom_mode()`

## Extraction Strategy

### Option A: Full Extraction (Recommended)

Extract setter methods as Reader impl methods that delegate to reader_settings functions.

**Signature Pattern**:
```rust
// In reader_settings.rs
pub(crate) fn apply_font_size_change(
    info: &mut Option<ReaderInfo>,
    doc: &Arc<Mutex<Box<dyn Document>>>,
    synthetic: bool,
    current_page: &mut usize,
    pages_count: &mut usize,
    cache: &mut BTreeMap<usize, Resource>,
    text: &mut FxHashMap<usize, Vec<BoundedText>>,
    reader_id: Id,
    rect: Rectangle,
    font_size: f32,
    context: &Context,
    hub: &Hub,
    rq: &mut RenderQueue,
) {
    // Handle document manipulation
    // Return new values for Reader fields
}

// In reader.rs - impl Reader
fn set_font_size(...) {
    reader_settings::apply_font_size_change(
        &mut self.info.reader,
        &self.doc,
        self.synthetic,
        &mut self.current_page,
        &mut self.pages_count,
        &mut self.cache,
        &mut self.text,
        self.id,
        self.rect,
        font_size,
        context,
        hub,
        rq,
    );
}
```

**Pros**:
- Modules become more independent
- Cleaner separation of concerns
- reader_settings handles all settings logic

**Cons**:
- Very long parameter lists
- Difficult to maintain
- Some parameters redundant

### Option B: Helper-Based Extraction

Create focused helper functions for the repetitive parts.

**Pattern**:
```rust
// In reader_settings.rs
pub(crate) fn prepare_document_layout(
    doc: &Arc<Mutex<Box<dyn Document>>>,
    dims: (u32, u32),
    dpi: u16,
    font_size: f32,
) -> Result<u32, Error> {
    let mut doc = doc.lock()?;
    doc.layout(dims.0, dims.1, font_size, dpi);
    Ok(doc.pages_count())
}

pub(crate) fn update_page_position(
    synthetic: bool,
    current_page: &mut usize,
    pages_count: &mut usize,
    new_page_count: u32,
) {
    if synthetic {
        *current_page = (*current_page).min(new_page_count as usize - 1);
    } else {
        let ratio = (new_page_count as usize) / *pages_count;
        *pages_count = new_page_count as usize;
        *current_page = (*current_page * ratio).min(*pages_count - 1);
    }
}
```

**Pros**:
- Smaller, more focused functions
- Reusable across multiple setters
- Easier to test

**Cons**:
- Still needs coordination in reader.rs
- Multiple helper calls per setter

### Option C: Keep As-Is + Documentation

Leave setters in reader.rs but add comprehensive documentation.

**Rationale**:
- These are core Reader functionality
- Complex state management requires tight coupling
- View trait methods often stay with main impl
- Documentation + tests sufficient for maintainability

**Pros**:
- Simplest approach
- Minimal risk
- Maintains current clarity

**Cons**:
- Keeps reader.rs at 3,300+ lines
- Doesn't follow modular design goal
- Future maintenance harder

## Recommended Path Forward

### Phase 3.1 - Documentation (0.5 hours)
1. Add detailed comments to setter methods
2. Document state transition flows
3. Create visual diagram of setter interdependencies

### Phase 3.2 - Helper Extraction (2 hours)
1. Extract common patterns to reader_settings helpers
2. Create `prepare_document_layout()` helper
3. Create `update_page_position()` helper
4. Refactor setters to use helpers

### Phase 3.3 - Full Extraction (3-4 hours) - OPTIONAL
1. Extract one setter fully as proof-of-concept
2. Determine parameter list impact
3. Decide whether to continue or revert to Option B

## Individual Setter Methods

### set_font_size (42 lines) - PRIORITY 1
Lines: 744-785
Complexity: High (document reflow)
Dependencies: Arc count check, document lock, layout operation

### set_text_align (38 lines) - PRIORITY 2
Lines: 787-825
Complexity: High (document manipulation)
Dependencies: Arc count check, set_text_align() on doc

### set_font_family (43 lines) - PRIORITY 3
Lines: 827-871
Complexity: High (font path lookup)
Dependencies: DEFAULT_FONT_FAMILY, context.settings

### set_line_height (38 lines) - PRIORITY 4
Lines: 873-911
Complexity: Medium-High
Dependencies: Document set_line_height()

### set_margin_width (55 lines) - PRIORITY 5
Lines: 913-969
Complexity: Medium
Dependencies: None (settings only)

### set_contrast_exponent (12 lines) - PRIORITY 6
Lines: 981-994
Complexity: Low
Dependencies: None (just state update)

### set_contrast_gray (12 lines) - PRIORITY 7
Lines: 996-1009
Complexity: Low
Dependencies: None (just state update)

### set_zoom_mode (26 lines) - PRIORITY 8
Lines: 1011-1036
Complexity: Medium (menu manipulation)
Dependencies: locate_by_id(), TitleMenu access

### set_scroll_mode (14 lines) - PRIORITY 9
Lines: 1038-1053
Complexity: Low
Dependencies: ZoomMode comparison

## Implementation Examples

### Example: Extracting set_contrast_exponent()

**Current Code (12 lines)**:
```rust
fn set_contrast_exponent(
    &mut self,
    exponent: f32,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    if let Some(ref mut r) = self.info.reader {
        r.contrast_exponent = Some(exponent);
    }
    self.contrast.exponent = exponent;
    self.update(None, hub, rq, context);
    self.update_tool_bar(rq, context);
}
```

**Extracted Version**:
```rust
// In reader_settings.rs
pub(crate) fn set_contrast_exponent(
    info: &mut Option<ReaderInfo>,
    contrast: &mut Contrast,
    exponent: f32,
) {
    if let Some(ref mut r) = info {
        r.contrast_exponent = Some(exponent);
    }
    contrast.exponent = exponent;
}

// In reader.rs
fn set_contrast_exponent(
    &mut self,
    exponent: f32,
    hub: &Hub,
    rq: &mut RenderQueue,
    context: &mut Context,
) {
    reader_settings::set_contrast_exponent(
        &mut self.info.reader,
        &mut self.contrast,
        exponent,
    );
    self.update(None, hub, rq, context);
    self.update_tool_bar(rq, context);
}
```

**Benefits**:
- Settings logic isolated in reader_settings
- Short parameter list
- Clear single responsibility
- Easy to test

**Effort**: ~20 minutes per method for the simple ones

## Testing Strategy

After extraction, test:
1. **Unit tests** - Test helpers independently
2. **Integration tests** - Test setters with full Reader state
3. **Regression tests** - Ensure display updates correctly
4. **Edge cases**:
   - Arc count > 1 scenarios
   - Lock poisoning scenarios
   - Invalid font families

## Rollback Plan

If extraction becomes too complex:
1. Revert changes to last stable commit
2. Update PHASE3_ANALYSIS.md with findings
3. Mark as "deferred" with documented reasons
4. Focus on Phase 4 work instead

## Success Metrics

✅ All 9 setters extracted OR  
✅ Helper functions created and working OR  
✅ Clear analysis completed on why extraction isn't worthwhile

## Estimated Timeline

- **Option A (Full Extraction)**: 4-5 hours
- **Option B (Helper-Based)**: 2-3 hours  
- **Option C (Documentation)**: 0.5-1 hour

**Recommendation**: Start with Option B (2-3 hours), reassess after helpers are extracted.

---

**Created**: April 7, 2026  
**Related**: PHASE2_EXTENDED_COMPLETION.md
