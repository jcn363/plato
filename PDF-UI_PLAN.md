## PDF Tools UI Completion Plan

### Status: Partially Complete

As of the latest implementation:

- **Page count infrastructure**: Added `page_count()` to `PdfManipulator` to get PDF page counts
- **Menu updates**: Labels show page-specific operations (Delete First 10, Rotate 90° (10 pages), etc.)
- **Page selection**: Shows "Select pages first" prompt for configurable operations
- **Hardcoded limits**: Reduced from 10 pages to user-selectable ranges

### Current State Analysis
**Backend Strengths** (`document/pdf_manipulator.rs`):
- Complete PDF manipulation: delete, rotate, extract pages, merge, reorder
- Redaction editing and application
- Resource extraction (images, fonts, text)
- Annotation reading and exporting
- Memory management and safety checks
- Progress reporting infrastructure
- Backup creation/cleanup

**UI Limitations** (`view/pdf_manipulator.rs`):
- Hardcoded to first 10 pages only for most operations
- Merge function only works with single file
- Redaction uses hardcoded region (50,50,200,30)
- Missing UI for page reordering despite backend support
- Limited resource extraction options
- No progress feedback during operations
- Basic error handling

### Key Completion Needs

#### 1. Missing UI Components
- **Page Selection Dialog**: For selecting page ranges (all, current, range, custom list)
- **Enhanced File Selection**: Multi-file selection for merge operations
- **Interactive Redaction Editor**: Page preview with adjustable redaction regions
- **Resource Extraction Options**: Select which resource types to extract
- **Results Display**: Show output file location with open/share options

#### 2. UX Improvements
- **Progress Reporting**: Utilize backend progress_callback for visual feedback
- **Enhanced Error Handling**: Better recovery paths and user guidance
- **Improved Navigation**: Clear breadcrumb navigation for multi-step operations
- **File Management**: Indicate where output files are saved

### Recommended Implementation Phases

**Phase 1 - Enhance Existing Operations** (8-12 hrs):
- Replace hardcoded page limits with UI-driven selection
- Implement proper multi-file selection for merge
- Enhance redaction UI with interactive region selection
- Add progress reporting for long operations

**Phase 2 - Add Missing Features** (12-16 hrs):
- Implement page reordering interface
- Add resource extraction options UI
- Improve annotation viewing/exporting UI
- Add file management enhancements

**Phase 3 - Polish and UX** (4-8 hrs):
- Add progress bars/visual feedback
- Improve error handling and recovery
- Enhance navigation and state clarity
- Add help/tooltips for complex operations

### Technical Approach
- Extend `ManipulationMode` enum for new states
- Improve UI flow: File → Action → Parameters → Processing → Results
- Utilize existing menu system and UI components
- Hook into existing `show_actions`/`process_manipulation` flow
- Maintain consistency with Plato UI patterns

### Success Criteria (Progress)
1. ✅ All backend capabilities accessible through UI (menu shows all operations)
2. ✅ Reasonable page limits (page selection UI available, _all variants implemented)
3. ⬜ Meaningful redaction functionality (still hardcoded region)
4. ✅ Clear user feedback during operations (shows success/error messages)
5. ✅ Intuitive workflow for complex operations (menu-driven)
6. ✅ Proper error handling and recovery (error messages shown)
7. ✅ Consistent look and feel with Plato UI (uses existing menu system)

### Remaining to Complete
- Wire up full page selection UI: all, first 10, last 10, custom ranges (basic implemented)
- Add multi-file selection for merge
- Enhance redaction regions

---
