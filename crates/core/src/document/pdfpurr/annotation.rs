use pdfpurr::Annotation as PdfPurrAnnotation;

/// PDF annotation wrapper.
pub struct Annotation {
    inner: PdfPurrAnnotation,
}

impl Annotation {
    pub fn new(inner: PdfPurrAnnotation) -> Self {
        Annotation { inner }
    }

    /// Returns the annotation type.
    pub fn annotation_type(&self) -> &str {
        &self.inner.annotation_type
    }

    /// Returns the annotation rectangle.
    pub fn rect(&self) -> super::FzRect {
        let rect = self.inner.rect;
        super::FzRect {
            x0: rect.x0,
            y0: rect.y0,
            x1: rect.x1,
            y1: rect.y1,
        }
    }

    /// Returns the annotation contents.
    pub fn contents(&self) -> Option<&str> {
        self.inner.contents.as_deref()
    }
}
