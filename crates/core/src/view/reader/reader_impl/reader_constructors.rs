//! Reader Constructor Functions
//!
//! Functions for creating Reader instances and opening documents.

use crate::context::Context;
use crate::geom::Rectangle;
use crate::log_error;
use crate::metadata::Info;
use crate::unit::scale_by_dpi;
use anyhow::{Context as AnyhowContext, Error};
use std::sync::{Arc, Mutex};

use crate::view::reader::tool_bar::ToolBar;
use crate::view::SMALL_BAR_HEIGHT;

/// Type alias for the document handle used throughout the reader
type DocumentHandle = Arc<Mutex<Box<dyn crate::document::Document>>>;

/// Result type for document opening operations
type DocumentResult = (DocumentHandle, usize, bool);

pub(crate) fn open_document(info: &Info) -> Option<DocumentResult> {
    let doc = match crate::document::open(&info.file.path) {
        Some(d) => d,
        None => {
            log_error!("Failed to open document: {}", info.file.path.display());
            return None;
        }
    };
    let doc = Arc::new(Mutex::new(doc));
    let pages_count = (*doc.lock().expect("doc lock")).pages_count();
    let reflowable = (*doc.lock().expect("doc lock")).is_reflowable();
    Some((doc, pages_count, reflowable))
}

pub(crate) fn create_toolbar(
    rect: Rectangle,
    reflowable: bool,
    info: &Info,
    context: &mut Context,
) -> Vec<Box<dyn crate::view::View>> {
    let dpi = crate::unit::get_device_dpi();
    let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;

    let top_bar_rect = rect![
        rect.min.x,
        rect.min.y,
        rect.max.x,
        rect.min.y + small_height
    ];
    let tool_bar = ToolBar::new(
        top_bar_rect,
        reflowable,
        info.reader.as_ref(),
        &context.settings.reader,
    );
    vec![Box::new(tool_bar) as Box<dyn crate::view::View>]
}

pub(crate) fn open_html_document(html: &str) -> Result<DocumentResult, Error> {
    let doc = crate::document::open_html(html).context("Failed to open HTML document")?;
    let doc = Arc::new(Mutex::new(doc));
    let pages_count = (*doc.lock().expect("doc lock")).pages_count();
    let reflowable = (*doc.lock().expect("doc lock")).is_reflowable();
    Ok((doc, pages_count, reflowable))
}

pub(crate) fn create_html_info(html: &str) -> Info {
    Info {
        file: crate::metadata::FileInfo {
            path: std::path::PathBuf::from("memory.html"),
            kind: "html".to_string(),
            size: html.len() as u64,
        },
        reader: None,
        ..Default::default()
    }
}
