use pdfpurr::Annotation;

/// PDF link wrapper.
pub struct Link {
    annotations: Vec<Annotation>,
    index: usize,
}

impl Link {
    pub fn new(annotations: Vec<Annotation>) -> Self {
        Link {
            annotations,
            index: 0,
        }
    }

    /// Returns the URI of this link.
    pub fn uri(&self) -> String {
        self.annotations.get(self.index)
            .and_then(|a| a.uri.clone())
            .unwrap_or_default()
    }

    /// Returns the rectangle bounds of this link.
    pub fn rect(&self) -> super::FzRect {
        self.annotations.get(self.index)
            .map(|a| {
                let rect = a.rect;
                super::FzRect {
                    x0: rect.x0,
                    y0: rect.y0,
                    x1: rect.x1,
                    y1: rect.y1,
                }
            })
            .unwrap_or_default()
    }

    /// Returns the next link.
    pub fn next(&self) -> Option<Self> {
        if self.index + 1 < self.annotations.len() {
            Some(Link {
                annotations: self.annotations.clone(),
                index: self.index + 1,
            })
        } else {
            None
        }
    }
}

/// Placeholder for FzRect type.
#[derive(Debug, Clone, Default)]
pub struct FzRect {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}
