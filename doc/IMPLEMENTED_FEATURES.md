# Implemented Features

> **Last Updated**: 2026-05-04
> **Related Documents**: [NOT_IMPLEMENTED.md](./NOT_IMPLEMENTED.md) | [CHANGES.md](../CHANGES.md)

This document tracks features that have been successfully implemented in Plato. For a high-level overview of recent changes, see [CHANGES.md](../CHANGES.md).

## Implementation History

The following features were implemented in recent updates (as of April 2026):

1. **Plugin Network Control** - Now checks for network usage in plugins and enforces `allow_network` setting
2. **Cover Editor UI** - Added full UI accessible from Applications menu
3. **External Storage Auto-Import** - Now imports from SD card during regular import
4. **WebDAV Sync** - Added file listing capability, improved sync detection, annotation/reading state sync
5. **Reading Statistics UI** - Added Statistics view accessible from Applications menu
6. **Password-protected Documents** - Infrastructure exists in PDF libraries (auto-handled)
7. **Series Management** - Metadata is fully supported; basic UI via metadata editing
8. **Batch Operations UI** - Added batch mode with delete and move operations
9. **KoboCloud Sync** - Implemented sync_with_kobocloud function for reading progress
10. **EPUB Editor Enhancements** - Added Undo, Preview, and improved error handling
11. **PDF Native Search** - Added PDF text search using PDFPurr
12. **Settings UI Improvements** - Added Manga Mode, PDF Search, Show Time, Show Battery, External Storage, and Dithering toggles to the in-app settings
13. **Manga Mode Navigation** - Implemented right-to-left reading navigation for manga mode (swipe gestures, bottom bar buttons, chapter/bookmark/annotation navigation all reversed)
14. **CBZ/CBR Comic Book Archive Support** - Added ComicDocument type for reading CBZ (ZIP) and CBR (RAR) comic book archives with full page navigation and image rendering
15. **PDF Document Manipulation** - Added full PDF manipulation library and UI with delete, rotate, extract, merge, and reorder pages using lopdf (pure Rust)
16. **Progressive Document Loading** - Added `ProgressiveDocLoader` with LRU caching and preloading for large PDFs
17. **Redaction Support** - Added `RedactionEditor` struct using lopdf for marking and permanently removing content from PDFs
18. **Resource Extraction** - Added `ResourceExtractor` using lopdf for extracting images, fonts, and listing PDF resources
19. **PDF-Native Annotations** - Added `PdfAnnotationExporter` using lopdf for exporting annotations to PDF (new file, preserves original)
20. **Memory Optimizations** - Fixed `get_available_memory_mb()` to actually read `/proc/meminfo`, reduced thumbnail memory by 75% (RGBA→grayscale), fixed page leaks on error paths, added `MAX_CACHED_PAGES` constant
21. **Performance Improvements** - Reduced context cache from 32MB→16MB, fixed Pixmap OOM panics, added PDF/A detection, improved error messages
22. **PDF Annotation Reading** - Added `read_annotations()` to read existing PDF annotations and display count
23. **EPUB Editor Search & Replace** - Added popup UX for searching and replacing text within EPUB chapters
24. **E-ink Crash Safety** - Fixed 11 unsafe mutex lock unwrap() calls in reader that could crash on Kobo OOM
25. **E-ink Touch Targets** - Increased margin cropper button diameter from 30px to 40px for better touch on cold/dry fingers
26. **Cache Memory Safety** - Fixed progressive loader cache size calculation to use actual data length, preventing OOM on Kobo
27. **Render Performance** - Pre-allocated RenderQueue capacity for faster e-ink rendering
28. **Dictionary Safety** - Fixed panic on empty chunk count in dictionary reader
29. **E-ink Notification Visibility** - Increased notification timeout from 4s to 6s, changed to UpdateMode::Full for better e-ink readability
30. **E-ink Keyboard Touch** - Increased keyboard padding ratio from 0.06 to 0.08 for larger, more comfortable key targets on e-ink
31. **E-ink Menu Touch** - Increased menu entry height from 5x to 6x x-height for easier selection with larger fingers
32. **E-ink Slider Visibility** - Increased progress track height from 7px to 12px for better readability on e-ink grayscale
33. **E-ink Keyboard Contrast** - Darkened keyboard background (GRAY12 -> GRAY11) for 27% better key visibility
34. **E-ink Text Contrast** - Improved disabled text contrast: TEXT_NORMAL from GRAY08 (1.85:1) to GRAY05 (3.6:1), TEXT_INVERTED_HARD from GRAY06 (2.7:1) to GRAY09 (3.5:1)
35. **E-ink Border Visibility** - Increased THICKNESS_SMALL from 1.0 to 1.5 so it rounds to 2px on high-DPI devices (was always 1px)
36. **PDF Tools Layout** - Replaced hardcoded pixel values in pdf_manipulator.rs with named constants (PADDING, BUTTON_HEIGHT, BUTTON_SPACING)
37. **E-ink Word Selection** - Increased touch jitter tolerance from 24px to 32px (~2.7mm) for more reliable word selection and link tapping on e-ink touchscreens
38. **E-ink Context Menus** - Increased popup radius from 24px to 32px at all three locations for larger context menu touch targets
39. **E-ink Book Progress** - Increased book card progress bar height from 13px to 16px for better visibility in library view
40. **E-ink Slider Track** - Increased slider track height from 12px to 16px for better visual feedback during font/contrast adjustment
41. **E-ink Selection Quality** - Changed text selection highlight UpdateMode from Fast to Gui for cleaner rendering without ghosting artifacts on e-ink
42. **E-ink Search/Replace DPI** - Added DPI scaling to all hardcoded pixel values in search_replace.rs for consistent appearance across devices
43. **E-ink Input Field Contrast** - Darkened TEXT_BUMP_SMALL background from GRAY14 (93% white) to GRAY13 (87% white) for better input field visibility
44. **E-ink Keyboard Ghosting** - Changed keyboard key press feedback from UpdateMode::Fast to FastMono for monochrome rendering with less ghosting
45. **Library Crash Safety** - Fixed 8+ unsafe unwrap() calls on I/O operations in library.rs (fs::read_dir, entry.metadata, fingerprint, DateTime) that crash on corrupted SD cards or NFS timeouts
46. **CPU Optimization** - Cached 3 regex compilations (PDF page, TOC page, search) as lazy_static constants to avoid repeated compilation on every event handling cycle
47. **Frontlight Graceful Degradation** - App now starts with no-op fallback instead of crashing when frontlight device files are unavailable (emulators, broken hardware)
48. **E-ink Button Ghosting** - Changed press feedback from UpdateMode::Fast (A2 grayscale) to UpdateMode::FastMono (monochrome) across 5 views: button, icon, menu_entry, rounded_button, preset - eliminates grayscale artifacts on button press
49. **Library Crash Safety (continued)** - Fixed remaining 4 unwrap() calls on metadata/fingerprint operations in library.rs
50. **Reader Crash Safety** - Fixed 7 dangerous unwrap() calls in reader/mod.rs (cache.get, selection.as_mut, text_excerpt, doc.dims, child_mut/downcast)
51. **EPUB Editor Performance** - Cached 10 regex compilations as lazy_static constants in epub_edit/src/lib.rs, eliminating repeated regex compilation on every EPUB parse
52. **PDF Manipulation Implementation** - Added PDF manipulation library using lopdf for page insert/delete/rotate, annotations, redactions, and image/font extraction
53. **lazy_static → LazyLock Migration** - Migrated 13 `lazy_static!` instances to `std::sync::LazyLock` across 9 files (constants, keyboard combos, regex patterns, hyphenation, i18n translations, dithering matrices, shelf mutex).
54. **Further Unwrap/Expect Reduction** - Replaced `.unwrap()` with proper error handling in `sync.rs`, `document/html/parse.rs`, and `fetcher/main.rs`
55. **IMPROVEMENTS.md Reorganization** - Condensed and organized improvement logs for better clarity
56. **Pocket/Instapaper Integration** - Full API integration with OAuth (Pocket), username/password (Instapaper), article sync, tag management, archive operations, reading progress sync, highlight export (Readwise/Obsidian), and folder support (Instapaper). Modules: `article.rs`, `pocket.rs`, `instapaper.rs`
