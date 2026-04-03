# OCR and TTS Not Implemented

This document explains why Optical Character Recognition (OCR) and Text-to-Speech (TTS) are not implemented in Plato, including analysis of implementation costs.

## OCR (Optical Character Recognition)

### Why Not Implemented

1. **MuPDF Does NOT Include Native OCR**: MuPDF has NO built-in OCR engine. The "OCR" in MuPDF is actually text *extraction* from existing text layers, not image-to-text conversion. For true OCR of scanned PDFs, Tesseract integration would be required separately.

2. **Hardware Limitations**: Kobo e-readers have limited CPU (typically ARM-based @ 1GHz) and memory (256MB). OCR requires significant computational power - a single page takes 10-60 seconds, meaning a 300-page book would require 50-300 minutes to process.

3. **Tesseract Integration Cost (Estimated: 8/10 - High)**:
   - Would need to bundle Tesseract library (~20MB+)
   - Requires language data files (each language ~2-20MB)
   - Significant memory footprint exceeds Kobo limits
   - Background processing needed to not block UI

4. **Battery Impact**: OCR processing is power-intensive and would significantly reduce battery life on portable devices.

5. **Already Handled by MuPDF**: Plato uses MuPDF for PDF rendering, which includes text extraction from PDFs that already have text layers. This enables text selection and search in scanned documents that have already been OCR'd.

6. **Use Case Mismatch**: The primary audience for e-readers wants to read text-based documents. OCR is primarily needed for image-only PDFs (scanned books), which are less common in the e-reader ecosystem.

### Alternative Solutions (Recommended)

- **Pre-OCR on PC**: Use desktop tools (OCRmyPDF, Adobe Acrobat) before transferring to Kobo
- **Cloud OCR**: Send to web service (requires WiFi, privacy concerns)
- **Selective OCR**: Only OCR selected pages/regions user chooses

### Implementation Verdict

**Not recommended for Kobo implementation** due to:
- Hardware limitations (256MB RAM, 1GHz CPU)
- Battery drain during intensive processing
- MuPDF doesn't actually do OCR - external Tesseract dependency required
- Better handled by pre-processing on computer

The existing text selection in MuPDF works for PDFs that already have an invisible text layer - true OCR for image-only PDFs is outside Plato's core mission.

## TTS (Text-to-Speech)

### Why Not Implemented

1. **No Audio Hardware Support**: The codebase contains no audio output subsystem. Kobo e-readers have basic audio capabilities (some models have speakers or headphone jacks), but Plato's architecture is focused on visual reading.

2. **Platform Constraints**: Even if audio were available, real-time TTS requires either:
   - On-device TTS engine (computationally expensive)
   - Network connectivity for cloud TTS services (inconsistent on e-readers)

3. **Alternative Solutions**: Users who need TTS can use:
   - Device-native accessibility features
   - Third-party Android apps on supported Kobo devices
   - External TTS applications

4. **Development Focus**: Plato's development is centered on providing the best possible reading experience for visual text, annotations, and document management—not audio features.

5. **E-ink Display Context**: E-readers are primarily designed for silent, visual reading. Adding TTS would deviate from the core use case and add complexity without significant benefit to the target user base.

## Summary

Both OCR and TTS are omitted because they:
- Require significant hardware resources
- Are outside Plato's core mission of document reading
- Have limited use cases for the typical e-reader user
- Would negatively impact device performance and battery life
- Are better served by native device features or third-party applications

Plato continues to focus on what it does best: providing an exceptional reading experience with support for multiple document formats, annotation tools, and reading customization.