//! Font Rendering Subsystem
//!
//! This module provides font handling for Plato, using pure Rust libraries:
//! - skrifa for font loading and parsing
//! - rustybuzz for text shaping and layout
//! - ab_glyph for glyph rasterization (when needed)
//!
//! ## Architecture
//!
//! - **skrifa_wrapper**: Safe wrapper around skrifa for font loading and metrics
//! - **rustybuzz_wrapper**: Safe wrapper around rustybuzz for text shaping
//!
//! The subsystem handles:
//! - Font discovery and loading from filesystem
//! - Embedded font resources
//! - Glyph rasterization via ab_glyph (optional)
//! - Text shaping (glyph positioning) via rustybuzz
//! - Variable font support
//! - Complex script handling

pub mod face;
pub mod library;
mod rustybuzz_wrapper;
pub mod shaper;
mod skrifa_wrapper;
mod types;

// Public re-exports
pub use self::face::Font;
pub use self::library::FontOpener;
pub use self::types::{GlyphPlan, RenderPlan};

// ===========================================================================
// Imports and Re-exports
// ===========================================================================

use crate::device::CURRENT_DEVICE;
use crate::helpers::walkdir_visible;
use crate::{log_error, log_warn};
use anyhow::{format_err, Error};
use bitflags::bitflags;
use globset::Glob;
use rustc_hash::FxHashMap;
use std::collections::BTreeSet;
use std::path::Path;
use std::sync::LazyLock;

// Font sizes in 1/64th of a point
pub const FONT_SIZES: [u32; 3] = [349, 524, 629];

pub const KEYBOARD_FONT_SIZES: [u32; 2] = [337, 843];

pub const DISPLAY_FONT_SIZE: u32 = 2516;

pub const NORMAL_STYLE: Style = Style {
    family: Family::SansSerif,
    variant: Variant::REGULAR,
    size: FONT_SIZES[1],
};

pub const SPECIAL_STYLE: Style = Style {
    family: Family::SansSerif,
    variant: Variant::ITALIC,
    size: FONT_SIZES[1],
};

pub const KBD_CHAR: Style = Style {
    family: Family::Keyboard,
    variant: Variant::REGULAR,
    size: KEYBOARD_FONT_SIZES[1],
};

pub const KBD_LABEL: Style = Style {
    family: Family::Keyboard,
    variant: Variant::REGULAR,
    size: FONT_SIZES[0],
};

pub const DISPLAY_STYLE: Style = Style {
    family: Family::Display,
    variant: Variant::REGULAR,
    size: DISPLAY_FONT_SIZE,
};

pub static MD_TITLE: LazyLock<Style> = LazyLock::new(|| {
    // Compute the ratio between the physical width of the
    // current device and that of the Aura ONE.
    let ratio = (CURRENT_DEVICE.dims.0 as f32 * 300.0) / (CURRENT_DEVICE.dpi as f32 * 1404.0);
    let size = ((FONT_SIZES[2] as f32 * ratio) as u32).clamp(FONT_SIZES[1], FONT_SIZES[2]);
    Style {
        family: Family::Serif,
        variant: Variant::ITALIC,
        size,
    }
});

// ===========================================================================
// Font Size Constants and Style Definitions
// ===========================================================================

pub const MD_AUTHOR: Style = Style {
    family: Family::Serif,
    variant: Variant::REGULAR,
    size: FONT_SIZES[1],
};

pub const MD_YEAR: Style = NORMAL_STYLE;

pub const MD_KIND: Style = Style {
    family: Family::SansSerif,
    variant: Variant::BOLD,
    size: FONT_SIZES[0],
};

pub const MD_SIZE: Style = Style {
    family: Family::SansSerif,
    variant: Variant::REGULAR,
    size: FONT_SIZES[0],
};

// ===========================================================================
// Embedded Font Data Module
// ===========================================================================
// NOTE: Removed - font data now loaded from filesystem using pure Rust stack
// (skrifa, rustybuzz, ab_glyph) instead of embedded MuPDF font data.

// ===========================================================================
// Font Family and Discovery Utilities
// ===========================================================================

pub const SLIDER_VALUE: Style = MD_SIZE;

pub struct FontFamily {
    pub regular: Font,
    pub italic: Font,
    pub bold: Font,
    pub bold_italic: Font,
}

pub fn family_names<P: AsRef<Path>>(search_path: P) -> Result<BTreeSet<String>, Error> {
    if !search_path.as_ref().exists() {
        return Err(format_err!("the search path doesn't exist"));
    }

    let opener = FontOpener::new()?;
    let glob = Glob::new("**/*.[ot]tf")?.compile_matcher();

    let mut families = BTreeSet::new();

    for entry in walkdir_visible(search_path.as_ref()) {
        let path = entry.path();
        if !glob.is_match(path) {
            continue;
        }
        if let Ok(font) = opener.open(path).map_err(|e| {
            log_error!(
                "Failed to load font '{}': {}. Please ensure the font file exists and is valid.",
                path.display(),
                e
            )
        }) {
            if let Some(family_name) = font.family_name() {
                families.insert(family_name.to_string());
            } else {
                log_warn!("Can't get the family name of '{}'.", path.display());
            }
        }
    }

    Ok(families)
}

impl FontFamily {
    pub fn from_name<P: AsRef<Path>>(
        family_name: &str,
        search_path: P,
    ) -> Result<FontFamily, Error> {
        let opener = FontOpener::new()?;
        let glob = Glob::new("**/*.[ot]tf")?.compile_matcher();
        let mut styles = FxHashMap::default();

        for entry in walkdir_visible(search_path.as_ref()) {
            let path = entry.path();
            if !glob.is_match(path) {
                continue;
            }
            if let Ok(font) = opener
                .open(path)
                .map_err(|e| log_error!("Failed to load font '{}': {}. Please ensure the font file exists and is valid.", path.display(), e))
            {
                if font.family_name().as_deref() == Some(family_name) {
                    styles.insert(
                        font.style_name().unwrap_or_else(|| "Regular".to_string()),
                        path.to_path_buf(),
                    );
                }
            }
        }

        let regular_path = if styles.len() == 1 {
            styles
                .values()
                .next()
                .ok_or_else(|| format_err!("styles is empty"))?
        } else {
            styles
                .get("Regular")
                .or_else(|| styles.get("Roman"))
                .or_else(|| styles.get("Book"))
                .ok_or_else(|| format_err!("can't find regular style"))?
        };
        let italic_path = styles
            .get("Italic")
            .or_else(|| styles.get("Book Italic"))
            .or_else(|| styles.get("Regular Italic"))
            .unwrap_or(regular_path);
        let bold_path = styles
            .get("Bold")
            .or_else(|| styles.get("Semibold"))
            .or_else(|| styles.get("SemiBold"))
            .or_else(|| styles.get("Medium"))
            .unwrap_or(regular_path);
        let bold_italic_path = styles
            .get("Bold Italic")
            .or_else(|| styles.get("SemiBold Italic"))
            .or_else(|| styles.get("Medium Italic"))
            .unwrap_or(italic_path);
        Ok(FontFamily {
            regular: opener.open(regular_path)?,
            italic: opener.open(italic_path)?,
            bold: opener.open(bold_path)?,
            bold_italic: opener.open(bold_italic_path)?,
        })
    }
}

pub struct Fonts {
    pub sans_serif: FontFamily,
    pub serif: FontFamily,
    pub monospace: FontFamily,
    pub keyboard: Font,
    pub display: Font,
}

impl Fonts {
    pub fn load() -> Result<Fonts, Error> {
        let opener = FontOpener::new()?;
        let mut fonts = Fonts {
            sans_serif: FontFamily {
                regular: opener.open("fonts/NotoSans-Regular.ttf")?,
                italic: opener.open("fonts/NotoSans-Italic.ttf")?,
                bold: opener.open("fonts/NotoSans-Bold.ttf")?,
                bold_italic: opener.open("fonts/NotoSans-BoldItalic.ttf")?,
            },
            serif: FontFamily {
                regular: opener.open("fonts/NotoSerif-Regular.ttf")?,
                italic: opener.open("fonts/NotoSerif-Italic.ttf")?,
                bold: opener.open("fonts/NotoSerif-Bold.ttf")?,
                bold_italic: opener.open("fonts/NotoSerif-BoldItalic.ttf")?,
            },
            monospace: FontFamily {
                regular: opener.open("fonts/SourceCodeVariable-Roman.otf")?,
                italic: opener.open("fonts/SourceCodeVariable-Italic.otf")?,
                bold: opener.open("fonts/SourceCodeVariable-Roman.otf")?,
                bold_italic: opener.open("fonts/SourceCodeVariable-Italic.otf")?,
            },
            keyboard: opener.open("fonts/VarelaRound-Regular.ttf")?,
            display: opener.open("fonts/Cormorant-Regular.ttf")?,
        };
        fonts.monospace.bold.set_variations(&["wght=600"]);
        fonts.monospace.bold_italic.set_variations(&["wght=600"]);
        Ok(fonts)
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Variant: u8 {
        const REGULAR = 0;
        const ITALIC = 1;
        const BOLD = 2;
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Family {
    SansSerif,
    Serif,
    Monospace,
    Keyboard,
    Display,
}

pub struct Style {
    family: Family,
    variant: Variant,
    pub size: u32,
}

pub fn font_from_variant(family: &mut FontFamily, variant: Variant) -> &mut Font {
    if variant.contains(Variant::ITALIC | Variant::BOLD) {
        &mut family.bold_italic
    } else if variant.contains(Variant::ITALIC) {
        &mut family.italic
    } else if variant.contains(Variant::BOLD) {
        &mut family.bold
    } else {
        &mut family.regular
    }
}

pub fn font_from_style<'a>(fonts: &'a mut Fonts, style: &Style, dpi: u16) -> &'a mut Font {
    let font = match style.family {
        Family::SansSerif => {
            let family = &mut fonts.sans_serif;
            font_from_variant(family, style.variant)
        }
        Family::Serif => {
            let family = &mut fonts.serif;
            font_from_variant(family, style.variant)
        }
        Family::Monospace => {
            let family = &mut fonts.monospace;
            font_from_variant(family, style.variant)
        }
        Family::Keyboard => &mut fonts.keyboard,
        Family::Display => &mut fonts.display,
    };
    font.set_size(style.size, dpi);
    font
}
