# EPUB Editor for Plato

A command-line EPUB editor that can be used alongside Plato on Kobo e-readers.

## Features

- List EPUB files in a directory
- View and edit EPUB metadata (title, author, language)
- List and edit chapter content (with multi-line support)
- **Undo** functionality for metadata and chapter edits
- **Preview** functionality for metadata and chapters before applying changes
- Improved error handling and descriptive error messages
- Automatic temporary directory cleanup

## Building for Kobo (ARM)

### Prerequisites

Install Rust and the ARM target:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add arm-unknown-linux-gnueabihf
```

### Cross-compile

```bash
cd epub_editor
cargo build --release --target arm-unknown-linux-gnueabihf
```

The binary will be at `target/arm-unknown-linux-gnueabihf/release/epub_editor`

### Copy to Kobo

```bash
scp target/arm-unknown-linux-gnueabihf/release/epub_editor kobo:/mnt/onboard/.adds/plato/bin/
```

## Usage

```bash
./epub_editor /mnt/onboard
```

### Menu Options

1. List EPUB files - Show all EPUB files in the library directory
2. Open EPUB file - Load an EPUB for editing
3. View metadata - Display current metadata
4. Edit metadata - Modify title, author, language (with preview)
5. List chapters - Show all chapters in the EPUB
6. Edit chapter - Edit a specific chapter's content (with multi-line support and preview)
7. Preview changes - Show a summary of current metadata and chapter states
8. Undo last change - Revert the most recent metadata or chapter edit
9. Save EPUB - Save all changes back to the original file
0. Exit - Quit the editor

## Advanced Editing

### Multi-line Chapter Editing

When editing a chapter, you can now enter multiple lines of text. To finish editing, type `END` on a new line.

### Undo Stack

The editor maintains an undo stack for the current session. You can revert multiple changes in reverse chronological order. The undo stack is cleared when you open a new EPUB or exit the application.

## Integration with Plato

To call the editor from Plato:

1. Copy the compiled binary to your Kobo's Plato bin directory
2. Currently, you need to exit Plato and launch the editor separately
3. Future versions may support direct integration

## Troubleshooting

- **File not found:** Ensure the library path provided as an argument is correct.
- **Permission denied:** The editor needs write access to the library directory to save EPUBs.
- **Invalid EPUB:** If an EPUB cannot be opened, it might be DRM-protected or corrupted. The editor works best with non-DRM EPUB files.

## Notes

- The editor creates a temporary directory for working with EPUB files
- Changes are not saved until you explicitly select "Save EPUB"
- The editor works with standard EPUB 2.0 and 3.0 formats

