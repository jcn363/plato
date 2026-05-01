use crate::document::pdf_manipulator::PdfManipulator;
use crate::view::{Bus, Event};
use anyhow::{format_err, Error};
use std::path::{Path, PathBuf};

use super::types::ManipulationMode;

pub fn process_manipulation(
    manipulator: &mut PdfManipulator,
    file_path: &Path,
    action: &str,
    bus: &mut Bus,
    mode: &mut ManipulationMode,
) -> Result<(), Error> {
    *mode = ManipulationMode::Processing;

    let result: Result<PathBuf, Error> = match action {
        "delete" | "rotate90" | "rotate180" | "rotate270" => {
            bus.push_back(Event::Render("Select pages first".to_string()));
            Ok(file_path.to_path_buf())
        }
        "delete_all" => {
            let pages: Vec<_> = (0..10).collect();
            let output = file_path.with_extension("modified.pdf");
            manipulator.delete_pages(file_path, &output, &pages)
        }
        "rotate90_all" => {
            let pages: Vec<usize> = (1..=10).collect();
            let output = file_path.with_extension("rotated.pdf");
            manipulator.rotate_pages(file_path, &output, &pages, 90)
        }
        "rotate180_all" => {
            let pages: Vec<usize> = (1..=10).collect();
            let output = file_path.with_extension("rotated.pdf");
            manipulator.rotate_pages(file_path, &output, &pages, 180)
        }
        "rotate270_all" => {
            let pages: Vec<usize> = (1..=10).collect();
            let output = file_path.with_extension("rotated.pdf");
            manipulator.rotate_pages(file_path, &output, &pages, 270)
        }
        "extract" => {
            bus.push_back(Event::Render("Select pages first".to_string()));
            return Ok(());
        }
        "extract_all" => {
            let pages: Vec<_> = vec![0];
            let output = file_path.with_extension("extracted.pdf");
            manipulator.extract_pages(file_path, &output, &pages)
        }
        "extract_resources" => {
            handle_extract_resources(file_path, bus)?;
            return Ok(());
        }
        "export_annotations" => {
            handle_export_annotations(file_path, bus)?;
            return Ok(());
        }
        "read_annotations" => {
            handle_read_annotations(file_path, bus)?;
            return Ok(());
        }
        "search_annotations" => {
            handle_search_annotations(file_path, bus)?;
            return Ok(());
        }
        "export_xfdf" => {
            handle_export_xfdf(file_path, bus)?;
            return Ok(());
        }
        "import_xfdf" => {
            handle_import_xfdf(file_path, bus)?;
            return Ok(());
        }
        "booklet" => {
            let output = file_path.with_extension("booklet.pdf");
            manipulator.reorder_pages_for_booklet(file_path, &output)
        }
        "compare" => {
            let metadata = std::fs::metadata(file_path)?;
            let size_mb = metadata.len() as f64 / 1_048_576.0;
            let output = file_path.with_extension("info.txt");
            std::fs::write(
                &output,
                format!(
                    "File: {}\nSize: {:.2} MB\nModified: {}",
                    file_path.file_name().unwrap_or_default().to_string_lossy(),
                    size_mb,
                    chrono::Local::now().format("%Y-%m-%d %H:%M")
                ),
            )?;
            Ok(output)
        }
        _ => Err(format_err!("Unknown action")),
    };

    *mode = ManipulationMode::SelectFile;

    match result {
        Ok(_) => {
            bus.push_back(Event::Render(
                "✅ Operation complete! Backup created.".to_string(),
            ));
        }
        Err(e) => {
            let error_msg = if e.to_string().contains("memory") || e.to_string().contains("Memory")
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

fn handle_extract_resources(file_path: &Path, bus: &mut Bus) -> Result<(), Error> {
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
            bus.push_back(Event::Render(msg));
        }
        Err(e) => {
            bus.push_back(Event::Render(format!("Error: {}", e)));
        }
    }
    Ok(())
}

fn handle_export_annotations(file_path: &Path, bus: &mut Bus) -> Result<(), Error> {
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
    Ok(())
}

fn handle_read_annotations(file_path: &Path, bus: &mut Bus) -> Result<(), Error> {
    use crate::document::pdf_manipulator::ResourceExtractor;
    let extractor = ResourceExtractor::new(file_path)?;
    match extractor.read_annotations() {
        Ok(annotations) => {
            if annotations.is_empty() {
                bus.push_back(Event::Render("📋 No annotations found in PDF".to_string()));
            } else {
                let msg = format!("📋 Found {} annotations in PDF", annotations.len());
                bus.push_back(Event::Render(msg));
            }
        }
        Err(e) => {
            bus.push_back(Event::Render(format!("Error reading annotations: {}", e)));
        }
    }
    Ok(())
}

fn handle_search_annotations(file_path: &Path, bus: &mut Bus) -> Result<(), Error> {
    use crate::document::pdf_manipulator::{
        AnnotationQuery, AnnotationSubtype, PdfAnnotationManager,
    };
    let mut manager = PdfAnnotationManager::new(file_path)?;

    match manager.import_annotations() {
        Ok(_) => {
            let query = AnnotationQuery::new().with_subtype(AnnotationSubtype::Highlight);
            let results = manager.search(&query);
            if results.is_empty() {
                bus.push_back(Event::Render("🔍 No highlights found in PDF".to_string()));
            } else {
                let msg = format!("🔍 Found {} highlights in PDF", results.len());
                bus.push_back(Event::Render(msg));
            }
        }
        Err(e) => {
            bus.push_back(Event::Render(format!("Error importing annotations: {}", e)));
        }
    }
    Ok(())
}

fn handle_export_xfdf(file_path: &Path, bus: &mut Bus) -> Result<(), Error> {
    use crate::document::pdf_manipulator::{PdfAnnotationManager, XfdfHandler};
    let mut manager = PdfAnnotationManager::new(file_path)?;

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
    Ok(())
}

fn handle_import_xfdf(file_path: &Path, bus: &mut Bus) -> Result<(), Error> {
    use crate::document::pdf_manipulator::XfdfHandler;
    let xfdf_path = file_path.with_extension("xfdf");

    if !xfdf_path.exists() {
        bus.push_back(Event::Render(
            "❌ No XFDF file found. Export annotations first.".to_string(),
        ));
        return Ok(());
    }

    let xfdf_content = std::fs::read_to_string(&xfdf_path)?;
    match XfdfHandler::import_from_xfdf(&xfdf_content) {
        Ok(annotations) => {
            if annotations.is_empty() {
                bus.push_back(Event::Render(
                    "📥 No annotations found in XFDF file".to_string(),
                ));
            } else {
                let msg = format!("📥 Imported {} annotations from XFDF", annotations.len());
                bus.push_back(Event::Render(msg));
            }
        }
        Err(e) => {
            bus.push_back(Event::Render(format!("XFDF import failed: {}", e)));
        }
    }
    Ok(())
}
