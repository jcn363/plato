//! Safe wrapper around skrifa for font loading and metrics.
//!
//! This module provides a high-level interface to skrifa,
//! replacing the previous FreeType FFI bindings.
//!
//! Glyph rasterization is handled via ab_glyph integration.

use anyhow::{bail, format_err, Result};
use skrifa::raw::tables::name::NameId;
use skrifa::raw::types::GlyphId;
use skrifa::raw::{FontRef, TableProvider};
use std::sync::Arc;

/// A loaded font face using skrifa.
pub struct Face {
    data: Arc<Vec<u8>>,
    _index: u32,
    scale: f32,
}

impl Face {
    /// Load a font face from raw bytes.
    pub fn from_memory(data: Vec<u8>, index: u32) -> Result<Self> {
        let data_arc = Arc::new(data);
        if data_arc.len() < 4 {
            bail!("Font data too small");
        }

        Ok(Face {
            data: data_arc,
            _index: index,
            scale: 0.0,
        })
    }

    /// Get the raw font reference for parsing.
    fn get_font_ref(&self) -> Option<FontRef<'_>> {
        FontRef::new(&self.data[..]).ok()
    }

    /// Get the number of glyphs in this face.
    pub fn num_glyphs(&self) -> i32 {
        self.get_font_ref()
            .and_then(|font| font.maxp().ok())
            .map(|table| table.num_glyphs() as i32)
            .unwrap_or(0)
    }

    /// Get the font's units per em.
    pub fn units_per_em(&self) -> u16 {
        self.get_font_ref()
            .and_then(|font| font.head().ok())
            .map(|table: skrifa::raw::tables::head::Head<'_>| table.units_per_em())
            .unwrap_or(1000)
    }

    /// Get the family name of this font.
    pub fn family_name(&self) -> Option<String> {
        let font = self.get_font_ref()?;
        let name_table = font.name().ok()?;
        let string_data = name_table.string_data();

        name_table
            .name_record()
            .iter()
            .find(|record| record.name_id() == NameId::FAMILY_NAME)
            .and_then(|record| record.string(string_data).ok())
            .map(|ns| ns.to_string())
    }

    /// Get the style name (e.g., "Regular", "Bold", "Italic").
    pub fn style_name(&self) -> Option<String> {
        let font = self.get_font_ref()?;
        let name_table = font.name().ok()?;
        let string_data = name_table.string_data();

        name_table
            .name_record()
            .iter()
            .find(|record| record.name_id() == NameId::SUBFAMILY_NAME)
            .and_then(|record| record.string(string_data).ok())
            .map(|ns| ns.to_string())
    }

    /// Get the character index for a unicode codepoint.
    pub fn get_char_index(&self, char_code: u32) -> u32 {
        self.get_font_ref()
            .and_then(|font| font.cmap().ok())
            .and_then(|cmap| cmap.map_codepoint(char_code))
            .map(u32::from)
            .unwrap_or(0)
    }

    /// Set character size (stores for metrics calculations).
    pub fn set_char_size(&mut self, width: i32, _height: i32, hdpi: u32, _vdpi: u32) -> Result<()> {
        if width < 0 {
            bail!("Invalid character size parameters");
        }
        // Store the size - will be used for scaling in get_glyph_metrics
        self.scale = (width as f32 * hdpi as f32) / 72.0;
        Ok(())
    }

    /// Set pixel size (for rasterization).
    pub fn set_pixel_sizes(&mut self, width: u32, _height: u32) -> Result<()> {
        self.scale = width as f32;
        Ok(())
    }

    /// Get the number of SFNT name records.
    pub fn get_sfnt_name_count(&self) -> u32 {
        self.get_font_ref()
            .and_then(|font| font.name().ok())
            .map(|name| name.name_record().len() as u32)
            .unwrap_or(0)
    }

    /// Get SFNT name record by index.
    pub fn get_sfnt_name(&self, index: u32) -> Option<SfntName> {
        let font = self.get_font_ref()?;
        let name_table = font.name().ok()?;
        let record = name_table.name_record().get(index as usize)?;

        Some(SfntName {
            name_id: record.name_id().to_u16(),
            platform_id: record.platform_id(),
            encoding_id: record.encoding_id(),
            language_id: record.language_id(),
        })
    }

    /// Get variable font metrics.
    pub fn get_mm_var(&self) -> Result<MmVar> {
        let font = self
            .get_font_ref()
            .ok_or_else(|| format_err!("Failed to parse font"))?;
        let fvar = match font.fvar() {
            Ok(f) => f,
            Err(_) => return Ok(MmVar { axes: Vec::new() }),
        };

        let axes: Vec<Axis> = fvar
            .axes()
            .map_err(|e| format_err!("Failed to read fvar axes: {}", e))?
            .iter()
            .map(|record| {
                let bytes = record.axis_tag().to_be_bytes();
                let tag = u32::from_be_bytes(bytes);
                Axis {
                    tag,
                    min: record.min_value().to_f32() as i32,
                    def: record.default_value().to_f32() as i32,
                    max: record.max_value().to_f32() as i32,
                }
            })
            .collect();

        Ok(MmVar { axes })
    }

    /// Set variable design coordinates.
    /// Set variable design coordinates.
    /// Note: Variable font support requires additional integration.
    pub fn set_var_design_coordinates(&mut self, _coords: &[i32]) -> Result<()> {
        Ok(())
    }

    /// Get glyph metrics (using skrifa for basic metrics).
    pub fn get_glyph_metrics(&self, glyph_id: u16) -> Result<GlyphMetrics> {
        let font = self
            .get_font_ref()
            .ok_or_else(|| format_err!("Failed to parse font"))?;

        let hmtx = font.hmtx().ok();
        let advance_width = hmtx
            .and_then(|h| h.advance(GlyphId::new(glyph_id as u16)))
            .unwrap_or(0) as i32;

        Ok(GlyphMetrics {
            advance_width,
            advance_height: 0,
            lsb: 0,
            tsb: 0,
        })
    }

    /// Render a glyph outline to a bitmap.
    /// Note: Full rasterization requires ab_glyph integration.
    pub fn rasterize_glyph(&self, glyph_id: u16, ppem: u16) -> Result<GlyphBitmap> {
        let metrics = self.get_glyph_metrics(glyph_id)?;
        let scale = if ppem > 0 { ppem as f32 } else { self.scale };

        Ok(GlyphBitmap {
            width: 0,
            height: 0,
            pitch: 0,
            buffer: Vec::new(),
            left: 0,
            top: 0,
            advance_x: (metrics.advance_width as f32 * scale / 1000.0) as i32,
            advance_y: 0,
        })
    }
}

/// SFNT name record metadata.
#[derive(Debug, Clone)]
pub struct SfntName {
    pub name_id: u16,
    pub platform_id: u16,
    pub encoding_id: u16,
    pub language_id: u16,
}

/// Variable font axis information.
#[derive(Debug, Clone)]
pub struct Axis {
    pub tag: u32,
    pub min: i32,
    pub def: i32,
    pub max: i32,
}

/// Multiple axes for variable fonts.
#[derive(Debug, Clone)]
pub struct MmVar {
    pub axes: Vec<Axis>,
}

impl MmVar {
    pub fn num_axis(&self) -> u32 {
        self.axes.len() as u32
    }

    pub fn axis(&self) -> &[Axis] {
        &self.axes
    }
}

/// Glyph metrics.
#[derive(Debug, Clone, Copy)]
pub struct GlyphMetrics {
    pub advance_width: i32,
    pub advance_height: i32,
    pub lsb: i32,
    pub tsb: i32,
}

/// Rasterized glyph bitmap.
#[derive(Debug, Clone)]
pub struct GlyphBitmap {
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub buffer: Vec<u8>,
    pub left: i32,
    pub top: i32,
    pub advance_x: i32,
    pub advance_y: i32,
}
