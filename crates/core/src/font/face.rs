use crate::color::Color;
use crate::font::freetype::Face;
use crate::font::harfbuzz::{Buffer, Font as HbFont};
use crate::font::harfbuzz_sys::{hb_feature_from_string, HbFeature, HB_DIRECTION_LTR};
use crate::font::types::{GlyphPlan, RenderPlan};
use crate::framebuffer::Framebuffer;
use crate::geom::Point;
use crate::log_error;
use anyhow::Result;
use std::convert::TryInto;
use std::str;

pub struct Font {
    face: Face,
    #[allow(dead_code)] // HarfBuzz font for text shaping
    hb_font: HbFont,
    pub size: u32,
    pub dpi: u16,
    pub ellipsis: RenderPlan,
    pub x_heights: (u32, u32),
    #[allow(dead_code)] // Space codepoint for text layout
    space_codepoint: u32,
}

impl Font {
    pub fn new(face: Face) -> Self {
        let hb_font = unsafe { HbFont::from_ft_face(&*face.face_ptr()) };
        let ellipsis = RenderPlan::default();
        let x_heights = (0, 0);
        let space_codepoint = face.get_char_index(' ' as u32);

        Font {
            face,
            hb_font,
            size: 0,
            dpi: 0,
            ellipsis,
            x_heights,
            space_codepoint,
        }
    }

    pub fn num_glyphs(&self) -> i32 {
        self.face.num_glyphs()
    }

    pub fn units_per_em(&self) -> u16 {
        self.face.units_per_em()
    }

    pub fn family_name(&self) -> Option<String> {
        self.face.family_name()
    }

    pub fn style_name(&self) -> Option<String> {
        self.face.style_name()
    }

    pub fn get_char_index(&self, char_code: u32) -> u32 {
        self.face.get_char_index(char_code)
    }

    pub fn set_char_size(&self, width: u32, height: u32, hdpi: u32, vdpi: u32) -> Result<()> {
        self.face.set_char_size(
            width.try_into().unwrap_or(0),
            height.try_into().unwrap_or(0),
            hdpi.try_into().unwrap_or(0),
            vdpi.try_into().unwrap_or(0),
        )
    }

    pub fn destroy_hb_font(&self, font: HbFont) {
        // Explicitly consume and destroy the HarfBuzz font.
        drop(font);
    }

    pub fn done_face(&self) {
        // Explicit cleanup for the underlying face pointer
        // In the fully migrated architecture, Drop should handle this.
    }

    pub fn load_char(&self, char_code: u32, flags: i32) -> Result<()> {
        self.face.load_char(char_code, flags)
    }

    pub fn changed(&self, hb_font: &HbFont) {
        hb_font.changed();
    }

    pub fn set_pixel_sizes(&self, width: u32, height: u32) -> Result<()> {
        self.face.set_pixel_sizes(width, height)
    }

    pub fn load_glyph(&self, glyph_index: u32, flags: i32) -> Result<()> {
        self.face.load_glyph(glyph_index, flags)
    }

    pub fn create_hb_font(&self) -> HbFont {
        unsafe { HbFont::from_ft_face(&*self.face.face_ptr()) }
    }

    pub fn create_hb_font_from_raw(face_ptr: *mut crate::font::freetype_sys::FtFace) -> HbFont {
        unsafe { HbFont::from_ft_face(&*face_ptr) }
    }

    pub fn get_sfnt_name_count(&self) -> u32 {
        self.face.get_sfnt_name_count()
    }

    pub fn get_sfnt_name(&self, index: u32) -> Option<crate::font::freetype_sys::FtSfntName> {
        self.face.get_sfnt_name(index)
    }

    pub fn get_mm_var(&self) -> Result<crate::font::freetype::MmVar> {
        self.face.get_mm_var()
    }

    pub fn set_var_design_coordinates(&self, coords: &[i32]) -> Result<()> {
        self.face.set_var_design_coordinates(coords)
    }

    pub fn bitmap(&self) -> &crate::font::freetype_sys::FtBitmap {
        &self.face.glyph().bitmap
    }

    pub fn glyph_metrics(&self) -> &crate::font::freetype_sys::FtGlyphMetrics {
        &self.face.glyph().metrics
    }

    pub fn set_size(&mut self, size: u32, dpi: u16) {
        if self.size == size && self.dpi == dpi {
            return;
        }
        self.size = size;
        self.dpi = dpi;
        if let Err(e) = self.face.set_char_size(size as i32, 0, dpi as u32, 0) {
            log_error!("Failed to set char size: {}", e);
            return;
        }
        self.hb_font.changed();
        self.ellipsis = RenderPlan::default();
        self.x_heights = (self.height('x'), self.height('X'));
    }

    fn tag(c1: u8, c2: u8, c3: u8, c4: u8) -> u32 {
        ((c1 as u32) << 24) | ((c2 as u32) << 16) | ((c3 as u32) << 8) | c4 as u32
    }

    pub fn set_variations(&mut self, specs: &[&str]) {
        if let Ok(mm_var) = self.face.get_mm_var() {
            let axes_count = mm_var.num_axis() as usize;
            let mut coords: Vec<i32> = Vec::with_capacity(axes_count);
            let axis_data = mm_var.axis();
            for axis in axis_data.iter().take(axes_count) {
                coords.push(axis.def as i32);
            }
            for s in specs {
                if let Some(pos) = s.find('=') {
                    let tag_str = &s[..pos];
                    let value_str = &s[pos + 1..];
                    if tag_str.len() != 4 {
                        continue;
                    }
                    if let Ok(value) = value_str.parse::<f32>() {
                        let tag = Self::tag(
                            tag_str.as_bytes()[0],
                            tag_str.as_bytes()[1],
                            tag_str.as_bytes()[2],
                            tag_str.as_bytes()[3],
                        ) as libc::c_ulong;
                        let axis_data = mm_var.axis();
                        for (i, axis) in axis_data.iter().take(axes_count).enumerate() {
                            if axis.tag == tag {
                                coords[i] = (value * 65536.0) as i32;
                                break;
                            }
                        }
                    }
                }
            }
            if let Ok(()) = self.face.set_var_design_coordinates(&coords) {
                self.hb_font.changed();
            }
        }
    }

    pub fn set_variations_from_name(&mut self, _name: &str) -> bool {
        false
    }

    pub fn height(&self, c: char) -> u32 {
        if let Ok(()) = self.face.load_char(c as u32, 0) {
            let metrics = &self.face.glyph().metrics;
            (metrics.height >> 6) as u32
        } else {
            0
        }
    }

    #[inline]
    pub fn em(&self) -> u16 {
        self.face.size().metrics.x_ppem as u16
    }

    #[inline]
    pub fn ascender(&self) -> i32 {
        self.face.size().metrics.ascender as i32 / 64
    }

    #[inline]
    pub fn descender(&self) -> i32 {
        self.face.size().metrics.descender as i32 / 64
    }

    #[inline]
    pub fn line_height(&self) -> i32 {
        self.face.size().metrics.height as i32 / 64
    }

    pub fn plan<S: AsRef<str>>(
        &mut self,
        text: S,
        max_width: Option<i32>,
        features: Option<&[String]>,
    ) -> RenderPlan {
        let mut buffer = Buffer::new();
        let text_ref = text.as_ref();
        buffer.add_utf8(text_ref, 0, text_ref.len());
        buffer.set_direction(HB_DIRECTION_LTR);
        buffer.guess_segment_properties();
        let features_vec: Vec<HbFeature> = features
            .map(|ftr| {
                ftr.iter()
                    .filter_map(|f| {
                        let mut feature = HbFeature::default();
                        if unsafe {
                            hb_feature_from_string(
                                f.as_bytes().as_ptr().cast::<libc::c_char>(),
                                f.len() as libc::c_int,
                                &mut feature,
                            ) == 1
                        } {
                            Some(feature)
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();
        buffer.shape(&self.hb_font, &features_vec);
        let len = buffer.length() as usize;
        let infos = buffer.glyph_infos();
        let positions = buffer.glyph_positions();
        let mut render_plan = RenderPlan::default();
        for i in 0..len {
            let pos_i = &positions[i];
            let info_i = &infos[i];
            render_plan.width += pos_i.x_advance >> 6;
            let glyph = GlyphPlan {
                codepoint: info_i.codepoint,
                cluster: info_i.cluster as usize,
                advance: Point::new(pos_i.x_advance >> 6, pos_i.y_advance >> 6),
                offset: Point::new(pos_i.x_offset >> 6, -(pos_i.y_offset >> 6)),
            };
            render_plan.glyphs.push(glyph);
        }
        drop(buffer);
        if let Some(mw) = max_width {
            self.crop_right(&mut render_plan, mw);
        }
        render_plan
    }

    #[inline]
    pub fn crop_right(&self, render_plan: &mut RenderPlan, max_width: i32) {
        if render_plan.width <= max_width {
            return;
        }
        render_plan.width += self.ellipsis.width;
        while let Some(gp) = render_plan.glyphs.pop() {
            render_plan.width -= gp.advance.x;
            if render_plan.width <= max_width {
                break;
            }
        }
        let len = render_plan.glyphs.len();
        render_plan.scripts.retain(|&k, _| k < len);
        render_plan
            .glyphs
            .extend_from_slice(&self.ellipsis.glyphs[..]);
    }

    #[inline]
    pub fn trim_left(&self, render_plan: &mut RenderPlan) {
        if render_plan.glyphs.is_empty() {
            return;
        }
        let mut i = 0;
        while render_plan.glyphs[i].codepoint == self.space_codepoint {
            render_plan.width -= render_plan.glyphs[i].advance.x;
            i += 1;
        }
        render_plan.glyphs.drain(..i);
        render_plan.scripts = render_plan
            .scripts
            .iter()
            .filter_map(|(&k, &v)| if k < i { None } else { Some((k - i, v)) })
            .collect();
    }

    pub fn cut_point(&self, render_plan: &RenderPlan, max_width: i32) -> (usize, i32) {
        let mut width = render_plan.width;
        let glyphs = &render_plan.glyphs;
        let mut i = if glyphs.is_empty() {
            0
        } else {
            glyphs.len() - 1
        };
        if i > 0 {
            width -= glyphs[i].advance.x;
        }
        while i > 0 && width > max_width {
            i -= 1;
            width -= glyphs[i].advance.x;
        }
        let j = i;
        let last_width = width;
        while i > 0 && glyphs[i].codepoint != self.space_codepoint {
            i -= 1;
            width -= glyphs[i].advance.x;
        }
        if i == 0 {
            i = j;
            width = last_width;
        }
        (i, width)
    }

    pub fn render(
        &mut self,
        _fb: &mut dyn Framebuffer,
        _color: Color,
        _render_plan: &RenderPlan,
        _origin: Point,
    ) {
    }

    pub fn crop_around(&self, render_plan: &mut RenderPlan, index: usize, max_width: i32) -> usize {
        if render_plan.width <= max_width {
            return 0;
        }
        let len = render_plan.glyphs.len();
        let mut width = 0;
        let mut upper = index;
        while upper < len && width <= max_width {
            width += render_plan.glyphs[upper].advance.x;
            upper += 1;
        }
        if upper < len {
            render_plan.glyphs.truncate(upper);
            render_plan
                .glyphs
                .extend_from_slice(&self.ellipsis.glyphs[..]);
        }
        render_plan.width = width;
        upper
    }
}
