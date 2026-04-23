# Plato Changelog

## Executive Summary

Recent work focused on completing UI feature implementation, migrating from FFI font dependencies to pure Rust alternatives, and comprehensive code quality improvements. All reserved UI features are now implemented and fully integrated.

## Version 0.9.38 (2026-04-22)

### 🚀 Major Features

- **Complete UI Implementation**: All reserved UI features implemented and integrated
- **Font Migration**: Successfully migrated from FFI dependencies (FreeType/HarfBuzz) to pure Rust (skrifa/rustybuzz/ab_glyph)
- **PDF Manipulator**: Fully functional with annotation copying and text extraction using PDFPurr/lopdf
- **Library Filters**: Implemented format and category filtering with toggle functionality

### 🎨 UI Improvements

- **Dialog System**: AboutDialog, ShareDialog, SystemInfoDialog, EmailDialog, CloudDialog
- **Settings Management**: Font size cycling, zoom modes, scroll modes, text alignment, search margins
- **EPUB Editor GUI**: Removed content truncation, added bulk replace, metadata editing, chapter navigation
- **Advanced Search**: Regex support, case-sensitive toggle, whole-word toggle, search history

### 🏗️ Code Quality

- **Error Handling**: Replaced `unwrap()` calls with proper `expect()` messages across buffer_pool, cache, and pdf_manipulator
- **Input Validation**: Added `validator::Validate` to Settings struct with range and length constraints
- **Documentation**: Enabled `#![warn(missing_docs)]` across all crate entry points
- **Safety**: Added comprehensive `// SAFETY:` comments to all unsafe blocks
- **Testing**: 204 tests passing, clean clippy results

### 🔧 Manager Integration

All 9 WIP module managers now integrated:

- ReaderAnnotationManager (annotation rendering)
- ReaderDialogManager (info dialogs)
- ReaderInputHandler (input processing)
- ReaderRenderCache (cache statistics)
- ReaderRenderEngine (viewport management)
- ReaderSearchHandler (search operations)
- ReaderSettingsManager (settings menu)
- ReaderStateManager (page tracking)
- ReaderTocManager (chapter navigation)

### 📦 Dependencies & Build

- Updated dependencies: signal-hook 0.4.4, reqwest 0.13.2, image 0.25.10
- Standardized workspace metadata
- Clean ARM target build with zero warnings
- All `#[allow(dead_code)]` attributes removed from actionable code

### 📚 EPUB Editor Enhancements

- **Content Management**: Full chapter loading, bulk replace, metadata editing
- **Navigation**: Previous/Next chapter buttons with visual modification indicators
- **Search & Replace**: Advanced options (regex, case-sensitive, whole-word), search history
- **Validation**: HTML structure check, broken link detection, external image detection
- **Chapter Tools**: Rename, delete, reorder chapters with statistics (word/character/paragraph counts)
- **Spell Check**: HTML tag stripping, word extraction, common word filtering
- **Import/Export**: Chapter text file export/import with bookmark management
- **TOC Management**: Generate and update table of contents from chapter list
- **Image Management**: Detect and list all images with chapter locations
- **CSS Tools**: Detect and list CSS files and inline styles
- **Undo/Redo**: Improved history with 50-action limit and clear method

### 🔄 Implementation Timeline

- **Phase 1** (`96eb50b`): Annotation System Integration
- **Phase 2** (`bb797bb`): WIP Module Integration
- **Phase 3** (`44f51b4`, `3784adb`): Complete Manager Wiring

### 🐛 Bug Fixes & TODO Resolution

- **PDF Manipulator**: Re-enabled module with complete PDFPurr/lopdf migration
- **Annotations**: Implemented copying using lopdf with proper error handling
- **Text Extraction**: PDFPurr Document.load_page API integration
- **Hardware Access**: Replaced eink controller TODOs with proper error messages
- **Library Filters**: Implemented format and category toggle filtering
- **Build Issues**: Fixed unused imports, method names, type mismatches

### 🧩 Component Integration

- UI Components: ResultsLabel → SearchBar, ChapterLabel → Bottom bar, MarginCropper → PDF tools
- Documentation: Created docs/README.md, archived outdated plans, consolidated PDF docs

### 📁 Code Modularity

- **epub_edit**: Extracted 11 types to `types.rs`, parser functions to `parser.rs` (386 lines reduced total)
- **pdfpurr**: MuPDF compatibility layer in `types.rs` (FzRect, FzPoint, PdfPurrPixmap)
- **epub_editor**: UI state to `state.rs`, helpers to `helpers.rs` (392 lines reduced total)
- **plato**: Constants, task management, helpers to separate modules (269 lines reduced)
- **plato**: Device event handler to `event.rs` (324 lines reduced)
- **pdf_manipulator**: Types and constants to `types.rs` (253 lines reduced total)
- **emulator**: Constants and helpers to separate modules (73 lines reduced)

### ✅ Quality Assurance

- **Code Standards**: All files < 1000 lines, all functions < 50 lines
- **Error Handling**: Proper anyhow/thiserror usage throughout
- **Validation**: Input validation at all API boundaries
- **Documentation**: Complete docs on public methods
- **Testing**: Unit test structure maintained, 204 tests passing
- **Build**: Clean cargo check and clippy with zero warnings

### 📊 Impact Summary

- **Total Lines Reduced**: 1100+ lines across multiple modules
- **Maintainability**: Improved code organization and separation of concerns
- **Dead Code**: All `#[allow(dead_code)]` attributes removed from actionable code
- **Integration**: Complete manager system, animation, and gesture handling wired up
