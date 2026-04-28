//! PDF/A and PDF/X Validation UI View
//!
//! Provides UI for PDF/A and PDF/X validation on desktop platforms.
//! Supports selecting validation standards and viewing validation results.

#![cfg(target_os = "linux")]

use crate::context::Context;
use crate::document::validation::{
    PdfALevel, PdfAValidator, PdfXLevel, PdfXValidator, ValidationResult,
};
use crate::font::Fonts;
use crate::geom::Rectangle;
use std::path::{Path, PathBuf};

use crate::unit::scale_by_dpi;
use crate::view::button::Button;
use crate::view::label::Label;
use crate::view::top_bar::TopBar;
use crate::view::{Align, Bus, Event, Hub, RenderQueue, View};
use crate::view::{EntryId, Id, ID_FEEDER};
use crate::view::{SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};
use anyhow::Error;

pub const BUTTON_HEIGHT: i32 = 48;
pub const BUTTON_SPACING: i32 = 12;
pub const PADDING: i32 = 16;

pub struct ValidationView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    pdf_path: PathBuf,
    validation_result: Option<ValidationResult>,
    #[cfg(feature = "ocr")]
    ocr_result: Option<String>,
}

impl ValidationView {
    pub fn new(
        rect: Rectangle,
        pdf_path: &Path,
        _rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<Self, Error> {
        let id = ID_FEEDER.next();
        let mut children = Vec::new();

        let top_bar_height =
            scale_by_dpi(SMALL_BAR_HEIGHT, context.fonts.sans_serif.regular.dpi) as i32;
        let content_y = rect.min.y + top_bar_height + THICKNESS_MEDIUM as i32;

        // Top bar
        let top_bar = TopBar::new(
            rect![
                rect.min.x,
                rect.min.y,
                rect.max.x,
                rect.min.y + top_bar_height
            ],
            Event::Back,
            "PDF Validation".to_string(),
            context,
        );
        children.push(Box::new(top_bar) as Box<dyn View>);

        let mut y = content_y;

        // Document info
        let doc_label = Label::new(
            rect![
                rect.min.x + PADDING,
                y,
                rect.max.x - PADDING,
                y + BUTTON_HEIGHT
            ],
            format!(
                "Document: {}",
                pdf_path.file_name().unwrap_or_default().to_string_lossy()
            ),
            Align::Left(0),
        );
        children.push(Box::new(doc_label) as Box<dyn View>);
        y += BUTTON_HEIGHT + BUTTON_SPACING;

        // PDF/A validation section
        let pdfa_header = Label::new(
            rect![
                rect.min.x + PADDING,
                y,
                rect.max.x - PADDING,
                y + BUTTON_HEIGHT
            ],
            "PDF/A Validation (Archival)".to_string(),
            Align::Left(0),
        );
        children.push(Box::new(pdfa_header) as Box<dyn View>);
        y += BUTTON_HEIGHT + BUTTON_SPACING;

        // PDF/A level buttons
        let pdfa_levels = vec![
            ("PDF/A-1b", crate::document::validation::PdfALevel::A1b),
            ("PDF/A-2b", crate::document::validation::PdfALevel::A2b),
            ("PDF/A-3b", crate::document::validation::PdfALevel::A3b),
        ];

        for (label, level) in pdfa_levels {
            let btn = Button::new(
                rect![
                    rect.min.x + PADDING,
                    y,
                    rect.min.x + PADDING + 200,
                    y + BUTTON_HEIGHT
                ],
                Event::Select(EntryId::ValidatePdfA(pdf_path.to_path_buf(), level)),
                label.to_string(),
            );
            children.push(Box::new(btn) as Box<dyn View>);
            y += BUTTON_HEIGHT + BUTTON_SPACING;
        }

        y += BUTTON_SPACING; // Extra spacing between sections

        // PDF/X validation section
        let pdfx_header = Label::new(
            rect![
                rect.min.x + PADDING,
                y,
                rect.max.x - PADDING,
                y + BUTTON_HEIGHT
            ],
            "PDF/X Validation (Print Production)".to_string(),
            Align::Left(0),
        );
        children.push(Box::new(pdfx_header) as Box<dyn View>);
        y += BUTTON_HEIGHT + BUTTON_SPACING;

        // PDF/X level buttons
        let pdfx_levels = vec![
            ("PDF/X-1a", crate::document::validation::PdfXLevel::X1a),
            ("PDF/X-3", crate::document::validation::PdfXLevel::X3),
            ("PDF/X-4", crate::document::validation::PdfXLevel::X4),
        ];

        for (label, level) in pdfx_levels {
            let btn = Button::new(
                rect![
                    rect.min.x + PADDING,
                    y,
                    rect.min.x + PADDING + 200,
                    y + BUTTON_HEIGHT
                ],
                Event::Select(EntryId::ValidatePdfX(pdf_path.to_path_buf(), level)),
                label.to_string(),
            );
            children.push(Box::new(btn) as Box<dyn View>);
            y += BUTTON_HEIGHT + BUTTON_SPACING;
        }

        #[cfg(feature = "ocr")]
        {
            // OCR section
            y += BUTTON_SPACING;
            let ocr_header = Label::new(
                rect![
                    rect.min.x + PADDING,
                    y,
                    rect.max.x - PADDING,
                    y + BUTTON_HEIGHT
                ],
                "OCR (Text Extraction from Scanned PDFs)".to_string(),
                Align::Left(0),
            );
            children.push(Box::new(ocr_header) as Box<dyn View>);
            y += BUTTON_HEIGHT + BUTTON_SPACING;

            let ocr_btn = Button::new(
                rect![
                    rect.min.x + PADDING,
                    y,
                    rect.min.x + PADDING + 200,
                    y + BUTTON_HEIGHT
                ],
                Event::Select(EntryId::OcrDocument(pdf_path.to_path_buf())),
                "OCR Current Page".to_string(),
            );
            children.push(Box::new(ocr_btn) as Box<dyn View>);
        }

        Ok(ValidationView {
            id,
            rect,
            children,
            pdf_path: pdf_path.to_path_buf(),
            validation_result: None,
            #[cfg(feature = "ocr")]
            ocr_result: None,
        })
    }

    pub fn validate_pdfa(&mut self, level: PdfALevel) -> Result<(), Error> {
        let pdf_data = std::fs::read(&self.pdf_path)?;
        let result = PdfAValidator::validate(&pdf_data, level)?;
        self.validation_result = Some(result);
        Ok(())
    }

    pub fn validate_pdfx(&mut self, level: PdfXLevel) -> Result<(), Error> {
        let pdf_data = std::fs::read(&self.pdf_path)
            .map_err(|e| anyhow::format_err!("Failed to read PDF: {}", e))?;

        let result = PdfXValidator::validate(&pdf_data, level)?;
        self.validation_result = Some(result);
        Ok(())
    }

    #[cfg(feature = "ocr")]
    pub fn ocr_page(&mut self) -> Result<(), Error> {
        use crate::document::ocr::OcrManager;

        let ocr_manager = OcrManager::new();
        let current_page = 0; // TODO: Get actual current page
        let text = ocr_manager
            .ocr_page(&self.pdf_path, current_page)
            .map_err(|e| anyhow::format_err!("Failed to OCR page: {}", e))?;

        // Store OCR result
        self.ocr_result = Some(text);
        Ok(())
    }

    pub fn validation_result(&self) -> Option<&ValidationResult> {
        self.validation_result.as_ref()
    }
}

impl View for ValidationView {
    fn handle_event(
        &mut self,
        _evt: &Event,
        _hub: &Hub,
        _bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        false
    }

    fn render(
        &self,
        _fb: &mut dyn crate::framebuffer::Framebuffer,
        _rect: Rectangle,
        _fonts: &mut Fonts,
    ) {
        // Rendering is handled by children
    }

    fn rect(&self) -> &Rectangle {
        &self.rect
    }

    fn rect_mut(&mut self) -> &mut Rectangle {
        &mut self.rect
    }

    fn children(&self) -> &Vec<Box<dyn View>> {
        &self.children
    }

    fn children_mut(&mut self) -> &mut Vec<Box<dyn View>> {
        &mut self.children
    }

    fn id(&self) -> Id {
        self.id
    }
}
