use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::PathBuf;

use crate::document::SimpleTocEntry;
use crate::helpers::datetime_format;

use super::annotation::Annotation;

pub type Metadata = Vec<Info>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Info {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub title: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub subtitle: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub author: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub year: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub language: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub publisher: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub series: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub edition: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub volume: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub number: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub identifier: String,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub categories: BTreeSet<String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub tags: BTreeSet<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    pub file: FileInfo,
    #[serde(skip_serializing)]
    pub reader: Option<ReaderInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reader_info: Option<ReaderInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toc: Option<Vec<SimpleTocEntry>>,
    #[serde(with = "datetime_format")]
    pub added: NaiveDateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct FileInfo {
    pub path: PathBuf,
    pub kind: String,
    pub size: u64,
}

impl Default for FileInfo {
    fn default() -> Self {
        FileInfo {
            path: PathBuf::default(),
            kind: String::default(),
            size: u64::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ReaderInfo {
    #[serde(with = "datetime_format")]
    pub opened: NaiveDateTime,
    pub current_page: usize,
    pub pages_count: usize,
    pub finished: bool,
    pub dithered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zoom_mode: Option<super::types::ZoomMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scroll_mode: Option<super::types::ScrollMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_offset: Option<crate::geom::Point>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<i8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cropping_margins: Option<super::types::CroppingMargins>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub margin_width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screen_margin_width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<super::types::TextAlign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_height: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contrast_exponent: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contrast_gray: Option<f32>,
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub page_names: std::collections::BTreeMap<usize, String>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub bookmarks: BTreeSet<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub annotations: Vec<Annotation>,
    #[serde(skip_serializing)]
    pub reading_time_seconds: u64,
}

impl ReaderInfo {
    pub fn progress(&self) -> f32 {
        (self.current_page / self.pages_count) as f32
    }

    pub fn total_reading_time(&self) -> u64 {
        self.reading_time_seconds
    }

    pub fn add_reading_time(&mut self, seconds: u64) {
        self.reading_time_seconds += seconds;
    }
}

impl Default for ReaderInfo {
    fn default() -> Self {
        ReaderInfo {
            opened: Local::now().naive_local(),
            current_page: 0,
            pages_count: 1,
            finished: false,
            dithered: false,
            zoom_mode: None,
            scroll_mode: None,
            page_offset: None,
            rotation: None,
            cropping_margins: None,
            margin_width: None,
            screen_margin_width: None,
            font_family: None,
            font_size: None,
            text_align: None,
            line_height: None,
            contrast_exponent: None,
            contrast_gray: None,
            page_names: std::collections::BTreeMap::new(),
            bookmarks: BTreeSet::new(),
            annotations: Vec::new(),
            reading_time_seconds: 0,
        }
    }
}

impl Default for Info {
    fn default() -> Self {
        Info {
            title: String::default(),
            subtitle: String::default(),
            author: String::default(),
            year: String::default(),
            language: String::default(),
            publisher: String::default(),
            series: String::default(),
            edition: String::default(),
            volume: String::default(),
            number: String::default(),
            identifier: String::default(),
            categories: BTreeSet::new(),
            tags: BTreeSet::new(),
            collection: None,
            file: FileInfo::default(),
            added: Local::now().naive_local(),
            reader: None,
            reader_info: None,
            toc: None,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Status {
    New,
    Reading(f32),
    Finished,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SimpleStatus {
    New,
    Reading,
    Finished,
}

impl std::fmt::Display for SimpleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

impl Info {
    pub fn status(&self) -> Status {
        if let Some(ref r) = self.reader {
            if r.finished {
                Status::Finished
            } else {
                Status::Reading(r.current_page as f32 / r.pages_count as f32)
            }
        } else {
            Status::New
        }
    }

    pub fn simple_status(&self) -> SimpleStatus {
        if let Some(ref r) = self.reader {
            if r.finished {
                SimpleStatus::Finished
            } else {
                SimpleStatus::Reading
            }
        } else {
            SimpleStatus::New
        }
    }

    pub fn file_stem(&self) -> String {
        self.file
            .path
            .file_stem()
            .expect("file_stem is missing")
            .to_string_lossy()
            .into_owned()
    }

    pub fn title(&self) -> String {
        if self.title.is_empty() {
            return self.file_stem();
        }

        let mut title = self.title.clone();

        if !self.number.is_empty() && self.series.is_empty() {
            title = format!("{} #{}", title, self.number);
        }

        if !self.volume.is_empty() {
            title = format!("{} — vol. {}", title, self.volume);
        }

        if !self.subtitle.is_empty() {
            title = if self
                .subtitle
                .chars()
                .next()
                .unwrap_or('_')
                .is_alphanumeric()
                && title
                    .chars()
                    .last()
                    .expect("title is empty")
                    .is_alphanumeric()
            {
                format!("{}: {}", title, self.subtitle)
            } else {
                format!("{} {}", title, self.subtitle)
            };
        }

        if !self.series.is_empty() && !self.number.is_empty() {
            title = format!("{} ({} #{})", title, self.series, self.number);
        }

        title
    }

    pub fn alphabetic_author(&self) -> String {
        let author_suffixes = [
            "jr.", "sr.", "ii", "iii", "iv", "esq.", "dr.", "mr.", "mrs.", "ms.",
        ];

        let mut parts = self
            .author
            .split(',')
            .next()
            .unwrap_or("")
            .split(' ')
            .collect::<Vec<_>>();

        while let Some(last) = parts.last() {
            let lower = last.to_lowercase().trim_end_matches('.').to_lowercase();
            if author_suffixes.contains(&lower.as_str()) || last.ends_with('.') {
                parts.pop();
            } else {
                break;
            }
        }

        parts.join(" ")
    }

    pub fn alphabetic_title(&self) -> &str {
        let mut start = 0;

        let lang = if self.language.is_empty() || self.language.starts_with("en") {
            "en"
        } else if self.language.starts_with("fr") {
            "fr"
        } else {
            &self.language
        };

        if let Some(m) = super::constants::TITLE_PREFIXES
            .get(lang)
            .and_then(|re| re.find(&self.title))
        {
            start = m.end()
        }

        &self.title[start..]
    }

    pub fn label(&self) -> String {
        if !self.author.is_empty() {
            format!("{} · {}", self.title(), &self.author)
        } else {
            self.title()
        }
    }
}
