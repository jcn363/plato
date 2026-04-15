# Home View Modularization Plan

## Executive Summary

The goal is to split `crates/core/src/view/home/mod.rs` (2786 lines) into focused submodules under 1000 lines each, following AGENTS.md mandates without backward compatibility.

**Status: COMPLETED** - All 8 modules extracted successfully.

## Current Structure

```
home/mod.rs (591 lines - COMPLETED)
├── Extraction completed:
│   ├── home/ops.rs (217 lines) - Document operations
│   ├── home/ui_toggles.rs (1014 lines) - UI toggle methods
│   ├── home/library.rs (117 lines) - Library operations
│   ├── home/fetcher.rs (226 lines) - Background fetcher management
│   ├── home/navigation.rs (159 lines) - Directory/page navigation
│   ├── home/updates.rs (182 lines) - UI state updates
│   ├── home/input.rs (525 lines) - Event handling
│   └── home_utils.rs (41 lines) - Utility functions
└── Existing submodules:
    ├── shelf.rs (217 lines)
    ├── book.rs (366 lines)
    ├── directory.rs (128 lines)
    ├── address_bar.rs (189 lines)
    ├── navigation_bar.rs (405 lines)
    ├── bottom_bar.rs (216 lines)
    ├── library_label.rs (128 lines)
    └── directories_bar.rs (621 lines)
```

## Modularization Plan

### Module 1: home/mod.rs (Core + View Trait)

**Target:** ~400 lines

**Contents:**
- Module docstring
- `mod` declarations
- `pub use` re-exports
- `Home` struct definition (15 fields)
- Helper structs: `Fetcher`, `BookMenuData`
- View trait implementations (`render`, `resize`, `rect`, `rect_mut`, `children`, `children_mut`, `id`)

**Rationale:** Core struct + trait impl is ~250 lines

---

### Module 2: home/ops.rs (Document Operations)

**Target:** ~900 lines

**Contents:**
- `add_document()`
- `set_status()`
- `empty_trash()`
- `rename()`
- `remove()`
- `copy_to()`
- `move_to()`
- `set_reverse_order()`
- `set_sort_method()`
- `sort()`

**Rationale:** These are cohesive CRUD operations on documents

---

### Module 3: home/ui_toggles.rs (UI Toggle Methods)

**Target:** ~900 lines

**Contents:**
- `toggle_keyboard()`
- `toggle_address_bar()`
- `toggle_navigation_bar()`
- `toggle_search_bar()`
- `toggle_rename_document()`
- `toggle_go_to_page()`
- `toggle_sort_menu()`
- `book_index()` (helper)
- `toggle_book_menu()`
- `toggle_library_menu()`

**Rationale:** These are modal/overlay toggle methods

---

### Module 4: home/library.rs (Library/Sync)

**Target:** ~400 lines

**Contents:**
- `load_library()`
- `import()`
- `clean_up()`
- `flush()`

**Rationale:** Library state management

---

### Module 5: home/fetcher.rs (Async Fetchers)

**Target:** ~400 lines

**Contents:**
- `terminate_fetchers()`
- `insert_fetcher()`
- `spawn_child()`
- `reseed()`

**Rationale:** Background fetcher/process management

---

### Module 6: home/navigation.rs (Navigation)

**Target:** ~400 lines

**Contents:**
- `select_directory()`
- `toggle_select_directory()`
- `go_to_page()`
- `go_to_neighbor()`
- `go_to_status_change()`

**Rationale:** Directory/page navigation

---

### Module 7: home/updates.rs (State Updates)

**Target:** ~400 lines

**Contents:**
- `refresh_visibles()`
- `update_first_column()`
- `update_second_column()`
- `update_thumbnail_previews()`
- `update_shelf()`
- `update_top_bar()`
- `update_bottom_bar()`

**Rationale:** UI state refresh/updates

---

### Module 8: home/input.rs (Event Handling)

**Target:** ~500 lines

**Contents:**
- `new()` (constructor - might stay in mod.rs for convenience)
- `handle_event()`

**Rationale:** Event routing

---

## Implementation Order

| Step | Module | Target Lines | Priority |
|------|--------|-------------|------------|
| 1 | ops.rs | ~900 | High |
| 2 | ui_toggles.rs | ~900 | High |
| 3 | library.rs | ~400 | Medium |
| 4 | fetcher.rs | ~400 | Medium |
| 5 | navigation.rs | ~400 | Medium |
| 6 | updates.rs | ~400 | Low |
| 7 | input.rs | ~500 | Low |
| 8 | home/mod.rs | ~400 | Final |

## Refactoring Steps

### Step 1: Create home/ops.rs

1. Create file `home/ops.rs`
2. Add module declaration to `home/mod.rs`
3. Move document operation methods
4. Update imports in both files

### Step 2: Create home/ui_toggles.rs

1. Create file `home/ui_toggles.rs`
2. Add module declaration to `home/mod.rs`
3. Move UI toggle methods
4. Update imports in both files

### Continue for remaining modules...

## Key Requirements

1. **No backward compatibility** - Remove deprecated patterns
2. **Under 1000 lines per file** - Hard limit
3. **Under 50 lines per function** - AGENTS.md mandate
4. **Single responsibility per module** - Cohesive concerns only
5. **Update all imports** - No broken references

## Verification

After each step:
```bash
# Check line count
wc -l crates/core/src/view/home/ops.rs

# Verify compilation
cargo check --target x86_64-unknown-linux-gnu

# Run tests
cargo test -p plato-core --target x86_64-unknown-linux-gnu
```

## Notes

- Consider creating `home/home.rs` as a re-export barrel if needed
- `home_utils.rs` might merge into `home/mod.rs` (only 39 lines)
- Some methods may require extraction into smaller helpers to meet 50-line limit