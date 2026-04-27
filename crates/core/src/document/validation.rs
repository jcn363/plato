//! PDF/A and PDF/X validation module
//!
//! This module provides validation functionality for PDF/A (archival) and PDF/X (print production) standards.
//! It uses PDFPurr's built-in validation capabilities where available.

use crate::document::pdfpurr;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;

/// PDF/A conformance levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PdfALevel {
    /// PDF/A-1b (Level B - basic)
    A1b,
    /// PDF/A-2b (Level B - basic)
    A2b,
    /// PDF/A-3b (Level B - basic)
    A3b,
}

impl fmt::Display for PdfALevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PdfALevel::A1b => write!(f, "PDF/A-1b"),
            PdfALevel::A2b => write!(f, "PDF/A-2b"),
            PdfALevel::A3b => write!(f, "PDF/A-3b"),
        }
    }
}

/// PDF/X conformance levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PdfXLevel {
    /// PDF/X-1a
    X1a,
    /// PDF/X-3
    X3,
    /// PDF/X-4
    X4,
}

impl fmt::Display for PdfXLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PdfXLevel::X1a => write!(f, "PDF/X-1a"),
            PdfXLevel::X3 => write!(f, "PDF/X-3"),
            PdfXLevel::X4 => write!(f, "PDF/X-4"),
        }
    }
}

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Critical,
    Error,
    Warning,
    Info,
}

impl fmt::Display for ValidationSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationSeverity::Critical => write!(f, "Critical"),
            ValidationSeverity::Error => write!(f, "Error"),
            ValidationSeverity::Warning => write!(f, "Warning"),
            ValidationSeverity::Info => write!(f, "Info"),
        }
    }
}

/// Validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Severity of the issue
    pub severity: ValidationSeverity,
    /// Issue category
    pub category: String,
    /// Issue message
    pub message: String,
    /// Page number (if applicable)
    pub page: Option<usize>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the document is compliant
    pub is_compliant: bool,
    /// Standard being validated against
    pub standard: String,
    /// Validation issues
    pub issues: Vec<ValidationIssue>,
    /// Total number of issues
    pub total_issues: usize,
    /// Number of critical issues
    pub critical_issues: usize,
    /// Number of warnings
    pub warnings: usize,
}

impl ValidationResult {
    /// Create a new validation result
    pub fn new(standard: String) -> Self {
        ValidationResult {
            is_compliant: true,
            standard,
            issues: Vec::new(),
            total_issues: 0,
            critical_issues: 0,
            warnings: 0,
        }
    }

    /// Add an issue to the result
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        if issue.severity == ValidationSeverity::Critical {
            self.critical_issues += 1;
            self.is_compliant = false;
        } else if issue.severity == ValidationSeverity::Warning {
            self.warnings += 1;
        }
        self.total_issues += 1;
        self.issues.push(issue);
    }

    /// Get a summary of the validation result
    pub fn summary(&self) -> String {
        if self.is_compliant {
            format!("✓ Document is compliant with {}", self.standard)
        } else {
            format!(
                "✗ Document is not compliant with {}: {} critical, {} warnings, {} total",
                self.standard, self.critical_issues, self.warnings, self.total_issues
            )
        }
    }
}

/// PDF/A validator
pub struct PdfAValidator;

impl PdfAValidator {
    /// Validate a PDF document against a specific PDF/A level
    ///
    /// # Arguments
    /// * `pdf_data` - The PDF document data
    /// * `level` - The PDF/A conformance level to validate against
    ///
    /// # Returns
    /// A validation result containing compliance status and any issues
    pub fn validate(pdf_data: &[u8], level: PdfALevel) -> Result<ValidationResult> {
        let doc = pdfpurr::Document::from_bytes(pdf_data)
            .map_err(|e| anyhow::format_err!("Failed to load PDF: {}", e))?;

        let standard = level.to_string();
        let mut result = ValidationResult::new(standard);

        // Use PDFPurr's built-in PDF/A validation
        let pdfpurr_level = match level {
            PdfALevel::A1b => PdfPurrALevel::A1b,
            PdfALevel::A2b => PdfPurrALevel::A2b,
            PdfALevel::A3b => PdfPurrALevel::A3b,
        };

        let report = doc.validate_pdfa(pdfpurr_level);

        // Convert PDFPurr's StandardsReport to our ValidationResult
        // Check if all checks passed
        result.is_compliant = report.checks.iter().all(|check| check.passed);

        for check in &report.checks {
            if !check.passed {
                result.add_issue(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    category: check.id.to_string(),
                    message: check.description.to_string(),
                    page: None,
                });
            }
        }

        Ok(result)
    }

    /// Validate against all PDF/A levels and return the best match
    pub fn validate_all(pdf_data: &[u8]) -> Result<Vec<ValidationResult>> {
        let levels = [PdfALevel::A1b, PdfALevel::A2b, PdfALevel::A3b];

        let mut results = Vec::new();
        for level in levels {
            match Self::validate(pdf_data, level) {
                Ok(result) => results.push(result),
                Err(e) => {
                    // If validation fails completely, create an error result
                    let mut result = ValidationResult::new(level.to_string());
                    result.is_compliant = false;
                    result.add_issue(ValidationIssue {
                        severity: ValidationSeverity::Critical,
                        category: "Validation Error".to_string(),
                        message: format!("Failed to validate: {}", e),
                        page: None,
                    });
                    results.push(result);
                }
            }
        }

        Ok(results)
    }
}

/// PDF/X validator
pub struct PdfXValidator;

impl PdfXValidator {
    /// Validate a PDF document against a specific PDF/X level
    ///
    /// # Arguments
    /// * `pdf_data` - The PDF document data
    /// * `level` - The PDF/X conformance level to validate against
    ///
    /// # Returns
    /// A validation result containing compliance status and any issues
    pub fn validate(pdf_data: &[u8], level: PdfXLevel) -> Result<ValidationResult> {
        let doc = pdfpurr::Document::from_bytes(pdf_data)
            .map_err(|e| anyhow::format_err!("Failed to load PDF: {}", e))?;

        let standard = level.to_string();
        let mut result = ValidationResult::new(standard);

        // Use PDFPurr's built-in PDF/X validation
        let pdfpurr_level = match level {
            PdfXLevel::X1a => PdfPurrXLevel::X1a,
            PdfXLevel::X3 => PdfPurrXLevel::X3,
            PdfXLevel::X4 => PdfPurrXLevel::X4,
        };

        let report = doc.validate_pdfx(pdfpurr_level);

        // Convert PDFPurr's StandardsReport to our ValidationResult
        // Check if all checks passed
        result.is_compliant = report.checks.iter().all(|check| check.passed);

        for check in &report.checks {
            if !check.passed {
                result.add_issue(ValidationIssue {
                    severity: ValidationSeverity::Error,
                    category: check.id.to_string(),
                    message: check.description.to_string(),
                    page: None,
                });
            }
        }

        Ok(result)
    }

    /// Validate against all PDF/X levels and return the best match
    pub fn validate_all(pdf_data: &[u8]) -> Result<Vec<ValidationResult>> {
        let levels = [PdfXLevel::X1a, PdfXLevel::X3, PdfXLevel::X4];

        let mut results = Vec::new();
        for level in levels {
            match Self::validate(pdf_data, level) {
                Ok(result) => results.push(result),
                Err(e) => {
                    // If validation fails completely, create an error result
                    let mut result = ValidationResult::new(level.to_string());
                    result.is_compliant = false;
                    result.add_issue(ValidationIssue {
                        severity: ValidationSeverity::Critical,
                        category: "Validation Error".to_string(),
                        message: format!("Failed to validate: {}", e),
                        page: None,
                    });
                    results.push(result);
                }
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pdfa_level_display() {
        assert_eq!(PdfALevel::A1a.to_string(), "PDF/A-1a");
        assert_eq!(PdfALevel::A2b.to_string(), "PDF/A-2b");
        assert_eq!(PdfALevel::A3u.to_string(), "PDF/A-3u");
    }

    #[test]
    fn test_pdfx_level_display() {
        assert_eq!(PdfXLevel::X1a.to_string(), "PDF/X-1a");
        assert_eq!(PdfXLevel::X4p.to_string(), "PDF/X-4p");
        assert_eq!(PdfXLevel::X4g.to_string(), "PDF/X-4g");
    }

    #[test]
    fn test_validation_result_new() {
        let result = ValidationResult::new("PDF/A-2b".to_string());
        assert!(result.is_compliant);
        assert_eq!(result.total_issues, 0);
        assert_eq!(result.critical_issues, 0);
        assert_eq!(result.warnings, 0);
    }

    #[test]
    fn test_validation_result_add_critical_issue() {
        let mut result = ValidationResult::new("PDF/A-2b".to_string());
        result.add_issue(ValidationIssue {
            severity: ValidationSeverity::Critical,
            category: "Metadata".to_string(),
            message: "Missing XMP metadata".to_string(),
            page: None,
        });

        assert!(!result.is_compliant);
        assert_eq!(result.total_issues, 1);
        assert_eq!(result.critical_issues, 1);
        assert_eq!(result.warnings, 0);
    }

    #[test]
    fn test_validation_result_add_warning() {
        let mut result = ValidationResult::new("PDF/A-2b".to_string());
        result.add_issue(ValidationIssue {
            severity: ValidationSeverity::Warning,
            category: "Fonts".to_string(),
            message: "Font not embedded".to_string(),
            page: Some(1),
        });

        assert!(result.is_compliant); // Warnings don't make it non-compliant
        assert_eq!(result.total_issues, 1);
        assert_eq!(result.critical_issues, 0);
        assert_eq!(result.warnings, 1);
    }

    #[test]
    fn test_validation_result_summary() {
        let mut result = ValidationResult::new("PDF/A-2b".to_string());
        assert_eq!(result.summary(), "✓ Document is compliant with PDF/A-2b");

        result.add_issue(ValidationIssue {
            severity: ValidationSeverity::Critical,
            category: "Metadata".to_string(),
            message: "Missing XMP metadata".to_string(),
            page: None,
        });

        assert!(result.summary().contains("not compliant"));
        assert!(result.summary().contains("1 critical"));
    }
}
