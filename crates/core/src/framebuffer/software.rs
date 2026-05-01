#! Software framebuffer for desktop/Linux systems
//!
//! This module provides a software framebuffer implementation that renders to
//! an in-memory buffer. It allows Plato to run on standard Linux desktop
//! systems without requiring a physical framebuffer device (/dev/fb0).
//!
//! The software framebuffer can be used for:
//! - Development and testing on desktop Linux
//! - Debugging and visual verification
//! - Creating screenshots for documentation
//! - Running Plato in headless CI environments
//!
//! ## Usage
//!
//! ```rust,ignore
//! use plato_core::framebuffer::SoftwareFramebuffer;
//! use plato_core::framebuffer::Framebuffer;
//!
//! let mut fb = Box::new(SoftwareFramebuffer::new(1404, 1872, None)?);
//! let rect = crate::geom::Rectangle::new(
//!     crate::geom::Point::new(0, 0),
//!     crate::geom::Point::new(1404, 1872),
//! );
//! fb.update(&rect, UpdateMode::Partial)?;
//! fb.save("/tmp/framebuffer.png")?;
//! ```

use std::path::Path;

use crate::color::Color;
use crate::geom::Rectangle;
use anyhow::{Context, Error, Result};
use image::{ImageBuffer, RgbImage};

/// Software framebuffer that renders to an in-memory buffer
///
/// This framebuffer stores all pixels in a Vec<Color> and provides methods
/// to manipulate and retrieve the pixel data. It can save the contents
/// as a PNG image for debugging purposes.
#[derive(Debug, Clone)]
pub struct SoftwareFramebuffer {
    width: u32,
    height: u32,
    pixels: Vec<Color>,
    update_count: usize,
    /// Optional path to save debug output
    debug_save_path: Option<String>,
}

impl SoftwareFramebuffer {
    /// Create a new software framebuffer with given dimensions
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `debug_save_path` - Optional path to save debug output (PNG format)
    ///
    /// # Returns
    ///
    /// New SoftwareFramebuffer instance
    pub fn new(width: u32, height: u32, debug_save_path: Option<String>) -> Result<Self> {
        let size = (width * height) as usize;
        Ok(Self {
            width,
            height,
            pixels: vec![Color::Gray(255); size], // Start with white background
            update_count: 0,
            debug_save_path,
        })
    }

    /// Create a new software framebuffer with given dimensions (no debug saving)
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    ///
    /// # Returns
    ///
    /// New SoftwareFramebuffer instance
    pub fn new_simple(width: u32, height: u32) -> Result<Self> {
        Self::new(width, height, None)
    }

    /// Get the number of update calls made
    pub fn update_count(&self) -> usize {
        self.update_count
    }

    /// Get pixel color at coordinates
    pub fn pixel(&self, x: u32, y: u32) -> Option<Color> {
        if x < self.width && y < self.height {
            Some(self.pixels[(y * self.width + x) as usize])
        } else {
            None
        }
    }

    /// Get a copy of all pixels
    pub fn get_pixels(&self) -> Vec<Color> {
        self.pixels.clone()
    }

    /// Save the framebuffer contents as a PNG image
    ///
    /// # Arguments
    ///
    /// * `path` - File path to save the image
    ///
    /// # Returns
    ///
    /// Result indicating success or failure
    pub fn save_as_png<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let mut imgbuf: RgbImage = ImageBuffer::new(self.width, self.height);

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let color = self.pixels[(y * self.width + x) as usize];
            let rgb = color.rgb();
            *pixel = image::Rgb([rgb[0], rgb[1], rgb[2]]);
        }

        imgbuf
            .save(path)
            .with_context(|| format!("failed to save framebuffer to {}", path.display()))
    }

    /// Internal method to save if debug path is configured
    fn maybe_save_debug(&self) -> Result<()> {
        if let Some(ref path) = self.debug_save_path {
            self.save_as_png(path)?;
        }
        Ok(())
    }
}

impl crate::framebuffer::Framebuffer for SoftwareFramebuffer {
    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize] = color;
        }
    }

    fn set_blended_pixel(&mut self, x: u32, y: u32, color: Color, alpha: f32) {
        if x < self.width && y < self.height {
            let current = self.pixels[(y * self.width + x) as usize];
            let blended = current.lerp(color, alpha);
            self.pixels[(y * self.width + x) as usize] = blended;
        }
    }

    fn invert_region(&mut self, rect: &Rectangle) {
        for y in rect.min.y..rect.max.y {
            for x in rect.min.x..rect.max.x {
                let ux = x as u32;
                let uy = y as u32;
                if ux < self.width && uy < self.height {
                    let idx = (uy * self.width + ux) as usize;
                    let mut color = self.pixels[idx];
                    color.invert();
                    self.pixels[idx] = color;
                }
            }
        }
        self.update_count += 1;
        let _ = self.maybe_save_debug();
    }

    fn shift_region(&mut self, rect: &Rectangle, drift: u8) {
        // Used for annotation highlighting effect
        for y in rect.min.y..rect.max.y {
            for x in rect.min.x..rect.max.x {
                let ux = x as u32;
                let uy = y as u32;
                if ux < self.width && uy < self.height {
                    let idx = (uy * self.width + ux) as usize;
                    let mut color = self.pixels[idx];
                    color.shift(drift);
                    self.pixels[idx] = color;
                }
            }
        }
        self.update_count += 1;
        let _ = self.maybe_save_debug();
    }

    fn update(
        &mut self,
        _rect: &Rectangle,
        _mode: crate::framebuffer::UpdateMode,
    ) -> Result<u32, Error> {
        self.update_count += 1;
        let _ = self.maybe_save_debug();
        Ok(0)
    }

    fn wait(&self, _token: u32) -> Result<i32, Error> {
        Ok(0)
    }

    fn save(&self, path: &str) -> Result<(), Error> {
        self.save_as_png(path)
            .with_context(|| format!("failed to save framebuffer to {}", path))
    }

    fn set_rotation(&mut self, _n: i8) -> Result<(u32, u32), Error> {
        // Software framebuffer doesn't support hardware rotation
        // Rotation would need to be handled at a higher level
        Ok((self.width, self.height))
    }

    fn set_monochrome(&mut self, _enable: bool) {
        // No-op for software framebuffer - color handling is done at pixel level
    }

    fn set_dithered(&mut self, _enable: bool) {
        // No-op for software framebuffer - dithering is handled at pixel level
    }

    fn set_inverted(&mut self, _enable: bool) {
        // No-op for software framebuffer - inversion is handled at pixel level
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::BLACK;
    use crate::framebuffer::{Framebuffer, UpdateMode};

    #[test]
    fn test_software_framebuffer_creation() {
        let fb = SoftwareFramebuffer::new_simple(100, 100).unwrap();
        assert_eq!(fb.width(), 100);
        assert_eq!(fb.height(), 100);
        assert_eq!(fb.update_count(), 0);
    }

    #[test]
    fn test_software_framebuffer_pixel() {
        let mut fb = SoftwareFramebuffer::new_simple(100, 100).unwrap();
        fb.set_pixel(50, 50, BLACK);
        assert_eq!(fb.pixel(50, 50), Some(BLACK));
    }

    #[test]
    fn test_software_framebuffer_update() {
        let mut fb = SoftwareFramebuffer::new_simple(100, 100).unwrap();
        let rect = crate::geom::Rectangle::new(
            crate::geom::Point::new(0, 0),
            crate::geom::Point::new(100, 100),
        );
        fb.update(&rect, UpdateMode::Partial).unwrap();
        assert_eq!(fb.update_count(), 1);
    }

    #[test]
    fn test_software_framebuffer_save() {
        let fb = SoftwareFramebuffer::new_simple(100, 100).unwrap();
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("test_framebuffer.png");
        let result = fb.save_as_png(path);
        assert!(result.is_ok());
    }
}
