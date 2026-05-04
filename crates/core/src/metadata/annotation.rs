use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::document::TextLocation;
use crate::helpers::datetime_format;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Annotation {
    #[serde(skip_serializing_if = "String::is_empty")]
    pub note: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub text: String,
    pub selection: [TextLocation; 2],
    #[serde(with = "datetime_format")]
    pub modified: NaiveDateTime,
}

impl Default for Annotation {
    fn default() -> Self {
        Annotation {
            note: String::new(),
            text: String::new(),
            selection: [TextLocation::Dynamic(0), TextLocation::Dynamic(1)],
            modified: Local::now().naive_local(),
        }
    }
}

impl Annotation {
    pub fn to_markdown(&self, _book_title: &str) -> String {
        let mut md = String::new();
        md.push_str(&format!("> {}\n\n", self.text.replace('\n', "\n> ")));
        if !self.note.is_empty() {
            md.push_str(&format!("**Note:** {}\n\n", self.note));
        }
        md.push_str(&format!(
            "- Location: {} - {}\n",
            self.selection[0], self.selection[1]
        ));
        md.push_str(&format!(
            "- Modified: {}\n",
            self.modified.format("%Y-%m-%d %H:%M")
        ));
        md
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

pub fn export_annotations_markdown(annotations: &[Annotation], book_title: &str) -> String {
    if annotations.is_empty() {
        return String::new();
    }
    let mut md = format!("# Annotations for \"{}\"\n\n", book_title);
    for (i, annotation) in annotations.iter().enumerate() {
        md.push_str(&format!("## Annotation {}\n\n", i + 1));
        md.push_str(&annotation.to_markdown(book_title));
        md.push_str("\n---\n\n");
    }
    md
}

pub fn export_annotations_json(annotations: &[Annotation]) -> String {
    if annotations.is_empty() {
        return String::new();
    }
    serde_json::to_string_pretty(annotations).unwrap_or_default()
}

pub fn export_to_readwise(
    annotations: &[Annotation],
    book_title: &str,
    book_author: Option<&str>,
) -> String {
    if annotations.is_empty() {
        return String::new();
    }

    let mut md = format!("# Highlights from \"{}\"\n\n", book_title);
    if let Some(author) = book_author {
        md.push_str(&format!("**Author:** {}\n\n", author));
    }
    md.push_str("---\n\n");

    for (i, annot) in annotations.iter().enumerate() {
        md.push_str(&format!("## Highlight {}\n\n", i + 1));
        md.push_str(&format!("> {}\n\n", annot.text.replace('\n', "\n> ")));
        if !annot.note.is_empty() {
            md.push_str(&format!("**Note:** {}\n\n", annot.note));
        }
        md.push_str(&format!(
            "- Location: {} - {}\n",
            annot.selection[0], annot.selection[1]
        ));
        md.push_str(&format!(
            "- Date: {}\n\n",
            annot.modified.format("%Y-%m-%d")
        ));
    }

    md
}

pub fn export_to_obsidian(
    annotations: &[Annotation],
    book_title: &str,
    book_author: Option<&str>,
) -> String {
    if annotations.is_empty() {
        return String::new();
    }

    let mut md = format!("# {}\n\n", book_title);
    if let Some(author) = book_author {
        md.push_str(&format!("**Author:** {}\n\n", author));
    }
    md.push_str("---\n\n## Highlights\n\n");

    for annot in annotations {
        md.push_str(&format!("> {}\n\n", annot.text.replace('\n', "\n> ")));
        if !annot.note.is_empty() {
            md.push_str(&format!("**Note:** {}\n\n", annot.note));
        }
    }

    md
}

pub fn generate_quote_card(text: &str, book_title: &str, author: Option<&str>) -> String {
    let mut card = String::new();
    card.push_str("┌─────────────────────────────────────┐\n");
    card.push_str("│                                     │\n");

    let wrapped = wrap_text(text, 36);
    for line in wrapped {
        let len = line.len();
        let padding = (36 - len) / 2;
        let pad = " ".repeat(padding);
        card.push_str(&format!("│{}{}{}│\n", pad, line, pad));
    }

    card.push_str("│                                     │\n");
    card.push_str("│ ─────────────────────────────────── │\n");
    card.push_str(&format!("│ {:^35} │\n", book_title));
    if let Some(a) = author {
        card.push_str(&format!("│ {:^35} │\n", a));
    }
    card.push_str("└─────────────────────────────────────┘");
    card
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();

    for word in text.split_whitespace() {
        if current.len() + word.len() + 1 > max_width {
            if !current.is_empty() {
                lines.push(current.clone());
                current.clear();
            }
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }

    lines
}

pub fn export_to_bibtex(
    annotations: &[Annotation],
    book_title: &str,
    author: Option<&str>,
) -> String {
    if annotations.is_empty() {
        return String::new();
    }

    let key = book_title
        .to_lowercase()
        .split_whitespace()
        .take(2)
        .collect::<Vec<_>>()
        .join("");

    let mut bib = format!("@book{}{{,\n", key);
    bib.push_str(&format!("  title = {{{}}},\n", book_title));
    if let Some(a) = author {
        bib.push_str(&format!("  author = {{{}}},\n", a));
    }
    bib.push_str(&format!(
        "  note = {{{} highlight(s)}},\n",
        annotations.len()
    ));
    for (i, annot) in annotations.iter().enumerate() {
        bib.push_str(&format!("  highlight{} = {{{}}},\n", i + 1, annot.text));
    }
    bib.push('}');
    bib
}

pub fn export_to_ris(annotations: &[Annotation], book_title: &str, author: Option<&str>) -> String {
    if annotations.is_empty() {
        return String::new();
    }

    let mut ris = String::new();

    for annot in annotations {
        ris.push_str("TY  - BOOK\n");
        ris.push_str(&format!("TI  - {}\n", book_title));
        if let Some(a) = author {
            ris.push_str(&format!("AU  - {}\n", a));
        }
        ris.push_str("QU  - \n");
        ris.push_str(&format!("TX  - {}\n", annot.text));
        if !annot.note.is_empty() {
            ris.push_str(&format!("AN  - {}\n", annot.note));
        }
        ris.push_str("ER  - \n\n");
    }

    ris
}

pub fn export_citations(
    annotations: &[Annotation],
    book_title: &str,
    author: Option<&str>,
    format: crate::settings::CitationFormat,
) -> String {
    match format {
        crate::settings::CitationFormat::BibTeX => {
            export_to_bibtex(annotations, book_title, author)
        }
        crate::settings::CitationFormat::RIS => export_to_ris(annotations, book_title, author),
        _ => String::new(),
    }
}
