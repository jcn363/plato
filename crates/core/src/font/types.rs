#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    Serif,
    SansSerif,
    Display,
    Keyboard,
    Monospace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    REGULAR,
    ITALIC,
    BOLD,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Style {
    pub family: Family,
    pub variant: Variant,
    pub size: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct GlyphPlan {
    pub codepoint: u32,
    pub cluster: usize,
    pub offset: crate::geom::Point,
    pub advance: crate::geom::Point,
}

#[derive(Debug, Clone)]
pub struct RenderPlan {
    pub width: i32,
    pub scripts: rustc_hash::FxHashMap<usize, crate::font::harfbuzz_sys::HbScript>,
    pub glyphs: Vec<GlyphPlan>,
}

impl Default for RenderPlan {
    fn default() -> RenderPlan {
        RenderPlan {
            width: 0,
            scripts: rustc_hash::FxHashMap::default(),
            glyphs: Vec::new(),
        }
    }
}

impl RenderPlan {
    pub fn scale(&self, scale: f32) -> RenderPlan {
        let width = (scale * self.width as f32) as i32;
        let scripts = self.scripts.clone();
        let glyphs = self
            .glyphs
            .iter()
            .map(|gp| GlyphPlan {
                offset: crate::geom::Point::from(scale * crate::geom::Vec2::from(gp.offset)),
                advance: crate::geom::Point::from(scale * crate::geom::Vec2::from(gp.advance)),
                ..*gp
            })
            .collect();
        RenderPlan {
            width,
            scripts,
            glyphs,
        }
    }

    pub fn space_out(&mut self, letter_spacing: i32) {
        if letter_spacing == 0 {
            return;
        }

        if let Some((_, start)) = self.glyphs.split_last_mut() {
            let len = start.len() as i32;
            for glyph in start {
                glyph.advance.x += letter_spacing;
            }
            self.width += len * letter_spacing;
        }
    }

    pub fn split_off(&mut self, index: usize, width: i32) -> RenderPlan {
        let mut next_scripts = rustc_hash::FxHashMap::default();
        if !self.scripts.is_empty() {
            for i in index..self.glyphs.len() {
                self.scripts
                    .remove_entry(&i)
                    .map(|(k, v)| next_scripts.insert(k - index, v));
            }
        }
        let next_glyphs = self.glyphs.split_off(index);
        let next_width = self.width - width;
        self.width = width;
        RenderPlan {
            width: next_width,
            scripts: next_scripts,
            glyphs: next_glyphs,
        }
    }

    pub fn index_from_advance(&self, advance: i32) -> usize {
        let mut sum = 0;
        let mut index = 0;
        while index < self.glyphs.len() {
            let gad = self.glyph_advance(index);
            sum += gad;
            if sum > advance {
                if sum - advance < advance - sum + gad {
                    index += 1;
                }
                break;
            }
            index += 1;
        }
        index
    }

    pub fn append(&mut self, other: &mut Self) {
        let next_index = self.glyphs.len();
        self.scripts
            .extend(other.scripts.iter().map(|(k, v)| (next_index + k, *v)));
        self.glyphs.append(&mut other.glyphs);
        self.width += other.width;
    }

    pub fn total_advance(&self, index: usize) -> i32 {
        self.glyphs.iter().take(index).map(|g| g.advance.x).sum()
    }

    #[inline]
    pub fn glyph_advance(&self, index: usize) -> i32 {
        self.glyphs[index].advance.x
    }
}
