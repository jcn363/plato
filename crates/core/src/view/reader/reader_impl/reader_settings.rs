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

use crate::document::{Location, SimpleTocEntry, TocEntry, TocLocation};
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

/// Build table of contents from document structure
///
/// Converts the simple TOC structure from document metadata into a hierarchical
/// TocEntry structure suitable for navigation UI.
///
/// # Arguments
/// - `info`: Metadata containing document TOC
/// - `find_page_fn`: Function to resolve page names to page indices
///
/// # Returns
/// Hierarchical TOC structure, or None if no TOC in document
#[allow(dead_code)]
pub(crate) fn build_toc<F>(info: &Info, find_page_fn: F) -> Option<Vec<TocEntry>>
where
    F: Fn(&str) -> Option<usize> + Copy,
{
    let mut index = 0;
    info.toc
        .as_ref()
        .map(|simple_toc| build_toc_aux(simple_toc, &mut index, find_page_fn))
}

/// Recursively build table of contents entries
///
/// Internal helper for building TOC hierarchy. Processes each TOC entry and
/// recursively processes child entries for containers.
///
/// # Arguments
/// - `simple_toc`: Flat list of TOC entries
/// - `index`: Mutable counter for assigning sequential indices
/// - `find_page_fn`: Function to resolve named page references
///
/// # Returns
/// Vector of TocEntry with full hierarchy populated
#[allow(dead_code)]
pub(crate) fn build_toc_aux<F>(
    simple_toc: &[SimpleTocEntry],
    index: &mut usize,
    find_page_fn: F,
) -> Vec<TocEntry>
where
    F: Fn(&str) -> Option<usize> + Copy,
{
    let mut toc = Vec::with_capacity(simple_toc.len());
    for entry in simple_toc {
        *index += 1;
        match entry {
            SimpleTocEntry::Leaf(title, location)
            | SimpleTocEntry::Container(title, location, _) => {
                let current_title = title.clone();
                let current_location = match location {
                    TocLocation::Uri(uri) if uri.starts_with('\'') => find_page_fn(&uri[1..])
                        .map(Location::Exact)
                        .unwrap_or_else(|| location.clone().into()),
                    _ => location.clone().into(),
                };
                let current_index = *index;
                let current_children = if let SimpleTocEntry::Container(_, _, children) = entry {
                    build_toc_aux(children, index, find_page_fn)
                } else {
                    Vec::new()
                };
                toc.push(TocEntry {
                    title: current_title,
                    location: current_location,
                    index: current_index,
                    children: current_children,
                });
            }
        }
    }
    toc
}
