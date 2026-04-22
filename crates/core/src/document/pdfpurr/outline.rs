use pdfpurr::Outline as PdfPurrOutline;

/// PDF outline (table of contents) wrapper.
pub struct Outline {
    outlines: Vec<PdfPurrOutline>,
}

impl Outline {
    pub fn new(outlines: Vec<PdfPurrOutline>) -> Self {
        Outline { outlines }
    }

    /// Returns the title of this outline entry.
    pub fn title(&self) -> &str {
        self.outlines.first()
            .map(|o| o.title.as_str())
            .unwrap_or("")
    }

    /// Returns the page location for this outline entry.
    pub fn page(&self) -> FzLocation {
        self.outlines.first()
            .map(|o| FzLocation {
                chapter: o.page.unwrap_or(0) as i32,
                page: 0,
            })
            .unwrap_or(FzLocation { chapter: -1, page: -1 })
    }

    /// Returns the URI if this is an external link.
    pub fn uri(&self) -> Option<String> {
        self.outlines.first()
            .and_then(|o| o.uri.clone())
    }

    /// Returns the next outline entry at the same level.
    pub fn next(&self) -> Option<Self> {
        if self.outlines.len() > 1 {
            Some(Outline {
                outlines: self.outlines[1..].to_vec(),
            })
        } else {
            None
        }
    }

    /// Returns child outline entries.
    pub fn down(&self) -> Option<Self> {
        self.outlines.first()
            .and_then(|o| {
                if !o.children.is_empty() {
                    Some(Outline {
                        outlines: o.children.clone(),
                    })
                } else {
                    None
                }
            })
    }

    /// Creates a clone of this outline.
    pub fn clone_outline(&self) -> Self {
        Outline {
            outlines: self.outlines.clone(),
        }
    }
}

/// Placeholder for FzLocation type.
#[derive(Debug, Clone, Default)]
pub struct FzLocation {
    pub chapter: i32,
    pub page: i32,
}
