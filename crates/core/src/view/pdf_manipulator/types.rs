//! Types and constants for PDF manipulation view.

/// Maximum file size warning threshold in MB.
pub const WARNING_FILE_SIZE: u64 = 30;

/// Padding around UI elements.
pub const PADDING: i32 = 10;

/// Height of buttons.
pub const BUTTON_HEIGHT: i32 = 60;

/// Spacing between buttons.
pub const BUTTON_SPACING: i32 = 10;

/// State for redaction selection.
#[derive(Clone, PartialEq)]
pub enum RedactionState {
    /// No redaction in progress.
    None,
    /// Selecting a redaction region.
    Selecting {
        /// Start coordinates (x, y).
        start: (i32, i32),
        /// End coordinates (x, y).
        end: (i32, i32),
    },
}

/// Current manipulation mode.
pub enum ManipulationMode {
    /// Selecting a PDF file.
    SelectFile,
    /// Selecting an action to perform.
    SelectAction,
    /// Selecting a page for redaction.
    SelectRedactionPage,
    /// Defining a redaction region.
    DefiningRedaction {
        /// Path to the PDF file.
        file_path: std::path::PathBuf,
        /// Index of the page to redact.
        page_index: usize,
        /// Redaction region being defined.
        region: Option<crate::document::pdf_manipulator::RedactionRegion>,
    },
    /// Processing a manipulation.
    Processing,
}
