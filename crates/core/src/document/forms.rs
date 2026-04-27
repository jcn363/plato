//! PDF Forms Handling
//!
//! This module provides PDF form field parsing, value storage, and manipulation for mobile/desktop platforms.
//! Uses lopdf to parse AcroForm data from PDF documents.
//!
//! ## Architecture
//!
//! - **FormParser**: Extracts form fields from PDF AcroForm
//! - **FormField**: Represents individual form fields (text, checkbox, radio, dropdown, signature)
//! - **FormValues**: Stores user-entered form field values
//! - **FormExporter**: Exports filled form values back to PDF
//!
//! ## Platform Support
//!
//! This module is only compiled on mobile (Android/iOS) and desktop (Linux) platforms.
//! Kobo e-readers exclude this feature due to poor e-ink UX for text input.

#![cfg(any(target_os = "android", target_os = "ios", target_os = "linux"))]

use anyhow::{bail, Context, Error};
use lopdf::{Dictionary, Document, Object};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

/// Form field types supported by PDF forms
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FormFieldType {
    /// Text input field
    Text,
    /// Checkbox field
    Checkbox,
    /// Radio button group
    Radio,
    /// Dropdown/combobox field
    Dropdown,
    /// List box field
    List,
    /// Signature field
    Signature,
    /// Button field
    Button,
}

/// Represents a single form field in a PDF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    /// Field name (from PDF /T key)
    pub name: String,
    /// Field type
    pub field_type: FormFieldType,
    /// Field label (optional, from /TU key)
    pub label: Option<String>,
    /// Default value (from /DV key)
    pub default_value: Option<String>,
    /// Current value (user-entered)
    pub value: Option<String>,
    /// Field flags (bitmask from /Ff key)
    pub flags: u32,
    /// Options for dropdown/radio fields (from /Opt key)
    pub options: Vec<String>,
    /// Page number where field appears
    pub page: u32,
    /// Field rectangle on page (from /Rect key)
    pub rect: [f32; 4],
    /// Whether field is read-only (flag bit 1)
    pub read_only: bool,
    /// Whether field is required (flag bit 2)
    pub required: bool,
    /// Whether field is multiline text (flag bit 13)
    pub multiline: bool,
}

impl FormField {
    /// Create a new FormField from a lopdf dictionary
    pub fn from_dict(
        name: String,
        dict: &Dictionary,
        page: u32,
        _doc: &Document,
    ) -> Result<Self, Error> {
        let field_type = Self::parse_field_type(dict)?;
        let label = dict
            .get(b"TU")
            .ok()
            .and_then(|obj| obj.as_str().ok())
            .map(|s| std::str::from_utf8(s).unwrap_or_default().to_string());
        let default_value = dict
            .get(b"DV")
            .ok()
            .and_then(|obj| obj.as_str().ok())
            .map(|s| std::str::from_utf8(s).unwrap_or_default().to_string());
        let flags = dict
            .get(b"Ff")
            .ok()
            .and_then(|obj| obj.as_i64().ok())
            .unwrap_or(0) as u32;
        let options = Self::parse_options(dict)?;
        let rect = Self::parse_rect(dict)?;
        let read_only = (flags & 1) != 0;
        let required = (flags & 2) != 0;
        let multiline = (flags & (1 << 13)) != 0;

        Ok(FormField {
            name,
            field_type,
            label,
            default_value: default_value.clone(),
            value: default_value,
            flags,
            options,
            page,
            rect,
            read_only,
            required,
            multiline,
        })
    }

    /// Parse field type from dictionary
    fn parse_field_type(dict: &Dictionary) -> Result<FormFieldType, Error> {
        // Get the field's parent dictionary to determine type
        if let Ok(parent_obj) = dict.get(b"Parent") {
            if let Ok(parent) = parent_obj.as_dict() {
                if let Ok(ft_obj) = parent.get(b"FT") {
                    if let Ok(ft) = ft_obj.as_name() {
                        return match ft {
                            b"Tx" => Ok(FormFieldType::Text),
                            b"Btn" => {
                                // Button can be checkbox, radio, or pushbutton
                                if let Ok(flags_obj) = parent.get(b"Ff") {
                                    if let Ok(flags) = flags_obj.as_i64() {
                                        let flags = flags as u32;
                                        if flags & (1 << 15) != 0 {
                                            Ok(FormFieldType::Radio)
                                        } else if flags & (1 << 16) != 0 {
                                            Ok(FormFieldType::Checkbox)
                                        } else {
                                            Ok(FormFieldType::Button)
                                        }
                                    } else {
                                        Ok(FormFieldType::Button)
                                    }
                                } else {
                                    Ok(FormFieldType::Button)
                                }
                            }
                            b"Ch" => {
                                // Choice field can be dropdown or list
                                if let Ok(flags_obj) = parent.get(b"Ff") {
                                    if let Ok(flags) = flags_obj.as_i64() {
                                        let flags = flags as u32;
                                        if flags & (1 << 18) != 0 {
                                            Ok(FormFieldType::List)
                                        } else {
                                            Ok(FormFieldType::Dropdown)
                                        }
                                    } else {
                                        Ok(FormFieldType::Dropdown)
                                    }
                                } else {
                                    Ok(FormFieldType::Dropdown)
                                }
                            }
                            b"Sig" => Ok(FormFieldType::Signature),
                            _ => bail!(
                                "Unknown field type: {:?}",
                                std::str::from_utf8(ft).unwrap_or_default()
                            ),
                        };
                    }
                }
            }
        }
        // Fallback: try to determine from structure
        if dict.get(b"Opt").is_ok() {
            Ok(FormFieldType::Dropdown)
        } else if dict.get(b"V").is_ok() {
            Ok(FormFieldType::Checkbox)
        } else {
            Ok(FormFieldType::Text)
        }
    }

    /// Parse options for dropdown/radio fields
    fn parse_options(dict: &Dictionary) -> Result<Vec<String>, Error> {
        let mut options = Vec::new();

        if let Ok(opt_obj) = dict.get(b"Opt") {
            if let Ok(opt_array) = opt_obj.as_array() {
                for item in opt_array {
                    if let Ok(str) = item.as_str() {
                        options.push(std::str::from_utf8(str).unwrap_or_default().to_string());
                    } else if let Ok(opt_dict) = item.as_dict() {
                        // Option may be a dict with display value and export value
                        if let Ok(display_obj) = opt_dict.get(b"") {
                            if let Ok(display) = display_obj.as_str() {
                                options.push(
                                    std::str::from_utf8(display).unwrap_or_default().to_string(),
                                );
                            }
                        }
                    }
                }
            }
        }

        Ok(options)
    }

    /// Parse rectangle from dictionary
    fn parse_rect(dict: &Dictionary) -> Result<[f32; 4], Error> {
        if let Ok(rect_obj) = dict.get(b"Rect") {
            if let Ok(rect_array) = rect_obj.as_array() {
                if rect_array.len() >= 4 {
                    let rect: Vec<f32> = rect_array
                        .iter()
                        .filter_map(|obj| obj.as_i64().ok().map(|f| f as f32))
                        .collect();
                    if rect.len() == 4 {
                        return Ok([rect[0], rect[1], rect[2], rect[3]]);
                    }
                }
            }
        }
        bail!("Failed to parse field rectangle");
    }

    /// Set the field value
    pub fn set_value(&mut self, value: String) {
        self.value = Some(value);
    }

    /// Clear the field value
    pub fn clear_value(&mut self) {
        self.value = self.default_value.clone();
    }

    /// Check if field has a value
    pub fn has_value(&self) -> bool {
        self.value.is_some() && !self.value.as_ref().map(|s| s.is_empty()).unwrap_or(false)
    }
}

/// Form parser for extracting form fields from PDF
pub struct FormParser;

impl FormParser {
    /// Parse all form fields from a PDF document
    pub fn parse_document(path: &Path) -> Result<Vec<FormField>, Error> {
        let doc = Document::load(path)
            .with_context(|| format!("Failed to load PDF: {}", path.display()))?;
        Self::parse_from_document(&doc)
    }

    /// Parse all form fields from a loaded lopdf Document
    pub fn parse_from_document(doc: &Document) -> Result<Vec<FormField>, Error> {
        let mut fields = Vec::new();

        // Get the AcroForm dictionary
        let catalog = doc.catalog().context("Failed to get catalog")?;

        let acroform = catalog
            .get(b"AcroForm")
            .ok()
            .and_then(|obj| obj.as_dict().ok())
            .context("No AcroForm found in PDF")?;

        // Get the Fields array
        let fields_array = acroform
            .get(b"Fields")
            .ok()
            .and_then(|obj| obj.as_array().ok())
            .context("No Fields array found in AcroForm")?;

        // Parse each field
        for field_obj in fields_array {
            if let Ok(field_dict) = field_obj.as_dict() {
                // Get field name
                let name = field_dict
                    .get(b"T")
                    .ok()
                    .and_then(|obj| obj.as_str().ok())
                    .map(|s| std::str::from_utf8(s).unwrap_or_default().to_string())
                    .unwrap_or_else(|| "unnamed".to_string());

                // Determine page number
                let page = Self::get_field_page(field_dict, doc)?;

                // Parse the field
                if let Ok(field) = FormField::from_dict(name, field_dict, page, doc) {
                    fields.push(field);
                }
            }
        }

        Ok(fields)
    }

    /// Get the page number for a field
    fn get_field_page(field_dict: &Dictionary, doc: &Document) -> Result<u32, Error> {
        // Try to get page from /P key
        if let Ok(page_obj) = field_dict.get(b"P") {
            if let Ok(page_ref) = page_obj.as_reference() {
                // Find page number from reference
                let pages = doc.get_pages();
                for (page_num, page_id) in pages.iter() {
                    // Compare the reference IDs directly
                    if page_id.0 == page_ref.0 && page_id.1 == page_ref.1 {
                        return Ok(*page_num);
                    }
                }
            }
        }
        // Default to page 1 if not found
        Ok(1)
    }
}

/// Stores form field values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormValues {
    /// Map of field name to value
    pub values: BTreeMap<String, String>,
}

impl FormValues {
    /// Create empty FormValues
    pub fn new() -> Self {
        FormValues {
            values: BTreeMap::new(),
        }
    }

    /// Set a field value
    pub fn set(&mut self, field_name: &str, value: String) {
        self.values.insert(field_name.to_string(), value);
    }

    /// Get a field value
    pub fn get(&self, field_name: &str) -> Option<&String> {
        self.values.get(field_name)
    }

    /// Clear a field value
    pub fn clear(&mut self, field_name: &str) {
        self.values.remove(field_name);
    }

    /// Check if all required fields have values
    pub fn is_complete(&self, fields: &[FormField]) -> bool {
        for field in fields {
            if field.required && !self.values.contains_key(&field.name) {
                return false;
            }
        }
        true
    }
}

impl Default for FormValues {
    fn default() -> Self {
        Self::new()
    }
}

/// Form exporter for writing filled form values back to PDF
pub struct FormExporter;

impl FormExporter {
    /// Export form values to a new PDF file
    pub fn export_to_pdf(
        input_path: &Path,
        output_path: &Path,
        values: &FormValues,
    ) -> Result<(), Error> {
        let mut doc = Document::load(input_path)
            .with_context(|| format!("Failed to load PDF: {}", input_path.display()))?;

        // Get the AcroForm dictionary
        let catalog = doc.catalog().context("Failed to get catalog")?;

        let acroform = catalog
            .get(b"AcroForm")
            .ok()
            .and_then(|obj| obj.as_dict().ok())
            .context("No AcroForm found in PDF")?;

        // Get the Fields array
        let fields_array = acroform
            .get(b"Fields")
            .ok()
            .and_then(|obj| obj.as_array().ok())
            .context("No Fields array found in AcroForm")?;

        // Collect field IDs and their values to update
        let mut field_updates: Vec<((u32, u16), String)> = Vec::new();

        for field_obj in fields_array {
            if let Ok(field_dict) = field_obj.as_dict() {
                if let Ok(name_obj) = field_dict.get(b"T") {
                    if let Ok(name_bytes) = name_obj.as_str() {
                        let name = std::str::from_utf8(name_bytes)
                            .unwrap_or_default()
                            .to_string();

                        if let Some(value) = values.get(&name) {
                            if let Ok(field_id) = field_obj.as_reference() {
                                field_updates.push((field_id, value.clone()));
                            }
                        }
                    }
                }
            }
        }

        // Update field values using the document's object map
        for (field_id, value) in field_updates {
            if let Some(obj) = doc.objects.get_mut(&field_id) {
                if let Ok(field_dict) = obj.as_dict_mut() {
                    let value_obj =
                        Object::String(value.clone().into_bytes(), lopdf::StringFormat::Literal);
                    field_dict.set(b"V", value_obj);
                }
            }
        }

        // Save the modified PDF
        doc.save(output_path)
            .with_context(|| format!("Failed to save PDF: {}", output_path.display()))?;

        Ok(())
    }

    /// Get form values as a serializable format
    pub fn export_values(values: &FormValues) -> Result<String, Error> {
        serde_json::to_string_pretty(values).context("Failed to serialize form values")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_form_field_creation() {
        let field = FormField {
            name: "test_field".to_string(),
            field_type: FormFieldType::Text,
            label: Some("Test Field".to_string()),
            default_value: Some("default".to_string()),
            value: Some("default".to_string()),
            flags: 0,
            options: vec![],
            page: 1,
            rect: [0.0, 0.0, 100.0, 20.0],
            read_only: false,
            required: false,
            multiline: false,
        };

        assert_eq!(field.name, "test_field");
        assert_eq!(field.field_type, FormFieldType::Text);
        assert!(field.has_value());
    }

    #[test]
    fn test_form_values() {
        let mut values = FormValues::new();
        values.set("field1", "value1".to_string());
        values.set("field2", "value2".to_string());

        assert_eq!(values.get("field1"), Some(&"value1".to_string()));
        assert_eq!(values.get("field2"), Some(&"value2".to_string()));
        assert!(values.get("field3").is_none());
    }

    #[test]
    fn test_form_values_completeness() {
        let mut values = FormValues::new();
        values.set("field1", "value1".to_string());

        let fields = vec![
            FormField {
                name: "field1".to_string(),
                field_type: FormFieldType::Text,
                label: None,
                default_value: None,
                value: Some("value1".to_string()),
                flags: 0,
                options: vec![],
                page: 1,
                rect: [0.0, 0.0, 100.0, 20.0],
                read_only: false,
                required: true,
                multiline: false,
            },
            FormField {
                name: "field2".to_string(),
                field_type: FormFieldType::Text,
                label: None,
                default_value: None,
                value: None,
                flags: 0,
                options: vec![],
                page: 1,
                rect: [0.0, 0.0, 100.0, 20.0],
                read_only: false,
                required: true,
                multiline: false,
            },
        ];

        assert!(!values.is_complete(&fields));
    }
}
