use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use crate::document::asciify;
use crate::document::epub::EpubDocument;
use crate::document::html::HtmlDocument;
use crate::document::pdf::PdfOpener;
use crate::document::Document;
use crate::{log_error, log_warn};
use titlecase::titlecase;

use super::info::Info;

#[inline]
pub fn extract_metadata_from_document(prefix: &Path, info: &mut Info) {
    let path = prefix.join(&info.file.path);

    match info.file.kind.as_ref() {
        "epub" => match EpubDocument::new(&path) {
            Ok(doc) => {
                info.title = doc.title().unwrap_or_default();
                info.author = doc.author().unwrap_or_default();
                info.year = doc.year().unwrap_or_default();
                info.publisher = doc.publisher().unwrap_or_default();
                if let Some((title, index)) = doc.series() {
                    info.series = title;
                    info.number = index;
                }
                info.language = doc.language().unwrap_or_default();
                info.categories.append(&mut doc.categories());
            }
            Err(e) => log_error!("Can't open {}: {:#}.", info.file.path.display(), e),
        },
        "html" | "htm" => match HtmlDocument::new(&path) {
            Ok(doc) => {
                info.title = doc.title().unwrap_or_default();
                info.author = doc.author().unwrap_or_default();
                info.language = doc.language().unwrap_or_default();
            }
            Err(e) => log_error!("Can't open {}: {:#}.", info.file.path.display(), e),
        },
        "pdf" => match PdfOpener::new().and_then(|o| o.open(path)) {
            Some(doc) => {
                info.title = doc.title().unwrap_or_default();
                info.author = doc.author().unwrap_or_default();
            }
            None => log_error!("Can't open {}.", info.file.path.display()),
        },
        _ => {
            log_warn!(
                "Don't know how to extract metadata from {}.",
                &info.file.kind
            );
        }
    }
}

pub fn extract_metadata_from_filename(_prefix: &Path, info: &mut Info) {
    if let Some(filename) = info.file.path.file_name().and_then(OsStr::to_str) {
        let mut start_index = 0;

        if filename.starts_with('(') {
            start_index += 1;
            if let Some(index) = filename[start_index..].find(')') {
                info.series = filename[start_index..start_index + index]
                    .trim_end()
                    .to_string();
                start_index += index + 1;
            }
        }

        if let Some(index) = filename[start_index..].find("- ") {
            info.author = filename[start_index..start_index + index]
                .trim()
                .to_string();
            start_index += index + 1;
        }

        let title_start = start_index;

        if let Some(index) = filename[start_index..].find('_') {
            info.title = filename[start_index..start_index + index]
                .trim_start()
                .to_string();
            start_index += index + 1;
        }

        if let Some(index) = filename[start_index..].find('-') {
            if title_start == start_index {
                info.title = filename[start_index..start_index + index]
                    .trim_start()
                    .to_string();
            } else {
                info.subtitle = filename[start_index..start_index + index]
                    .trim_start()
                    .to_string();
            }
            start_index += index + 1;
        }

        if let Some(index) = filename[start_index..].find('(') {
            info.publisher = filename[start_index..start_index + index]
                .trim_end()
                .to_string();
            start_index += index + 1;
        }

        if let Some(index) = filename[start_index..].find(')') {
            info.year = filename[start_index..start_index + index].to_string();
        }
    }
}

pub fn consolidate(_prefix: &Path, info: &mut Info) {
    if info.subtitle.is_empty() {
        if let Some(index) = info.title.find(':') {
            let cur_title = info.title.clone();
            let (title, subtitle) = cur_title.split_at(index);
            info.title = title.trim_end().to_string();
            info.subtitle = subtitle[1..].trim_start().to_string();
        }
    }

    if info.language.is_empty() || info.language.starts_with("en") {
        info.title = titlecase(&info.title);
        info.subtitle = titlecase(&info.subtitle);
    }

    info.title = info.title.replace('\'', "’");
    info.subtitle = info.subtitle.replace('\'', "’");
    info.author = info.author.replace('\'', "’");
    if info.year.len() > 4 {
        info.year = info.year[..4].to_string();
    }
    info.series = info.series.replace('\'', "’");
    info.publisher = info.publisher.replace('\'', "’");
}

pub fn rename_from_info(prefix: &Path, info: &mut Info) {
    let new_file_name = file_name_from_info(info);
    if !new_file_name.is_empty() {
        let old_path = prefix.join(&info.file.path);
        let new_path = old_path.with_file_name(&new_file_name);
        if old_path != new_path {
            match fs::rename(&old_path, &new_path) {
                Err(e) => log_error!(
                    "Can't rename {} to {}: {:#}.",
                    old_path.display(),
                    new_path.display(),
                    e
                ),
                Ok(..) => {
                    let relat = new_path.strip_prefix(prefix).unwrap_or(&new_path);
                    info.file.path = relat.to_path_buf();
                }
            }
        }
    }
}

pub fn file_name_from_info(info: &Info) -> String {
    if info.title.is_empty() {
        return "".to_string();
    }
    let mut base = asciify(&info.title);
    if !info.subtitle.is_empty() {
        base = format!("{} - {}", base, asciify(&info.subtitle));
    }
    if !info.volume.is_empty() {
        base = format!("{} - {}", base, info.volume);
    }
    if !info.number.is_empty() && info.series.is_empty() {
        base = format!("{} - {}", base, info.number);
    }
    if !info.author.is_empty() {
        base = format!("{} - {}", base, asciify(&info.author));
    }
    base = format!("{}.{}", base, info.file.kind);
    base.replace("..", ".")
        .replace('/', " ")
        .replace('?', "")
        .replace('!', "")
        .replace(':', "")
}
