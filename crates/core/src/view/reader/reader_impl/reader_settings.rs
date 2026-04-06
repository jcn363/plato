//! Reader Settings Module
//!
//! Handles all font, contrast, zoom settings menus and configuration.
//!
//! ## Methods to Move Here
//! - `toggle_font_family_menu()` - Font selection (~60 lines)
//! - `toggle_font_size_menu()` - Font size selection (~60 lines)
//! - `toggle_text_align_menu()` - Text alignment (~60 lines)
//! - `toggle_line_height_menu()` - Line height settings (~55 lines)
//! - `toggle_contrast_exponent_menu()` - Contrast exponent (~50 lines)
//! - `toggle_contrast_gray_menu()` - Contrast gray level (~50 lines)
//! - `toggle_margin_width_menu()` - Margin width (~70 lines)
//! - `toggle_page_menu()` - Page display options (~60 lines)
//! - Settings setter methods: `set_font_size()`, `set_text_align()`, etc. (~120 lines total)
//! - `toggle_title_menu()` - Title/document menu
//! - `toggle_selection_menu()` - Selection/context menu
//!
//! ## Size
//! Large (~700 lines total), but mostly menu UI creation code.
//!
//! ## Dependencies
//! Moderate dependencies on Reader state and document properties.
//!
//! ## Future Use
//! Types from reader_core will be imported when methods are extracted.
