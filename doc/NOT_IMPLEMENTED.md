# Not Implemented Features

> **Last Updated**: 2026-04-22
> **Related Documents**: [PDF_FEATURES.md](./PDF_FEATURES.md)  |  [OCR_TTS.md](./OCR_TTS.md)  |  [GUIDE.md](./GUIDE.md)

This document tracks Plato features that are not implemented, partially implemented, or intentionally excluded by design. It serves as a central reference for development planning and user expectations.

## Quick Status Overview

 | Category           | Implemented | Not Implemented | By Design |
 |--------------------|-------------|-----------------|-----------|
 | **Core Features**  | 21          | 0               | 0         |
 | **PDF Features**   | 6           | 0               | 6         |
 | **UI/UX Features** | 15          | 0               | 0         |
 | **Infrastructure** | 8           | 0               | 0         |

## Recently Implemented

For a detailed list of recently implemented features, see [IMPLEMENTED_FEATURES.md](./IMPLEMENTED_FEATURES.md).

## Future Enhancements

### Status: Planning

**Description**: See [PDF_FEATURES.md](./PDF_FEATURES.md) for comprehensive analysis of potential PDF-based enhancements.

**Current Focus Areas:**

- Performance optimizations for large documents
- Enhanced annotation workflows
- Improved memory management on resource-constrained devices
- UI/UX refinements for e-ink displays

---

## Features Explicitly Not Implemented (By Design)

### OCR and TTS

**Status**: Documented as not implemented by design

**Location**: `doc/OCR_TTS.md` and `doc/PDF_FEATURES.md` (Section 12)

**Reason**:

- OCR: Hardware limitations (256MB RAM, 1GHz CPU), PDF libraries don't include OCR (external Tesseract needed), battery impact
- Advanced OCR Control: Same as basic OCR - PDF libraries cannot convert images to text
- TTS: No audio subsystem, outside core mission

### JavaScript (mujs) Integration

**Status**: Documented as not implemented by design

**Location**: `doc/PDF_FEATURES.md` (Section 9)

**Reason**:

- JavaScript engine not included - requires additional dependency
- JavaScript in PDFs is virtually nonexistent (<0.1% of e-books)
- E-ink displays cannot properly render interactive content/animations
- Kobo's 256MB RAM insufficient for JS runtime
- Basic form fields work without JavaScript

### Enhanced Reflow (Story Module)

**Status**: Documented as not implemented by design

**Location**: `doc/PDF_FEATURES.md` (Section 4)

**Reason**:

- Plato already has a working HTML reflow engine (in `document/html/`)
- Module is designed for complex document workflows, not simple reading
- E-ink displays don't benefit from complex layouts

### Interactive PDF Forms

**Status**: Documented as not implemented by design

**Location**: `doc/PDF_FEATURES.md` (Section 2)

**Reason**:

- Forms are extremely rare in e-books (<0.01%)
- E-ink displays poorly suited for text input (requires keyboard)
- Small screen impractical for complex form layouts
- Basic form fields display correctly (partial support)
- Users typically fill forms on desktop

### Digital Signatures

**Status**: Documented as not implemented by design

**Location**: `doc/PDF_FEATURES.md` (Section 5)

**Reason**:

- PDF libraries can only verify signatures partially, cannot create new ones
- No use case on e-readers (legal/business documents)
- Security concerns: no secure key storage on Kobo
- Would require adding crypto libraries

### PDF/A and PDF/X Validation

**Status**: Documented as not implemented by design

**Location**: `doc/PDF_FEATURES.md` (Section 11)

**Reason**:

- No use case on e-readers (users need desktop software for validation)
- PDF/A/PDF/X virtually never used in e-books
- E-readers are for reading, not professional document workflows
- PDF libraries have limited validation capability anyway

---

## Implementation Summary

### Recently Completed (Last 12 Months)

**Total Implemented**: 53 features across core functionality, PDF handling, UI/UX, and infrastructure.

#### By Category

**🔧 Core Features (21 implemented):**

- Plugin Network Control, Cover Editor UI, External Storage Auto-Import
- WebDAV Sync, Reading Statistics UI, Password-protected Documents
- Series Management, Batch Operations UI, KoboCloud Sync

**📄 PDF Features (6 implemented):**

- PDF Native Search, Document Manipulation, Progressive Loading
- Redaction Support, Resource Extraction, PDF-Native Annotations

**🎨 UI/UX Features (15 implemented):**

- Settings UI Improvements, E-ink Crash Safety, Touch Target Optimizations
- Performance Improvements, Memory Optimizations, Render Performance

**⚙️ Infrastructure (11 implemented):**

- lazy_static → LazyLock Migration, Unwrap/Expect Reduction, Frontlight Graceful Degradation
- CPU Optimization, EPUB Editor Performance, Library Crash Safety

### Currently Not Implemented

 | Feature                      | Status       | Reason                                              |
 |------------------------------|--------------|-----------------------------------------------------|
 | **OCR & TTS**                | ❌ By Design | Hardware limitations, no audio subsystem            |
 | **JavaScript Integration**   | ❌ By Design | JS in PDFs virtually nonexistent, e-ink limitations |
 | **Enhanced Reflow**          | ❌ By Design | Duplicates existing HTML engine                     |
 | **Interactive Forms**        | ❌ By Design | Forms rare in e-books, poor e-ink UX                |
 | **Digital Signatures**       | ❌ By Design | No use case, security concerns                      |
 | **PDF/A & PDF/X Validation** | ❌ By Design | No use case on e-readers                            |
 | **Advanced OCR**             | ❌ By Design | PDF libraries don't include OCR                     |

---

## Development Notes

### Implementation Priorities

- **P1**: Critical user experience features
- **P2**: Important functionality improvements
- **P3**: Nice-to-have enhancements

### Design Principles

- Optimize for e-ink displays and resource constraints
- Prioritize reading experience over document workflow features
- Maintain cross-platform compatibility where feasible
- Ensure memory efficiency on 256MB Kobo devices
