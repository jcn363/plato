//! Digital Signatures UI View
//!
//! Provides UI for digital signature operations on desktop platforms.
//! Supports certificate selection, signing, and signature verification.

#![cfg(target_os = "linux")]

use crate::context::Context;
use crate::document::signatures::{Certificate, SignatureManager};
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::Rectangle;
use std::path::{Path, PathBuf};

use crate::unit::scale_by_dpi;
use crate::view::button::Button;
use crate::view::label::Label;
use crate::view::top_bar::TopBar;
use crate::view::{Align, Bus, Event, Hub, RenderData, RenderQueue, View};
use crate::view::{EntryId, Id, ID_FEEDER};
use crate::view::{SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};
use anyhow::Error;

pub const BUTTON_HEIGHT: i32 = 48;
pub const BUTTON_SPACING: i32 = 12;
pub const PADDING: i32 = 16;

pub struct SignaturesView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    pdf_path: PathBuf,
    certificates: Vec<Certificate>,
    selected_certificate: Option<usize>,
}

impl SignaturesView {
    pub fn new(
        rect: Rectangle,
        pdf_path: &Path,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<Self, Error> {
        let id = ID_FEEDER.next();
        let mut children = Vec::new();

        let top_bar_height = scale_by_dpi(SMALL_BAR_HEIGHT, context.fonts.sans_serif.regular.dpi) as i32;
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
            "Digital Signatures".to_string(),
            context,
        );
        children.push(Box::new(top_bar) as Box<dyn View>);

        // Load available certificates
        let certificates = SignatureManager::list_certificates()
            .unwrap_or_default();

        let mut y = content_y;

        if certificates.is_empty() {
            let no_certs_label = Label::new(
                rect![
                    rect.min.x + PADDING,
                    y,
                    rect.max.x - PADDING,
                    y + BUTTON_HEIGHT
                ],
                "No certificates found. Import a certificate to sign documents.".to_string(),
                Align::Left(0),
            );
            children.push(Box::new(no_certs_label) as Box<dyn View>);
            y += BUTTON_HEIGHT + BUTTON_SPACING;

            // Import certificate button
            let import_btn = Button::new(
                rect![
                    rect.min.x + PADDING,
                    y,
                    rect.min.x + PADDING + 200,
                    y + BUTTON_HEIGHT
                ],
                Event::Select(EntryId::ImportCertificate),
                "📁 Import Certificate".to_string(),
            );
            children.push(Box::new(import_btn) as Box<dyn View>);
        } else {
            // Show certificate list
            let header_label = Label::new(
                rect![
                    rect.min.x + PADDING,
                    y,
                    rect.max.x - PADDING,
                    y + BUTTON_HEIGHT
                ],
                "Select a certificate to sign:".to_string(),
                Align::Left(0),
            );
            children.push(Box::new(header_label) as Box<dyn View>);
            y += BUTTON_HEIGHT + BUTTON_SPACING;

            for (i, cert) in certificates.iter().enumerate() {
                let cert_label = format!("{} - {} ({} to {})", 
                    cert.subject,
                    cert.issuer,
                    cert.valid_from,
                    cert.valid_until
                );
                let cert_btn = Button::new(
                    rect![
                        rect.min.x + PADDING,
                        y,
                        rect.max.x - PADDING,
                        y + BUTTON_HEIGHT
                    ],
                    Event::Select(EntryId::SelectCertificate(i)),
                    cert_label,
                );
                children.push(Box::new(cert_btn) as Box<dyn View>);
                y += BUTTON_HEIGHT + BUTTON_SPACING;
            }

            // Sign button
            let sign_btn = Button::new(
                rect![
                    rect.min.x + PADDING,
                    y,
                    rect.min.x + PADDING + 200,
                    y + BUTTON_HEIGHT
                ],
                Event::Select(EntryId::SignDocument(pdf_path.to_path_buf())),
                "✍️ Sign Document".to_string(),
            );
            children.push(Box::new(sign_btn) as Box<dyn View>);
            y += BUTTON_HEIGHT + BUTTON_SPACING;
        }

        // Verify signatures button
        let verify_btn = Button::new(
            rect![
                rect.min.x + PADDING,
                y,
                rect.min.x + PADDING + 200,
                y + BUTTON_HEIGHT
            ],
            Event::Select(EntryId::VerifySignatures),
            "🔍 Verify Signatures".to_string(),
        );
        children.push(Box::new(verify_btn) as Box<dyn View>);

        rq.add(RenderData::new(id, rect, UpdateMode::Full));

        Ok(SignaturesView {
            id,
            rect,
            children,
            pdf_path: pdf_path.to_path_buf(),
            certificates,
            selected_certificate: None,
        })
    }

    pub fn is_complete(&self) -> bool {
        self.selected_certificate.is_some()
    }
}

impl View for SignaturesView {
    fn handle_event(
        &mut self,
        event: &Event,
        _hub: &Hub,
        bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) -> bool {
        match event {
            Event::Select(EntryId::SelectCertificate(index)) => {
                self.selected_certificate = Some(*index);
                bus.push_back(Event::Notify(format!("Selected certificate: {}", 
                    self.certificates[*index].subject)));
                return true;
            }
            Event::Select(EntryId::SignDocument(_)) => {
                if let Some(index) = self.selected_certificate {
                    let cert = &self.certificates[index];
                    let output_path = self.pdf_path.with_extension("_signed.pdf");
                    match SignatureManager::sign_pdf(&self.pdf_path, &output_path, cert) {
                        Ok(signature) => {
                            bus.push_back(Event::Notify(format!("Document signed by: {}", signature.signer_name)));
                        }
                        Err(e) => {
                            bus.push_back(Event::Notify(format!("Failed to sign document: {}", e)));
                        }
                    }
                } else {
                    bus.push_back(Event::Notify("Please select a certificate first".to_string()));
                }
                return true;
            }
            Event::Select(EntryId::VerifySignatures) => {
                match SignatureManager::verify_signature(&self.pdf_path) {
                    Ok(signatures) => {
                        if signatures.is_empty() {
                            bus.push_back(Event::Notify("No signatures found in document".to_string()));
                        } else {
                            bus.push_back(Event::Notify(format!("Found {} signature(s)", signatures.len())));
                        }
                    }
                    Err(e) => {
                        bus.push_back(Event::Notify(format!("Failed to verify signatures: {}", e)));
                    }
                }
                return true;
            }
            Event::Select(EntryId::ImportCertificate) => {
                bus.push_back(Event::Notify("Certificate import not yet implemented".to_string()));
                return true;
            }
            _ => {}
        }
        false
    }

    fn render(&self, _fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {
        // Rendering handled by children
    }

    fn resize(&mut self, rect: Rectangle, _hub: &Hub, _rq: &mut RenderQueue, _context: &mut Context) {
        self.rect = rect;
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
