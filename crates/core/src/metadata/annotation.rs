use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};

use crate::document::TextLocation;
use crate::helpers::datetime_format;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
    let mut md = format!("# Annotations for \"{}\"\n\n", book_title);
    for (i, annotation) in annotations.iter().enumerate() {
        md.push_str(&format!("## Annotation {}\n\n", i + 1));
        md.push_str(&annotation.to_markdown(book_title));
        md.push_str("\n---\n\n");
    }
    md
}

pub fn export_annotations_json(annotations: &[Annotation]) -> String {
    serde_json::to_string_pretty(annotations).unwrap_or_default()
}
