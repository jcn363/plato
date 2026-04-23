use crate::color;
use crate::context::Context;
use crate::document::pdf_manipulator::{PdfManipulator, RedactionRegion};
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::Rectangle;
use crate::input::{DeviceEvent, FingerStatus};
use crate::theme;
use std::path::{Path, PathBuf};

use crate::unit::scale_by_dpi;
use crate::view::button::Button;
use crate::view::common::locate_by_id;
use crate::view::label::Label;
use crate::view::top_bar::TopBar;
use crate::view::{Align, Bus, Event, Hub, RenderData, RenderQueue, View};
use crate::view::{EntryId, EntryKind, Id, ViewId, ID_FEEDER};
use crate::view::{SMALL_BAR_HEIGHT, THICKNESS_MEDIUM};
use anyhow::{format_err, Error};

mod types;
pub use types::{
    ManipulationMode, RedactionState, BUTTON_HEIGHT, BUTTON_SPACING, PADDING, WARNING_FILE_SIZE,
};

pub struct PdfManipulatorView {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    manipulator: PdfManipulator,
    mode: ManipulationMode,
    selected_file: Option<PathBuf>,
    redaction_state: RedactionState,
}

impl PdfManipulatorView {
    pub fn new(
        rect: Rectangle,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<PdfManipulatorView, Error> {
        let id = ID_FEEDER.next();
        let dpi = crate::unit::get_device_dpi();
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
            "Large PDFs may cause memory issues.
Max: 30MB, 500 pages. Keep battery charged."
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
            redaction_state: RedactionState::None,
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
        self.mode = ManipulationMode::SelectAction;

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
                "🗑️ Delete First 10".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "delete_all".to_string()),
            ),
            EntryKind::Command(
                "🔄 Rotate 90° (10 pages)".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "rotate90_all".to_string()),
            ),
            EntryKind::Command(
                "🔄 Rotate 180° (10 pages)".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "rotate180_all".to_string()),
            ),
            EntryKind::Command(
                "🔄 Rotate 270° (10 pages)".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "rotate270_all".to_string()),
            ),
            EntryKind::Command(
                "📄 Extract First Page".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "extract_all".to_string()),
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
            EntryKind::Command(
                "🔍 Search Annotations".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "search_annotations".to_string()),
            ),
            EntryKind::Command(
                "📤 Export to XFDF".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "export_xfdf".to_string()),
            ),
            EntryKind::Command(
                "📥 Import from XFDF".to_string(),
                EntryId::PdfManipulate(file_path.clone(), "import_xfdf".to_string()),
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
        file_path: &Path,
        total_pages: usize,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> Result<(), Error> {
        self.mode = ManipulationMode::SelectRedactionPage;

        let mut entries = vec![
            EntryKind::Message(
                format!("📄 PDF has {} pages", total_pages),
                Some(EntryId::Back),
            ),
            EntryKind::Separator,
        ];

        // Change EntryId to DefineRedaction to transition to the new mode
        for page in 0..total_pages.min(50) {
            entries.push(EntryKind::Command(
                format!("Page {}", page + 1),
                EntryId::OpenRedactionEditor(file_path.to_path_buf(), page),
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

    // New function to start the process of defining a redaction region
    fn start_defining_redaction(
        &mut self,
        file_path: PathBuf,
        page_index: usize,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        self.mode = ManipulationMode::DefiningRedaction {
            file_path,
            page_index,
            region: None,
        };
        self.redaction_state = RedactionState::None;
        self.children.retain(|child| child.id() == self.id);
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    #[allow(clippy::ptr_arg)]
    fn process_redaction(&mut self, file_path: &PathBuf, page: usize) -> Result<PathBuf, Error> {
        use crate::document::pdf_manipulator::{RedactionEditor, RedactionRegion};

        let output = file_path.with_extension("redacted.pdf");
        let mut editor = RedactionEditor::new(file_path)?;

        let region = if let ManipulationMode::DefiningRedaction {
            region: Some(r), ..
        } = &self.mode
        {
            r.clone()
        } else {
            RedactionRegion {
                page,
                x: 50.0,
                y: 50.0,
                width: 200.0,
                height: 30.0,
            }
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
        self.mode = ManipulationMode::Processing;

        let result = match action {
            "delete" | "rotate90" | "rotate180" | "rotate270" => {
                bus.push_back(Event::Render("Select pages first".to_string()));
                Ok(file_path.clone())
            }
            "delete_all" => {
                let pages: Vec<_> = (0..10).collect();
                let output = file_path.with_extension("modified.pdf");
                self.manipulator.delete_pages(file_path, &output, &pages)
            }
            "rotate90_all" => {
                let pages: Vec<usize> = (1..=10).collect();
                let output = file_path.with_extension("rotated.pdf");
                self.manipulator
                    .rotate_pages(file_path, &output, &pages, 90)
            }
            "rotate180_all" => {
                let pages: Vec<usize> = (1..=10).collect();
                let output = file_path.with_extension("rotated.pdf");
                self.manipulator
                    .rotate_pages(file_path, &output, &pages, 180)
            }
            "rotate270_all" => {
                let pages: Vec<usize> = (1..=10).collect();
                let output = file_path.with_extension("rotated.pdf");
                self.manipulator
                    .rotate_pages(file_path, &output, &pages, 270)
            }
            "extract" => {
                bus.push_back(Event::Render("Select pages first".to_string()));
                return Ok(());
            }
            "extract_all" => {
                let pages: Vec<_> = vec![0];
                let output = file_path.with_extension("extracted.pdf");
                self.manipulator.extract_pages(file_path, &output, &pages)
            }
            "merge" => {
                self.show_file_picker_for_merge(file_path.clone(), rq, context);
                return Ok(());
            }
            "redact_page" => {
                use crate::document::pdf_manipulator::RedactionEditor;
                let editor = RedactionEditor::new(file_path)?;
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
                let extractor = ResourceExtractor::new(file_path)?;
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
            "search_annotations" => {
                use crate::document::pdf_manipulator::{
                    AnnotationQuery, AnnotationSubtype, PdfAnnotationManager,
                };
                let mut manager = match PdfAnnotationManager::new(file_path) {
                    Ok(m) => m,
                    Err(e) => {
                        bus.push_back(Event::Render(format!("Error: {}", e)));
                        return Ok(());
                    }
                };

                match manager.import_annotations() {
                    Ok(_) => {
                        // Search for all highlights
                        let query =
                            AnnotationQuery::new().with_subtype(AnnotationSubtype::Highlight);

                        let results = manager.search(&query);
                        if results.is_empty() {
                            bus.push_back(Event::Render(
                                "🔍 No highlights found in PDF".to_string(),
                            ));
                        } else {
                            let msg = format!("🔍 Found {} highlights in PDF", results.len());
                            bus.push_back(Event::Render(msg));
                        }
                    }
                    Err(e) => {
                        bus.push_back(Event::Render(format!("Error importing annotations: {}", e)));
                    }
                }
                return Ok(());
            }
            "export_xfdf" => {
                use crate::document::pdf_manipulator::{PdfAnnotationManager, XfdfHandler};
                let mut manager = match PdfAnnotationManager::new(file_path) {
                    Ok(m) => m,
                    Err(e) => {
                        bus.push_back(Event::Render(format!("Error: {}", e)));
                        return Ok(());
                    }
                };

                match manager.import_annotations() {
                    Ok(annotations) => {
                        let xfdf_path = file_path.with_extension("xfdf");
                        match XfdfHandler::export_to_xfdf(&annotations, &xfdf_path) {
                            Ok(_) => {
                                let msg = format!(
                                    "✅ Exported to: {}",
                                    xfdf_path.file_name().unwrap_or_default().to_string_lossy()
                                );
                                bus.push_back(Event::Render(msg));
                            }
                            Err(e) => {
                                bus.push_back(Event::Render(format!("XFDF export failed: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        bus.push_back(Event::Render(format!("Error importing annotations: {}", e)));
                    }
                }
                return Ok(());
            }
            "import_xfdf" => {
                use crate::document::pdf_manipulator::XfdfHandler;
                let xfdf_path = file_path.with_extension("xfdf");

                if !xfdf_path.exists() {
                    bus.push_back(Event::Render(
                        "❌ No XFDF file found. Export annotations first.".to_string(),
                    ));
                    return Ok(());
                }

                match std::fs::read_to_string(&xfdf_path) {
                    Ok(xfdf_content) => match XfdfHandler::import_from_xfdf(&xfdf_content) {
                        Ok(annotations) => {
                            if annotations.is_empty() {
                                bus.push_back(Event::Render(
                                    "📥 No annotations found in XFDF file".to_string(),
                                ));
                            } else {
                                let msg = format!(
                                    "📥 Imported {} annotations from XFDF",
                                    annotations.len()
                                );
                                bus.push_back(Event::Render(msg));
                            }
                        }
                        Err(e) => {
                            bus.push_back(Event::Render(format!("XFDF import failed: {}", e)));
                        }
                    },
                    Err(e) => {
                        bus.push_back(Event::Render(format!("Error reading XFDF file: {}", e)));
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

    fn show_file_picker_for_merge(
        &mut self,
        _primary_file: PathBuf,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        self.mode = ManipulationMode::SelectFile;

        let home_dir = std::env::var("PLATO_HOME")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("/mnt/onboard"));

        let mut entries = vec![
            EntryKind::Message(
                "Select PDF files to merge with the current document.".to_string(),
                Some(EntryId::Back),
            ),
            EntryKind::Separator,
        ];

        if let Ok(dir_iter) = std::fs::read_dir(&home_dir) {
            for entry in dir_iter.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map(|ext| ext == "pdf").unwrap_or(false) {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        entries.push(EntryKind::Command(
                            format!("📄 {}", name),
                            EntryId::SelectFile(path.clone()),
                        ));
                    }
                }
            }
        }

        if entries.len() <= 2 {
            entries.push(EntryKind::Message(
                "No PDF files found in the home directory.".to_string(),
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
                ManipulationMode::SelectFile => {
                    bus.push_back(Event::Close(ViewId::PdfManipulator));
                    return true;
                }
                ManipulationMode::SelectAction => {
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
                ManipulationMode::SelectRedactionPage => {
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
                ManipulationMode::DefiningRedaction {
                    file_path,
                    page_index,
                    ..
                } => {
                    let file_path_cloned = file_path.clone();
                    let _page_index_val = *page_index;
                    self.mode = ManipulationMode::SelectRedactionPage;
                    if let Ok(total_pages) = self.manipulator.page_count(&file_path_cloned) {
                        self.show_redaction_menu(&file_path_cloned, total_pages, rq, context)
                            .ok();
                    }
                    return true;
                }
                ManipulationMode::Processing => {
                    // Allow back to cancel processing
                    self.mode = ManipulationMode::SelectFile;
                    bus.push_back(Event::Render("Operation cancelled.".to_string()));
                    return true;
                }
            },
            Event::Select(EntryId::CleanUp) => {
                self.cleanup_backups(hub, bus, rq, context);
                return true;
            }
            Event::Select(EntryId::PdfManipulate(path, action)) => {
                if let Err(e) = self.process_manipulation(path, action, hub, bus, rq, context) {
                    bus.push_back(Event::Render(format!("Error: {}", e)));
                }
                return true;
            }
            // Handle the new entry ID for defining redaction
            Event::Select(EntryId::OpenRedactionEditor(file_path, page_index)) => {
                self.start_defining_redaction(file_path.clone(), *page_index, rq, context);
                return true;
            }
            Event::Select(EntryId::SelectFile(merge_file)) => {
                if let Some(primary_file) = &self.selected_file {
                    if merge_file.exists() {
                        match self.manipulator.merge_pdfs(
                            &[primary_file, merge_file],
                            &primary_file.with_extension("merged.pdf"),
                        ) {
                            Ok(_output_path) => {
                                let msg = format!(
                                    "✅ Merged with: {}",
                                    merge_file.file_name().unwrap_or_default().to_string_lossy()
                                );
                                bus.push_back(Event::Render(msg));
                            }
                            Err(e) => {
                                bus.push_back(Event::Render(format!("❌ Merge failed: {}", e)));
                            }
                        }
                    } else {
                        bus.push_back(Event::Render("❌ File does not exist.".to_string()));
                    }
                }
                return true;
            }
            Event::Device(DeviceEvent::Finger {
                status, position, ..
            }) => {
                if let ManipulationMode::DefiningRedaction {
                    file_path,
                    page_index,
                    ..
                } = &self.mode
                {
                    match status {
                        FingerStatus::Down => {
                            self.redaction_state = RedactionState::Selecting {
                                start: (position.x, position.y),
                                end: (position.x, position.y),
                            };
                            rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                            return true;
                        }
                        FingerStatus::Move => {
                            // Ignore move events during redaction selection
                        }
                        FingerStatus::Motion => {
                            if let RedactionState::Selecting { start, .. } = &self.redaction_state {
                                self.redaction_state = RedactionState::Selecting {
                                    start: *start,
                                    end: (position.x, position.y),
                                };
                                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                                return true;
                            }
                        }
                        FingerStatus::Up => {
                            if let RedactionState::Selecting { start, end } = &self.redaction_state
                            {
                                let x0 = start.0.min(end.0).max(self.rect.min.x);
                                let y0 = start.1.min(end.1).max(self.rect.min.y);
                                let x1 = start.0.max(end.0).min(self.rect.max.x);
                                let y1 = start.1.max(end.1).min(self.rect.max.y);

                                if x1 - x0 > 10 && y1 - y0 > 10 {
                                    // Use standard PDF page dimensions (A4: 595x842 points)
                                    const PDF_PAGE_WIDTH: f32 = 595.0;
                                    const PDF_PAGE_HEIGHT: f32 = 842.0;
                                    let scale_x = PDF_PAGE_WIDTH / self.rect.width() as f32;
                                    let scale_y = PDF_PAGE_HEIGHT / self.rect.height() as f32;

                                    let region = RedactionRegion {
                                        page: *page_index,
                                        x: (x0 - self.rect.min.x) as f32 * scale_x,
                                        y: (y0 - self.rect.min.y) as f32 * scale_y,
                                        width: (x1 - x0) as f32 * scale_x,
                                        height: (y1 - y0) as f32 * scale_y,
                                    };

                                    self.mode = ManipulationMode::DefiningRedaction {
                                        file_path: file_path.clone(),
                                        page_index: *page_index,
                                        region: Some(region),
                                    };
                                }
                                self.redaction_state = RedactionState::None;
                                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                                return true;
                            }
                        }
                    }
                }
                return false;
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

        // Draw redaction selection rectangle
        if let ManipulationMode::DefiningRedaction { .. } = &self.mode {
            if let RedactionState::Selecting { start, end } = &self.redaction_state {
                let x0 = start.0.min(end.0).max(self.rect.min.x);
                let y0 = start.1.min(end.1).max(self.rect.min.y);
                let x1 = start.0.max(end.0).min(self.rect.max.x);
                let y1 = start.1.max(end.1).min(self.rect.max.y);

                if let Some(selection_rect) = rect![pt!(x0, y0), pt!(x1, y1)].intersection(&rect) {
                    fb.draw_rectangle(&selection_rect, color::BLACK);
                    fb.draw_rectangle(&selection_rect, color::BLACK);
                }
            }
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
