# Feature Opportunities

> **Last Updated**: 2026-04-27
> **Related Documents**: [NOT_IMPLEMENTED.md](./NOT_IMPLEMENTED.md) | [IMPLEMENTED_FEATURES.md](./IMPLEMENTED_FEATURES.md) | [PDF_FEATURES.md](./PDF_FEATURES.md) | [AGENTS.md](../AGENTS.md)

## Quick Status Overview

- ✅ **All 15 features implemented** (P1: 5, P2: 5, P3: 5)
- Pure Rust migration complete (no C dependencies)
- All files AGENTS.md compliant (under 1,000 lines)
- ✅ **TTS implemented** for Mobile/Desktop (Android, Linux, macOS, Windows)
- **Deferred**: PKCS#7/CMS signatures, system keyring integration
- **Excluded by design**: JavaScript integration

---

## Implemented Features

| Priority | Feature                            | Status                   |
|----------|------------------------------------|--------------------------|
| **P1**   | Advanced Library Search            | ✅ Backend + UI complete |
| **P1**   | Collection/Folder Organization     | ✅ Backend + UI complete |
| **P1**   | Reading Progress Visualization     | ✅ Backend + UI complete |
| **P1**   | Cross-Device Reading Position Sync | ✅ Backend + UI complete |
| **P1**   | Accessibility Improvements         | ✅ Backend + UI complete |
| **P2**   | Calibre Wireless Integration       | ✅ Backend + UI complete |
| **P2**   | EPUB to PDF Conversion             | ✅ Backend + UI complete |
| **P2**   | Custom Sorting Options             | ✅ Backend + UI complete |
| **P2**   | Gesture Customization              | ✅ Backend + UI complete |
| **P2**   | Text Spacing/Line Height Controls  | ✅ Backend + UI complete |
| **P3**   | Goodreads Integration              | ✅ Backend + UI complete |
| **P3**   | Pocket/Instapaper Integration      | ✅ Backend + UI complete |
| **P3**   | Cloud Storage Integration          | ✅ Backend + UI complete |
| **P3**   | Document Comparison (Diff)         | ✅ Backend + UI complete |
| **P3**   | Booklet Printing Mode              | ✅ Backend + UI complete |

---

## Platform-Specific Features

| Feature                      | Kobo        | Mobile         | Desktop        | Notes              |
|------------------------------|-------------|----------------|----------------|--------------------|
| **Text-to-Speech**           | ❌ Excluded | ✅ Implemented | ✅ Implemented | No audio on Kobo   |
| **OCR**                      | ❌ Excluded | ✅ Implemented | ✅ Implemented | RAM constraint     |
| **Interactive PDF Forms**    | ❌ Excluded | ✅ Implemented | ✅ Implemented | Poor e-ink UX      |
| **Digital Signatures**       | ❌ Excluded | ❌ Excluded    | ✅ Implemented | SHA256 + cert mgmt |
| **PDF/A & PDF/X Validation** | ❌ Excluded | ❌ Excluded    | ✅ Implemented | Desktop-only       |
| **JavaScript**               | ❌ Excluded | ❌ Excluded    | ❌ Excluded    | <0.1% in e-books   |

### Deferred Components

| Component                  | Status      | Reason                               |
|----------------------------|-------------|--------------------------------------|
| PKCS#7/CMS signatures      | ⏳ Deferred | Requires complex cert chain handling |
| System keyring integration | ⏳ Deferred | zbus async dependency conflicts      |

---

## Excluded by Design

See [NOT_IMPLEMENTED.md](./NOT_IMPLEMENTED.md) and [PDF_FEATURES.md](./PDF_FEATURES.md):

- **JavaScript Integration** — Virtually nonexistent (<0.1%), e-ink limitations
- **OCR on Kobo** — 256MB RAM insufficient for Tesseract
- **TTS on Kobo** — No audio subsystem

---

## Next Steps

For implementation history, see [IMPLEMENTED_FEATURES.md](./IMPLEMENTED_FEATURES.md).  
For design rationale, see [NOT_IMPLEMENTED.md](./NOT_IMPLEMENTED.md).
