# Mu-OUT: Complete MuPDF Replacement with PDFPurr

## Executive Summary

This document outlines a comprehensive plan to replace MuPDF (C library) with PDFPurr (pure Rust) plus custom e-ink display drivers in Plato. This migration will eliminate FreeType+HarfBuzz dependencies and achieve a 100% Rust PDF rendering stack.

**Goal**: Remove MuPDF, FreeType, and HarfBuzz entirely from Plato, replacing them with PDFPurr + custom e-ink optimization layer.

**Impact**:

- Eliminates 3 C library dependencies
- Reduces build complexity (no cross-compilation of C libraries)
- Enables pure Rust PDF rendering with e-ink optimization
- Maintains feature parity with current MuPDF implementation

---

## Current State Analysis

### MuPDF Usage in Plato

**Location**: `crates/core/src/document/mupdf/` (~1,867 lines)

| Module      | File            | Lines | Purpose                       |
|-------------|-----------------|-------|-------------------------------|
| FFI sys     | `mupdf_sys.rs`  | 531   | Core types/FFI bindings       |
| Context     | `context.rs`    | 164   | Document context management   |
| Document    | `document.rs`   | 213   | PDF document operations       |
| Page        | `page.rs`       | 238   | Page rendering and operations |
| Text        | `text.rs`       | 245   | Text extraction and analysis  |
| Pixmap      | `pixmap.rs`     | 179   | Rendering to bitmaps          |
| Outline     | `outline.rs`    | 102   | Table of contents             |
| Annotations | `annotation.rs` | 71    | PDF annotations               |
| Links       | `link.rs`       | 59    | Hyperlink handling            |
| Images      | `image.rs`      | 35    | Image extraction              |
| Module      | `mod.rs`        | 30    | Re-exports                    |

**Key Features Used**:

- PDF document loading and parsing
- Page rendering to bitmaps
- Text extraction with positions
- Table of contents (outline) parsing
- Annotation reading/writing
- Link extraction
- Image extraction
- Search within PDFs
- Redaction support
- PDF manipulation (merge, split, rotate)

**Dependencies**:

- MuPDF C library (via mupdf_wrapper)
- FreeType (for font rendering within PDFs)
- HarfBuzz (for text shaping within PDFs)
- Third-party C libraries: zlib, bzip2, libpng, libjpeg, openjpeg, jbig2dec, gumbo, djvulibre

### Build System Impact

**Files requiring modification**:

- `crates/core/build.rs` - Remove MuPDF/FreeType/HarfBuzz linking
- `thirdparty/build.sh` - Remove from build list
- `thirdparty/mupdf/kobo.patch` - No longer needed
- `thirdparty/download.sh` - Remove MuPDF download
- `dist.sh` - Remove from distribution bundle

---

## Target State Architecture

### Stack Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                    Plato Application                        │
│  (document loading, UI, library, settings)                  │
├─────────────────────────────────────────────────────────────┤
│              PDF Rendering Abstraction Layer                │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  PdfDocument trait (load, render, extract, etc.)    │  │
│  └──────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│                   PDFPurr (Rust PDF Library)                 │
│  - Document loading and parsing                           │
│  - Page rendering via tiny-skia                            │
│  - Text extraction                                         │
│  - Forms, annotations, encryption                          │
│  - OCR support                                             │
├─────────────────────────────────────────────────────────────┤
│              E-Ink Optimization Layer (Custom)              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │  Partial update tracking (damage regions)            │  │
│  │  Ghosting reduction algorithms                        │  │
│  │  Grayscale quantization (16-level)                   │  │
│  │  Waveform mode selection (GC16, GL16, DU, A2)        │  │
│  │  Delta compression (send only changed pixels)         │  │
│  └──────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────┤
│              Kobo Display Controller Drivers               │
│  ┌──────────────────┐  ┌──────────────────┐              │
│  │  sunxi disp2     │  │  MXC EPDC        │              │
│  │  (Elipsa, Sage)  │  │  (older Kobos)   │              │
│  └──────────────────┘  └──────────────────┘              │
└─────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

**PDFPurr**:

- PDF parsing and loading
- Page rendering to RGBA bitmaps via tiny-skia
- Text extraction with bounding boxes
- Form field handling
- Annotation reading/writing
- Encryption support (RC4, AES)
- OCR integration (optional)
- Metadata extraction

**E-Ink Optimization Layer**:

- **Damage tracking**: Track which regions changed between renders
- **Partial refresh**: Only update changed display regions
- **Ghosting reduction**: Apply clean refresh after N partial updates
- **Grayscale conversion**: Convert RGBA to 16-level grayscale with dithering
- **Waveform selection**: Choose optimal refresh mode (GC16 for quality, DU for speed, A2 for text)
- **Delta compression**: Compress pixel differences before sending to display

**Display Controller Drivers**:

- **sunxi disp2** (Allwinner B300 - Elipsa, Sage):
  - DISP_EINK_UPDATE2 ioctl interface
  - ION buffer management
  - G2D hardware acceleration
  - Waveform LUT programming
- **MXC EPDC** (Freescale i.MX - older Kobos):
  - MXCFB_SEND_UPDATE ioctl
  - EPDC waveform programming
  - TPS65185 PMIC integration
  - Temperature compensation

---

## Implementation Phases

### Phase 1: E-Ink Optimization Layer (Weeks 1-6) ✅ COMPLETE

**Goal**: Build custom e-ink optimization layer independently (PDF-agnostic)

**Status**: COMPLETED - All tasks implemented and tested

**Tasks**:

1. Create e-ink optimization module `crates/core/src/eink/`
   - `mod.rs` - Module exports
   - `damage_tracker.rs` - Track changed regions
   - `grayscale.rs` - RGBA to 16-level grayscale with dithering
   - `waveform.rs` - Waveform mode selection logic
   - `partial_refresh.rs` - Partial update management
   - `ghosting.rs` - Ghosting reduction algorithms

2. Implement damage tracking

   ```rust
   pub struct DamageTracker {
       previous_frame: Option<FrameBuffer>,
       damage_regions: Vec<Rectangle>,
   }
   
   impl DamageTracker {
       pub fn track_changes(&mut self, current: &FrameBuffer) -> Vec<Rectangle>;
       pub fn should_full_refresh(&self) -> bool;
   }
   ```

3. Implement grayscale conversion with dithering
   - Floyd-Steinberg dithering for smooth gradients
   - 16-level grayscale quantization
   - Gamma correction for e-ink displays
   - Threshold tuning for optimal contrast

4. Implement waveform selection logic

   ```rust
   pub enum WaveformMode {
       GC16,  // High quality, slow
       GL16,  // Grayscale, medium
       DU,    // Direct update, fast
       A2,    // Monochrome text, very fast
   }
   
   pub fn select_waveform(content_type: ContentType, update_type: UpdateType) -> WaveformMode;
   ```

5. Implement ghosting reduction
   - Track number of partial updates since last full refresh
   - Force full refresh after N partial updates (configurable)
   - Apply clean refresh mode periodically

6. Create display controller abstraction

   ```rust
   pub trait EInkController {
       fn update(&self, region: Rectangle, data: &[u8], waveform: WaveformMode) -> Result<()>;
       fn full_refresh(&self) -> Result<()>;
       fn set_waveform_lut(&self, lut: &[u8]) -> Result<()>;
   }
   
   pub struct SunxiController { /* sunxi disp2 implementation */ }
   pub struct MxcController { /* MXC EPDC implementation */ }
   ```

7. Implement sunxi disp2 driver (Elipsa, Sage)
   - DISP_EINK_UPDATE2 ioctl wrapper
   - ION buffer allocation and management
   - G2D rotation and scaling
   - Waveform LUT programming

8. Implement MXC EPDC driver (older Kobos)
   - MXCFB_SEND_UPDATE ioctl wrapper
   - EPDC waveform programming
   - TPS65185 PMIC control
   - Temperature-based compensation

9. Create test harness for e-ink layer
   - Mock display controller for testing
   - Test with synthetic RGBA buffers
   - Verify damage tracking accuracy
   - Validate grayscale conversion quality

**Deliverables**:

- E-ink optimization module (~1,500 lines)
- Display controller drivers (~800 lines)
- Damage tracking and partial refresh
- Waveform selection algorithms
- Ghosting reduction logic
- Test harness for e-ink layer

**Acceptance Criteria**:

- Partial updates work correctly (only changed regions refresh)
- Ghosting is controlled (no visible artifacts after 10+ updates)
- Grayscale rendering quality meets standards
- Waveform selection is appropriate for content type
- E-ink layer is PDF-agnostic (works with any RGBA input)

---

### Phase 2: MuPDF Feature Extension (Weeks 7-12)

**Goal**: Extend PDFPurr or add custom code to match MuPDF features that PDFPurr lacks

**Tasks**:

1. Audit PDFPurr capabilities vs MuPDF requirements
   - Create feature gap matrix
   - Identify missing features
   - Prioritize by usage frequency in Plato

2. Implement missing text extraction features
   - Character position tracking (if PDFPurr lacks)
   - Complex script support (Arabic, CJK)
   - Text layout preservation

3. Implement missing outline/TOC features
   - PDF outline parsing (if PDFPurr lacks)
   - Nested section support
   - TOC tree building

4. Implement missing annotation features
   - Annotation types not in PDFPurr
   - Custom annotation rendering
   - Annotation persistence

5. Implement missing link features
   - Internal link handling
   - External link extraction
   - Link rectangle preservation

6. Implement missing image features
   - Image format support gaps
   - Image metadata extraction
   - Image compression handling

7. Implement search functionality
   - Full-text search across document
   - Search result highlighting
   - Match navigation

8. Implement PDF manipulation features
   - Merge operations (if PDFPurr lacks)
   - Split operations
   - Page rotation
   - Page removal

9. Implement redaction support
   - Text redaction
   - Image redaction
   - Secure content removal

10. Implement form support
    - Form field reading
    - Form field filling
    - Form saving

11. Implement encryption support
    - Password-protected PDFs
    - RC4/AES encryption handling
    - Permission restrictions

12. Evaluate need for supplementary libraries
    - Use `lopdf` for missing manipulation features
    - Use custom code for rendering gaps
    - Keep MuPDF as fallback for critical missing features

**Deliverables**:

- Feature gap analysis document
- Extended PDFPurr wrapper (~800 lines)
- Supplementary library integrations (lopdf, etc.)
- Test suite for each added feature

**Acceptance Criteria**:

- All MuPDF features have Rust equivalents
- Feature parity verified against test PDFs
- No critical feature gaps remain
- Fallback strategy documented for any remaining gaps

---

### Phase 3: PDFPurr Integration (Weeks 13-16)

**Goal**: Integrate PDFPurr with extended features and connect to e-ink optimization layer

**Tasks**:

1. Add PDFPurr dependency to `crates/core/Cargo.toml`

   ```toml
   [dependencies]
   pdfpurr = "0.4"
   tiny-skia = "0.11"
   lopdf = "0.31"  # For missing manipulation features
   ```

2. Create PDFPurr wrapper module `crates/core/src/document/pdfpurr/`
   - `mod.rs` - Module exports
   - `document.rs` - Document loading and parsing
   - `page.rs` - Page operations
   - `text.rs` - Text extraction
   - `render.rs` - Rendering via tiny-skia
   - `extensions.rs` - Custom extensions from Phase 2

3. Implement `PdfDocument` trait to abstract PDF libraries

   ```rust
   pub trait PdfDocument {
       fn open(path: &Path) -> Result<Self>;
       fn page_count(&self) -> u32;
       fn page(&self, index: u32) -> Result<Page>;
       fn metadata(&self) -> Result<DocumentMetadata>;
       fn render_to_rgba(&self, page: u32) -> Result<FrameBuffer>;
   }
   ```

4. Integrate extended features from Phase 2
   - Connect custom text extraction
   - Connect custom outline parsing
   - Connect custom annotation handling
   - Connect supplementary libraries (lopdf)

5. Connect PDFPurr to e-ink optimization layer
   - Pipe PDFPurr RGBA output to grayscale converter
   - Apply damage tracking to rendered pages
   - Select appropriate waveform for content
   - Route to correct display controller (sunxi or MXC)

6. Migrate basic PDF loading from MuPDF to PDFPurr
   - Replace `PdfOpener` with PDFPurr-based implementation
   - Update `crates/core/src/document/pdf.rs` to use PDFPurr
   - Keep MuPDF as fallback via feature flag

7. Add rendering pipeline integration
   - PDFPurr renders to RGBA
   - E-ink layer converts to grayscale
   - Display controller updates screen
   - Test end-to-end pipeline

8. Test with diverse PDFs
   - Text-only documents
   - Complex layouts
   - Images and graphics
   - Forms and annotations
   - Encrypted PDFs

**Deliverables**:

- Complete PDFPurr wrapper module (~1,000 lines including extensions)
- Integrated rendering pipeline (PDFPurr → E-ink → Display)
- Test suite for integration
- Feature flag for MuPDF fallback

**Acceptance Criteria**:

- PDFPurr loads and renders all test PDFs correctly
- Rendering quality matches MuPDF
- E-ink optimization works with PDFPurr output
- Integration pipeline is stable
- MuPDF fallback works if needed

---

### Phase 4: Performance Optimization (Weeks 17-20)

**Goal**: Optimize performance to match or exceed MuPDF

**Tasks**:

1. Implement caching strategies
   - Cache rendered pages
   - Cache extracted text
   - Cache metadata
   - LRU eviction policy

2. Optimize partial refresh
   - Minimize damage regions
   - Merge adjacent regions
   - Batch small updates

3. Optimize grayscale conversion
   - SIMD acceleration for dithering
   - Lookup tables for quantization
   - Parallel processing for large pages

4. Optimize memory usage
   - Reuse buffers
   - Pool allocations
   - Stream large PDFs

5. Optimize PDFPurr integration
   - Lazy loading of pages
   - Parallel text extraction
   - Incremental rendering

6. Profile and benchmark
   - Compare performance with MuPDF
   - Identify bottlenecks
   - Optimize hot paths

**Deliverables**:

- Performance benchmarks
- Optimized rendering pipeline
- Memory usage profiling

**Acceptance Criteria**:

- Rendering speed matches or exceeds MuPDF
- Memory usage is within limits (<200MB for typical PDF)
- Page turn latency <500ms

---

### Phase 5: Testing and Validation (Weeks 21-24)

**Goal**: Comprehensive testing and validation

**Tasks**:

1. Create test PDF suite
   - Simple text PDFs
   - Complex layouts
   - Images and graphics
   - Forms and annotations
   - Encrypted PDFs
   - Large documents (1000+ pages)

2. Implement automated tests
   - Unit tests for each module
   - Integration tests for workflows
   - Regression tests against MuPDF output

3. Manual testing on devices
   - Test on Kobo Elipsa (sunxi)
   - Test on Kobo Sage (sunxi)
   - Test on older Kobo devices (MXC)
   - Test on emulator (desktop)

4. Performance validation
   - Measure rendering speed
   - Measure battery impact
   - Measure memory usage

5. User acceptance testing
   - Beta testing with real users
   - Gather feedback
   - Fix issues

6. Documentation
   - Update BUILD.md with new dependencies
   - Update DEVELOPMENT_SETUP.md
   - Create migration guide for contributors
   - Document e-ink optimization techniques

**Deliverables**:

- Comprehensive test suite
- Test report with results
- Performance benchmarks
- Updated documentation

**Acceptance Criteria**:

- All tests pass on all target platforms
- Performance meets or exceeds MuPDF
- No critical bugs found in user testing
- Documentation is complete

---

## Risk Assessment

### High-Risk Items

| Risk                          | Probability | Impact | Mitigation                                                              |
|-------------------------------|-------------|--------|-------------------------------------------------------------------------|
| E-ink optimization complexity | High        | High   | Start with simple full refresh, add partial refresh incrementally       |
| PDFPurr feature gaps          | Medium      | High   | Keep MuPDF as fallback during migration, use lopdf for missing features |
| Performance regression        | Medium      | High   | Benchmark early, optimize hot paths, use caching                        |
| Display controller bugs       | Medium      | Medium | Test on actual devices, have fallback to full refresh                   |
| Encryption compatibility      | Low         | Medium | Test with various encryption schemes, have MuPDF fallback               |

### Medium-Risk Items

| Risk                       | Probability | Impact | Mitigation                                        |
|----------------------------|-------------|--------|---------------------------------------------------|
| Text extraction accuracy   | Medium      | Medium | Compare with MuPDF output, tune algorithms        |
| Annotation compatibility   | Medium      | Medium | Test with various annotation types                |
| Form field support         | Low         | Medium | Implement basic forms first, advanced forms later |
| Large document performance | Medium      | Medium | Implement streaming, lazy loading                 |

### Low-Risk Items

| Risk                 | Probability | Impact | Mitigation                                |
|----------------------|-------------|--------|-------------------------------------------|
| Build system changes | Low         | Low    | Keep MuPDF build until migration complete |
| Dependency conflicts | Low         | Low    | Use feature flags for gradual migration   |
| Documentation gaps   | Low         | Low    | Document as we go, review at end          |

---

## Timeline Estimate

| Phase                             | Duration                 | Dependencies       |
|-----------------------------------|--------------------------|--------------------|
| Phase 1: E-Ink Optimization       | 6 weeks                  | None               |
| Phase 2: MuPDF Feature Extension  | 6 weeks                  | None               |
| Phase 3: PDFPurr Integration      | 4 weeks                  | Phase 1, 2         |
| Phase 4: Performance Optimization | 4 weeks                  | Phase 1, 2, 3      |
| Phase 5: Testing and Validation   | 4 weeks                  | Phase 1, 2, 3, 4   |
| **Total**                         | **24 weeks** (~6 months) |                    |

**Critical Path**: Phase 1 + Phase 2 → Phase 3 → Phase 4 → Phase 5

**Parallel Opportunities**:

- Phase 1 (E-Ink) and Phase 2 (Feature Extension) can run in parallel (independent)
- Phase 5 (testing) can start during Phase 4 for completed features

---

## Success Criteria

### Functional Requirements

- ✅ All PDF formats supported (PDF 1.0-2.0)
- ✅ Text extraction with positions matches MuPDF
- ✅ Rendering quality matches MuPDF
- ✅ Annotations read/write works
- ✅ Forms work
- ✅ Encryption works
- ✅ Search works
- ✅ PDF manipulation works
- ✅ Redaction works

### Performance Requirements

- ✅ Page rendering speed ≤ MuPDF (target: <500ms)
- ✅ Memory usage ≤ MuPDF (target: <200MB)
- ✅ Battery impact ≤ MuPDF (target: <5% difference)
- ✅ Partial refresh latency <300ms
- ✅ Full refresh latency <2s

### E-Ink Requirements

- ✅ Partial updates work correctly
- ✅ Ghosting controlled (no artifacts after 10+ updates)
- ✅ Grayscale quality matches MuPDF
- ✅ Waveform selection appropriate
- ✅ Works on all Kobo devices (sunxi and MXC)

### Build Requirements

- ✅ No C library dependencies for PDF rendering
- ✅ Pure Rust build for PDF stack
- ✅ Build time reduced (no C compilation)
- ✅ Cross-compilation simplified
- ✅ Distribution bundle smaller

### Code Quality Requirements

- ✅ All modules under 1,000 lines
- ✅ No unsafe code where feasible
- ✅ Clippy passes with 0 warnings
- ✅ Test coverage >80%
- ✅ Documentation complete

---

## Migration Strategy

### Gradual Migration Approach

**Stage 1**: Foundation Building (Weeks 1-12)

- Build E-Ink optimization layer (independent of PDF library)
- Extend PDFPurr with missing MuPDF features
- Keep MuPDF as default
- Test components independently

**Stage 2**: Integration (Weeks 13-16)

- Integrate PDFPurr with E-Ink layer
- Add feature flag for new stack
- Test integrated pipeline
- Keep MuPDF as fallback

**Stage 3**: Feature-flagged switch (Weeks 17-20)

- Add `use_pdfpurr` feature flag
- Allow users to opt-in to new stack
- Gather feedback
- Fix issues
- Performance validation

**Stage 4**: Default switch and MuPDF removal (Weeks 21-24)

- Make new stack default
- Remove MuPDF code
- Remove C library dependencies
- Update build system
- Final testing

### Rollback Plan

If critical issues are found:

- Revert to MuPDF by disabling feature flag
- Keep MuPDF fallback for 2 releases after default switch
- Document known issues and workarounds

---

## Resource Requirements

### Development Resources

- **Senior Rust Developer**: 1 FTE (24 weeks)
- **E-Ink Specialist**: 0.5 FTE (12 weeks, Phases 1-5)
- **PDF Specialist**: 0.5 FTE (12 weeks, Phases 2-3)
- **QA Engineer**: 0.5 FTE (12 weeks, Phase 5)
- **Technical Writer**: 0.25 FTE (6 weeks, Phase 5)

### Hardware Resources

- **Kobo Elipsa** (for sunxi testing): 1 device
- **Kobo Sage** (for sunxi testing): 1 device
- **Older Kobo device** (for MXC testing): 1 device
- **Desktop machine** (for development/testing): 1 machine

### Software Resources

- **PDFPurr**: v0.4+ (may need custom patches)
- **tiny-skia**: v0.11+
- **Rust toolchain**: stable 1.80+
- **Cross-compilation tools**: arm-linux-gnueabihf, aarch64-linux-gnu

---

## Open Questions

1. **PDFPurr Maturity**: Is PDFPurr production-ready for all use cases?
   - **Action**: Evaluate PDFPurr with test PDF suite in Phase 1
   - **Fallback**: Use lopdf for missing features if needed

2. **E-Ink Waveform Data**: Do we have access to waveform LUT data for all Kobo devices?
   - **Action**: Extract from existing MuPDF integration or obtain from Kobo
   - **Fallback**: Use generic waveforms if device-specific unavailable

3. **Performance**: Can PDFPurr + custom e-ink layer match MuPDF performance?
   - **Action**: Benchmark in Phase 4, optimize as needed
   - **Fallback**: Keep MuPDF for performance-critical paths if needed

4. **Encryption**: Does PDFPurr support all encryption schemes MuPDF supports?
   - **Action**: Test with various encrypted PDFs in Phase 3
   - **Fallback**: Use MuPDF for unsupported encryption schemes

5. **Testing**: How do we ensure quality across all Kobo devices?
   - **Action**: Create device test farm or partner with device owners
   - **Fallback**: Focus on most popular devices first

---

## Appendix A: File Structure

### New Files

```text
crates/core/src/
├── document/
│   ├── pdfpurr/
│   │   ├── mod.rs
│   │   ├── document.rs
│   │   ├── page.rs
│   │   ├── text.rs
│   │   └── render.rs
│   └── pdf.rs (modified to use PDFPurr)
├── eink/
│   ├── mod.rs
│   ├── damage_tracker.rs
│   ├── grayscale.rs
│   ├── waveform.rs
│   ├── partial_refresh.rs
│   ├── ghosting.rs
│   ├── controller.rs (trait)
│   ├── sunxi_controller.rs
│   └── mxc_controller.rs
```

### Modified Files

```text
crates/core/
├── Cargo.toml (add PDFPurr dependencies)
├── build.rs (remove MuPDF/FreeType/HarfBuzz linking)
└── src/
    ├── document/mod.rs (update exports)
    └── document/pdf.rs (use PDFPurr)

thirdparty/
├── build.sh (remove MuPDF from build list)
├── download.sh (remove MuPDF download)
└── mupdf/ (can be deleted after migration)

dist.sh (remove MuPDF from distribution)
```

### Deleted Files

```text
crates/core/src/document/mupdf/ (entire directory)
crates/core/src/document/mupdf_sys.rs
thirdparty/mupdf/ (entire directory after migration)
```

---

## Appendix B: Dependencies

### New Rust Dependencies

```toml
[dependencies]
pdfpurr = "0.4"
tiny-skia = "0.11"
# Existing dependencies retained:
skrifa = "0.41"
rustybuzz = "0.20"
ab_glyph = "0.2"
```

### Removed C Dependencies

- MuPDF (mupdf_wrapper)
- FreeType (freetype2)
- HarfBuzz (harfbuzz)
- zlib (can use Rust zlib)
- bzip2 (can use Rust bzip2)
- libpng (can use Rust png)
- libjpeg (can use Rust jpeg-decoder)
- openjpeg (may need to keep for JPEG2000)
- jbig2dec (may need to keep for JBIG2)
- gumbo (may need to keep for HTML)
- djvulibre (keep for DjVu support)

---

## Appendix C: Testing Strategy

### Unit Tests

- Test each module in isolation
- Mock external dependencies
- Cover edge cases

### Integration Tests

- Test complete workflows
- Use real PDF files
- Compare output with MuPDF

### Device Tests

- Test on actual Kobo devices
- Test on emulator
- Test on desktop

### Regression Tests

- Compare rendering output with MuPDF
- Compare text extraction with MuPDF
- Compare performance with MuPDF

### Performance Tests

- Benchmark rendering speed
- Benchmark memory usage
- Benchmark battery impact

---

## Appendix D: Success Metrics

### Quantitative Metrics

- **Build time**: Reduced by 30% (no C compilation)
- **Binary size**: Reduced by 20% (no C libraries)
- **Rendering speed**: Within 10% of MuPDF
- **Memory usage**: Within 10% of MuPDF
- **Test coverage**: >80%
- **Bug count**: <10 critical bugs in beta

### Qualitative Metrics

- User satisfaction: >4/5 in beta testing
- Code quality: No clippy warnings
- Documentation: Complete and accurate
- Maintainability: All modules under 1,000 lines

---

## Conclusion

This plan provides a comprehensive roadmap for replacing MuPDF with PDFPurr and custom e-ink optimization. The migration is estimated to take 24 weeks and will eliminate all C library dependencies for PDF rendering, achieving a pure Rust PDF stack.

The key challenges are e-ink optimization and feature parity, but these are mitigated by a phased approach with fallback options. The benefits include simplified builds, reduced dependencies, and a more maintainable codebase.

**Next Steps**:

1. Review and approve this plan
2. Assign resources
3. Begin Phase 1: E-Ink Optimization Layer (can run in parallel with Phase 2)
4. Begin Phase 2: MuPDF Feature Extension (can run in parallel with Phase 1)
5. Set up tracking and reporting
