use crate::font::freetype_sys::FtFace;
use crate::font::harfbuzz_sys::*;
use std::ptr;

pub struct Font {
    font: *mut HbFont,
}

impl Font {
    pub fn from_ft_face(face: &FtFace) -> Self {
        unsafe {
            let font = hb_ft_font_create(face as *const FtFace as *mut FtFace, ptr::null());
            Font { font }
        }
    }

    #[inline]
    pub fn changed(&self) {
        unsafe { hb_ft_font_changed(self.font) }
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut HbFont {
        self.font
    }
}

impl Drop for Font {
    fn drop(&mut self) {
        if !self.font.is_null() {
            unsafe { hb_font_destroy(self.font) }
        }
    }
}

pub struct Buffer {
    buf: *mut HbBuffer,
}

impl Buffer {
    pub fn new() -> Self {
        unsafe {
            let buf = hb_buffer_create();
            Buffer { buf }
        }
    }

    #[inline]
    pub fn add_utf8(&mut self, text: &str, offset: usize, len: usize) {
        unsafe {
            hb_buffer_add_utf8(
                self.buf,
                text.as_ptr() as *const libc::c_char,
                len as libc::c_int,
                offset as libc::c_uint,
                len as libc::c_int,
            );
        }
    }

    #[inline]
    pub fn set_direction(&mut self, direction: HbDirection) {
        unsafe { hb_buffer_set_direction(self.buf, direction) }
    }

    #[inline]
    pub fn guess_segment_properties(&mut self) {
        unsafe { hb_buffer_guess_segment_properties(self.buf) }
    }

    pub fn shape(&mut self, font: &Font, features: &[HbFeature]) {
        unsafe {
            hb_shape(
                font.as_ptr(),
                self.buf,
                features.as_ptr(),
                features.len() as libc::c_uint,
            );
        }
    }

    #[inline]
    pub fn length(&self) -> u32 {
        unsafe { hb_buffer_get_length(self.buf) }
    }

    pub fn glyph_infos(&self) -> Vec<HbGlyphInfo> {
        unsafe {
            let mut len: libc::c_uint = 0;
            let infos = hb_buffer_get_glyph_infos(self.buf, &mut len);
            if infos.is_null() || len == 0 {
                return Vec::new();
            }
            std::slice::from_raw_parts(infos, len as usize).to_vec()
        }
    }

    pub fn glyph_positions(&self) -> Vec<HbGlyphPosition> {
        unsafe {
            let mut len: libc::c_uint = 0;
            let positions = hb_buffer_get_glyph_positions(self.buf, &mut len);
            if positions.is_null() || len == 0 {
                return Vec::new();
            }
            std::slice::from_raw_parts(positions, len as usize).to_vec()
        }
    }

    #[inline]
    pub fn direction(&self) -> HbDirection {
        unsafe { hb_buffer_get_direction(self.buf) }
    }

    #[inline]
    pub fn language(&self) -> *const HbLanguage {
        unsafe { hb_buffer_get_language(self.buf) }
    }

    #[inline]
    pub fn script(&self) -> HbScript {
        unsafe { hb_buffer_get_script(self.buf) }
    }

    #[inline]
    pub fn clear(&mut self) {
        unsafe { hb_buffer_clear_contents(self.buf) }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if !self.buf.is_null() {
            unsafe { hb_buffer_destroy(self.buf) }
        }
    }
}

pub fn feature_from_string(s: &str) -> Option<HbFeature> {
    unsafe {
        let mut feature = HbFeature::default();
        let result = hb_feature_from_string(
            s.as_ptr() as *const libc::c_char,
            s.len() as libc::c_int,
            &mut feature,
        );
        if result != 0 {
            Some(feature)
        } else {
            None
        }
    }
}

pub const DIRECTION_LTR: HbDirection = HB_DIRECTION_LTR;
pub const DIRECTION_RTL: HbDirection = HB_DIRECTION_RTL;
pub const DIRECTION_TTB: HbDirection = HB_DIRECTION_TTB;
pub const DIRECTION_BTT: HbDirection = HB_DIRECTION_BTT;
