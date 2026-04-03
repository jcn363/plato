# Not Implemented Features

This document lists features mentioned in documentation or settings that are not implemented or only partially implemented.

## Recently Implemented

The following features were implemented in the latest updates:

1. **Plugin Network Control** - Now checks for network usage in plugins and enforces `allow_network` setting
2. **Cover Editor UI** - Added full UI accessible from Applications menu
3. **External Storage Auto-Import** - Now imports from SD card during regular import
4. **WebDAV Sync** - Added file listing capability, improved sync detection, annotation/reading state sync
5. **Reading Statistics UI** - Added Statistics view accessible from Applications menu
6. **Password-protected Documents** - Infrastructure exists in MuPDF (auto-handled)
7. **Series Management** - Metadata is fully supported; basic UI via metadata editing
8. **Batch Operations UI** - Added batch mode with delete and move operations
9. **KoboCloud Sync** - Implemented sync_with_kobocloud function for reading progress
10. **EPUB Editor Enhancements** - Added Undo, Preview, and improved error handling
11. **MuPDF Native Search** - Added option to use MuPDF's `fz_search_page` for PDF text search
12. **Settings UI Improvements** - Added Manga Mode, MuPDF Search, Show Time, Show Battery, External Storage, and Dithering toggles to the in-app settings
13. **PDF Document Manipulation** - Added full PDF manipulation library and UI with delete, rotate, extract, merge, and reorder pages
14. **Progressive Document Loading** - Added `ProgressiveDocLoader` with LRU caching and preloading for large PDFs
15. **Redaction Support** - Added `RedactionEditor` struct with FFI bindings for marking and permanently removing content from PDFs
16. **Resource Extraction** - Added `ResourceExtractor` for extracting images, fonts, and listing PDF resources
17. **PDF-Native Annotations** - Added `PdfAnnotationExporter` for exporting annotations to PDF (new file, preserves original)
18. **Memory Optimizations** - Fixed `get_available_memory_mb()` to actually read `/proc/meminfo`, reduced thumbnail memory by 75% (RGBA→grayscale), fixed page leaks on error paths, added `MAX_CACHED_PAGES` constant
19. **Performance Improvements** - Reduced MuPDF context cache from 32MB→16MB, fixed Pixmap OOM panics, added PDF/A detection, improved error messages
20. **PDF Annotation Reading** - Added `read_annotations()` to read existing PDF annotations and display count
21. **EPUB Editor Search & Replace** - Added popup UX for searching and replacing text within EPUB chapters
22. **E-ink Crash Safety** - Fixed 11 unsafe mutex lock unwrap() calls in reader that could crash on Kobo OOM
23. **E-ink Touch Targets** - Increased margin cropper button diameter from 30px to 40px for better touch on cold/dry fingers
24. **Cache Memory Safety** - Fixed progressive loader cache size calculation to use actual data length, preventing OOM on Kobo
25. **Render Performance** - Pre-allocated RenderQueue capacity for faster e-ink rendering
26. **Dictionary Safety** - Fixed panic on empty chunk count in dictionary reader
27. **E-ink Notification Visibility** - Increased notification timeout from 4s to 6s, changed to UpdateMode::Full for better e-ink readability
28. **E-ink Keyboard Touch** - Increased keyboard padding ratio from 0.06 to 0.08 for larger, more comfortable key targets on e-ink
29. **E-ink Menu Touch** - Increased menu entry height from 5x to 6x x-height for easier selection with larger fingers
30. **E-ink Slider Visibility** - Increased progress track height from 7px to 12px for better readability on e-ink grayscale
31. **E-ink Keyboard Contrast** - Darkened keyboard background (GRAY12 -> GRAY11) for 27% better key visibility
32. **E-ink Text Contrast** - Improved disabled text contrast: TEXT_NORMAL from GRAY08 (1.85:1) to GRAY05 (3.6:1), TEXT_INVERTED_HARD from GRAY06 (2.7:1) to GRAY09 (3.5:1)
33. **E-ink Border Visibility** - Increased THICKNESS_SMALL from 1.0 to 1.5 so it rounds to 2px on high-DPI devices (was always 1px)
34. **PDF Tools Layout** - Replaced hardcoded pixel values in pdf_manipulator.rs with named constants (PADDING, BUTTON_HEIGHT, BUTTON_SPACING)
35. **E-ink Word Selection** - Increased touch jitter tolerance from 24px to 32px (~2.7mm) for more reliable word selection and link tapping on e-ink touchscreens
36. **E-ink Context Menus** - Increased popup radius from 24px to 32px at all three locations for larger context menu touch targets
37. **E-ink Book Progress** - Increased book card progress bar height from 13px to 16px for better visibility in library view
38. **E-ink Slider Track** - Increased slider track height from 12px to 16px for better visual feedback during font/contrast adjustment
39. **E-ink Selection Quality** - Changed text selection highlight UpdateMode from Fast to Gui for cleaner rendering without ghosting artifacts on e-ink
40. **E-ink Search/Replace DPI** - Added DPI scaling to all hardcoded pixel values in search_replace.rs for consistent appearance across devices
41. **E-ink Input Field Contrast** - Darkened TEXT_BUMP_SMALL background from GRAY14 (93% white) to GRAY13 (87% white) for better input field visibility
42. **E-ink Keyboard Ghosting** - Changed keyboard key press feedback from UpdateMode::Fast to FastMono for monochrome rendering with less ghosting
43. **Library Crash Safety** - Fixed 8+ unsafe unwrap() calls on I/O operations in library.rs (fs::read_dir, entry.metadata, fingerprint, DateTime) that crash on corrupted SD cards or NFS timeouts
44. **CPU Optimization** - Cached 3 regex compilations (PDF page, TOC page, search) as lazy_static constants to avoid repeated compilation on every event handling cycle
45. **Frontlight Graceful Degradation** - App now starts with no-op fallback instead of crashing when frontlight device files are unavailable (emulators, broken hardware)
46. **E-ink Button Ghosting** - Changed press feedback from UpdateMode::Fast (A2 grayscale) to UpdateMode::FastMono (monochrome) across 5 views: button, icon, menu_entry, rounded_button, preset - eliminates grayscale artifacts on button press
47. **Library Crash Safety (continued)** - Fixed remaining 4 unwrap() calls on metadata/fingerprint operations in library.rs
48. **Reader Crash Safety** - Fixed 7 dangerous unwrap() calls in reader/mod.rs (cache.get, selection.as_mut, text_excerpt, doc.dims, child_mut/downcast)
49. **EPUB Editor Performance** - Cached 10 regex compilations as lazy_static constants in epub_edit/src/lib.rs, eliminating repeated regex compilation on every EPUB parse
50. **MuPDF Wrapper Expansion** - Added 20+ custom FFI wrapper functions to `mupdf_wrapper.c` for PDF manipulation (page insert/delete/rotate, annotations, redactions, image/font extraction), resolving linker failures from incomplete pre-compiled `libmupdf.so`

---

## Remaining Not Implemented

### Future Enhancements

**Status**: Planning

**Description**: See `doc/MUPDF_FEATURES.md` for potential MuPDF-based enhancements.

---

## Features Explicitly Not Implemented (By Design)

### OCR and TTS

**Status**: Documented as not implemented by design

**Location**: `doc/OCR_TTS.md` and `doc/MUPDF_FEATURES.md` (Section 12)

**Reason**:
- OCR: Hardware limitations (256MB RAM, 1GHz CPU), MuPDF doesn't include OCR (external Tesseract needed), battery impact
- Advanced OCR Control: Same as basic OCR - MuPDF cannot convert images to text
- TTS: No audio subsystem, outside core mission

### JavaScript (mujs) Integration

**Status**: Documented as not implemented by design

**Location**: `doc/MUPDF_FEATURES.md` (Section 9)

**Reason**:
- mujs NOT included in Plato's MuPDF build - requires recompilation
- JavaScript in PDFs is virtually nonexistent (<0.1% of e-books)
- E-ink displays cannot properly render interactive content/animations
- Kobo's 256MB RAM insufficient for JS runtime
- Basic form fields work without JavaScript

### Enhanced Reflow (Story Module)

**Status**: Documented as not implemented by design

**Location**: `doc/MUPDF_FEATURES.md` (Section 4)

**Reason**:
- Story module NOT included in Plato's MuPDF build - requires recompilation
- Plato already has a working HTML reflow engine (in `document/html/`)
- Module is designed for complex document workflows, not simple reading
- E-ink displays don't benefit from complex layouts

### Interactive PDF Forms

**Status**: Documented as not implemented by design

**Location**: `doc/MUPDF_FEATURES.md` (Section 2)

**Reason**:
- Forms are extremely rare in e-books (<0.01%)
- E-ink displays poorly suited for text input (requires keyboard)
- Small screen impractical for complex form layouts
- Basic form fields display correctly (partial support)
- Users typically fill forms on desktop

### Digital Signatures

**Status**: Documented as not implemented by design

**Location**: `doc/MUPDF_FEATURES.md` (Section 5)

**Reason**:
- MuPDF can only verify signatures partially, cannot create new ones
- No use case on e-readers (legal/business documents)
- Security concerns: no secure key storage on Kobo
- Would require adding crypto libraries

### PDF/A and PDF/X Validation

**Status**: Documented as not implemented by design

**Location**: `doc/MUPDF_FEATURES.md` (Section 11)

**Reason**:
- No use case on e-readers (users need desktop software for validation)
- PDF/A/PDF/X virtually never used in e-books
- E-readers are for reading, not professional document workflows
- MuPDF has limited validation capability anyway

---

## Summary

| Feature | Priority | Status |
|---------|----------|--------|
| Plugin Network Control | P1 | ✅ Implemented |
| Cover Editor UI | P1 | ✅ Implemented |
| External Storage Import | P1 | ✅ Implemented |
| WebDAV Sync | P2 | ✅ Implemented |
| Reading Statistics | P2 | ✅ Implemented |
| Password UI | P3 | ✅ Auto-handled |
| KoboCloud Sync | P1 | ✅ Implemented |
| Batch Operations | P2 | ✅ Implemented |
| EPUB Editor Improvements | P2 | ✅ Implemented |
| MuPDF Native Search | P2 | ✅ Implemented |
| Settings UI | P2 | ✅ Implemented |
| PDF Document Manipulation | P2 | ✅ Implemented |
| Progressive Loading | P2 | ✅ Implemented |
| Redaction Support | P3 | ✅ Implemented |
| Resource Extraction | P3 | ✅ Implemented |
| PDF-Native Annotations | P3 | ⚠️ Partial (Export only) |
