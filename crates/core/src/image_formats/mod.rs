//! Image handling module
//!
//! This module provides image loading and processing capabilities for various
//! image formats including standard formats and specialized formats like JPEG 2000 and JBIG2.
//!
//! ## Architecture
//!
//! The image module is organized by format and function:
//!
//! ### Format-Specific Modules
//! - **jp2**: JPEG 2000 (JP2) format support using justjp2
//!   - JPEG 2000 decoding and conversion
//!   - Support for JP2, JPX, and J2K file formats
//! - **jbig2**: JBIG2 format support using hayro-jbig2
//!   - JBIG2 decoding for PDF image compression
//!   - Support for bi-level image decompression

pub mod jbig2;
pub mod jp2;
