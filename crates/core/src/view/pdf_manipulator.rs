use crate::color;
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::document::pdf_manipulator::PdfManipulator;
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::Rectangle;
use crate::theme;

use crate::unit::scale_by_dpi;
use crate::view::button::Button;
use crate::view::common::locate_by_id;
use crate::view::label::Label;
use crate::view::top_bar::TopBar;
use crate::view::{Align, Bus, Event, Hub, RenderData, RenderQueue, View};
use crate::view::{EntryId, EntryKind, Id, ViewId, ID_FEEDER};
use crate::view::{SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};
use anyhow::{format_err, Error};
use std::path::PathBuf;

const WARNING_FILE_SIZE: u64 = 30;
const PADDING: i32 = 10;
const BUTTON_HEIGHT: i32 = 60;
const BUTTON_SPACING: i32 = 10;

// Note: ManipulationMode variants for file selection, redaction, and processing
// are reserved for future implementation of file browser integration and advanced features.
// Fields within these variants are currently unused but preserved for future expansion.
#[allow(dead_code)]
enum ManipulationMode {
    SelectFile,
    SelectAction(PathBuf),
    SelectRedactionPage(PathBuf, usize),
    Processing(PathBuf, String),
}

pub struct PdfManipulatorView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    manipulator: PdfManipulator,
    mode: ManipulationMode,
    selected_file: Option<PathBuf>,
}

impl PdfManipulatorView {
    pub fn new(
        rect: Rectangle,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<PdfManipulatorView, Error> {
        let id = ID_FEEDER.next();
        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;

        let manipulator = PdfManipulator::new()?;

        let mut children = Vec::new();

        let top_bar = TopBar::new(
            rect![
                rect.min.x,
                rect.min.y,
                rect.max.x,
                rect.min.y + small_height
            ],
            Event::Back,
            "PDF Tools".to_string(),
            context,
        );
        children.push(Box::new(top_bar) as Box<dyn View>);

        let content_y = rect.min.y + small_height + thickness;

        let warning_label = Label::new(
            rect![
                rect.min.x + PADDING,
                content_y,
                rect.max.x - PADDING,
                content_y + BUTTON_HEIGHT
            ],
            "Large PDFs may cause memory issues.\nMax: 30MB, 500 pages. Keep battery charged."
                .to_string(),
            Align::Left(0),
        );
        children.push(Box::new(warning_label) as Box<dyn View>);

        let button_y = content_y + BUTTON_HEIGHT + BUTTON_SPACING;
        let button_width = rect.width() - 2 * PADDING as u32;
        let cleanup_btn = Button::new(
            rect![
                rect.min.x + PADDING,
                button_y,
                rect.min.x + PADDING + button_width as i32,
                button_y + BUTTON_HEIGHT
            ],
            Event::Select(EntryId::CleanUp),
            "🗑️ Clean Temp Backups".to_string(),
        );
        children.push(Box::new(cleanup_btn) as Box<dyn View>);

        rq.add(RenderData::new(id, rect, UpdateMode::Full));

        Ok(PdfManipulatorView {
            id,
            rect,
            children,
            manipulator,
            mode: ManipulationMode::SelectFile,
            selected_file: None,
        })
    }

    fn show_message(&mut self, msg: String, rq: &mut RenderQueue, bus: &mut Bus) {
        bus.push_back(Event::Render(msg));
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Full));
    }

    pub fn for_file(
        rect: Rectangle,
        file_path: PathBuf,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<PdfManipulatorView, Error> {
        let mut view = PdfManipulatorView::new(rect, rq, context)?;
        view.selected_file = Some(file_path.clone());
        view.show_actions(file_path, rq, context);
        Ok(view)
    }

    fn show_actions(&mut self, file_path: PathBuf, rq: &mut RenderQueue, context: &mut Context) {
        self.mode = ManipulationMode::SelectAction(file_path.clone());

        let file_size = std::fs::metadata(&file_path)
            .map(|m| m.len() / (1024 * 1024))
            .unwrap_or(0);

        let warning_msg = if file_size > WARNING_FILE_SIZE {
            format!("⚠️ Large file ({}MB). May be slow on Kobo.", file_size)
        } else {
            "".to_string()
        };

        let mut entries = vec![];

        if !warning_msg.is_empty() {
            entries.push(EntryKind::Message(warning_msg, Some(EntryId::Back)));
        }

        entries.extend(vec![
            EntryKind::Command(
                "🗑️ Delete Pages".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "delete".to_string()),
            ),
            EntryKind::Command(
                "🔄 Rotate 90°".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "rotate90".to_string()),
            ),
            EntryKind::Command(
                "🔄 Rotate 180°".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "rotate180".to_string()),
            ),
            EntryKind::Command(
                "🔄 Rotate 270°".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "rotate270".to_string()),
            ),
            EntryKind::Command(
                "📄 Extract Pages".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "extract".to_string()),
            ),
            EntryKind::Command(
                "📚 Merge PDFs".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "merge".to_string()),
            ),
            EntryKind::Command(
                "✏️ Redact Areas".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "redact_page".to_string()),
            ),
            EntryKind::Command(
                "🖼️ Extract Resources".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "extract_resources".to_string()),
            ),
            EntryKind::Command(
                "📝 Export with Annotations".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "export_annotations".to_string()),
            ),
            EntryKind::Command(
                "📋 Read PDF Annotations".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "read_annotations".to_string()),
            ),
        ]);

        let menu = crate::view::menu::Menu::new(
            self.rect,
            ViewId::PdfManipulatorMenu,
            crate::view::menu::MenuKind::Contextual,
            entries,
            context,
        );
        rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
        self.children.push(Box::new(menu) as Box<dyn View>);
    }

    fn show_redaction_menu(
        &mut self,
        file_path: &PathBuf,
        total_pages: usize,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<(), Error> {
        self.mode = ManipulationMode::SelectRedactionPage(file_path.clone(), total_pages);

        let mut entries = vec![
            EntryKind::Message(
                format!("📄 PDF has {} pages", total_pages),
                Some(EntryId::Back),
            ),
            EntryKind::Separator,
        ];

        for page in 0..total_pages.min(50) {
            entries.push(EntryKind::Command(
                format!("Page {}", page + 1),
                EntryId::PdfManipulate(file_path.clone(), format!("redact_apply:{}", page)),
            ));
        }

        if total_pages > 50 {
            entries.push(EntryKind::Message(
                "... and more pages".to_string(),
                Some(EntryId::Back),
            ));
        }

        let menu = crate::view::menu::Menu::new(
            self.rect,
            ViewId::PdfManipulatorMenu,
            crate::view::menu::MenuKind::Contextual,
            entries,
            context,
        );
        rq.add(RenderData::new(menu.id(), *menu.rect(), UpdateMode::Gui));
        self.children.push(Box::new(menu) as Box<dyn View>);

        Ok(())
    }

    fn process_redaction(&mut self, file_path: &PathBuf, page: usize) -> Result<PathBuf, Error> {
        use crate::document::pdf_manipulator::{RedactionEditor, RedactionRegion};

        let output = file_path.with_extension("redacted.pdf");
        let mut editor = RedactionEditor::new(file_path)?;

        let region = RedactionRegion {
            page,
            x: 50.0,
            y: 50.0,
            width: 200.0,
            height: 30.0,
        };
        editor.add_redaction(region);

        editor.apply_redactions(&output)
    }

    fn process_manipulation(
        &mut self,
        file_path: &PathBuf,
        action: &str,
        _hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<(), Error> {
        self.mode = ManipulationMode::Processing(file_path.clone(), action.to_string());

        let result = match action {
            "delete" => {
                let pages: Vec<_> = (0..10).collect();
                let output = file_path.with_extension("modified.pdf");
                self.manipulator.delete_pages(file_path, &output, &pages)
            }
            "rotate90" => {
                let pages: Vec<_> = (0..10).map(|i| (i, 90)).collect();
                let output = file_path.with_extension("rotated.pdf");
                self.manipulator.rotate_pages(file_path, &output, &pages)
            }
            "rotate180" => {
                let pages: Vec<_> = (0..10).map(|i| (i, 180)).collect();
                let output = file_path.with_extension("rotated.pdf");
                self.manipulator.rotate_pages(file_path, &output, &pages)
            }
            "rotate270" => {
                let pages: Vec<_> = (0..10).map(|i| (i, 270)).collect();
                let output = file_path.with_extension("rotated.pdf");
                self.manipulator.rotate_pages(file_path, &output, &pages)
            }
            "extract" => {
                let pages: Vec<_> = vec![0];
                let output = file_path.with_extension("extracted.pdf");
                self.manipulator.extract_pages(file_path, &output, &pages)
            }
            "merge" => {
                let output = file_path.with_extension("merged.pdf");
                self.manipulator.merge_pdfs(&[file_path], &output)
            }
            "redact_page" => {
                use crate::document::pdf_manipulator::RedactionEditor;
                let editor = match RedactionEditor::new(file_path) {
                    Ok(e) => e,
                    Err(e) => return Err(e),
                };
                self.selected_file = Some(file_path.clone());
                self.show_redaction_menu(file_path, editor.page_count(), rq, context)?;
                return Ok(());
            }
            action if action.starts_with("redact_apply:") => {
                let page: usize = action
                    .trim_start_matches("redact_apply:")
                    .parse()
                    .map_err(|_| format_err!("Invalid page number"))?;
                return self.process_redaction(file_path, page).map(|_| ());
            }
            "extract_resources" => {
                use crate::document::pdf_manipulator::ResourceExtractor;
                let extractor = match ResourceExtractor::new(file_path) {
                    Ok(e) => e,
                    Err(e) => return Err(e),
                };
                match extractor.list_resources() {
                    Ok(summary) => {
                        let msg = if summary.is_pdf_a {
                            format!(
                                "📄 Pages: {} | 🖼️ Images: {} | 🔤 Fonts: {} | 📋 PDF/A: {}",
                                summary.total_pages,
                                summary.total_images,
                                summary.total_fonts,
                                summary.pdf_a_version
                            )
                        } else {
                            format!(
                                "📄 Pages: {} | 🖼️ Images: {} | 🔤 Fonts: {}",
                                summary.total_pages, summary.total_images, summary.total_fonts
                            )
                        };
                        self.show_message(msg, rq, bus);
                    }
                    Err(e) => {
                        bus.push_back(Event::Render(format!("Error: {}", e)));
                    }
                }
                return Ok(());
            }
            "export_annotations" => {
                use crate::document::pdf_manipulator::PdfAnnotationExporter;
                let output = file_path.with_extension("annotated.pdf");

                match PdfAnnotationExporter::new(file_path, &output) {
                    Ok(exporter) => match exporter.save() {
                        Ok(path) => {
                            let msg = format!(
                                "✅ Exported to: {}",
                                path.file_name().unwrap_or_default().to_string_lossy()
                            );
                            bus.push_back(Event::Render(msg));
                        }
                        Err(e) => {
                            bus.push_back(Event::Render(format!("Export failed: {}", e)));
                        }
                    },
                    Err(e) => {
                        bus.push_back(Event::Render(format!("Error: {}", e)));
                    }
                }
                return Ok(());
            }
            "read_annotations" => {
                use crate::document::pdf_manipulator::ResourceExtractor;
                let extractor = match ResourceExtractor::new(file_path) {
                    Ok(e) => e,
                    Err(e) => {
                        bus.push_back(Event::Render(format!("Error: {}", e)));
                        return Ok(());
                    }
                };
                match extractor.read_annotations() {
                    Ok(annotations) => {
                        if annotations.is_empty() {
                            bus.push_back(Event::Render(
                                "📋 No annotations found in PDF".to_string(),
                            ));
                        } else {
                            let msg = format!("📋 Found {} annotations in PDF", annotations.len());
                            bus.push_back(Event::Render(msg));
                        }
                    }
                    Err(e) => {
                        bus.push_back(Event::Render(format!("Error reading annotations: {}", e)));
                    }
                }
                return Ok(());
            }
            _ => Err(format_err!("Unknown action")),
        };

        self.mode = ManipulationMode::SelectFile;

        match result {
            Ok(_) => {
                bus.push_back(Event::Render(
                    "✅ Operation complete! Backup created.".to_string(),
                ));
            }
            Err(e) => {
                let error_msg = if e.to_string().contains("memory")
                    || e.to_string().contains("Memory")
                {
                    "❌ Memory error. Try smaller PDF or close apps.".to_string()
                } else if e.to_string().contains("too large") || e.to_string().contains("exceeds") {
                    "❌ File too large. Max 30MB, 500 pages.".to_string()
                } else if e.to_string().contains("Insufficient memory") {
                    "❌ Low memory. Close other apps and retry.".to_string()
                } else {
                    format!("❌ Error: {}", e)
                };
                bus.push_back(Event::Render(error_msg));
            }
        }

        Ok(())
    }

    fn cleanup_backups(
        &mut self,
        _hub: &Hub,
        bus: &mut Bus,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        if let Ok(home) = std::env::var("PLATO_HOME").map(PathBuf::from) {
            match self.manipulator.cleanup_temp_files(&home) {
                Ok(bytes) => {
                    let msg = format!("Cleaned {} bytes from temp files", bytes);
                    bus.push_back(Event::Render(msg));
                }
                Err(e) => {
                    bus.push_back(Event::Render(format!("Cleanup error: {}", e)));
                }
            }
        }
    }
}

impl View for PdfManipulatorView {
    fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match evt {
            Event::Back => match &self.mode {
                ManipulationMode::SelectRedactionPage(_, _) => {
                    self.mode = ManipulationMode::SelectFile;
                    self.selected_file = None;
                    if let Some(index) = locate_by_id(self, ViewId::PdfManipulatorMenu) {
                        rq.add(RenderData::expose(
                            *self.child(index).rect(),
                            UpdateMode::Gui,
                        ));
                        self.children.remove(index);
                    }
                    return true;
                }
                _ => {
                    bus.push_back(Event::Close(ViewId::PdfManipulator));
                    return true;
                }
            },
            Event::Select(EntryId::CleanUp) => {
                self.cleanup_backups(hub, bus, rq, context);
                return true;
            }
            Event::Select(EntryId::PdfManipulate(path, action)) => {
                if let Err(e) = self.process_manipulation(&path, action, hub, bus, rq, context) {
                    bus.push_back(Event::Render(format!("Error: {}", e)));
                }
                return true;
            }
            _ => {}
        }
        for child in self.children_mut().iter_mut() {
            if child.handle_event(evt, hub, bus, rq, context) {
                return true;
            }
        }
        false
    }

    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, fonts: &mut Fonts) {
        if let Some(r) = self.rect().intersection(&rect) {
            fb.draw_rectangle(&r, color::background(theme::is_dark_mode()));
        }
        for child in self.children().iter() {
            child.render(fb, rect, fonts);
        }
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
