# Plan: Replacing MuPDF with Pure Rust PDF Libraries

## Executive Summary

This document outlines a phased plan to replace MuPDF (FFI/C-based PDF library) with pure Rust alternatives in Plato. The goal is to eliminate unsafe FFI code, simplify the build system, and improve memory safety while maintaining feature parity for core functionality.

**Key Decision**: Full replacement is not recommended. A hybrid approach is proposed:
- **Replace**: PDF manipulation, text extraction, matrix math
- **Keep**: Rendering, annotations, redaction, EPUB reflow

---

## 1. Current State Analysis

### 1.1 MuPDF Usage in Plato

| File | Purpose | FFI Functions Used |
|------|---------|---------------------|
| `crates/core/src/document/pdf.rs` | PDF document handling | 35+ FFI calls |
| `crates/core/src/document/pdf_manipulator.rs` | PDF manipulation/annotations | 50+ FFI calls |
| `crates/core/src/document/progressive_loader.rs` | Progressive loading | 8 FFI calls |
| `crates/core/src/document/mupdf_sys.rs` | FFI bindings | 70+ functions |
| `crates/core/src/font/mod.rs` | Font embedding | 2 FFI calls |

### 1.2 MuPDF Features Used

| Feature | Status |
|---------|--------|
| Document opening (file/memory) | Used |
| Page loading/rendering | Used |
| Text extraction (words, lines) | Used |
| Link extraction | Used |
| Metadata (title, author) | Used |
| Password-protected PDFs | Used |
| EPUB reflow | Used |
| PDF manipulation (delete/insert/rotate pages) | Used |
| Annotations (create/read/modify) | Used |
| Redaction | Used |
| Search | Used (via setting) |

### 1.3 Build Dependencies

- `mupdf_wrapper/mupdf_wrapper.c` — Custom C wrapper (376 lines)
- Pre-compiled `libs/libmupdf.so` — Incomplete build
- `build.rs` — Cross-compilation logic for ARM

---

## 2. Goals and Objectives

### 2.1 Primary Goals

1. **Eliminate unsafe FFI code** — Remove `unsafe` blocks tied to MuPDF
2. **Simplify build system** — Remove C wrapper compilation, cross-compilation complexity
3. **Maintain feature parity** — Ensure all existing PDF features work after replacement
4. **Improve memory safety** — Leverage Rust's ownership model

### 2.2 Success Metrics

| Metric | Target |
|--------|--------|
| Unsafe FFI blocks removed | 80%+ |
| Build time reduction | 30% (removing C compilation) |
| Test pass rate | 100% of existing tests |
| Binary size change | < 5% increase |
| Runtime performance | < 10% degradation |

---

## 3. Replacement Strategy: Hybrid Approach

Instead of full replacement, adopt a hybrid strategy:

```
┌─────────────────────────────────────────────────────────────┐
│                    Keep MuPDF (Core)                       │
├─────────────────────────────────────────────────────────────┤
│  • Rendering to pixmap                                      │
│  • Annotations (read/create)                                │
│  • Redaction                                                │
│  • EPUB reflow                                              │
│  • Search (optional)                                        │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                  Replace with Pure Rust                    │
├─────────────────────────────────────────────────────────────┤
│  • PDF manipulation (lopdf)                                │
│  • Text extraction (pdf_oxide)                              │
│  • Matrix math (pure Rust)                                 │
└─────────────────────────────────────────────────────────────┘
```

### 3.1 Library Selection

| Use Case | Library | Version | Reason |
|----------|---------|---------|--------|
| PDF manipulation | lopdf | 0.40+ | Mature, MIT licensed, 169 reverse deps |
| Text extraction | pdf_oxide | 0.3+ | 5× faster than MuPDF, pure Rust |
| Matrix math | — | — | Implement in pure Rust |
| Rendering | Keep MuPDF | — | No adequate Rust replacement |
| Annotations | Keep MuPDF | — | rpdfium lacks full parity |
| EPUB | Keep MuPDF | — | No Rust equivalent |

---

## 4. Implementation Plan

### Phase 1: Matrix Math (Weeks 1-2)

**Goal**: Replace FFI matrix operations with pure Rust

**Changes**:
- Replace `fz_scale`, `fz_concat`, `fz_invert_matrix` with pure Rust implementation
- Create new module `crates/core/src/document/matrix.rs`

**Files to modify**:
- `crates/core/src/document/pdf.rs` (replace FFI calls)
- `crates/core/src/document/mupdf_sys.rs` (remove unused functions)

**Testing**:
- Verify rendering output matches (pixel-identical not required, visually similar)
- Benchmark: ensure no regression in page rendering time

### Phase 2: PDF Manipulation (Weeks 3-6)

**Goal**: Replace PDF edit operations with lopdf

**Changes**:
- Add lopdf dependency to `Cargo.toml`
- Create new module `crates/core/src/document/pdf_editor.rs`
- Replace: delete page, insert page, rotate page, save document

**Functions to replace**:
```rust
// Current (MuPDF FFI)
fz_pdf_delete_page(ctx, doc, number)
fz_pdf_insert_page(ctx, doc, page, after)
fz_pdf_rotate_page(ctx, doc, number, rotation)
fz_save_document(ctx, doc, filename, opts, fmt)
fz_pdf_move_page(ctx, doc, src, dst)
fz_new_pdf_document(ctx)
fz_pdf_count_pages(ctx, doc)
fz_pdf_can_move_pages(ctx, doc)
fz_pdf_output_intent(ctx, doc)
```

**Implementation approach**:
```rust
// New (lopdf)
use lopdf::Document;

// Replace: fz_pdf_delete_page
pub fn delete_page(doc: &mut Document, page_num: usize) -> Result<()> {
    doc.delete_pages(&[page_num as u32])
}

// Replace: fz_pdf_rotate_page  
pub fn rotate_page(doc: &mut Document, page_num: usize, degrees: i32) -> Result<()> {
    let pages = doc.get_pages();
    let page_id = pages.get(&(page_num as u32)).ok_or("Page not found")?;
    // Apply rotation via lopdf
}

// Replace: fz_save_document
pub fn save_document(doc: &Document, path: &Path) -> Result<()> {
    doc.save(path)
}
```

**Testing**:
- Unit tests for each operation
- Integration tests: open PDF → rotate page → save → reopen → verify
- Test edge cases: empty document, single page, last page deletion

### Phase 3: Text Extraction (Weeks 7-10)

**Goal**: Replace text extraction with pdf_oxide for search/indexing

**Changes**:
- Add pdf_oxide dependency to `Cargo.toml`
- Create module `crates/core/src/document/text_extractor.rs`
- Replace: word extraction, line extraction, search

**Note**: Keep MuPDF for display rendering, use pdf_oxide only for text extraction (faster)

**Functions to replace**:
```rust
// Replace with pdf_oxide
mp_new_stext_page_from_page(ctx, page, opts)  // text structure
fz_search_page(ctx, page, text, hits, count)   // search
```

**Implementation approach**:
```rust
// New (pdf_oxide)
use pdf_oxide::PdfReader;

pub struct TextExtractor {
    reader: PdfReader,
}

impl TextExtractor {
    pub fn extract_words(&self, page: usize) -> Vec<Word> {
        // Use pdf_oxide for fast extraction
    }
    
    pub fn search(&self, page: usize, query: &str) -> Vec<Rect> {
        // Use pdf_oxide's search
    }
}
```

**Testing**:
- Compare word extraction output with existing implementation
- Verify search results match
- Benchmark: should be faster than MuPDF

### Phase 4: Annotation Read (Weeks 11-14)

**Goal**: Replace annotation reading with pure Rust (or keep MuPDF)

**Analysis**: 
- rpdfium has annotation support but not full parity
- Recommendation: Keep MuPDF for annotations until rpdfium matures

**Alternative**: If rpdfium is chosen:
- Add rpdfium dependency
- Replace: first_annot, next_annot, annot_contents, annot_rect

**Decision**: Defer to Phase 5 evaluation

### Phase 5: Evaluation & Cleanup (Weeks 15-18)

**Goal**: Assess progress, make final decisions

**Tasks**:
1. Evaluate Phase 4 decision on annotations
2. Remove unused MuPDF FFI bindings
3. Update documentation
4. Final performance testing
5. Update AGENTS.md with new dependencies

---

## 5. Detailed Technical Changes

### 5.1 New Dependencies (Cargo.toml)

```toml
[dependencies]
lopdf = "0.40"  # PDF manipulation
pdf_oxide = "0.3"  # Text extraction

[dev-dependencies]
# Add test PDFs
```

### 5.2 New Module Structure

```
crates/core/src/document/
├── mod.rs           # Add: mod matrix; mod pdf_editor; mod text_extractor;
├── matrix.rs        # NEW: Pure Rust matrix operations
├── pdf_editor.rs    # NEW: PDF manipulation via lopdf
├── text_extractor.rs # NEW: Text extraction via pdf_oxide
├── pdf.rs           # MODIFY: Use new modules, keep MuPDF for rendering
├── pdf_manipulator.rs # MODIFY: Use lopdf for editing
└── mupdf_sys.rs    # MODIFY: Remove replaced functions, keep rendering
```

### 5.3 Code Changes Summary

| File | Changes | Risk |
|------|---------|------|
| `Cargo.toml` | Add lopdf, pdf_oxide | Low |
| `document/mod.rs` | Add 3 new modules | Low |
| `document/matrix.rs` | New file | Low |
| `document/pdf_editor.rs` | New file | Medium |
| `document/text_extractor.rs` | New file | Medium |
| `document/pdf.rs` | Replace 6 FFI calls | Medium |
| `document/pdf_manipulator.rs` | Replace 10 FFI calls | Medium |
| `document/progressive_loader.rs` | Minor changes | Low |

---

## 6. Testing Strategy

### 6.1 Unit Tests

```rust
#[cfg(test)]
mod matrix_tests {
    #[test]
    fn test_scale() { /* ... */ }
    #[test]
    fn test_concat() { /* ... */ }
    #[test]
    fn test_invert() { /* ... */ }
}

#[cfg(test)]
mod pdf_editor_tests {
    #[test]
    fn test_delete_page() { /* ... */ }
    #[test]
    fn test_rotate_page() { /* ... */ }
}
```

### 6.2 Integration Tests

- Open standard PDF → render → compare output
- Open PDF → delete page → save → reopen → verify page count
- Open PDF → rotate page → save → reopen → verify rotation
- Search test: find text in PDF → verify locations

### 6.3 Performance Benchmarks

```
bench_render_page_100x.png     (before/after)
bench_text_extraction_10mb.pdf (before/after)
bench_pdf_save_50_pages.pdf   (before/after)
```

---

## 7. Risk Analysis

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Feature gap in lopdf | Low | High | Validate before Phase 2; fallback to MuPDF |
| Performance regression | Medium | Medium | Benchmark early; optimize if needed |
| pdf_oxide parsing errors | Low | Medium | Test with 100+ PDFs; fallback to MuPDF |
| Annotation parity missing | High | Medium | Keep MuPDF; re-evaluate later |
| EPUB reflow breaks | Low | High | Keep MuPDF for EPUB (no replacement planned) |

---

## 8. Timeline

```
Week  1-2:  Phase 1 - Matrix Math
Week  3-6:  Phase 2 - PDF Manipulation (lopdf)
Week  7-10: Phase 3 - Text Extraction (pdf_oxide)
Week  11-14: Phase 4 - Annotations (evaluate)
Week  15-18: Phase 5 - Cleanup & Final Testing

Total: 18 weeks (~4.5 months)
```

---

## 9. Resources

### 9.1 Developer Tasks

| Phase | Primary Developer | Reviewer |
|-------|------------------|----------|
| Phase 1 | 1 developer | 1 reviewer |
| Phase 2 | 1 developer | 1 reviewer |
| Phase 3 | 1 developer | 1 reviewer |
| Phase 4-5 | 1 developer | 1 reviewer |

### 9.2 Test Resources

- 50+ PDF test files (varied: simple, complex, encrypted, multi-page)
- Performance test suite
- Regression test suite

---

## 10. Post-Implementation

### 10.1 Maintenance

- Update lopdf/pdf_oxide versions quarterly
- Monitor for security vulnerabilities (cargo-audit)
- Track feature gaps in Rust PDF ecosystem

### 10.2 Future Considerations

- **rpdfium**: Monitor for annotation parity
- **PDFPurr**: New library, monitor maturity
- **EPUB**: Consider epub-rs when stable

---

## 11. Summary

| Aspect | Before | After |
|--------|--------|-------|
| FFI functions | 70+ | ~40 (rendering only) |
| Unsafe blocks | Many | Minimal (rendering only) |
| Build complexity | High (C wrapper) | Low (pure Rust) |
| Dependencies | MuPDF + wrapper | lopdf + pdf_oxide |
| Memory safety | FFI risks | Safer (pure Rust) |

**Recommendation**: Execute this plan in 5 phases over 18 weeks, prioritizing low-risk replacements (matrix, PDF manipulation) and deferring complex features (annotations, EPUB) to future evaluation.