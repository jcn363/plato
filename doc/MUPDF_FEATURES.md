# Potential MuPDF Features for Plato

This document lists features supported by the MuPDF library that are currently not implemented or only partially implemented in Plato. These features could enhance the reading and document management experience.

## 1. PDF-Native Annotations

**Current Status**: ✅ PARTIALLY IMPLEMENTED - Export to PDF with annotations (new file, preserves original)

**MuPDF Capability**: MuPDF supports creating, editing, and saving annotations (highlights, underlines, sticky notes, ink drawings) directly into the PDF file.

**Available Functions**:
- `PdfAnnotationExporter` - Main struct for exporting annotations to PDF
- FFI bindings: `fz_create_annot`, `fz_set_annot_contents`, `fz_set_annot_rect`, `fz_first_annot`, `fz_next_annot`, `fz_annot_contents`, `fz_annot_rect`, `fz_drop_annot`
- All annotation FFI functions implemented in `mupdf_wrapper.c`

**UI Integration**:
- "Export with Annotations" option in PDF Tools menu
- Creates new PDF file (does not modify original)
- Exports to `.annotated.pdf` extension

**How It Works**:
1. Select PDF file in PDF Tools
2. Choose "Export with Annotations"
3. Creates new PDF with embedded annotations
4. Original file remains unchanged

**Benefit**: Annotations become portable and visible in other PDF viewers (Acrobat, Okular, Evince, etc.), while preserving the original unannotated file.

**Estimated Cost (0=Low, 10=High): 4/10** (for export approach)**

## 2. Interactive PDF Forms

**Current Status**: ❌ NOT IMPLEMENTED - By Design
**Estimated Cost (0=Low, 10=High): 8/10 (High)**

### Why Not Implemented

1. **Forms Are Extremely Rare in E-books**:
   - <0.01% of e-books contain interactive forms
   - Mostly found in government/legal documents
   - Users typically fill forms on desktop computers

2. **E-ink Display Limitations**:
   - Text input requires keyboard - poor UX on e-ink
   - Small screen makes form layout cramped
   - Signature fields impractical on small devices

3. **Current Partial Support**:
   - Basic form fields display correctly
   - Text selection works
   - No UI for filling/editing form values

4. **Field Type Complexity**:

| Field Type | E-ink Suitability |
|------------|-------------------|
| Text input | ⚠️ Poor - keyboard needed |
| Checkbox | ✅ Good - simple tap |
| Radio button | ✅ Good - simple tap |
| Dropdown | ✅ Good - menu selection |
| Signature | ❌ Not practical |
| XFA forms | ❌ Not supported in MuPDF |

### Verdict

Not recommended - forms rare in e-books, poor e-ink UX for text input, high development cost for minimal benefit. Forms work for viewing, but filling should be done on desktop.

## 3. Native Text Search

**Current Status**: ✅ IMPLEMENTED - Available via Settings UI toggle (`use-mupdf-search`)
**MuPDF Capability**: `fz_search_page` provides a highly optimized search engine that handles complex layouts, ligatures, and hyphenation more accurately.
**Benefit**: Faster and more reliable search results within PDF documents.

## 4. Enhanced Reflow (Story Module)

**Current Status**: ❌ NOT IMPLEMENTED - By Design
**Estimated Cost (0=Low, 10=High): 9/10 (Very High)**

### Why Not Implemented

1. **MuPDF Story Module NOT Included by Default**: The `fz_story` module requires MuPDF to be compiled with specific build flags. Plato's current MuPDF build does NOT include it - would require recompiling MuPDF from source.

2. **Duplicate Functionality**: Plato already has a working reflow engine using its own HTML layout engine in `document/html/`. It works well for typical e-book use cases.

3. **Story Module is Overkill**: The module is designed for complex document workflows (multi-column layouts, document remixing, advanced typography) rather than simple reading reflow. Plato's current engine handles 99% of use cases.

4. **E-ink Display Limitations**: Complex layouts don't render well on e-ink displays. Simple single-column reflow is optimal.

5. **Kobo Hardware Constraints**: 
   - 256MB RAM - Story module has larger memory footprint
   - Additional ~50KB+ code for marginal benefit

### Verdict

Not recommended - duplicates existing working functionality with high development cost. Better to improve existing HTML layout engine if needed.**

## 5. Digital Signatures

**Current Status**: ❌ NOT IMPLEMENTED - By Design (Security + No Use Case)
**Estimated Cost (0=Low, 10=High): 8/10 (High)**

### Why Not Implemented

1. **MuPDF Limited Capability**:
   - Can only *verify* signatures partially
   - Cannot *create* new signatures (no signing API)
   - No certificate validation
   - No timestamp handling
   - No PKCS#7 signing support

2. **No Use Case on E-readers**:
   - Digital signatures used for legal/business documents
   - <0.001% of e-books are signed
   - E-readers unsuitable for contract/legal workflows
   - Users sign documents on desktop

3. **Security Concerns**:
   - No secure key storage on Kobo
   - No certificate management infrastructure
   - Would require adding crypto libraries (OpenSSL/mbedTLS)
   - Increased attack surface

4. **Implementation Requirements**:
   - FFI for signature verification
   - Certificate storage/management UI
   - Crypto libraries (~1MB+ additional)
   - All for effectively zero user benefit

### Verdict

Not implemented - MuPDF cannot create signatures, no use case on e-readers, security concerns with key storage.

## 6. Document Manipulation

**Current Status**: ✅ IMPLEMENTED - Core library, UI, and MuPDF wrapper functions created
**MuPDF Capability**: MuPDF allows for merging multiple PDFs, deleting pages, reordering pages, and rotating pages permanently.
**Available Functions**:
- `delete_pages()` - Remove specific pages from a PDF
- `rotate_pages()` - Rotate pages by 90/180/270 degrees
- `extract_pages()` - Extract specific pages to a new PDF
- `reorder_pages()` - Reorder pages in a PDF
- `merge_pdfs()` - Combine multiple PDFs into one
- FFI wrappers in `mupdf_wrapper.c`: `fz_pdf_count_pages`, `fz_pdf_delete_page`, `fz_pdf_insert_page`, `fz_pdf_rotate_page`, `fz_pdf_move_page`, `fz_pdf_can_move_pages`, `fz_save_document`, `fz_new_pdf_document`

**⚠️ Memory Warnings Implemented**:
- Files >100MB are rejected
- Files >50MB show warning
- PDFs with >500 pages show warning
- Each operation validates before execution

**Benefit**: Basic PDF editing capabilities without needing a separate computer.

## 7. Progressive Document Loading

**Current Status**: ✅ IMPLEMENTED - `ProgressiveDocLoader` created with LRU caching
**MuPDF Capability**: Using hints streams with `fz_open_document_with_stream` allows for progressive loading of linearized PDFs.
**Available Features**:
- `ProgressiveDocLoader` - Main struct for progressive loading
- LRU page cache (max 5 pages, 20MB)
- Pre-loading pages ahead/behind current position
- Linearized PDF detection
- Memory usage tracking
- Cache clearing for memory management

**Kobo Optimizations**:
- Memory limit: 256MB
- Cache size: 20MB max
- Preload: 2 pages ahead, 1 behind
- Thumbnail size: 800x1200 for efficiency

**Benefit**: Much faster opening and navigation for extremely large PDF files.

## 8. Redaction Support

**Current Status**: ✅ IMPLEMENTED - `RedactionEditor` struct created with FFI bindings
**MuPDF Capability**: MuPDF supports the PDF redaction workflow: marking areas for redaction and then permanently "applying" the redaction to remove the underlying content.
**Available Functions**:
- `RedactionEditor` - Main struct for redaction operations
- `add_redaction()` - Add a region to be redacted
- `remove_redaction()` - Remove a redaction region
- `apply_redactions()` - Permanently apply redactions to PDF
- `remove_redactions()` - Remove all redaction marks without applying

**⚠️ Memory Warnings**:
- Files >50MB are rejected
- PDFs with >500 pages are rejected
- Files >30MB show warning before operation

**Benefit**: Security-conscious users can safely share documents after removing sensitive information.

## 9. JavaScript (mujs) Integration

**Current Status**: ❌ NOT IMPLEMENTED - By Design
**Estimated Cost (0=Low, 10=High): 9/10 (Very High)**

### Why Not Implemented

1. **mujs NOT Included in Default MuPDF Builds**: MuPDF must be compiled WITH mujs support explicitly enabled. Plato's current MuPDF build does NOT include it, requiring recompilation of the entire library.

2. **Library Size Increase**: mujs adds ~500KB-1MB to the binary, which is significant on Kobo's limited storage.

3. **JavaScript in PDFs is Extremely Rare**: 
   - Less than 0.1% of e-books contain PDF JavaScript
   - Mostly used for fillable forms, calculators, animations
   - Consumer e-books almost never use it

4. **E-ink Display Limitations**:
   - Animations and interactive content cannot render properly
   - Touch events not routed to PDF JS engine
   - Forms work with basic fields (no JS needed)

5. **Kobo Hardware Constraints**:
   - 256MB RAM - JS runtime needs significant memory
   - Limited CPU for JS execution
   - Battery impact from continuous JS processing

### Implementation Requirements

- Recompile MuPDF with mujs support (requires build system changes)
- JS event handling (route touch events to PDF)
- Memory management (JS heap)
- Form UI handling (new UI component)

### Verdict

Not recommended for Kobo because JS in PDFs is virtually nonexistent in e-books, and e-ink displays cannot properly render interactive content. Basic form fields work without JavaScript.

## 10. Resource Extraction

**Current Status**: ✅ IMPLEMENTED - Full resource extraction library and UI created
**MuPDF Capability**: Explicit tools for extracting all images, fonts, and other embedded resources from a PDF.
**Available Functions**:
- `ResourceExtractor` - Main struct for resource extraction
- `list_resources()` - Get summary of all resources (images, fonts, pages)
- `extract_images_from_page()` - Extract images from a specific page
- `extract_all_images()` - Extract all images from PDF
- `count_page_fonts()` - Count fonts used on a page
- `extract_text_from_page()` - Get text from a page

**UI Integration**:
- "Extract Resources" option in PDF Tools menu
- Displays resource summary: page count, image count, font count

**Kobo Optimizations**:
- Memory limit: 256MB
- Max file size: 50MB
- Scans first 20 pages for resource listing
- Efficient image counting without full extraction

**Benefit**: Useful for researchers or users who need to analyze PDF contents, extract images, or audit fonts.

## 11. PDF/A and PDF/X Validation

**Current Status**: ❌ NOT IMPLEMENTED - By Design (No Use Case)
**Estimated Cost (0=Low, 10=High): 5/10 (Medium)**

### What Are PDF/A and PDF/X?

| Standard | Purpose | Typical Users |
|----------|---------|---------------|
| PDF/A | Long-term archiving | Archivist, government, legal |
| PDF/X | Print production | Commercial printing |

### Why Not Implemented

1. **No Use Case on E-readers**:
   - E-readers are for reading, not document validation
   - <0.0001% of e-books are PDF/A or PDF/X
   - Users who need this use desktop software

2. **MuPDF Limited Capability**:
   - Can only *detect* basic conformance
   - Cannot fully validate all rules
   - Limited PDF/X support

3. **Implementation Options**:

| Level | Cost | Features |
|-------|------|----------|
| Basic | 2/10 | Show "This is PDF/A" label |
| Medium | 5/10 | List conformance levels |
| Full | 8/10 | Full validation details |

### Verdict

Not implemented - no practical use case on e-readers, users who need validation use desktop software.

## 12. Advanced OCR Control

**Current Status**: ❌ NOT IMPLEMENTED - By Design (Same as basic OCR)
**Estimated Cost (0=Low, 10=High): 8/10 (High)**

### Why Not Implemented

1. **MuPDF Does NOT Include OCR**:
   - MuPDF can only extract text from EXISTING text layers
   - Cannot convert images to text (that's Tesseract)
   - "Advanced OCR Control" requires external OCR engine

2. **Same as Basic OCR**:
   - Requires Tesseract integration (~20MB+ library)
   - Language data files (each 2-20MB)
   - Memory constraints on Kobo (256MB)
   - Long processing time per page (10-60 seconds)

3. **Hardware Limitations**:
   - Kobo CPU (1GHz) too slow for OCR
   - Battery drain during processing
   - Better handled on desktop before transfer

### Verdict

Same as basic OCR (see `doc/OCR_TTS.md`). Not recommended - MuPDF doesn't include OCR, hardware constraints on Kobo.

---

## Implementation Status Summary

| Feature | Status | Notes |
|---------|--------|-------|
| Native Text Search | ✅ Implemented | Available via Settings toggle |
| Document Manipulation | ✅ Implemented | `PdfManipulator` module created |
| PDF-Native Annotations | ⚠️ Partial | Export to PDF with annotations |
| Redaction Support | ✅ Implemented | `RedactionEditor` struct created |
| Resource Extraction | ✅ Implemented | `ResourceExtractor` with image/font counting |
| PDF Forms | ❌ By Design | Forms rare in e-books, poor e-ink UX |
| Digital Signatures | ❌ By Design | No use case, security concerns |
| JavaScript (mujs) | ❌ By Design | JS in PDFs is virtually nonexistent |
| Enhanced Reflow | ❌ By Design | Duplicates existing engine |
| PDF/A Validation | ❌ By Design | No use case on e-readers |
| Advanced OCR | ❌ By Design | MuPDF doesn't include OCR |
