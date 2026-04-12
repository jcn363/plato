# Font Module Migration Plan: Migrate to Safe Wrappers

## Overview
This plan outlines the migration of the font module (`crates/core/src/font/mod.rs`) from direct FFI calls to using the existing safe wrapper modules (`freetype.rs` and `harfbuzz.rs`). This addresses multiple AGENTS.md mandates:
- Reducing file size from 2,400 lines to under 1,000 lines
- Eliminating direct FFI usage in favor of safe wrappers with RAII
- Improving modular design and separation of concerns

## Current State Analysis

### Problems in `font/mod.rs`
1. **Size violation**: 2,400 lines (exceeds 1,000 line limit)
2. **Direct FFI usage**: Throughout the file, raw FreeType (`FT_*`) and HarfBuzz (`hb_*`) calls are made
3. **Unsafe pointer management**: Structs directly hold `*mut FtLibrary`, `*mut FtFace`, `*mut HbFont` pointers
4. **Missing RAII**: Manual resource management instead of leveraging Drop implementations
5. **Monolithic design**: All font handling logic in a single file

### Existing Safe Wrappers
The following safe wrapper modules already exist:
- `freetype.rs`: Provides `Library` and `Face` structs with proper Drop implementations
- `harfbuzz.rs`: Provides `Font` and `Buffer` structs with proper Drop implementations
- These modules wrap FFI calls with safe Rust abstractions

## Migration Strategy

### Phase 1: Preparation and Analysis
1. **Identify all FFI usage points** in `font/mod.rs`
2. **Map current struct responsibilities**:
   - `FontLibrary`: Wraps `*mut FtLibrary`
   - `FontOpener`: Wraps `Rc<FontLibrary>`
   - `Font`: Combines library, face, hb_font, and additional metadata
3. **Determine how to replace each with safe wrappers**:
   - `FontLibrary` → `freetype::Library`
   - `FontOpener` → Could be replaced with direct `freetype::Library` usage or kept as thin wrapper
   - `Font` face and hb_font components → `freetype::Face` and `harfbuzz::Font`

### Phase 2: Implementation Approach
Replace direct FFI usage with safe wrapper method calls:

#### Replacements needed:
| Current (Direct FFI) | Replacement (Safe Wrapper) |
|----------------------|----------------------------|
| `FT_Init_FreeType(&mut lib)` | `freetype::Library::new()` |
| `FT_New_Face(lib.0, path, 0, &mut face)` | `library.new_face(path, 0)?` |
| `FT_Set_Char_Size(face, ...)` | `face.set_char_size(...)` |
| `FT_Load_Glyph(face, ...)` | `face.load_glyph(...)` |
| `FT_Get_Char_Index(face, ...)` | `face.get_char_index(...)` |
| `hb_ft_font_create(face, null)` | `harfbuzz::Font::from_ft_face(&face)` |
| `hb_font_destroy(font)` | Automatic via Drop |
| `FT_Done_Face(face)` | Automatic via Drop |
| `FT_Done_FreeType(lib)` | Automatic via Drop |

#### Struct Changes:
1. **FontLibrary**: Remove entirely, use `freetype::Library` directly
2. **FontOpener**: Evaluate if needed; likely replace with direct `freetype::Library` usage or simple wrapper
3. **Font**:
   - Replace `lib: Rc<FontLibrary>` with `library: freetype::Library` (or `Rc<freetype::Library>` if shared)
   - Replace `face: *mut FtFace` with `face: freetype::Face`
   - Replace `font: *mut HbFont` with `hb_font: harfbuzz::Font`
   - Remove manual Drop implementation (rely on struct field Drop)

### Phase 3: File Splitting
To meet the 1,000-line requirement, split the monolithic file:

#### Proposed Module Structure:
- `mod.rs`: Public re-exports and high-level API
- `library.rs`: Font library discovery and loading logic
- `rasterizer.rs`: Glyph rasterization and caching
- `shaper.rs`: Text shaping and layout logic
- `types.rs`: Font-related type definitions (already exists, move relevant parts)
- `constants.rs`: Font constants (already exists)
- `freetype.rs`: Keep as-is (safe wrapper)
- `harfbuzz.rs`: Keep as-is (safe wrapper)
- `freetype_sys.rs`: Keep as-is (FFI bindings)
- `harfbuzz_sys.rs`: Keep as-is (FFI bindings)
- `freetype_error.rs`: Keep as-is (error types)
- `md_title.rs`: Keep as-is (special style calculation)

### Phase 4: Implementation Details

#### Key Changes in `Font` struct:
**Before:**
```rust
pub struct Font {
    lib: Rc<FontLibrary>,
    face: *mut FtFace,
    font: *mut HbFont,
    size: u32,
    dpi: u16,
    ellipsis: RenderPlan,
    x_heights: (u32, u32),
    space_codepoint: u32,
}
```

**After:**
```rust
pub struct Font {
    library: freetype::Library, // or Rc<freetype::Library> if shared
    face: freetype::Face,
    hb_font: harfbuzz::Font,
    size: u32,
    dpi: u16,
    ellipsis: RenderPlan,
    x_heights: (u32, u32),
    space_codepoint: u32,
}
```

#### Method Updates:
All methods that currently make direct FFI calls need to be updated to call methods on the safe wrapper structs instead.

Example transformation:
**Before:**
```rust
impl Font {
    pub fn set_pixel_sizes(&self, width: u32, height: u32) -> Result<(), Error> {
        unsafe {
            let result = FT_Set_Pixel_Sizes(self.face, width, height);
            if result != FT_ERR_OK {
                bail!("Failed to set pixel sizes: {}", result);
            }
            Ok(())
        }
    }
}
```

**After:**
```rust
impl Font {
    pub fn set_pixel_sizes(&self, width: u32, height: u32) -> Result<(), Error> {
        self.face.set_pixel_sizes(width, height)
    }
}
```

### Phase 5: Verification and Testing
1. **Ensure no direct FFI calls remain** in the font module hierarchy
2. **Verify all existing functionality works** through unit and integration tests
3. **Confirm file sizes** are all under 1,000 lines
4. **Check that Drop implementations** work correctly for automatic cleanup
5. **Validate performance** is not degraded by the abstraction layer

## Acceptance Criteria
1. ✅ Zero direct FFI calls (`FT_*`, `hb_*`) in `src/font/` directory
2. ✅ All font modules under 1,000 lines each
3. ✅ All font-related structs use safe wrappers instead of raw pointers
4. ✅ Proper RAII resource management through Drop implementations
5. ✅ All existing font functionality preserved and working
6. ✅ No regression in text rendering, shaping, or font loading performance

## Estimated Effort
- Analysis and mapping: 2-3 hours
- Implementation: 8-12 hours
- File splitting and reorganization: 3-5 hours
- Testing and verification: 4-6 hours
- **Total: 17-26 hours**

## Dependencies
- Relies on existing safe wrapper modules (`freetype.rs`, `harfbuzz.rs`)
- No new external dependencies required
- Maintains compatibility with existing font API consumers

## Risks and Mitigations
1. **Risk**: Breaking changes to font API
   **Mitigation**: Maintain public API compatibility; only change internal implementation

2. **Risk**: Performance regression from abstraction layers
   **Mitigation**: Safe wrappers already use `#[inline]` where appropriate; benchmark critical paths

3. **Risk**: Missing FFI error handling in wrappers
   **Mitigation**: Verify existing wrappers properly propagate errors; enhance if needed

4. **Risk**: Resource leaks during transition
   **Mitigation**: Leverage Drop implementations in safe wrappers; verify with miri/valgrind

## Implementation Notes
- Follow existing code patterns in the crate (error handling with `anyhow::Result`, logging macros)
- Use `#[inline]` on performance-critical wrapper methods
- Ensure proper thread safety considerations (Send/Sync bounds where needed)
- Maintain backward compatibility for all public font APIs

---
