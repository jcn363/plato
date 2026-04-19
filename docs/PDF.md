# PDFPurr Migration - MuPDF Replacement

## Overview

This document tracks the migration from MuPDF (C library) to PDFPurr (pure Rust PDF library) in the Plato codebase.

## Status: BLOCKED - API Compatibility Issues

### Summary

The migration from MuPDF to PDFPurr is blocked by significant API compatibility issues:

- **System has Rust 1.95.0** (dependency blocker resolved)
- **PDFPurr v0.4.0** API is significantly different from MuPDF
- **Wrapper modules created** but don't match actual PDFPurr API
- **74 compilation errors** due to type mismatches and missing methods

### Dependency Status

- **PDFPurr v0.4.0** depends on **zip v8.5.1**
- **zip v8.5.1** depends on **constant_time_eq v0.4.3**
- **constant_time_eq v0.4.3** requires **Rust 1.95.0**
- **Current system**: Rust 1.95.0

### API Compatibility Issues

The PDFPurr wrapper modules created were based on assumptions about the API that don't match reality:

1. **Type mismatches**: PDFPurr uses different types for Rect, Document, Page, etc.
2. **Missing methods**: Many MuPDF methods don't have direct PDFPurr equivalents
3. **Struct fields vs methods**: PDFPurr uses public fields where MuPDF uses methods
4. **Drop cycle**: Circular dependency in Context implementation

### Compilation Errors

```txt
error[E0107]: missing generics for struct `FileOptions`
error[E0308]: mismatched types
error[E0391]: cycle detected when computing drop
error[E0422]: cannot find type in this scope
error[E0599]: no method named
... (74 total errors)
```

### Current State

- **Code**: Reverted to MuPDF (pdf.rs uses mupdf module)
- **Dependencies**: PDFPurr removed from Cargo.toml
- **Modules**: pdfpurr module removed from codebase
- **zip**: Restored to 8.5.1 (compatible with Rust 1.95.0)

## Recommendations

### Option A: Keep MuPDF (Recommended)

Continue using MuPDF for the time being. MuPDF is battle-tested, has e-ink optimizations, and works well on Kobo devices.

**Pros**:

- Stable and mature
- E-ink optimizations already implemented
- No API compatibility issues
- Works on ARM Kobo devices

**Cons**:

- C library requires FFI
- More complex build process
- Not pure Rust

### Option B: Study PDFPurr API and Recreate Wrappers

Requires:

1. Study PDFPurr documentation and source code
2. Create accurate wrapper modules that match the actual API
3. Handle API differences (fields vs methods, types, etc.)
4. Implement missing functionality (e-ink rendering, etc.)
5. Extensive testing

**Time estimate**: 2-3 weeks of dedicated work

### Option C: Wait for PDFPurr to Mature

PDFPurr is in early development (published March 2026) with breaking changes expected. Waiting for a more stable API may be more efficient.

## Future Work (If Migration Proceeds)

1. **Study PDFPurr API**
   - Read PDFPurr documentation thoroughly
   - Examine PDFPurr source code
   - Understand type system and patterns

2. **Create Accurate Wrapper Modules**
   - Match PDFPurr's actual API exactly
   - Handle type conversions properly
   - Implement missing functionality

3. **Handle API Differences**
   - Map MuPDF concepts to PDFPurr equivalents
   - Implement MuPDF methods that don't exist in PDFPurr
   - Handle e-ink rendering custom implementation

4. **Replace MuPDF in Other Files**
   - `crates/core/src/document/pdf_manipulator.rs`
   - `crates/core/src/document/progressive_loader.rs`

5. **Remove MuPDF Build Dependencies**
   - Update `build.sh` to remove MuPDF compilation
   - Update `thirdparty/build.sh` and `download.sh`
   - Remove `mupdf_wrapper` build configuration
   - Remove `mupdf_sys.rs` FFI bindings

6. **Build and Test for ARM Kobo**
   - Build with `--target arm-unknown-linux-gnueabihf`
   - Fix all compilation errors, warnings, and bugs
   - Test e-ink rendering on actual Kobo device

## Files Modified (During Attempted Migration)

- `crates/core/Cargo.toml` - PDFPurr dependency added then removed
- `crates/epub_edit/Cargo.toml` - zip restored to 8.5.1
- `Cargo.toml` - zip restored to 8.5.1 in workspace
- `crates/core/src/document/mod.rs` - pdfpurr module added then removed
- `crates/core/src/document/pdf.rs` - Reverted to use MuPDF
- `PDF.md` - This file updated with current status

## Notes

- PDFPurr is in early development (published March 2026)
- Breaking changes expected in future versions
- API is significantly different from MuPDF
- E-ink optimizations are critical for Kobo devices and not available in PDFPurr
- MuPDF remains the best option for e-ink devices until PDFPurr matures
- Migration requires significant API study and wrapper development
