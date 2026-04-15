# STUB Implementation Plan for Plato Codebase

This document outlines the step-by-step plan to replace stub implementations with real functionality in the Plato codebase, following the guidelines in AGENTS.md.

## Overview

The Plato codebase contains several stub methods that need to be replaced with real implementations. These stubs are primarily found in trait definitions where methods have empty bodies (`{}`) or minimal implementations that don't provide actual functionality.

## Identified Stubs

Based on code analysis, the following stub methods need real implementations:

### 1. Framebuffer Trait (`crates/core/src/framebuffer/mod.rs`)
- `fn shift_region(&mut self, _rect: &Rectangle, _drift: u8) {}` (line 42)
- `fn set_monochrome(&mut self, _enable: bool) {}` (line 51) - Note: Implemented in concrete types
- `fn set_dithered(&mut self, _enable: bool) {}` (line 55) - Note: Implemented in concrete types  
- `fn set_inverted(&mut self, _enable: bool) {}` (line 59) - Note: Implemented in concrete types

### 2. Document Trait (`crates/core/src/document/mod.rs`)
- `fn set_font_family(&mut self, _family_name: &str, _search_path: &str) {}` (line 124)
- `fn set_margin_width(&mut self, _width: i32) {}` (line 128)
- `fn set_text_align(&mut self, _text_align: TextAlign) {}` (line 132)
- `fn set_line_height(&mut self, _line_height: f32) {}` (line 136)
- `fn set_hyphen_penalty(&mut self, _hyphen_penalty: i32) {}` (line 140)
- `fn set_stretch_tolerance(&mut self, _stretch_tolerance: f32) {}` (line 144)
- `fn set_ignore_document_css(&mut self, ignore: bool);` (line 146) - Note: No body, needs implementation

### 3. Document Plugin Trait (`crates/core/src/document/plugin.rs`)
- `fn set_ignore_document_css(&mut self, _ignore: bool) {}` (line 228)

### 4. Frontlight Trait (`crates/core/src/frontlight/standard.rs`)
- `fn set_warmth(&mut self, _value: f32) {}` (line 35)

## Implementation Approach

For each stub, we need to:

1. **Understand the purpose** - Determine what the method should actually do based on its name, documentation, and usage
2. **Check existing implementations** - See if concrete types already implement this functionality
3. **Design the implementation** - Create a proper implementation that follows Plato's architecture
4. **Implement in concrete types** - Add the implementation to each concrete type that implements the trait
5. **Update documentation** - Remove "Not supported" comments when implementing real functionality
6. **Test the changes** - Verify that the implementation works correctly

## Detailed Implementation Plan

### Framebuffer Stubs

#### shift_region
- **Purpose**: Shifts pixel values in a region by a drift value (used for annotation highlighting effect)
- **Current Status**: Empty stub in trait, needs implementation
- **Implementation Approach**: 
  - For each pixel in the rectangle, shift its color value by the drift amount
  - Handle edge cases (drift causing values to go out of bounds)
  - Implement in KoboFramebuffer1 and KoboFramebuffer2 concrete types

#### set_monochrome, set_dithered, set_inverted
- **Note**: These already have implementations in KoboFramebuffer1 and KoboFramebuffer2
- **Action**: Verify implementations are correct and complete
- **Documentation**: Update trait documentation to remove "Not supported on Kobo e-readers" if implementations are valid

### Document Trait Stubs

These methods relate to text layout and styling. Since Plato supports multiple document formats (EPUB, HTML, etc.), implementations will vary by format.

#### set_font_family
- **Purpose**: Sets the font family for text rendering
- **Current Status**: Stub in Document trait
- **Implementation Approach**:
  - EPUB: Already implemented in `crates/core/src/document/epub/document.rs`
  - HTML: Already implemented in `crates/core/src/document/html/mod.rs` and `engine.rs`
  - Need to ensure PDF documents handle this appropriately (may need to fall back or provide alternative)

#### set_margin_width, set_text_align, set_line_height, set_hyphen_penalty, set_stretch_tolerance
- **Purpose**: Various text layout settings
- **Current Status**: Stubs in Document trait
- **Implementation Approach**:
  - EPUB: Implement in `epub/document.rs`
  - HTML: Implement in `html/mod.rs` and `engine.rs` 
  - PDF: Determine if these apply to PDF documents via MuPDF

#### set_ignore_document_css
- **Purpose**: Sets whether to ignore document CSS
- **Current Status**: Declared without body in Document trait, implemented in plugin.rs
- **Implementation Approach**:
  - Implement in Document trait for each document type
  - EPUB: May not apply (EPUB uses its own styling)
  - HTML: Should actually ignore/process CSS based on this flag
  - PDF: May not apply

### Document Plugin Trait Stub

#### set_ignore_document_css
- **Purpose**: Sets whether to ignore document CSS for plugins
- **Current Status**: Stub in Plugin trait
- **Implementation Approach**:
  - Implement in concrete plugin types
  - Store the setting and use it when processing documents

### Frontlight Trait Stub

#### set_warmth
- **Purpose**: Sets the warmth (color temperature) of the frontlight
- **Current Status**: Stub in Standard frontlight implementation
- **Implementation Approach**:
  - Check if hardware supports warmth adjustment
  - If supported, implement actual hardware control
  - If not supported, document why and potentially remove stub

## Implementation Steps

For each stub method, follow these steps:

1. **Analysis Phase**
   - Read method documentation and surrounding code
   - Search for usages of the method to understand how it's called
   - Check if concrete types already have implementations
   - Determine what a proper implementation should do

2. **Design Phase**
   - Design the implementation for each concrete type (KoboFramebuffer1/2, EPUB document, HTML document, etc.)
   - Consider error handling and edge cases
   - Follow Plato's coding conventions from AGENTS.md

3. **Implementation Phase**
   - Implement the method in each concrete type
   - Add proper error handling using `anyhow::Error` where appropriate
   - Use `.with_context()` for meaningful error messages
   - Follow import conventions (std, external, crate::)

4. **Validation Phase**
   - Run `cargo check` to ensure no compilation errors
   - Run `cargo test` to ensure no test regressions
   - Run `cargo fmt` and `cargo clippy` to ensure code quality
   - Verify on host target (`x86_64-unknown-linux-gnu`)

5. **Documentation Phase**
   - Update/remove misleading documentation comments
   - Add implementation details where appropriate
   - Ensure docstrings follow Plato's documentation conventions

## Priority Order

Based on usage and importance:

1. **Framebuffer.shift_region** - Used for annotation highlighting
2. **Document text layout methods** (font_family, margin_width, etc.) - Core reading experience
3. **Frontlight.set_warmth** - Device feature
4. **Document.set_ignore_document_css** - Styling control

## Completion Criteria

A stub is considered implemented when:
- The method has a meaningful implementation (not just `{}` or `unimplemented!()`)
- The implementation follows Plato's architecture and conventions
- All concrete types that implement the trait have proper implementations
- The code compiles without warnings or errors
- Tests pass (both existing and any new tests added)
- Documentation is updated appropriately
- No `#[allow(dead_code)]` or similar attributes are needed for the implementation

## References

- AGENTS.md: Plato's coding guidelines and conventions
- Existing implementations in concrete types for reference
- MuPDF, FreeType, and HarfBuzz safe wrapper documentation
- Kobo device hardware specifications
