//! JPEG 2000 image support
//!
//! This module provides JPEG 2000 (JP2) format support using the `justjp2` crate.
//! JPEG 2000 is a compression standard and coding system for digital images.
//!
//! ## Features
//!
//! - Decode JPEG 2000 images
//! - Convert to standard image formats for rendering
//! - Support for both JP2 and J2K file formats with auto-detection
//!
//! ## Dependencies
//!
//! - `justjp2` - Pure Rust JPEG 2000 encoder and decoder

use anyhow::{Context, Error};
use image::{DynamicImage, GrayImage, ImageBuffer, RgbImage};
use std::path::Path;

/// Load a JPEG 2000 (JP2) image from a file path
///
/// # Arguments
/// * `path` - Path to the JPEG 2000 file
///
/// # Returns
/// A DynamicImage from the image crate
pub fn load_jp2<P: AsRef<Path>>(path: P) -> Result<image::DynamicImage, Error> {
    let path = path.as_ref();

    // Validate path
    if !path.exists() {
        return Err(Error::msg(format!(
            "JPEG 2000 file not found: {}",
            path.display()
        )));
    }

    // Read JPEG 2000 file data
    let data = std::fs::read(path).context("Failed to read JPEG 2000 file")?;

    // Decode JPEG 2000 image using justjp2 (auto-detects JP2 vs J2K)
    let jp2_image = justjp2::decode(&data)
        .with_context(|| "Failed to decode JPEG 2000 image")?;

    // Convert justjp2::Image to image::DynamicImage
    let width = jp2_image.width;
    let height = jp2_image.height;

    if jp2_image.components.is_empty() {
        return Err(Error::msg("JPEG 2000 image has no components"));
    }

    // Handle different color spaces
    match jp2_image.components.len() {
        1 => {
            // Grayscale
            let comp = &jp2_image.components[0];
            let buffer: Vec<u8> = comp.data.iter().map(|&v| v as u8).collect();
            let gray_image: GrayImage = ImageBuffer::from_raw(width, height, buffer)
                .with_context(|| "Failed to create GrayImage from JPEG 2000 data")?;
            Ok(DynamicImage::ImageLuma8(gray_image))
        }
        3 => {
            // RGB
            let r = &jp2_image.components[0];
            let g = &jp2_image.components[1];
            let b = &jp2_image.components[2];

            let buffer: Vec<u8> = r.data.iter()
                .zip(g.data.iter())
                .zip(b.data.iter())
                .flat_map(|((rv, gv), bv)| [*rv as u8, *gv as u8, *bv as u8])
                .collect();

            let rgb_image: RgbImage = ImageBuffer::from_raw(width, height, buffer)
                .with_context(|| "Failed to create RgbImage from JPEG 2000 data")?;
            Ok(DynamicImage::ImageRgb8(rgb_image))
        }
        4 => {
            // RGBA - convert to RGB for now
            let r = &jp2_image.components[0];
            let g = &jp2_image.components[1];
            let b = &jp2_image.components[2];

            let buffer: Vec<u8> = r.data.iter()
                .zip(g.data.iter())
                .zip(b.data.iter())
                .flat_map(|((rv, gv), bv)| [*rv as u8, *gv as u8, *bv as u8])
                .collect();

            let rgb_image: RgbImage = ImageBuffer::from_raw(width, height, buffer)
                .with_context(|| "Failed to create RgbImage from JPEG 2000 data")?;
            Ok(DynamicImage::ImageRgb8(rgb_image))
        }
        n => {
            Err(Error::msg(format!(
                "JPEG 2000 image has unsupported number of components: {}",
                n
            )))
        }
    }
}

/// Check if a file is a JPEG 2000 format
///
/// # Arguments
/// * `path` - Path to the file
///
/// # Returns
/// True if the file is a JPEG 2000 format
pub fn is_jp2<P: AsRef<Path>>(path: P) -> bool {
    let path = path.as_ref();

    // Check file extension
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    matches!(ext.as_deref(), Some("jp2") | Some("jpx") | Some("j2k"))
}
