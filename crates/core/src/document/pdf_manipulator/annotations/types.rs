//! Types for PDF annotation and manipulation.

use anyhow::{format_err, Error};
use chrono::{DateTime, Utc};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

/// Annotation subtypes supported by Plato
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AnnotationSubtype {
    Text,
    Highlight,
    Underline,
    StrikeOut,
    Squiggly,
    Popup,
    FreeText,
    Line,
    Square,
    Circle,
    Polygon,
    PolyLine,
    Caret,
    Ink,
    FileAttachment,
    Sound,
    Movie,
    Widget,
    Screen,
    PrinterMark,
    TrapNet,
    Watermark,
    ThreeD,
    Redact,
}

impl AnnotationSubtype {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s.to_lowercase().as_str() {
            "text" => Ok(AnnotationSubtype::Text),
            "highlight" => Ok(AnnotationSubtype::Highlight),
            "underline" => Ok(AnnotationSubtype::Underline),
            "strikeout" => Ok(AnnotationSubtype::StrikeOut),
            "squiggly" => Ok(AnnotationSubtype::Squiggly),
            "popup" => Ok(AnnotationSubtype::Popup),
            "freetext" => Ok(AnnotationSubtype::FreeText),
            "line" => Ok(AnnotationSubtype::Line),
            "square" => Ok(AnnotationSubtype::Square),
            "circle" => Ok(AnnotationSubtype::Circle),
            "polygon" => Ok(AnnotationSubtype::Polygon),
            "polyline" => Ok(AnnotationSubtype::PolyLine),
            "caret" => Ok(AnnotationSubtype::Caret),
            "ink" => Ok(AnnotationSubtype::Ink),
            "fileattachment" => Ok(AnnotationSubtype::FileAttachment),
            "sound" => Ok(AnnotationSubtype::Sound),
            "movie" => Ok(AnnotationSubtype::Movie),
            "widget" => Ok(AnnotationSubtype::Widget),
            "screen" => Ok(AnnotationSubtype::Screen),
            "printermark" => Ok(AnnotationSubtype::PrinterMark),
            "trapnet" => Ok(AnnotationSubtype::TrapNet),
            "watermark" => Ok(AnnotationSubtype::Watermark),
            "3d" => Ok(AnnotationSubtype::ThreeD),
            "redact" => Ok(AnnotationSubtype::Redact),
            _ => Err(format_err!("Unknown annotation subtype: {}", s)),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AnnotationSubtype::Text => "Text",
            AnnotationSubtype::Highlight => "Highlight",
            AnnotationSubtype::Underline => "Underline",
            AnnotationSubtype::StrikeOut => "StrikeOut",
            AnnotationSubtype::Squiggly => "Squiggly",
            AnnotationSubtype::Popup => "Popup",
            AnnotationSubtype::FreeText => "FreeText",
            AnnotationSubtype::Line => "Line",
            AnnotationSubtype::Square => "Square",
            AnnotationSubtype::Circle => "Circle",
            AnnotationSubtype::Polygon => "Polygon",
            AnnotationSubtype::PolyLine => "PolyLine",
            AnnotationSubtype::Caret => "Caret",
            AnnotationSubtype::Ink => "Ink",
            AnnotationSubtype::FileAttachment => "FileAttachment",
            AnnotationSubtype::Sound => "Sound",
            AnnotationSubtype::Movie => "Movie",
            AnnotationSubtype::Widget => "Widget",
            AnnotationSubtype::Screen => "Screen",
            AnnotationSubtype::PrinterMark => "PrinterMark",
            AnnotationSubtype::TrapNet => "TrapNet",
            AnnotationSubtype::Watermark => "Watermark",
            AnnotationSubtype::ThreeD => "3D",
            AnnotationSubtype::Redact => "Redact",
        }
    }
}

/// PDF annotation information with full metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PdfAnnotation {
    /// Unique identifier for the annotation
    pub id: String,
    /// Page number (0-indexed)
    pub page: usize,
    /// Annotation subtype
    pub subtype: AnnotationSubtype,
    /// Annotation contents/text
    pub contents: String,
    /// Bounding rectangle (x1, y1, x2, y2)
    pub rect: Option<(f32, f32, f32, f32)>,
    /// RGB color (0-255)
    pub color: Option<(u8, u8, u8)>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modification timestamp
    pub modified_at: DateTime<Utc>,
    /// Author/creator name
    pub author: Option<String>,
    /// Subject/title
    pub subject: Option<String>,
    /// Additional custom properties
    pub properties: FxHashMap<String, String>,
}

impl PdfAnnotation {
    /// Create a new annotation with default metadata
    pub fn new(page: usize, subtype: AnnotationSubtype, contents: String) -> Self {
        if contents.is_empty() {
            let now = Utc::now();
            return Self {
                id: uuid::Uuid::new_v4().to_string(),
                page,
                subtype,
                contents: String::new(),
                rect: None,
                color: None,
                created_at: now,
                modified_at: now,
                author: None,
                subject: None,
                properties: FxHashMap::default(),
            };
        }
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            page,
            subtype,
            contents,
            rect: None,
            color: None,
            created_at: now,
            modified_at: now,
            author: None,
            subject: None,
            properties: FxHashMap::default(),
        }
    }

    /// Update the modification timestamp
    pub fn touch(&mut self) {
        self.modified_at = Utc::now();
    }

    /// Check if annotation matches search criteria
    pub fn matches(&self, query: &AnnotationQuery) -> bool {
        if let Some(subtype) = &query.subtype {
            if &self.subtype != subtype {
                return false;
            }
        }
        if let Some(text) = &query.text {
            if !self.contents.to_lowercase().contains(&text.to_lowercase())
                && !self
                    .subject
                    .as_ref()
                    .is_none_or(|s| s.to_lowercase().contains(&text.to_lowercase()))
            {
                return false;
            }
        }
        if let Some(author) = &query.author {
            if self
                .author
                .as_ref()
                .is_some_and(|a| a.to_lowercase().contains(&author.to_lowercase()))
            {
                return false;
            }
        }
        if let Some(page) = &query.page {
            if self.page != *page {
                return false;
            }
        }
        if let Some(start) = &query.after {
            if self.created_at < *start {
                return false;
            }
        }
        if let Some(end) = &query.before {
            if self.created_at > *end {
                return false;
            }
        }
        true
    }
}

/// Query parameters for annotation search
#[derive(Debug, Clone, Default)]
pub struct AnnotationQuery {
    pub subtype: Option<AnnotationSubtype>,
    pub text: Option<String>,
    pub author: Option<String>,
    pub page: Option<usize>,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
}

impl AnnotationQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_subtype(mut self, subtype: AnnotationSubtype) -> Self {
        self.subtype = Some(subtype);
        self
    }

    pub fn with_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }

    pub fn with_author(mut self, author: String) -> Self {
        self.author = Some(author);
        self
    }

    pub fn with_page(mut self, page: usize) -> Self {
        self.page = Some(page);
        self
    }

    pub fn with_after(mut self, after: DateTime<Utc>) -> Self {
        self.after = Some(after);
        self
    }

    pub fn with_before(mut self, before: DateTime<Utc>) -> Self {
        self.before = Some(before);
        self
    }
}
