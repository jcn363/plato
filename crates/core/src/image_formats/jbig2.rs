//! JBIG2 image support
//!
//! This module provides JBIG2 format support using the `hayro-jbig2` crate.
//! JBIG2 (Joint Bi-level Image Experts Group) is an image compression standard
//! for bi-level images, commonly used in PDF documents for scanned pages.
//!
//! ## Features
//!
//! - Decode JBIG2 images
//! - Convert to standard image formats for rendering
//! - Support for both embedded and standalone JBIG2 files
//!
//! ## Dependencies
//!
//! - `hayro-jbig2` - Rust bindings for the JBIG2 decoder

use anyhow::{Context, Error};
use image::{DynamicImage, GrayImage, ImageBuffer, ImageDecoder};

/// Decode JBIG2 image data
///
/// # Arguments
/// * `data` - Raw JBIG2 encoded data
///
/// # Returns
/// A DynamicImage from the image crate
pub fn decode_jbig2(data: &[u8]) -> Result<image::DynamicImage, Error> {
    // Validate input
    if data.is_empty() {
        return Err(Error::msg("JBIG2 data is empty"));
    }

    // Parse JBIG2 image using hayro-jbig2
    let jbig2_image =
        hayro_jbig2::Image::new(data).with_context(|| "Failed to parse JBIG2 image data")?;

    // Get image dimensions
    let width = jbig2_image.width();
    let height = jbig2_image.height();

    // Validate dimensions
    if width == 0 || height == 0 {
        return Err(Error::msg("JBIG2 image has invalid dimensions"));
    }

    // Create buffer for decoded image data
    let mut buffer = vec![0u8; (width * height) as usize];

    // Decode the image data using the ImageDecoder trait
    jbig2_image
        .read_image(&mut buffer)
        .with_context(|| "Failed to decode JBIG2 image data")?;

    // Convert to GrayImage (JBIG2 is bi-level)
    let gray_image: GrayImage = ImageBuffer::from_raw(width, height, buffer)
        .with_context(|| "Failed to create GrayImage from JBIG2 data")?;

    Ok(DynamicImage::ImageLuma8(gray_image))
}

/// Check if data is JBIG2 format
///
/// # Arguments
/// * `data` - Raw image data
///
/// # Returns
/// True if the data appears to be JBIG2 format
pub fn is_jbig2(data: &[u8]) -> bool {
    // JBIG2 files typically start with specific magic bytes
    // This is a basic check - full validation would require more sophisticated detection
    data.len() > 4 && &data[0..4] == b"JB2\x00"
}
