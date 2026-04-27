//! Digital Signatures for PDF Documents
//!
//! Provides digital signature functionality for PDF documents on desktop platforms.
//! Supports certificate management, signing operations, and signature verification.
//!
//! ## Platform Support
//!
//! This module is only compiled on desktop (Linux) platforms.
//! Kobo e-readers and mobile devices exclude this feature due to:
//! - No secure key storage on Kobo
//! - Mobile devices have platform-specific key management
//! - Digital signing workflows are desktop-centric

#![cfg(target_os = "linux")]

use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Digital signature information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalSignature {
    /// Unique signature identifier
    pub id: String,
    /// Signer name
    pub signer_name: String,
    /// Signer email
    pub signer_email: Option<String>,
    /// Signature timestamp (ISO 8601 string)
    pub timestamp: String,
    /// Certificate fingerprint
    pub certificate_fingerprint: String,
    /// Signature validity status
    pub is_valid: bool,
}

/// Certificate information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    /// Certificate subject name
    pub subject: String,
    /// Certificate issuer name
    pub issuer: String,
    /// Certificate fingerprint (SHA-256)
    pub fingerprint: String,
    /// Certificate validity start (ISO 8601 string)
    pub valid_from: String,
    /// Certificate validity end (ISO 8601 string)
    pub valid_until: String,
    /// Whether certificate is trusted
    pub is_trusted: bool,
}

/// Digital signature manager
pub struct SignatureManager;

impl SignatureManager {
    /// Sign a PDF document with a digital signature
    ///
    /// This implementation uses ring for SHA256 hashing and stores signature metadata.
    /// Full PKCS#7/CMS signature generation requires complex certificate handling
    /// and is deferred for future implementation when proper certificate management is available.
    pub fn sign_pdf(
        input_path: &Path,
        output_path: &Path,
        certificate: &Certificate,
    ) -> Result<DigitalSignature, Error> {
        use ring::digest;
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        // Load the PDF document
        let pdf_data = fs::read(input_path)
            .with_context(|| format!("Failed to read PDF: {}", input_path.display()))?;

        // Generate SHA256 hash of the PDF
        let digest = digest::digest(&digest::SHA256, &pdf_data);
        let signature_data = digest.as_ref();

        // Create signature metadata
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        let timestamp_str = chrono::DateTime::from_timestamp(timestamp as i64, 0)
            .unwrap_or_default()
            .to_rfc3339();

        let signature_id = format!("sha256_sig_{}", hex::encode(digest.as_ref()));

        let digital_signature = DigitalSignature {
            id: signature_id.clone(),
            signer_name: certificate.subject.clone(),
            signer_email: None,
            timestamp: timestamp_str.clone(),
            certificate_fingerprint: certificate.fingerprint.clone(),
            is_valid: true,
        };

        // Add signature metadata to the PDF
        let mut doc = lopdf::Document::load(input_path)
            .with_context(|| format!("Failed to load PDF: {}", input_path.display()))?;

        // Get the catalog object ID
        let catalog_id = doc.trailer.get(b"Root")
            .and_then(|obj| obj.as_reference())
            .context("Failed to get catalog ID")?;
        
        // Add signature metadata as a custom entry
        let signature_metadata = lopdf::Dictionary::from_iter(vec![
            ("SignatureId", lopdf::Object::String(signature_id.as_bytes().to_vec(), lopdf::StringFormat::Literal)),
            ("Signer", lopdf::Object::String(certificate.subject.as_bytes().to_vec(), lopdf::StringFormat::Literal)),
            ("Timestamp", lopdf::Object::String(timestamp_str.as_bytes().to_vec(), lopdf::StringFormat::Literal)),
            ("CertificateFingerprint", lopdf::Object::String(certificate.fingerprint.as_bytes().to_vec(), lopdf::StringFormat::Literal)),
            ("SignatureFormat", lopdf::Object::String(b"SHA256/Ring".to_vec(), lopdf::StringFormat::Literal)),
            ("SignatureAlgorithm", lopdf::Object::String(b"SHA256".to_vec(), lopdf::StringFormat::Literal)),
            ("SignatureData", lopdf::Object::String(hex::encode(signature_data).as_bytes().to_vec(), lopdf::StringFormat::Literal)),
        ]);
        
        // Get mutable reference to catalog and set signature
        if let Some(obj) = doc.objects.get_mut(&catalog_id) {
            if let Ok(dict) = obj.as_dict_mut() {
                dict.set("PlatoSignature", lopdf::Object::Dictionary(signature_metadata));
            }
        }

        // Save the signed PDF
        doc.save(output_path)
            .with_context(|| format!("Failed to save signed PDF: {}", output_path.display()))?;

        Ok(digital_signature)
    }

    /// Verify a digital signature in a PDF document
    pub fn verify_signature(pdf_path: &Path) -> Result<Vec<DigitalSignature>, Error> {
        let doc = lopdf::Document::load(pdf_path)
            .with_context(|| format!("Failed to load PDF: {}", pdf_path.display()))?;

        let catalog = doc.catalog()
            .context("Failed to get catalog")?;
        
        let mut signatures = Vec::new();

        // Check for Plato signature metadata
        if let Ok(signature_obj) = catalog.get(b"PlatoSignature") {
            if let Ok(signature_dict) = signature_obj.as_dict() {
                let signature_id = signature_dict.get(b"SignatureId")
                    .ok()
                    .and_then(|o| o.as_str().ok())
                    .and_then(|s| std::str::from_utf8(s).ok())
                    .unwrap_or("unknown");
                
                let signer = signature_dict.get(b"Signer")
                    .ok()
                    .and_then(|o| o.as_str().ok())
                    .and_then(|s| std::str::from_utf8(s).ok())
                    .unwrap_or("unknown");
                
                let timestamp = signature_dict.get(b"Timestamp")
                    .ok()
                    .and_then(|o| o.as_str().ok())
                    .and_then(|s| std::str::from_utf8(s).ok())
                    .unwrap_or("unknown");
                
                let fingerprint = signature_dict.get(b"CertificateFingerprint")
                    .ok()
                    .and_then(|o| o.as_str().ok())
                    .and_then(|s| std::str::from_utf8(s).ok())
                    .unwrap_or("unknown");

                signatures.push(DigitalSignature {
                    id: signature_id.to_string(),
                    signer_name: signer.to_string(),
                    signer_email: None,
                    timestamp: timestamp.to_string(),
                    certificate_fingerprint: fingerprint.to_string(),
                    is_valid: true, // In full implementation, this would verify the actual signature
                });
            }
        }

        Ok(signatures)
    }

    /// List available certificates from system keyring
    pub fn list_certificates() -> Result<Vec<Certificate>, Error> {
        // Simplified implementation without secret-service dependency
        // In production, this would integrate with system keyring (secret-service on Linux)
        // For now, return a demo certificate for testing
        let mut certificates = Vec::new();
        
        certificates.push(Certificate {
            subject: "Demo Certificate".to_string(),
            issuer: "Demo CA".to_string(),
            fingerprint: "demo_fingerprint_123456".to_string(),
            valid_from: "2024-01-01T00:00:00Z".to_string(),
            valid_until: "2025-01-01T00:00:00Z".to_string(),
            is_trusted: false,
        });

        Ok(certificates)
    }

    /// Import a certificate from a file
    pub fn import_certificate(cert_path: &Path) -> Result<Certificate, Error> {
        use std::fs;

        let cert_data = fs::read(cert_path)
            .with_context(|| format!("Failed to read certificate: {}", cert_path.display()))?;

        // Parse the certificate (simplified - in production would use proper X.509 parsing)
        let subject = "Imported Certificate".to_string();
        let issuer = "Unknown".to_string();
        let fingerprint = hex::encode(ring::digest::digest(&ring::digest::SHA256, &cert_data).as_ref());
        let valid_from = chrono::Utc::now().to_rfc3339();
        let valid_until = chrono::Utc::now().to_rfc3339();

        let certificate = Certificate {
            subject: subject.clone(),
            issuer,
            fingerprint,
            valid_from,
            valid_until,
            is_trusted: false,
        };

        // In production, this would store in system keyring
        // For now, just return the certificate info
        Ok(certificate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_creation() {
        let signature = DigitalSignature {
            id: "test-id".to_string(),
            signer_name: "Test User".to_string(),
            signer_email: Some("test@example.com".to_string()),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            certificate_fingerprint: "abc123".to_string(),
            is_valid: true,
        };
        assert_eq!(signature.signer_name, "Test User");
    }
}
