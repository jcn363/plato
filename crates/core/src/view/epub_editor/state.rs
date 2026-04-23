//! State types for the EPUB editor view.

/// Current state of the editor UI.
pub enum EditorState {
    /// Showing the list of chapters to choose from
    ChapterList,
    /// Currently editing a specific chapter
    EditingChapter { index: usize },
}

/// State for search and replace functionality.
pub struct SearchReplaceState {
    /// The current search text.
    pub search_text: String,
    /// The current replacement text.
    pub replace_text: String,
}
