# Plato Codebase Modularization Plan

## Overview
This plan outlines the steps to modularize the Plato codebase following the AGENTS.md guidelines, with a focus on eliminating files over 1000 lines and ensuring each module has a single responsibility. Backward compatibility is not a concern as instructed.

## Current State Analysis
Based on code analysis, the following files exceed the 1000-line limit mandated by AGENTS.md:

### Critical Violations (>1000 lines)
1. `crates/core/src/document/html/engine.rs` - 2,679 lines
2. `crates/core/src/view/reader/reader_impl/reader.rs` - 2,653 lines  
3. `crates/core/src/document/html/engine_text.rs` - 1,076 lines
4. `crates/core/src/view/home/ui_toggles.rs` - 1,014 lines

### Already Compliant (<1000 lines)
- `crates/core/src/view/home/mod.rs` - 596 lines (8 modules extracted)
- `crates/core/src/font/mod.rs` - 802 lines (modularized with safe wrappers)
- `crates/core/src/view/reader/reader_impl/reader_settings.rs` - 911 lines
- `crates/core/src/document/pdf_manipulator.rs` - 872 lines

## Modularization Strategy

### Phase 1: Split Large Files (>1000 lines)
For each violating file, split into focused submodules by concern:

#### 1.1 HTML Engine (`document/html/engine.rs`)
Split by responsibility:
- `document/html/engine/mod.rs` - Public interface and coordination
- `document/html/engine/dom_parser.rs` - DOM parsing logic
- `document/html/engine/css_parser.rs` - CSS parsing and application
- `document/html/engine/layout_engine.rs` - Layout and text positioning
- `document/html/engine/image_handler.rs` - Image processing and rendering
- `document/html/engine/text_renderer.rs` - Text rendering and shaping
- `document/html/engine/table_handler.rs` - Table-specific logic
- `document/html/engine/event_dispatcher.rs` - Event handling and interaction

#### 1.2 Reader Module (`view/reader/reader_impl/reader.rs`)
Split by responsibility:
- `view/reader/reader_impl/mod.rs` - Public interface
- `view/reader/reader_impl/state.rs` - Reader state management
- `view/reader/reader_impl/rendering.rs` - All rendering logic
- `view/reader/reader_impl/input_handler.rs` - Input and gesture processing
- `view/reader/reader_impl/navigation.rs` - Page navigation and positioning
- `view/reader/reader_impl/annotations.rs` - Annotation handling
- `view/reader/reader_impl/settings_ui.rs` - Settings menu implementations
- `view/reader/reader_impl/search_handler.rs` - Search functionality
- `view/reader/reader_impl/dialog_manager.rs` - Dialog creation and management

#### 1.3 HTML Engine Text (`document/html/engine_text.rs`)
Split by responsibility:
- `document/html/engine_text/mod.rs` - Public interface
- `document/html/engine_text/text_layout.rs` - Text layout algorithms
- `document/html/engine_text/hyphenation.rs` - Hyphenation logic
- `document/html/engine_text/text_shaping.rs` - Text shaping with HarfBuzz
- `document/html/engine_text/font_cache.rs` - Font glyph caching
- `document/html/engine_text/line_breaker.rs` - Line breaking logic
- `document/html/engine_text/text_renderer.rs` - Actual text rendering to pixmap

#### 1.4 Home UI Toggles (`view/home/ui_toggles.rs`)
Split by responsibility:
- `view/home/ui_toggles/mod.rs` - Public interface
- `view/home/ui_toggles/keyboard_toggle.rs` - Keyboard visibility
- `view/home/ui_toggles/address_bar_toggle.rs` - Address bar visibility
- `view/home/ui_toggles/navigation_bar_toggle.rs` - Navigation bar visibility
- `view/home/ui_toggles/shelf_view_toggle.rs` - Shelf view toggle
- `view/home/ui_toggles/book_view_toggle.rs` - Book view toggle
- `view/home/ui_toggles/directory_view_toggle.rs` - Directory view toggle
- `view/home/ui_toggles/settings_toggle.rs` - Settings menu toggle
- `view/home/ui_toggles/library_toggle.rs` - Library view toggle

### Phase 2: Extract Shared Patterns
Create common modules for duplicated patterns:

#### 2.1 View Helper Macros (`crates/core/src/view/common.rs`)
- Continue expanding `with_child!` macro family
- Add macros for common view operations
- Ensure all locate_by_id patterns use these helpers

#### 2.2 Menu System Abstraction (`crates/core/src/view/menu_system.rs`)
- Standardize menu creation and management
- Abstract common menu item patterns
- Create reusable menu containers

#### 2.3 Rendering Queue Utilities (`crates/core/src/view/render_queue.rs`)
- Centralize render queue operations
- Create helpers for batched rendering
- Add profiling capabilities if needed

#### 2.4 Error Handling Patterns (`crates/core/src/error_handling.rs`)
- Standardized error creation with context
- Common error types for the application
- Helper functions for frequent error scenarios

### Phase 3: Enforce Single Responsibility
For each new module, ensure:
- One clear purpose per module
- Maximum 1000 lines per file (strive for <800)
- Maximum 50 lines per function (strive for <30)
- Clear separation of data structures, business logic, and I/O
- Use of `pub(crate)` for internal helpers
- Proper trait abstractions where polymorphism is needed

### Phase 4: Dependency Management
- Audit all `use` statements for each module
- Ensure dependencies flow in logical directions
- Prevent circular dependencies by introducing abstractions
- Use `pub mod` for public API, `mod` for internal

### Phase 5: Verification
After each change:
1. `cargo check --target x86_64-unknown-linux-gnu` - Ensure no compilation errors
2. `cargo clippy --target x86_64-unknown-linux-gnu -- -D warnings` - Ensure no new warnings
3. `cargo fmt` - Ensure consistent formatting
4. Run relevant tests to ensure functionality preserved
5. Verify file sizes are under limits

## Implementation Guidelines per AGENTS.md

### File Size
- **Mandatory**: No source file should exceed 1000 lines
- **Target**: Aim for 500-800 lines per file for readability
- **Action**: When approaching 800 lines, plan extraction

### Function Size
- **Mandatory**: No function should exceed 50 lines
- **Target**: Aim for 20-30 lines per function
- **Action**: Extract inner logic into helper functions when approaching 40 lines

### Module Responsibility
- **Mandatory**: Each module should have a single clear responsibility
- **Action**: If a module handles multiple concerns, split it
- **Example**: Separate parsing from rendering, UI from logic

### Large mod.rs Files
- **Mandatory**: When mod.rs grows large, extract into sibling files
- **Action**: If mod.rs > 600 lines, begin extracting to `module_name/*.rs`

### Public API
- **Mandatory**: Use `pub mod` for public API, plain `mod` for internal
- **Action**: Carefully consider what needs to be public
- **Internal helpers**: Use `pub(crate)` when sharing within crate

### Data Separation
- **Mandatory**: Separate data structures, business logic, and I/O
- **Action**: Put structs in `types.rs`, logic in `service.rs`, I/O in `io.rs`

### Traits for Abstraction
- **Mandatory**: Use traits for major components to improve testability
- **Action**: Define traits for services, mock in tests
- **Dependency**: Depend on abstractions, not concrete implementations

## Priority Order
1. Fix critical violations (>1500 lines): HTML engine, Reader module
2. Address moderate violations (1000-1500 lines): HTML engine text, home ui toggles
3. Extract shared patterns to reduce duplication
4. Enforce function and module size limits across codebase
5. Verify all changes with testing and linting

## Success Criteria
- All Rust files under 1000 lines
- All functions under 50 lines
- Each module has a single, clear responsibility documented in its mod.rs
- Zero `#[allow(dead_code)]` without justification
- All builds pass (host and ARM targets)
- All tests pass
- No clippy warnings with `-D warnings`

## References
- AGENTS.md: Modular Design, Modular Architecture, Module Hierarchy sections
- Existing modularized modules as examples (home/, font/)
- Rust API design guidelines

This plan provides a clear, actionable path to fully modularize the Plato codebase while strictly adhering to the AGENTS.md guidelines and eliminating backward compatibility concerns.