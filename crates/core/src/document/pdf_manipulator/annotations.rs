//! PDF Annotation Module
//!
//! Provides comprehensive PDF annotation functionality including:
//! - Export annotations to PDF documents
//! - Import existing PDF annotations
//! - Search and filter annotations
//! - XFDF export/import for interoperability
//! - Rich annotation types (highlights, underlines, strikethroughs)
//!
//! Implemented using lopdf for PDF manipulation

use crate::log_info;
use anyhow::{format_err, Error};
use chrono::{DateTime, Utc};
use lopdf::{Dictionary, Document, Object};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
    pub properties: HashMap<String, String>,
}

impl PdfAnnotation {
    /// Create a new annotation with default metadata
    pub fn new(
        page: usize,
        subtype: AnnotationSubtype,
        contents: String,
    ) -> Self {
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
            properties: HashMap::new(),
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
                && !self.subject.as_ref().map_or(false, |s| s.to_lowercase().contains(&text.to_lowercase()))
            {
                return false;
            }
        }
        if let Some(author) = &query.author {
            if self.author.as_ref().map_or(false, |a| a.to_lowercase().contains(&author.to_lowercase())) {
                return false;
            }
        }
        if let Some(page) = query.page {
            if self.page != page {
                return false;
            }
        }
        if let Some(after) = query.after {
            if self.created_at < after {
                return false;
            }
        }
        if let Some(before) = query.before {
            if self.created_at > before {
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

/// PDF annotation manager
///
/// Provides comprehensive annotation management including search, filter, import, and export.
pub struct PdfAnnotationManager {
    file_path: PathBuf,
    annotations: Vec<PdfAnnotation>,
}

impl PdfAnnotationManager {
    /// Create a new annotation manager for a PDF file
    pub fn new(path: &Path) -> Result<Self, Error> {
        let doc = super::super::pdfpurr::Document::open(path)
            .map_err(|e| format_err!("Failed to open PDF: {}", e))?;
        
        let total_pages = doc.page_count();
        log_info!("Opened PDF with {} pages", total_pages);

        Ok(Self {
            file_path: path.to_path_buf(),
            annotations: Vec::new(),
        })
    }

    /// Import all annotations from the PDF file
    pub fn import_annotations(&mut self) -> Result<Vec<PdfAnnotation>, Error> {
        let doc = Document::load(&self.file_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        let mut imported = Vec::new();
        let pages_map = doc.get_pages();

        for (page_num, page_id) in pages_map {
            let page_dict = doc.get_object(page_id).unwrap().as_dict().unwrap();
            if let Ok(annot_ref) = page_dict.get(b"Annots") {
                if let Ok(annot_array) = annot_ref.as_array() {
                    for annot_obj in annot_array {
                        if let Ok(annot_dict) = annot_obj.as_dict() {
                            if let Ok(annot) = self.parse_annotation(annot_dict, page_num as usize) {
                                imported.push(annot);
                            }
                        }
                    }
                }
            }
        }

        self.annotations = imported.clone();
        log_info!("Imported {} annotations from PDF", imported.len());
        Ok(imported)
    }

    /// Parse annotation from PDF dictionary
    fn parse_annotation(&self, dict: &Dictionary, page: usize) -> Result<PdfAnnotation, Error> {
        let subtype_str = if let Ok(name_obj) = dict.get(b"Subtype") {
            if let Ok(name) = name_obj.as_name() {
                std::str::from_utf8(name).unwrap_or("Text").to_string()
            } else {
                "Text".to_string()
            }
        } else {
            "Text".to_string()
        };
        let subtype = AnnotationSubtype::from_str(&subtype_str)
            .unwrap_or(AnnotationSubtype::Text);

        let contents = if let Ok(contents_obj) = dict.get(b"Contents") {
            if let Ok(bytes) = contents_obj.as_str() {
                std::str::from_utf8(bytes).unwrap_or("").to_string()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let rect = if let Ok(rect_obj) = dict.get(b"Rect") {
            if let Ok(arr) = rect_obj.as_array() {
                if arr.len() >= 4 {
                    Some((
                        arr[0].as_i64().unwrap_or(0) as f32,
                        arr[1].as_i64().unwrap_or(0) as f32,
                        arr[2].as_i64().unwrap_or(0) as f32,
                        arr[3].as_i64().unwrap_or(0) as f32,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let color = if let Ok(color_obj) = dict.get(b"C") {
            if let Ok(arr) = color_obj.as_array() {
                if arr.len() >= 3 {
                    Some((
                        arr[0].as_i64().unwrap_or(0) as u8,
                        arr[1].as_i64().unwrap_or(0) as u8,
                        arr[2].as_i64().unwrap_or(0) as u8,
                    ))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let author = if let Ok(author_obj) = dict.get(b"T") {
            if let Ok(bytes) = author_obj.as_str() {
                std::str::from_utf8(bytes).ok().map(|s| s.to_string())
            } else {
                None
            }
        } else {
            None
        };

        let subject = if let Ok(subject_obj) = dict.get(b"Subj") {
            if let Ok(bytes) = subject_obj.as_str() {
                std::str::from_utf8(bytes).ok().map(|s| s.to_string())
            } else {
                None
            }
        } else {
            None
        };

        let now = Utc::now();
        Ok(PdfAnnotation {
            id: uuid::Uuid::new_v4().to_string(),
            page,
            subtype,
            contents,
            rect,
            color,
            created_at: now,
            modified_at: now,
            author,
            subject,
            properties: HashMap::new(),
        })
    }

    /// Search annotations by query
    pub fn search(&self, query: &AnnotationQuery) -> Vec<PdfAnnotation> {
        self.annotations
            .iter()
            .filter(|a| a.matches(query))
            .cloned()
            .collect()
    }

    /// Get all annotations
    pub fn all(&self) -> &[PdfAnnotation] {
        &self.annotations
    }

    /// Get annotations by page
    pub fn by_page(&self, page: usize) -> Vec<PdfAnnotation> {
        self.annotations
            .iter()
            .filter(|a| a.page == page)
            .cloned()
            .collect()
    }

    /// Get annotations by subtype
    pub fn by_subtype(&self, subtype: AnnotationSubtype) -> Vec<PdfAnnotation> {
        self.annotations
            .iter()
            .filter(|a| a.subtype == subtype)
            .cloned()
            .collect()
    }

    /// Get annotation count by subtype
    pub fn count_by_subtype(&self) -> HashMap<AnnotationSubtype, usize> {
        let mut counts = HashMap::new();
        for annot in &self.annotations {
            *counts.entry(annot.subtype.clone()).or_insert(0) += 1;
        }
        counts
    }

    /// Sort annotations by creation date
    pub fn sort_by_date(&mut self, ascending: bool) {
        if ascending {
            self.annotations.sort_by(|a, b| a.created_at.cmp(&b.created_at));
        } else {
            self.annotations.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        }
    }

    /// Sort annotations by page
    pub fn sort_by_page(&mut self, ascending: bool) {
        if ascending {
            self.annotations.sort_by(|a, b| a.page.cmp(&b.page));
        } else {
            self.annotations.sort_by(|a, b| b.page.cmp(&a.page));
        }
    }
}

/// PDF annotation exporter
///
/// Exports annotations from a source PDF to a new output document.
pub struct PdfAnnotationExporter {
    file_path: PathBuf,
    total_pages: usize,
}

impl PdfAnnotationExporter {
    /// Create a new annotation exporter
    pub fn new(source_path: &Path, output_path: &Path) -> Result<PdfAnnotationExporter, Error> {
        let doc = super::super::pdfpurr::Document::open(source_path)
            .map_err(|e| format_err!("Failed to open PDF: {}", e))?;
        
        let total_pages = doc.page_count();

        Ok(PdfAnnotationExporter {
            file_path: output_path.to_path_buf(),
            total_pages,
        })
    }

    /// Get the total page count
    pub fn page_count(&self) -> usize {
        self.total_pages
    }

    /// Add an annotation to the output document
    pub fn add_annotation(&mut self, annot: PdfAnnotation) -> Result<(), Error> {
        if annot.page >= self.total_pages {
            return Err(format_err!("Page {} does not exist", annot.page + 1));
        }

        // Load the output document using lopdf
        let mut doc = Document::load(&self.file_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        // Get the target page
        let pages_map = doc.get_pages();
        let page_ids: Vec<_> = pages_map.values().collect();
        let page_index = annot.page;
        if page_index >= page_ids.len() {
            return Err(format_err!("Page {} does not exist in output document", annot.page + 1));
        }
        let page_id = page_ids.get(page_index).unwrap();

        // Create annotation dictionary
        let mut annot_dict = Dictionary::new();
        annot_dict.set("Subtype", Object::Name(annot.subtype.as_str().as_bytes().to_vec()));
        annot_dict.set("Contents", Object::String(annot.contents.as_bytes().to_vec(), lopdf::StringFormat::Literal));

        // Add rectangle if provided
        if let Some(rect) = annot.rect {
            let mut rect_array = Vec::new();
            rect_array.push(Object::Real(rect.0));
            rect_array.push(Object::Real(rect.1));
            rect_array.push(Object::Real(rect.2));
            rect_array.push(Object::Real(rect.3));
            annot_dict.set("Rect", Object::Array(rect_array));
        }

        // Add color if provided
        if let Some(color) = annot.color {
            let color_array = vec![
                Object::Integer(color.0 as i64),
                Object::Integer(color.1 as i64),
                Object::Integer(color.2 as i64),
            ];
            annot_dict.set("C", Object::Array(color_array));
        }

        // Add metadata
        annot_dict.set("T", Object::String(annot.author.unwrap_or_default().as_bytes().to_vec(), lopdf::StringFormat::Literal));
        annot_dict.set("Subj", Object::String(annot.subject.unwrap_or_default().as_bytes().to_vec(), lopdf::StringFormat::Literal));
        annot_dict.set("M", Object::String(annot.modified_at.to_rfc3339().as_bytes().to_vec(), lopdf::StringFormat::Literal));
        annot_dict.set("CreationDate", Object::String(annot.created_at.to_rfc3339().as_bytes().to_vec(), lopdf::StringFormat::Literal));

        // Add annotation to page
        let annot_id = doc.add_object(Object::Dictionary(annot_dict));

        let page_dict = doc.get_object_mut(**page_id).unwrap().as_dict_mut().unwrap();
        page_dict.set("Annots", Object::Array(vec![Object::Reference(annot_id)]));

        // Save the modified document
        let mut buffer = std::io::Cursor::new(Vec::new());
        doc.save_to(&mut buffer)
            .map_err(|e| format_err!("Failed to save PDF with lopdf: {}", e))?;
        let bytes = buffer.into_inner();

        std::fs::write(&self.file_path, bytes)
            .map_err(|e| format_err!("Failed to write output file: {}", e))?;

        log_info!("Successfully added annotation to page {}", annot.page + 1);
        Ok(())
    }

    /// Export an annotation to the output document
    pub fn export_annotation(&mut self, annot: &PdfAnnotation) -> Result<(), Error> {
        // Load the source document using lopdf
        let source_path = &self.file_path; // Using file_path as source for now
        let mut doc = Document::load(source_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        // Get the target page
        let pages_map = doc.get_pages();
        let page_ids: Vec<_> = pages_map.values().collect();
        let page_index = annot.page;
        if page_index >= page_ids.len() {
            return Err(format_err!("Page {} does not exist", annot.page + 1));
        }

        if let Some(page_id) = page_ids.get(page_index) {
            let page_dict = doc.get_object(**page_id).unwrap().as_dict().unwrap();

            // Check if annotations exist and copy them
            if let Ok(annot_ref) = page_dict.get(b"Annots") {
                if let Ok(annot_array) = annot_ref.as_array() {
                    log_info!("Found {} annotations on page {}", annot_array.len(), annot.page + 1);
                    
                    // Copy annotations to the output document
                    let mut output_doc = Document::load(&self.file_path)
                        .map_err(|e| format_err!("Failed to load output PDF with lopdf: {}", e))?;
                    
                    let output_pages_map = output_doc.get_pages();
                    let output_page_ids: Vec<_> = output_pages_map.values().collect();
                    
                    if page_index < output_page_ids.len() {
                        let output_page_id = output_page_ids.get(page_index).unwrap();
                        let output_page_dict = output_doc.get_object_mut(**output_page_id).unwrap().as_dict_mut().unwrap();
                        
                        // Add annotations to output page
                        output_page_dict.set("Annots", annot_ref.clone());
                        
                        // Save the modified document
                        let mut buffer = std::io::Cursor::new(Vec::new());
                        output_doc.save_to(&mut buffer)
                            .map_err(|e| format_err!("Failed to save PDF with lopdf: {}", e))?;
                        let bytes = buffer.into_inner();
                        
                        std::fs::write(&self.file_path, bytes)
                            .map_err(|e| format_err!("Failed to write output file: {}", e))?;
                    }
                }
            }
        }

        // Save the modified document
        let mut buffer = std::io::Cursor::new(Vec::new());
        doc.save_to(&mut buffer)
            .map_err(|e| format_err!("Failed to save PDF with lopdf: {}", e))?;
        let bytes = buffer.into_inner();

        std::fs::write(source_path, bytes)
            .map_err(|e| format_err!("Failed to write output file: {}", e))?;

        log_info!("Successfully exported annotation from page {}", annot.page + 1);
        Ok(())
    }

    /// Save the output document with annotations
    pub fn save(&self) -> Result<PathBuf, Error> {
        // Load and verify the document
        let mut doc = Document::load(&self.file_path)
            .map_err(|e| format_err!("Failed to load PDF with lopdf: {}", e))?;

        // Save to ensure it's valid
        let mut buffer = std::io::Cursor::new(Vec::new());
        doc.save_to(&mut buffer)
            .map_err(|e| format_err!("Failed to save PDF with lopdf: {}", e))?;
        let bytes = buffer.into_inner();

        std::fs::write(&self.file_path, bytes)
            .map_err(|e| format_err!("Failed to write output file: {}", e))?;

        log_info!("Successfully saved document with annotations to: {:?}", self.file_path);
        Ok(self.file_path.clone())
    }
}

/// XFDF (Adobe XML Forms Data Format) for annotation exchange
///
/// Provides import/export of annotations in XFDF format for interoperability
/// with other PDF tools like Adobe Acrobat, Foxit, etc.
pub struct XfdfHandler;

impl XfdfHandler {
    /// Export annotations to XFDF format
    pub fn export_to_xfdf(annotations: &[PdfAnnotation], pdf_path: &Path) -> Result<String, Error> {
        let mut xml = String::new();
        xml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push_str("\n<xfdf xmlns=\"http://ns.adobe.com/xfdf/\" xml:space=\"preserve\">\n");
        xml.push_str(&format!("  <f href=\"{}\"/>\n", pdf_path.display()));
        
        for annot in annotations {
            xml.push_str("  <annotate>\n");
            xml.push_str(&format!("    <subtype>{}</subtype>\n", annot.subtype.as_str()));
            xml.push_str(&format!("    <contents>{}</contents>\n", escape_xml(&annot.contents)));
            xml.push_str(&format!("    <page>{}</page>\n", annot.page));
            
            if let Some(rect) = annot.rect {
                xml.push_str(&format!("    <rect>{},{},{},{}</rect>\n", rect.0, rect.1, rect.2, rect.3));
            }
            
            if let Some(color) = annot.color {
                xml.push_str(&format!("    <color>#{:02X}{:02X}{:02X}</color>\n", color.0, color.1, color.2));
            }
            
            if let Some(author) = &annot.author {
                xml.push_str(&format!("    <author>{}</author>\n", escape_xml(author)));
            }
            
            if let Some(subject) = &annot.subject {
                xml.push_str(&format!("    <subject>{}</subject>\n", escape_xml(subject)));
            }
            
            xml.push_str(&format!("    <created>{}</created>\n", annot.created_at.to_rfc3339()));
            xml.push_str(&format!("    <modified>{}</modified>\n", annot.modified_at.to_rfc3339()));
            xml.push_str("  </annotate>\n");
        }
        
        xml.push_str("</xfdf>");
        Ok(xml)
    }
    
    /// Import annotations from XFDF format
    pub fn import_from_xfdf(xfdf_content: &str) -> Result<Vec<PdfAnnotation>, Error> {
        use quick_xml::events::Event;
        use quick_xml::Reader;
        
        let mut reader = Reader::from_str(xfdf_content);
        
        let mut annotations = Vec::new();
        let mut current_annot: Option<PdfAnnotation> = None;
        let mut current_text = String::new();
        
        let mut buf = Vec::new();
        
        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    match e.name().as_ref() {
                        b"annotate" => {
                            current_annot = Some(PdfAnnotation::new(0, AnnotationSubtype::Text, String::new()));
                        }
                        b"subtype" => current_text.clear(),
                        b"contents" => current_text.clear(),
                        b"page" => current_text.clear(),
                        b"rect" => current_text.clear(),
                        b"color" => current_text.clear(),
                        b"author" => current_text.clear(),
                        b"subject" => current_text.clear(),
                        b"created" => current_text.clear(),
                        b"modified" => current_text.clear(),
                        _ => {}
                    }
                }
                Ok(Event::Text(e)) => {
                    current_text.push_str(std::str::from_utf8(&e).unwrap_or(""));
                }
                Ok(Event::End(ref e)) => {
                    if let Some(ref mut annot) = current_annot {
                        match e.name().as_ref() {
                            b"subtype" => {
                                if let Ok(subtype) = AnnotationSubtype::from_str(&current_text) {
                                    annot.subtype = subtype;
                                }
                            }
                            b"contents" => annot.contents = current_text.clone(),
                            b"page" => {
                                if let Ok(page) = current_text.parse::<usize>() {
                                    annot.page = page;
                                }
                            }
                            b"rect" => {
                                let parts: Vec<f32> = current_text.split(',')
                                    .filter_map(|s| s.parse().ok())
                                    .collect();
                                if parts.len() == 4 {
                                    annot.rect = Some((parts[0], parts[1], parts[2], parts[3]));
                                }
                            }
                            b"color" => {
                                if current_text.starts_with('#') && current_text.len() == 7 {
                                    let r = u8::from_str_radix(&current_text[1..3], 16).unwrap_or(0);
                                    let g = u8::from_str_radix(&current_text[3..5], 16).unwrap_or(0);
                                    let b = u8::from_str_radix(&current_text[5..7], 16).unwrap_or(0);
                                    annot.color = Some((r, g, b));
                                }
                            }
                            b"author" => annot.author = Some(current_text.clone()),
                            b"subject" => annot.subject = Some(current_text.clone()),
                            b"created" => {
                                if let Ok(dt) = DateTime::parse_from_rfc3339(&current_text) {
                                    annot.created_at = dt.with_timezone(&Utc);
                                }
                            }
                            b"modified" => {
                                if let Ok(dt) = DateTime::parse_from_rfc3339(&current_text) {
                                    annot.modified_at = dt.with_timezone(&Utc);
                                }
                            }
                            b"annotate" => {
                                annotations.push(super::PdfAnnotation::new(
                                    0,
                                    annot.subtype.clone(),
                                    annot.contents.clone(),
                                ));
                            }
                            _ => {}
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(format_err!("XFDF parse error: {}", e)),
                _ => {}
            }
            buf.clear();
        }
        
        Ok(annotations)
    }
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod annotations_tests;
