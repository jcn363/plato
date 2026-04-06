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
//! - `find_page_by_name()` - Page lookup utility ✓
//!
//! ## Size
//! Large (~700 lines total), but mostly menu UI creation code.
//!
//! ## Dependencies
//! Moderate dependencies on Reader state and document properties.
//!
//! ## Future Use
//! Types from reader_core will be imported when methods are extracted.

use crate::helpers::AsciiExtension;
use crate::metadata::Info;
use septem::Roman;

/// Find page index by named page reference
///
/// Resolves page names using multiple formats:
/// - Numeric pages (e.g., "42" resolves to page 42)
/// - Alphabetic pages (e.g., "B" resolves to first page starting with "B")
/// - Roman numerals (e.g., "V" resolves to Roman numeral page 5)
///
/// # Arguments
/// - `info`: Metadata containing reader info and page names
/// - `name`: Page name to look up
///
/// # Returns
/// Page index if found, or None if name doesn't match any page
#[allow(dead_code)]
pub(crate) fn find_page_by_name(info: &Info, name: &str) -> Option<usize> {
    info.reader.as_ref().and_then(|r| {
        if let Ok(a) = name.parse::<u32>() {
            r.page_names
                .iter()
                .filter_map(|(i, s)| s.parse::<u32>().ok().map(|b| (b, i)))
                .filter(|(b, _)| *b <= a)
                .max_by(|x, y| x.0.cmp(&y.0))
                .map(|(b, i)| *i + (a - b) as usize)
        } else if let Some(a) = name.chars().next().and_then(|c| c.to_alphabetic_digit()) {
            r.page_names
                .iter()
                .filter_map(|(i, s)| {
                    s.chars()
                        .next()
                        .and_then(|c| c.to_alphabetic_digit())
                        .map(|c| (c, i))
                })
                .filter(|(b, _)| *b <= a)
                .max_by(|x, y| x.0.cmp(&y.0))
                .map(|(b, i)| *i + (a - b) as usize)
        } else if let Ok(a) = name.parse::<Roman>() {
            let a_val = *a;
            r.page_names
                .iter()
                .filter_map(|(i, s)| s.parse::<Roman>().ok().map(|b| (b, i)))
                .filter(|(b, _)| {
                    (*b).cmp(&Roman::from_unchecked(a_val)) != std::cmp::Ordering::Greater
                })
                .max_by(|x, y| x.0.cmp(&y.0))
                .map(|(b, i)| *i + (a_val - *b) as usize)
        } else {
            None
        }
    })
}
