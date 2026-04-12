use std::rc::Rc;
use crate::font::library::FontLibrary;
use crate::font::freetype::Face;
use crate::font::harfbuzz::Font as HbFont;
use crate::font::RenderPlan;

pub struct Font {
    #[allow(dead_code)] library: Rc<FontLibrary>,
    face: Face,
    #[allow(dead_code)] hb_font: HbFont,
    pub size: u32,
    pub dpi: u16,
    pub ellipsis: RenderPlan,
    pub x_heights: (u32, u32),
    #[allow(dead_code)] space_codepoint: u32,
}

impl Font {
    pub fn new(library: Rc<FontLibrary>, face: Face) -> Self {
        let hb_font = unsafe { HbFont::from_ft_face(&*face.face_ptr()) };
        let ellipsis = RenderPlan::default();
        let x_heights = (0, 0);
        let space_codepoint = face.get_char_index(' ' as u32);
        
        Font {
            library,
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
}
