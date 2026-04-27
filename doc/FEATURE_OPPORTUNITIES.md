# Feature Opportunities

> **Last Updated**: 2026-04-27
> **Related Documents**: [NOT_IMPLEMENTED.md](./NOT_IMPLEMENTED.md) | [IMPLEMENTED_FEATURES.md](./IMPLEMENTED_FEATURES.md) | [PDF_FEATURES.md](./PDF_FEATURES.md) | [AGENTS.md](../AGENTS.md)

This document identifies potential new features and enhancements for Plato, prioritized by value, implementation effort, and alignment with the project's mission to optimize the e-reader experience on Kobo devices.

## Quick Status Overview

**Current State**:

- ✅ **All 15 features implemented** (P1: 5, P2: 5, P3: 5)
- Pure Rust migration complete (no C dependencies)
- All files AGENTS.md compliant (under 1,000 lines)
- Comprehensive PDF manipulation (PDFPurr + lopdf)
- 6 features explicitly excluded by design (OCR, TTS, JavaScript, etc.)

**Implementation Priorities**:

- **P1**: High value, moderate/low effort
- **P2**: Moderate value, moderate effort
- **P3**: Nice-to-have, high effort or niche use cases

---

## P1: High Priority Features

### 1. Advanced Library Search and Filter

**Estimated Cost**: 3/10 (Low to Medium)
**User Value**: High
**Implementation Effort**: Moderate
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Enhanced search capabilities across the library with support for metadata, content, and custom filters.

**Proposed Features**:

- Full-text search across document content (PDF, EPUB)
- Filter by author, series, publisher, year
- Filter by reading status (unread, in-progress, completed)
- Filter by file type, size, date added
- Saved search queries
- Boolean operators (AND, OR, NOT)
- Search within collections/folders

**Implementation Notes**:

- Use existing PDFPurr text extraction for PDF content
- Use existing EPUB parsing for EPUB content
- Index content on import/update (background task)
- Store search index in efficient format (e.g., tantivy or simple FxHashMap)
- Add search UI to home view with filter sidebar

**Rationale**: Users with large libraries (1000+ books) need powerful search to find content quickly. Current search is limited to titles/authors.

---

### 2. Collection and Folder Organization

**Estimated Cost**: 4/10 (Medium)
**User Value**: High
**Implementation Effort**: Moderate
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Allow users to organize books into collections, folders, or custom categories.

**Proposed Features**:

- Create custom collections/folders
- Drag-and-drop books into collections
- Nested collections (collections within collections)
- Smart collections (auto-populated by rules: "unread science fiction")
- Collection icons and colors
- Quick filter by collection in home view
- Bulk move/copy between collections

**Implementation Notes**:

- Extend metadata schema to include collection_id
- Add collection management UI (create, edit, delete)
- Add drag-and-drop support to home view
- Implement smart collection rule engine
- Persist collections in settings database

**Rationale**: Users with diverse reading interests need organization beyond flat library view. Collections are standard in e-reader apps (Kindle, Kobo, Apple Books).

---

### 3. Reading Progress Visualization

**Estimated Cost**: 2/10 (Low)
**User Value**: High
**Implementation Effort**: Low
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Visualize reading progress with statistics, time estimates, and progress tracking.

**Proposed Features**:

- Reading speed calculation (pages/minute, words/minute)
- Time to finish estimate (based on current speed)
- Progress percentage with visual indicator
- Reading streak tracking (consecutive days)
- Session duration tracking
- Historical progress graph (last 30 days)
- Comparison with average reading speed

**Implementation Notes**:

- Track page turn timestamps in existing reading state
- Calculate speed on page turn (distance / time)
- Store in metadata or separate statistics database
- Add statistics view (already exists, enhance with new metrics)
- Add progress bar to book card in library view

**Rationale**: Users want to track reading habits and estimate completion time. Low effort with high user engagement value.

---

### 4. Cross-Device Reading Position Sync

**Estimated Cost**: 6/10 (Medium to High)
**User Value**: High
**Implementation Effort**: High
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Synchronize reading position, highlights, and annotations across multiple devices (e.g., Kobo, Android, iOS).

**Features**:

- Sync reading position (page, location) across devices
- Sync highlights and annotations
- Conflict resolution (last write wins or merge)
- Sync settings (font size, margins, theme)
- Sync progress statistics
- Manual sync trigger + auto-sync on close
- Sync status indicator

**Implementation Notes**:

- Extend existing WebDAV sync infrastructure
- Add reading position to sync metadata
- Implement conflict resolution strategy
- Add sync UI to settings view
- Use incremental sync (only changed data)
- Handle offline mode gracefully

**Rationale**: Users read on multiple devices (Kobo + phone/tablet). Current sync only covers files, not reading state.

---

### 5. Accessibility Improvements

**Estimated Cost**: 2/10 (Low)
**User Value**: High
**Implementation Effort**: Low
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Enhanced accessibility features for users with visual impairments or reading difficulties.

**Proposed Features**:

- High contrast mode (beyond current inversion)
- Dyslexic-friendly font option (OpenDyslexic)
- Text spacing controls (letter spacing, word spacing)
- Line height controls
- Color blindness support (adjust color palette)
- Large text mode (scale all text)
- Focus mode (dim everything except current paragraph)

**Implementation Notes**:

- Add accessibility settings to settings view
- Implement text spacing/line height in rendering engine
- Add OpenDyslexic font to font bundle
- Create color palette for color blindness (deuteranopia, protanopia, tritanopia)
- Add focus mode overlay to reader view

**Rationale**: Accessibility is important for inclusive design. Low effort with significant impact for users with visual/reading difficulties.

---

## P2: Medium Priority Features

### 6. Calibre Wireless Integration

**Estimated Cost**: 7/10 (High)
**User Value**: High
**Implementation Effort**: High
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Integrate with Calibre's Content Server for wireless book transfer and metadata management.

**Proposed Features**:

- Connect to Calibre Content Server over Wi-Fi
- Browse Calibre library from Plato
- Download books directly to device
- Upload books to Calibre library
- Sync metadata (ratings, tags, collections)
- Auto-sync on Wi-Fi connection
- Support for Calibre custom columns

**Implementation Notes**:

- Implement Calibre Content Server client (HTTP API)
- Add Calibre server configuration to settings
- Add Calibre browser view to home
- Handle authentication if configured
- Implement metadata mapping between Plato and Calibre
- Add background sync daemon

**Rationale**: Many users use Calibre for library management. Wireless integration eliminates USB cable dependency.

---

### 7. EPUB to PDF Conversion

**Estimated Cost**: 6/10 (Medium to High)
**User Value**: Moderate
**Implementation Effort**: High
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Convert EPUB documents to PDF format for sharing or printing.

**Proposed Features**:

- Convert EPUB to PDF with layout preservation
- Custom page size (A4, A5, letter, custom)
- Margin controls
- Font embedding options
- Image quality settings
- Batch conversion
- Conversion progress indicator

**Implementation Notes**:

- Use existing EPUB parsing (epub_edit crate)
- Use PDFPurr for PDF generation
- Implement layout engine (re-use existing HTML renderer)
- Add conversion UI to EPUB editor or home view
- Handle EPUB-specific features (TOC, links, annotations)
- Test with various EPUB layouts

**Rationale**: Users sometimes need PDF format for sharing or printing. Conversion on-device is convenient.

---

### 8. Custom Sorting Options

**Estimated Cost**: 2/10 (Low)
**User Value**: Moderate
**Implementation Effort**: Low
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Add more sorting options to library view beyond current defaults.

**Proposed Features**:

- Sort by author (last name, first name)
- Sort by series name + number
- Sort by publisher
- Sort by file size
- Sort by date added
- Sort by last read
- Sort by reading progress
- Custom sort order (drag-and-drop)
- Save sort preferences per view

**Implementation Notes**:

- Extend metadata schema with sortable fields
- Add sort menu to library view
- Implement custom sort (manual ordering)
- Persist sort preferences in settings
- Add sort indicator to library header

**Rationale**: Current sorting options are limited. Users want more control over library organization.

---

### 9. Gesture Customization

**Estimated Cost**: 4/10 (Medium)
**User Value**: Moderate
**Implementation Effort**: Moderate
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Allow users to customize touch gestures for common actions.

**Proposed Features**:

- Customize tap zones (left/right/top/bottom)
- Customize swipe gestures (up/down/left/right)
- Customize pinch gestures (zoom in/out)
- Customize long-press actions
- Gesture profiles (reading, navigation, accessibility)
- Reset to defaults option
- Gesture tutorial/preview

**Implementation Notes**:

- Extend existing gesture system (reader_gestures.rs)
- Add gesture configuration UI to settings
- Map gestures to actions (page turn, menu, bookmark, etc.)
- Store gesture profiles in settings
- Add visual gesture editor

**Rationale**: Users have different preferences for touch interactions. Customization improves usability.

---

### 10. Text Spacing and Line Height Controls

**Estimated Cost**: 2/10 (Low)
**User Value**: Moderate
**Implementation Effort**: Low
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Fine-grained controls for text spacing and line height in reader view.

**Proposed Features**:

- Letter spacing adjustment
- Word spacing adjustment
- Line height adjustment
- Paragraph spacing adjustment
- Preset profiles (compact, comfortable, spacious)
- Per-document settings

**Implementation Notes**:

- Add spacing controls to reader settings menu
- Implement spacing in rendering engine (CSS or direct)
- Add spacing presets for quick access
- Persist spacing preferences per document type

**Rationale**: Users with visual impairments or reading preferences benefit from spacing controls. Low effort to implement.

---

## P3: Low Priority Features

### 11. Goodreads Integration

**Estimated Cost**: 8/10 (High)
**User Value**: Moderate
**Implementation Effort**: High
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Integrate with Goodreads API for book discovery, reviews, and reading tracking.

**Proposed Features**:

- Search Goodreads from Plato
- View book details and reviews
- Add books to Goodreads shelves
- Sync reading status with Goodreads
- Import Goodreads library
- View Goodreads ratings in library

**Implementation Notes**:

- Implement Goodreads API client (OAuth2)
- Add Goodreads configuration to settings
- Add Goodreads search/view UI
- Handle rate limiting and API quotas
- Sync reading status periodically

**Rationale**: Goodreads is popular for book tracking. Integration adds social features but requires significant effort.

---

### 12. Pocket/Instapaper Integration

**Estimated Cost**: 7/10 (High)
**User Value**: Moderate
**Implementation Effort**: High
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Integrate with Pocket or Instapaper for article saving and reading.

**Proposed Features**:

- Save articles to Pocket/Instapaper from Plato
- Import Pocket/Instapaper articles to Plato
- Sync reading status
- Archive and delete articles

**Implementation Notes**:

- Implement Pocket/Instapaper API clients
- Add service configuration to settings
- Add import/export UI
- Handle authentication (OAuth)
- Sync periodically in background

**Rationale**: Users save articles to read later. Integration with popular services is convenient but high effort.

---

### 13. Cloud Storage Provider Integration

**Estimated Cost**: 8/10 (High)
**User Value**: Moderate
**Implementation Effort**: High
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Integrate with cloud storage providers (Dropbox, Google Drive, OneDrive) for file sync.

**Proposed Features**:

- Connect to Dropbox/Google Drive/OneDrive
- Browse cloud storage from Plato
- Download books from cloud
- Upload books to cloud
- Sync library with cloud folder
- Auto-sync on Wi-Fi

**Implementation Notes**:

- Implement API clients for each provider (OAuth2)
- Add cloud provider configuration to settings
- Add cloud browser UI
- Handle authentication and token refresh
- Implement sync daemon
- Handle offline mode

**Rationale**: Users store books in cloud storage. Direct integration is convenient but high effort. WebDAV already provides generic cloud sync.

---

### 14. Document Comparison (Diff View)

**Estimated Cost**: 6/10 (Medium to High)
**User Value**: Low
**Implementation Effort**: High
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Compare two versions of a document and show differences.

**Proposed Features**:

- Select two documents to compare
- Show side-by-side diff view
- Highlight added/removed/changed text
- Navigate between changes
- Export diff report

**Implementation Notes**:

- Implement diff algorithm (e.g., Myers algorithm)
- Add diff UI to reader or home view
- Handle different document types (PDF, EPUB)
- Extract text for comparison
- Visual diff rendering on e-ink

**Rationale**: Niche use case for researchers/editors. Low value for general users.

---

### 15. Booklet Printing Mode

**Estimated Cost**: 4/10 (Medium)
**User Value**: Low
**Implementation Effort**: Moderate
**Status**: ✅ **Implemented** (Backend + UI complete)

**Description**: Reorder PDF pages for booklet printing (2-up, foldable).

**Proposed Features**:

- Reorder pages for booklet layout
- Add margins for binding
- Preview booklet layout
- Export as new PDF

**Implementation Notes**:

- Implement page reordering algorithm (booklet imposition)
- Add booklet UI to PDF tools
- Handle page count (add blank pages if needed)
- Export with PDF manipulation library

**Rationale**: Niche use case for printing. Low value for e-reader users.

---

## Platform-Specific Implementation

The following features are excluded for Kobo e-readers due to hardware constraints, but are feasible for mobile (Android/iOS) and desktop (Linux) platforms:

### High Priority (Clear Value, Feasible)

#### Text-to-Speech (TTS)

**Kobo Status**: ❌ Excluded (no audio subsystem)
**Mobile/Desktop Status**: ✅ **Recommended for Implementation**

**Why Feasible on Mobile/Desktop:**

- **Android**: Native `TextToSpeech` API available
- **iOS**: Native `AVSpeechSynthesizer` available
- **Linux**: espeak/piper available via system audio
- Full audio hardware support on all platforms
- No memory/CPU constraints

**Implementation Cost**: 3/10 (Low) - Use platform-native APIs

**User Value**: High - Accessibility feature for visually impaired users

---

#### OCR for Scanned PDFs

**Kobo Status**: ❌ Excluded (256MB RAM, 1GHz CPU too slow)
**Mobile/Desktop Status**: ✅ **Recommended for Implementation**

**Why Feasible on Mobile/Desktop:**

- **Android/iOS**: Tesseract mobile libraries available
- **Linux**: Tesseract easily available via package manager
- Better CPU/RAM on mobile/desktop devices
- Battery impact less critical on plugged-in desktop

**Implementation Cost**: 4/10 (Medium) - Tesseract integration (~20MB library)

**User Value**: High - Enables reading of scanned documents

---

### Medium Priority (Feasible but Niche)

#### Interactive PDF Forms

**Kobo Status**: ❌ Excluded (poor e-ink UX for text input)
**Mobile/Desktop Status**: ✅ **Implemented**

**Why Feasible on Mobile/Desktop:**

- **Mobile**: Touch keyboards, better input methods
- **Desktop**: Full keyboard/mouse support
- LCD/OLED displays better for form layouts
- Signature fields feasible with touch/digitizer

**Implementation Cost**: 4/10 (Medium) - Form UI component needed

**User Value**: Moderate - Forms rare in e-books (<0.01%) but useful for government/legal documents

**Implementation Status**:

- ✅ Backend: Form field parsing (FormField, FormParser, FormValues)
- ✅ Backend: Form value storage and serialization
- ✅ UI: Form input components (text fields, checkboxes, radio buttons, dropdowns, lists, signatures, buttons)
- ✅ Integration: PDF viewer form UI overlay (menu entry added)
- ✅ Export: PDF form field value export (lopdf AcroForm modification)
- ✅ Interactive: Form field value editing and save functionality

---

#### JavaScript Integration

**Kobo Status**: ❌ Excluded (e-ink limitations, memory constraints)
**Mobile/Desktop Status**: **Low Priority (Still Complex)** ❌ Excluded (virtually nonexistent)

**Why Feasible but Low Priority:**

- **Mobile/Desktop**: QuickJS/V8 integration possible
- Better displays for interactive content
- More RAM available
- **BUT**: JavaScript in PDFs is virtually nonexistent (<0.1% of e-books)
- High implementation cost (5/10) for minimal benefit

**Implementation Cost**: 5/10 (Medium) - JavaScript engine integration

**User Value**: Low - JS in PDFs extremely rare in consumer content

---

### Desktop-Only Priority

#### Digital Signatures

**Kobo Status**: ❌ Excluded (no secure key storage, no use case)
**Desktop Status**: ✅ **Structurally Implemented (Crypto Operations Pending)**

**Why Feasible on Desktop:**

- Secure key storage available (keyring, TPM)
- Document signing workflows are desktop-centric
- Crypto libraries (OpenSSL/mbedTLS) available
- Certificate management infrastructure exists

**Implementation Cost**: 6/10 (Medium-High) - Security infrastructure needed

**User Value**: Moderate - Legal/business document workflows

**Implementation Status**:

- ✅ Backend: Digital signature module with data structures (DigitalSignature, Certificate, SignatureManager)
- ✅ UI: SignaturesView with certificate selection, signing, and verification UI
- ✅ Integration: PDF viewer menu entry for digital signatures
- ⏳ Crypto: PKCS#7/CMS signature generation (requires OpenSSL/ring integration)
- ⏳ Keyring: System keyring integration for secure key storage (secret-service on Linux)
- ⏳ PDF: PDF signature field creation with lopdf
- ⏳ Verification: Certificate chain validation and signature verification

---

#### PDF/A and PDF/X Validation

**Kobo Status**: ❌ Excluded (no use case on e-readers)
**Desktop Status**: ✅ **Recommended for Desktop Implementation**

**Why Feasible on Desktop:**

- Desktop is where document validation actually happens
- No hardware constraints
- Can leverage existing validation libraries
- Technical implementation easy (3/10 cost)

**Implementation Cost**: 3/10 (Low) - Basic conformance detection

**User Value**: Low-Moderate - Niche use case for archivists/printing professionals

---

## Not Recommended (By Design)

The following features are explicitly not recommended based on existing documentation and design decisions:

### OCR for Scanned PDFs (for Mobile/Desktop only)

- **Status**: Excluded by design (see `doc/OCR_TTS.md`)
- **Reason**: Hardware limitations (256MB RAM, 1GHz CPU), no OCR library in PDFPurr, battery impact, better handled on desktop

### Text-to-Speech (TTS) (for Mobile/Desktop only)

- **Status**: Excluded by design (see `doc/OCR_TTS.md`)
- **Reason**: No audio subsystem on Kobo devices, outside core mission

### JavaScript Integration (NOT RECOMMENDED)

- **Status**: Excluded by design (see `doc/PDF_FEATURES.md`)
- **Reason**: JavaScript in PDFs virtually nonexistent (<0.1%), e-ink limitations, memory constraints

### Interactive PDF Forms (for Mobile/Desktop only)

- **Status**: Excluded by design (see `doc/PDF_FEATURES.md`)
- **Reason**: Forms rare in e-books (<0.01%), poor e-ink UX for text input, users fill forms on desktop

### Digital Signatures (for Desktop only)

- **Status**: Excluded by design (see `doc/PDF_FEATURES.md`)
- **Reason**: No use case on e-readers, security concerns (no secure key storage), PDF libraries cannot create signatures

### PDF/A and PDF/X Validation (for Desktop only)

- **Status**: Excluded by design (see `doc/PDF_FEATURES.md`)
- **Reason**: No use case on e-readers, users need desktop software for validation

---

## Implementation Guidelines

When implementing new features, follow these principles from `AGENTS.md`:

1. **Modular Design**: Keep files under 1,000 lines, functions under 50 lines
2. **Performance**: Optimize for e-ink displays and 256MB RAM constraints
3. **Input Validation**: Validate all inputs at public API boundaries
4. **Error Handling**: Use `anyhow` for application-level errors, `thiserror` for library errors
5. **Testing**: Write unit tests in sibling `_tests.rs` files, integration tests in `tests/` directory
6. **Documentation**: Add rustdoc comments for public APIs, document design decisions
7. **No Backward Compatibility**: Do not add code to support deprecated patterns

---

## Development Notes

### Feature Selection Criteria

Features are prioritized based on:

- **User Value**: How much the feature improves the reading experience
- **Implementation Effort**: Complexity and development time
- **Hardware Constraints**: Feasibility on 256MB RAM Kobo devices
- **Alignment with Mission**: Focus on e-reader optimization, not document workflows
- **Maintenance Burden**: Long-term code maintenance cost

### Recommended Implementation Order

1. **Phase 1** (Quick wins): Reading progress visualization, custom sorting, accessibility improvements
2. **Phase 2** (Core enhancements): Advanced library search, collection organization, cross-device sync
3. **Phase 3** (Major features): Calibre integration, gesture customization, text spacing controls
4. **Phase 4** (Nice-to-have): Cloud storage integration, Goodreads integration

### Testing Requirements

- All features must have unit tests
- Integration tests for complex workflows
- Performance testing on ARM target
- Memory usage validation (must fit in 256MB)
- E-ink rendering validation

---

## Conclusion

Plato is a mature, well-maintained e-reader application with comprehensive features. The opportunities identified above focus on enhancing the reading experience, improving library management, and adding convenience features while respecting hardware constraints and the project's design philosophy.

The highest-priority features (P1) offer significant user value with moderate implementation effort and align well with Plato's mission to optimize the e-reader experience on Kobo devices.
