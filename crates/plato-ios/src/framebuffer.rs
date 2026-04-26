//! iOS framebuffer implementation
//!
//! This module provides the iOS-specific framebuffer implementation using
//! Metal for hardware-accelerated rendering. It converts Plato's Color buffer
//! to a format suitable for Metal display.

#![cfg(feature = "ios")]
#![deny(warnings)]

use anyhow::{Context, Result};
use plato_core::color::Color;
use plato_core::framebuffer::Framebuffer;
use plato_core::framebuffer::UpdateMode;
use plato_core::geom::Rectangle;
use std::sync::{Arc, Mutex};

/// iOS framebuffer implementation using Metal for hardware-accelerated rendering
#[derive(Clone)]
pub struct IOSFramebuffer {
    width: u32,
    height: u32,
    buffer: Vec<Color>,
    rgba_cache: Vec<u8>,
    monochrome: bool,
    dithered: bool,
    inverted: bool,
    // Metal layer and texture would be managed by Swift bridge
    // For MVP, we use a software buffer that will be copied to Metal
}

impl IOSFramebuffer {
    /// Create a new IOSFramebuffer with the given dimensions
    pub fn new(width: u32, height: u32) -> Result<Self> {
        // Allocate buffer for pixel data
        let buffer_size = (width * height) as usize;
        let buffer = vec![Color::Gray(0); buffer_size];

        // Pre-allocate RGBA cache to avoid per-frame allocations
        let rgba_cache = Vec::with_capacity(width as usize * height as usize * 4);

        Ok(Self {
            width,
            height,
            buffer,
            rgba_cache,
            monochrome: false,
            dithered: false,
            inverted: false,
        })
    }

    /// Get a pointer to the buffer for Metal texture upload
    /// Returns (pointer, width, height) for Swift to use
    pub fn buffer_ptr(&self) -> (*const u8, u32, u32) {
        (self.buffer.as_ptr() as *const u8, self.width, self.height)
    }

    /// Get the buffer as RGBA8888 format for Metal
    /// This converts from Plato's Color format to RGBA8888
    pub fn rgba8888_buffer(&self) -> Vec<u8> {
        let mut rgba = Vec::with_capacity(self.buffer.len() * 4);
        for color in &self.buffer {
            let rgb = color.rgb();
            rgba.push(rgb[0]); // R
            rgba.push(rgb[1]); // G
            rgba.push(rgb[2]); // B
            rgba.push(255); // A (fully opaque)
        }
        rgba
    }

    /// Fill a caller-provided buffer with RGBA8888 data without allocation
    /// Writes directly into the provided slice
    pub fn fill_rgba_buffer(&self, out: &mut [u8]) {
        let mut i = 0;
        for color in &self.buffer {
            if i + 4 <= out.len() {
                let rgb = color.rgb();
                out[i] = rgb[0]; // R
                out[i + 1] = rgb[1]; // G
                out[i + 2] = rgb[2]; // B
                out[i + 3] = 255; // A (fully opaque)
                i += 4;
            }
        }
    }
}

impl Framebuffer for IOSFramebuffer {
    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            if index < self.buffer.len() {
                self.buffer[index] = color;
            }
        }
    }

    fn set_blended_pixel(&mut self, x: u32, y: u32, color: Color, alpha: f32) {
        if x < self.width && y < self.height {
            let index = (y * self.width + x) as usize;
            if index < self.buffer.len() {
                // Simple alpha blending: new_color = old_color * (1 - alpha) + color * alpha
                let old_color = self.buffer[index];
                let blended = old_color.lerp(color, alpha);
                self.buffer[index] = blended;
            }
        }
    }

    fn invert_region(&mut self, rect: &Rectangle) {
        for y in rect.min.y..rect.max.y {
            for x in rect.min.x..rect.max.x {
                if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
                    let index = (y as u32 * self.width + x as u32) as usize;
                    if index < self.buffer.len() {
                        let mut color = self.buffer[index];
                        color.invert();
                        self.buffer[index] = color;
                    }
                }
            }
        }
    }

    fn shift_region(&mut self, rect: &Rectangle, drift: u8) {
        for y in rect.min.y..rect.max.y {
            for x in rect.min.x..rect.max.x {
                if x >= 0 && y >= 0 && (x as u32) < self.width && (y as u32) < self.height {
                    let index = (y as u32 * self.width + x as u32) as usize;
                    if index < self.buffer.len() {
                        let mut color = self.buffer[index];
                        color.shift(drift);
                        self.buffer[index] = color;
                    }
                }
            }
        }
    }

    fn update(&mut self, _rect: &Rectangle, _mode: UpdateMode) -> Result<u32, anyhow::Error> {
        // The actual update is handled by the Swift bridge via Metal
        // This just marks that an update is needed
        Ok(0)
    }

    fn wait(&self, _token: u32) -> Result<i32, anyhow::Error> {
        // No-op, return Ok(0)
        Ok(0)
    }

    fn save(&self, _path: &str) -> Result<(), anyhow::Error> {
        // Write PNG via Pixmap or stub with Ok(())
        // For now, stub with Ok(())
        Ok(())
    }

    fn set_rotation(&mut self, _n: i8) -> Result<(u32, u32), anyhow::Error> {
        // No-op, return current dims
        Ok((self.width, self.height))
    }

    fn set_monochrome(&mut self, enable: bool) {
        self.monochrome = enable;
    }

    fn set_dithered(&mut self, enable: bool) {
        self.dithered = enable;
    }

    fn set_inverted(&mut self, enable: bool) {
        self.inverted = enable;
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn monochrome(&self) -> bool {
        self.monochrome
    }

    fn dithered(&self) -> bool {
        self.dithered
    }

    fn inverted(&self) -> bool {
        self.inverted
    }
}

/// Wrapper for accessing the global framebuffer without per-pixel locking
/// This provides direct mutable access to the global FRAMEBUFFER static
/// Rendering happens on the main thread, so no synchronization is needed
pub struct GlobalFramebuffer;

impl Framebuffer for GlobalFramebuffer {
    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        unsafe {
            if let Some(ref mut fb) = crate::get_framebuffer_mut() {
                fb.set_pixel(x, y, color);
            }
        }
    }

    fn set_blended_pixel(&mut self, x: u32, y: u32, color: Color, alpha: f32) {
        unsafe {
            if let Some(ref mut fb) = crate::get_framebuffer_mut() {
                fb.set_blended_pixel(x, y, color, alpha);
            }
        }
    }

    fn invert_region(&mut self, rect: &Rectangle) {
        unsafe {
            if let Some(ref mut fb) = crate::get_framebuffer_mut() {
                fb.invert_region(rect);
            }
        }
    }

    fn shift_region(&mut self, rect: &Rectangle, drift: u8) {
        unsafe {
            if let Some(ref mut fb) = crate::get_framebuffer_mut() {
                fb.shift_region(rect, drift);
            }
        }
    }

    fn update(&mut self, rect: &Rectangle, mode: UpdateMode) -> Result<u32, anyhow::Error> {
        unsafe {
            if let Some(ref mut fb) = crate::get_framebuffer_mut() {
                fb.update(rect, mode)
            } else {
                Ok(0)
            }
        }
    }

    fn wait(&self, token: u32) -> Result<i32, anyhow::Error> {
        unsafe {
            if let Some(fb) = crate::get_framebuffer() {
                fb.wait(token)
            } else {
                Ok(0)
            }
        }
    }

    fn save(&self, path: &str) -> Result<(), anyhow::Error> {
        unsafe {
            if let Some(fb) = crate::get_framebuffer() {
                fb.save(path)
            } else {
                Ok(())
            }
        }
    }

    fn set_rotation(&mut self, n: i8) -> Result<(u32, u32), anyhow::Error> {
        unsafe {
            if let Some(ref mut fb) = crate::get_framebuffer_mut() {
                fb.set_rotation(n)
            } else {
                Ok((0, 0))
            }
        }
    }

    fn set_monochrome(&mut self, enable: bool) {
        unsafe {
            if let Some(ref mut fb) = crate::get_framebuffer_mut() {
                fb.set_monochrome(enable);
            }
        }
    }

    fn set_dithered(&mut self, enable: bool) {
        unsafe {
            if let Some(ref mut fb) = crate::get_framebuffer_mut() {
                fb.set_dithered(enable);
            }
        }
    }

    fn set_inverted(&mut self, enable: bool) {
        unsafe {
            if let Some(ref mut fb) = crate::get_framebuffer_mut() {
                fb.set_inverted(enable);
            }
        }
    }

    fn width(&self) -> u32 {
        unsafe { crate::get_framebuffer().map(|fb| fb.width()).unwrap_or(0) }
    }

    fn height(&self) -> u32 {
        unsafe { crate::get_framebuffer().map(|fb| fb.height()).unwrap_or(0) }
    }

    fn monochrome(&self) -> bool {
        unsafe {
            crate::get_framebuffer()
                .map(|fb| fb.monochrome())
                .unwrap_or(false)
        }
    }

    fn dithered(&self) -> bool {
        unsafe {
            crate::get_framebuffer()
                .map(|fb| fb.dithered())
                .unwrap_or(false)
        }
    }

    fn inverted(&self) -> bool {
        unsafe {
            crate::get_framebuffer()
                .map(|fb| fb.inverted())
                .unwrap_or(false)
        }
    }
}
