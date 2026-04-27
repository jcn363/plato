# OCR and TTS Implementation

This document explains the implementation status and constraints for Optical Character Recognition (OCR) and Text-to-Speech (TTS) in Plato.

## OCR (Optical Character Recognition)

### Implementation Status

**Desktop**: ✅ **Recommended for Implementation**
**Android**: ✅ **Recommended for Implementation**
**Kobo**: ❌ **Not Recommended** (hardware limitations)

### Why Not Implemented on Kobo

1. **Hardware Limitations**: Kobo e-readers have limited CPU (typically ARM-based @ 1GHz) and memory (256MB). OCR requires significant computational power - a single page takes 10-60 seconds, meaning a 300-page book would require 50-300 minutes to process.

2. **Tesseract Integration Cost (Estimated: 8/10 - High)**:
   - Would need to bundle Tesseract library (~20MB+)
   - Requires language data files (each language ~2-20MB)
   - Significant memory footprint exceeds Kobo limits
   - Background processing needed to not block UI

3. **Battery Impact**: OCR processing is power-intensive and would significantly reduce battery life on portable devices.

4. **Use Case Mismatch**: The primary audience for e-readers wants to read text-based documents. OCR is primarily needed for image-only PDFs (scanned books), which are less common in the e-reader ecosystem.

### Desktop and Android Implementation Plan

**Why Feasible on Desktop/Android:**

- Desktop has no hardware constraints (multi-core CPU, GBs of RAM)
- Android devices have significantly more resources than Kobo
- PDFPurr includes OCR capabilities with multiple engine options
- Can leverage existing OCR libraries (Tesseract, Windows OCR, ocrs)

**Implementation Strategy:**

1. **Use PDFPurr's Built-in OCR**:
   - PDFPurr supports three OCR engines:
     - Windows OCR (~95% accuracy, zero dependencies, Windows only)
     - Tesseract (~85-89%, requires tesseract CLI, cross-platform)
     - ocrs (pure Rust, Latin only, requires "ocr" feature)
   - Use Tesseract for Linux/Android (most widely available)
   - Use Windows OCR on Windows when available
   - Use ocrs as fallback for pure Rust solution

2. **Desktop Implementation**:
   - Add OCR feature flag to Cargo.toml
   - Implement OCR manager with engine selection
   - Add OCR UI to validation view or separate OCR view
   - Support batch OCR for entire documents
   - Save OCR results as invisible text layer (rendering mode 3)
   - Tag OCR output for screen reader accessibility

3. **Android Implementation**:
   - Use Tesseract Android library
   - Implement background OCR processing
   - Show progress during OCR
   - Cache OCR results for faster subsequent access
   - Integrate with Android's accessibility framework

**Implementation Cost**: 5/10 (Medium) - PDFPurr provides OCR API, need UI integration

**User Value**: Moderate - Enables reading of scanned PDFs on devices with sufficient resources

### Alternative Solutions (Recommended for Kobo)

- **Pre-OCR on PC**: Use desktop tools (OCRmyPDF, Adobe Acrobat) before transferring to Kobo
- **Cloud OCR**: Send to web service (requires WiFi, privacy concerns)
- **Selective OCR**: Only OCR selected pages/regions user chooses

### Implementation Verdict

**Kobo**: Not recommended due to hardware limitations (256MB RAM, 1GHz CPU), battery drain, and better alternatives.

**Desktop/Android**: Recommended implementation using PDFPurr's OCR capabilities.

## TTS (Text-to-Speech)

### Implementation Status

**Desktop**: ✅ **Implemented** (via `tts` crate using system TTS)
**Android**: ✅ **Implemented** (via Android TextToSpeech API)
**Kobo**: ❌ **Not Available** (no audio subsystem, hardware limitations)

### Why Not Implemented on Kobo

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

### Implementation Details

**Android Implementation:**

- Uses Android's `TextToSpeech` API via JNI (Java Native Interface)
- Located in `crates/core/src/tts_android.rs`
- Supports: play, stop, rate adjustment
- UI integration in `crates/core/src/view/tts.rs`

**Desktop Implementation:**

- Uses the `tts` crate for cross-platform TTS
- Linux: speech-dispatcher
- macOS: AVSpeechSynthesizer
- Windows: SAPI5
- Located in `crates/core/src/tts_desktop.rs`

**UI Features:**

- Play/Pause/Stop controls
- Speed slider (0.5x to 2.0x)
- Status indicator
- Only shown on supported platforms

**Usage:**

```rust
use plato_core::tts::{TtsEngine, TtsOptions, create_tts_engine};

let mut tts = create_tts_engine()?;
tts.initialize()?;
tts.speak("Hello, world!", TtsOptions::default())?;
```

**Feature Flag:**
Enable TTS with the `tts` feature in `Cargo.toml`:

```toml
plato-core = { path = "../core", features = ["tts"] }
```

### Implementation Verdict

**Kobo**: Not available due to no audio subsystem, hardware limitations, and better alternatives.

**Desktop**: ✅ **Implemented** - Uses system TTS via `tts` crate.

**Android**: ✅ **Implemented** - Uses Android's built-in TextToSpeech API via JNI.

## Summary

**OCR Implementation:**

- **Kobo**: Not recommended due to hardware limitations (256MB RAM, 1GHz CPU), battery drain, and better alternatives.
- **Desktop**: Recommended implementation using PDFPurr's OCR capabilities (Tesseract, Windows OCR, ocrs engines).
- **Android**: Recommended implementation using Tesseract Android library with background processing.

**TTS Implementation:**

- **Kobo**: Not recommended due to no audio subsystem, hardware limitations, and better alternatives.
- **Desktop**: Not recommended - outside core mission, better served by system TTS tools.
- **Android**: Recommended implementation using Android's built-in TextToSpeech API.

Both OCR and TTS are omitted on Kobo because they:

- Require significant hardware resources
- Are outside Plato's core mission of document reading on e-readers
- Have limited use cases for the typical e-reader user
- Would negatively impact device performance and battery life
- Are better served by native device features or third-party applications

Plato continues to focus on what it does best: providing an exceptional reading experience with support for multiple document formats, annotation tools, and reading customization.
