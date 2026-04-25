# EPUB Editor Features

Plato includes a powerful EPUB editing suite designed to optimize digital books for the best possible reading experience on Kobo e-readers.

## Core Optimization Features

### 1. Image Optimization

This tool processes all images within an EPUB to reduce file size and improve rendering on E-Ink displays.

- **Grayscale Conversion**: Converts color images to 8-bit grayscale (Luma8). Since Kobo devices use E-Ink, color data is often unnecessary and removing it significantly reduces memory usage and dithering artifacts.
- **Intelligent Resizing**: Resizes images that exceed a maximum dimension (default 1600px). This prevents the device from wasting CPU and memory on high-resolution images that exceed the screen's physical pixels.

### 2. CSS Sanitization for E-Ink

E-Ink screens have different requirements than LCD/OLED displays. The CSS sanitizer automatically modifies book styles for better readability:

- **High Contrast**: Forces black text on a white background, removing complex background colors or images that can cause ghosting.
- **Dynamic Layout**: Replaces fixed widths (e.g., `width: 800px`) with fluid constraints (`max-width: 100%`) to ensure text doesn't overflow the screen.
- **Margin Optimization**: Resets large hardcoded margins that waste screen real estate.

### 3. TOC Recovery

If a book has a missing or corrupted Table of Contents (TOC), this tool can automatically rebuild it:

- **Heading Detection**: Scans all chapters for standard HTML headings (`h1` through `h6`).
- **Automated Mapping**: Generates a new `nav.xhtml` structure based on the detected headings, providing functional navigation for previously unnavigable books.

## Technical Standards

### EPUB Spec Compliance

The editor ensures that saved files adhere strictly to the EPUB specification:

- **Mimetype Integrity**: The `mimetype` file is guaranteed to be the first entry in the ZIP archive and is stored without compression, ensuring compatibility with all hardware readers.
- **Pure Rust Implementation**: All processing is done using high-performance, memory-safe Rust libraries (e.g., `image`, `zip`, `walkdir`).

## Using the CLI Tool

Power users can access these features via the `epub_editor` command-line tool:

```bash
./epub_editor path/to/book.epub
```

From the interactive menu, select options 11-13 to apply these optimizations.

## Advanced Maintenance Tools

In addition to the core optimization features, Plato includes several advanced tools for deep cleaning and performance tuning.

### 1. HTML Minification

This tool aggressively reduces the size of all XHTML chapters by:

- Removing unnecessary whitespace and newlines.
- Consolidating redundant tags.
- Stripping HTML comments.
**Benefit**: Significantly reduces the time required for the e-reader to parse and render each chapter, leading to snappier page turns.

### 2. Metadata Scrubbing

E-books often contain "junk" metadata added by various management tools (e.g., Calibre timestamps, version strings, or internal tracking IDs).

- **Privacy**: Removes non-essential metadata fields that may contain information about your library or computer.
- **Efficiency**: Cleans up the `content.opf` file, reducing overhead for the e-reader's library parser.

### 3. Smart Cover Recovery

Ensures every book has a visible cover in the Plato library view:

- If a cover is not explicitly defined in the OPF metadata, the tool scans the first few chapters for large images.
- It automatically updates the OPF manifest to mark the most likely candidate as the official cover image.
