# PDF Features for Plato

> **Last Updated**: 2026-04-22
> **Related Documents**: [NOT_IMPLEMENTED.md](./NOT_IMPLEMENTED.md)  |  [OCR_TTS.md](./OCR_TTS.md)  |  [BUILD.md](./BUILD.md)

This document catalogs PDF features in Plato, detailing implementation status, technical decisions, and future possibilities. PDF rendering is handled by **PDFPurr** (pure Rust), and PDF manipulation is handled by **lopdf** (pure Rust).

## Quick Reference

 | Feature                                                | Status         | Priority | Implementation Cost |
 |--------------------------------------------------------|----------------|----------|---------------------|
 | [PDF-Native Annotations](#1-pdf-native-annotations)    | ✅ Implemented | P3       | 2/10                |
 | [Document Manipulation](#6-document-manipulation)      | ✅ Implemented | P2       | 3/10                |
 | [Progressive Loading](#7-progressive-document-loading) | ✅ Implemented | P2       | 4/10                |
 | [Resource Extraction](#10-resource-extraction)         | ✅ Implemented | P3       | 5/10                |
 | [Redaction Support](#8-redaction-support)              | ✅ Implemented | P3       | 6/10                |
 | [Native Text Search](#3-native-text-search)            | ✅ Implemented | P2       | 2/10                |
 | Interactive Forms                                      | ❌ By Design   | —        | 8/10                |
 | Digital Signatures                                     | ❌ By Design   | —        | 8/10                |
 | JavaScript Integration                                 | ❌ By Design   | —        | 9/10                |
 | Enhanced Reflow                                        | ❌ By Design   | —        | 9/10                |
 | PDF/A & PDF/X Validation                               | ❌ By Design   | —        | 5/10                |
 | Advanced OCR                                           | ❌ By Design   | —        | 8/10                |

## 1. PDF-Native Annotations

**Current Status**: ✅ IMPLEMENTED - Comprehensive annotation system with import, export, search, and XFDF support

**Implementation**: Annotations are managed using lopdf (pure Rust PDF manipulation library) with full metadata support.

**Available Functions**:

- `PdfAnnotation` - Rich annotation struct with metadata (id, timestamp, author, subject)
- `PdfAnnotationManager` - Import, search, filter, and sort annotations
- `PdfAnnotationExporter` - Export annotations to PDF documents
- `XfdfHandler` - XFDF export/import for cross-platform interoperability
- `AnnotationQuery` - Search and filter annotations by various criteria
- `AnnotationSubtype` - Support for 25+ annotation types (Text, Highlight, Underline, StrikeOut, etc.)

**New Capabilities**:

- **Import existing PDF annotations** - Read annotations from PDFs created by other viewers (Acrobat, Okular, etc.)
- **Search and filter** - Find annotations by type, text content, author, page, or date range
- **XFDF export/import** - Exchange annotations with other PDF tools via Adobe's XFDF format
- **Rich annotation types** - Support for highlights, underlines, strikethroughs, squiggly lines, and more
- **Annotation metadata** - Timestamps (created/modified), author tracking, subject/title, custom properties
- **Sorting** - Sort annotations by date or page number
- **Statistics** - Count annotations by type

**UI Integration**:

- "Export with Annotations" option in PDF Tools menu
- Creates new PDF file (does not modify original)
- Exports to `.annotated.pdf` extension
- "Search Annotations" option in PDF Tools menu - searches for specific annotation types (e.g., highlights)
- "Export to XFDF" option in PDF Tools menu - exports annotations to XFDF format for cross-platform exchange
- "Import from XFDF" option in PDF Tools menu - imports annotations from XFDF files

**How It Works**:

1. **Import**: Use `PdfAnnotationManager::import_annotations()` to read existing annotations from a PDF
2. **Search**: Use `AnnotationQuery` to filter annotations by type, text, author, page, or date
3. **Export**: Use `PdfAnnotationExporter` to write annotations to a new PDF
4. **XFDF**: Use `XfdfHandler::export_to_xfdf()` or `import_from_xfdf()` for cross-platform exchange

**Benefit**: Full annotation interoperability with other PDF tools, powerful search/filtering, and rich metadata support for better organization.

**Estimated Cost (0=Low, 10=High): 2/10** (fully implemented)**

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

 | Field Type   | E-ink Suitability                 |
 |--------------|-----------------------------------|
 | Text input   | ⚠️ Poor - keyboard needed         |
 | Checkbox     | ✅ Good - simple tap              |
 | Radio button | ✅ Good - simple tap              |
 | Dropdown     | ✅ Good - menu selection          |
 | Signature    | ❌ Not practical                  |
 | XFA forms    | ❌ Not supported in PDF libraries |

### Verdict

Not recommended - forms rare in e-books, poor e-ink UX for text input, high development cost for minimal benefit. Forms work for viewing, but filling should be done on desktop.

## 3. Native Text Search

**Current Status**: ✅ IMPLEMENTED - PDF text search is available
**Implementation**: Text search is handled by PDFPurr with support for complex layouts, ligatures, and hyphenation.
**Benefit**: Faster and more reliable search results within PDF documents.

## 4. Enhanced Reflow

**Current Status**: ❌ NOT IMPLEMENTED - By Design
**Estimated Cost (0=Low, 10=High): 9/10 (Very High)**

### Why is Not Implemented

1. **Duplicate Functionality**: Plato already has a working reflow engine using its own HTML layout engine in `document/html/`. It works well for typical e-book use cases.

2. **Overkill for Reading**: Enhanced reflow is designed for complex document workflows (multi-column layouts, document remixing, advanced typography) rather than simple reading reflow. Plato's current engine handles 99% of use cases.

3. **E-ink Display Limitations**: Complex layouts don't render well on e-ink displays. Simple single-column reflow is optimal.

4. **Kobo Hardware Constraints**:
   - 256MB RAM - Enhanced reflow has larger memory footprint
   - Additional code for marginal benefit

### The Verdict

Not recommended - duplicates existing working functionality with high development cost. Better to improve existing HTML layout engine if needed.

## 5. Digital Signatures

**Current Status**: ❌ NOT IMPLEMENTED - By Design (Security + No Use Case)
**Estimated Cost (0=Low, 10=High): 8/10 (High)**

### Why is Not yet Implemented

1. **PDF Library Limited Capability**:
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
   - Certificate storage/management UI
   - Crypto libraries (~1MB+ additional)
   - All for effectively zero user benefit

### This is the Verdict

Not implemented - PDF libraries cannot create signatures, no use case on e-readers, security concerns with key storage.

## 6. Document Manipulation

**Current Status**: ✅ IMPLEMENTED - Core library and UI created
**Implementation**: PDF manipulation is handled by lopdf (pure Rust PDF manipulation library).
**Available Functions**:

- `delete_pages()` - Remove specific pages from a PDF
- `rotate_pages()` - Rotate pages by 90/180/270 degrees
- `extract_pages()` - Extract specific pages to a new PDF
- `reorder_pages()` - Reorder pages in a PDF
- `merge_pdfs()` - Combine multiple PDFs into one

**⚠️ Memory Warnings Implemented**:

- Files >100MB are rejected
- Files >50MB show warning
- PDFs with >500 pages show warning
- Each operation validates before execution

**Benefit**: Basic PDF editing capabilities without needing a separate computer.

## 7. Progressive Document Loading

**Current Status**: ✅ IMPLEMENTED - `ProgressiveDocLoader` created with LRU caching
**Implementation**: Progressive loading is handled by PDFPurr with LRU caching for large documents.
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

**Current Status**: ✅ IMPLEMENTED - `RedactionEditor` struct created
**Implementation**: PDF redaction is handled by lopdf (pure Rust PDF manipulation library).
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

## 9. JavaScript Integration

**Current Status**: ❌ NOT IMPLEMENTED - By Design
**Estimated Cost (0=Low, 10=High): 9/10 (Very High)**

### Why it is Not Implemented

1. **Library Size Increase**: JavaScript engine adds ~500KB-1MB to the binary, which is significant on Kobo's limited storage.

2. **JavaScript in PDFs is Extremely Rare**:
   - Less than 0.1% of e-books contain PDF JavaScript
   - Mostly used for fillable forms, calculators, animations
   - Consumer e-books almost never use it

3. **E-ink Display Limitations**:
   - Animations and interactive content cannot render properly
   - Touch events not routed to PDF JS engine
   - Forms work with basic fields (no JS needed)

4. **Kobo Hardware Constraints**:
   - 256MB RAM - JS runtime needs significant memory
   - Limited CPU for JS execution
   - Battery impact from continuous JS processing

### Implementation Requirements

- JavaScript engine integration
- JS event handling (route touch events to PDF)
- Memory management (JS heap)
- Form UI handling (new UI component)

### Clear Verdict

Not recommended for Kobo because JS in PDFs is virtually nonexistent in e-books, and e-ink displays cannot properly render interactive content. Basic form fields work without JavaScript.

## 10. Resource Extraction

**Current Status**: ✅ IMPLEMENTED - Full resource extraction library and UI created
**Implementation**: Resource extraction is handled by lopdf (pure Rust PDF manipulation library).
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

 | Standard | Purpose             | Typical Users                |
 |----------|---------------------|------------------------------|
 | PDF/A    | Long-term archiving | Archivist, government, legal |
 | PDF/X    | Print production    | Commercial printing          |

### Why will Not be Implemented

1. **No Use Case on E-readers**:
   - E-readers are for reading, not document validation
   - <0.0001% of e-books are PDF/A or PDF/X
   - Users who need this use desktop software

2. **Limited Capability**:
   - Can only *detect* basic conformance
   - Cannot fully validate all rules
   - Limited PDF/X support

3. **Implementation Options**:

 | Level  | Cost | Features                   |
 |--------|------|----------------------------|
 | Basic  | 2/10 | Show "This is PDF/A" label |
 | Medium | 5/10 | List conformance levels    |
 | Full   | 8/10 | Full validation details    |

### Real Verdict

Not implemented - no practical use case on e-readers, users who need validation use desktop software.

## 12. Advanced OCR Control

**Current Status**: ❌ NOT IMPLEMENTED - By Design (Same as basic OCR)
**Estimated Cost (0=Low, 10=High): 8/10 (High)**

### Why should Not be Implemented

1. **PDF Libraries Do NOT Include OCR**:
   - PDF libraries can only extract text from EXISTING text layers
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

### Current Verdict

Same as basic OCR (see `doc/OCR_TTS.md`). Not recommended - PDF libraries don't include OCR, hardware constraints on Kobo.

---

## Implementation Status Summary

 | Feature                      | Status         | Implementation Details                 | Notes                                   |
 |------------------------------|----------------|----------------------------------------|-----------------------------------------|
 | **Native Text Search**       | ✅ Implemented | PDFPurr integration                    | Available via Settings toggle           |
 | **Document Manipulation**    | ✅ Implemented | `PdfManipulator` module                | Delete, rotate, extract, merge, reorder |
 | **PDF-Native Annotations**   | ✅ Implemented | `PdfAnnotationManager` + `XfdfHandler` | Import, export, search, XFDF support    |
 | **Redaction Support**        | ✅ Implemented | `RedactionEditor` struct               | Mark and permanently remove content     |
 | **Resource Extraction**      | ✅ Implemented | `ResourceExtractor`                    | Images, fonts, page analysis            |
 | **Progressive Loading**      | ✅ Implemented | `ProgressiveDocLoader`                 | LRU caching, preloading                 |
 | **Interactive Forms**        | ❌ By Design   | —                                      | Forms rare in e-books, poor e-ink UX    |
 | **Digital Signatures**       | ❌ By Design   | —                                      | No use case, security concerns          |
 | **JavaScript Integration**   | ❌ By Design   | —                                      | JS in PDFs virtually nonexistent        |
 | **Enhanced Reflow**          | ❌ By Design   | —                                      | Duplicates existing HTML engine         |
 | **PDF/A & PDF/X Validation** | ❌ By Design   | —                                      | No use case on e-readers                |
 | **Advanced OCR**             | ❌ By Design   | —                                      | PDF libraries don't include OCR         |

---

## Technical Architecture

**PDF Processing Stack:**

- **Rendering**: [PDFPurr](https://github.com/slintab/pdfpurr) - Pure Rust PDF rendering
- **Manipulation**: [lopdf](https://github.com/J-F-Liu/lopdf) - Pure Rust PDF operations
- **Annotations**: Custom implementation with XFDF interoperability
- **Memory Management**: LRU caching with 256MB Kobo limits

**Design Philosophy**: Prioritize e-ink optimization, memory efficiency, and reading experience over document workflow features.
