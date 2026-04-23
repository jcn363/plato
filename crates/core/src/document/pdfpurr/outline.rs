//! Outline and link structures for PDFPurr

use pdfpurr::structure::{Annotation, Outline as PdfPurrOutline};
use super::types::{FzLocation, FzRect};

/// Link wrapper for PDFPurr annotations
pub struct Link {
    annots: Vec<Annotation>,
    index: usize,
}

impl Link {
    pub fn new(annots: Vec<Annotation>, index: usize) -> Self {
        Self { annots, index }
    }

    pub fn uri(&self) -> String {
        if self.index < self.annots.len() {
            if let Some(uri) = &self.annots[self.index].uri {
                return uri.clone();
            }
        }
        String::new()
    }

    pub fn rect(&self) -> FzRect {
        if self.index < self.annots.len() {
            let rect = self.annots[self.index].rect;
            FzRect {
                x0: rect[0] as f32,
                y0: rect[1] as f32,
                x1: rect[2] as f32,
                y1: rect[3] as f32,
            }
        } else {
            FzRect::default()
        }
    }

    pub fn next(&self) -> Option<Link> {
        if self.index + 1 < self.annots.len() {
            Some(Link {
                annots: self.annots.clone(),
                index: self.index + 1,
            })
        } else {
            None
        }
    }
}

/// Outline wrapper for PDFPurr outlines
pub struct Outline {
    outlines: Vec<PdfPurrOutline>,
}

impl Outline {
    pub fn new(outlines: Vec<PdfPurrOutline>) -> Self {
        Self { outlines }
    }

    pub fn clone_outline(&self) -> Outline {
        Outline {
            outlines: self.outlines.clone(),
        }
    }

    pub fn page(&self) -> FzLocation {
        if let Some(first) = self.outlines.first() {
            FzLocation {
                chapter: 0,
                page: first.page.unwrap_or(0) as i32,
            }
        } else {
            FzLocation {
                chapter: 0,
                page: 0,
            }
        }
    }

    pub fn uri(&self) -> Option<String> {
        self.outlines.first().and_then(|o| o.uri.clone())
    }

    pub fn next(&self) -> Option<Outline> {
        if self.outlines.len() > 1 {
            Some(Outline {
                outlines: self.outlines[1..].to_vec(),
            })
        } else {
            None
        }
    }

    pub fn title(&self) -> String {
        self.outlines
            .first()
            .map(|o| o.title.clone())
            .unwrap_or_default()
    }

    pub fn down(&self) -> Option<Outline> {
        self.outlines.first().and_then(|o| {
            if !o.children.is_empty() {
                Some(Outline {
                    outlines: o.children.clone(),
                })
            } else {
                None
            }
        })
    }
}
