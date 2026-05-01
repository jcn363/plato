//! Android framebuffer implementation
//!
//! This module provides the Android-specific framebuffer implementation using
//! ANativeWindow for software rendering. It converts Plato's Color buffer to
//! RGB565 format for Android display.

#![cfg(target_os = "android")]

use plato_core::color::Color;
use plato_core::framebuffer::Framebuffer;
use plato_core::framebuffer::UpdateMode;
use plato_core::geom::Rectangle;
use anyhow::{Context, Result};
use ndk::native_window::NativeWindow;

/// Android framebuffer implementation using ANativeWindow for software rendering
pub struct AndroidFramebuffer {
    window: NativeWindow,
    width: u32,
    height: u32,
    buffer: Vec<Color>,
    monochrome: bool,
    dithered: bool,
    inverted: bool,
}

impl AndroidFramebuffer {
    /// Create a new AndroidFramebuffer from an ANativeWindow
    pub fn new(window: NativeWindow) -> Result<Self> {
        let width = window.width() as u32;
        let height = window.height() as u32;

        // Allocate buffer for pixel data
        let buffer_size = (width * height) as usize;
        let buffer = vec![Color::Gray(0); buffer_size];

        Ok(Self {
            window,
            width,
            height,
            buffer,
            monochrome: false,
            dithered: false,
            inverted: false,
        })
    }

    /// Post the buffer to the ANativeWindow
    fn post_buffer(&self) -> Result<()> {
        // Lock the native window to get a buffer (None for full window)
        let mut buffer = self
            .window
            .lock(None)
            .with_context(|| "Failed to lock native window")?;

        // Convert our Color buffer to RGB565 format for Android
        let stride = buffer.stride() as usize;
        let bits = buffer.bits() as *mut u16;

        for y in 0..self.height {
            for x in 0..self.width {
                let index = (y * self.width + x) as usize;
                if index < self.buffer.len() {
                    let color = self.buffer[index];
                    let rgb = color.rgb();
                    // Convert RGB888 to RGB565
                    let r = (rgb[0] >> 3) as u16;
                    let g = (rgb[1] >> 2) as u16;
                    let b = (rgb[2] >> 3) as u16;
                    let pixel = (r << 11) | (g << 5) | b;

                    unsafe {
                        *bits.add(y as usize * stride + x as usize) = pixel;
                    }
                }
            }
        }

        // Unlock and post the buffer - ndk 0.9 uses buffer.unlock_and_post()
        // The buffer is automatically posted when dropped
        drop(buffer);

        Ok(())
    }
}

impl Framebuffer for AndroidFramebuffer {
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
        self.post_buffer()?;
        Ok(0) // Return token 0 as specified
    }

    fn wait(&self, _token: u32) -> Result<i32, anyhow::Error> {
        // No-op, return Ok(0)
        Ok(0)
    }

    fn save(&self, _path: &str) -> Result<(), anyhow::Error> {
        // PNG export not implemented for Android platform.
        // Android uses ANativeWindow for display, which doesn't provide direct
        // framebuffer readback suitable for PNG export.
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

pub struct AndroidBattery {}

impl AndroidBattery {
    pub fn new() -> Self {
        Self {}
    }
}

impl plato_core::battery::Battery for AndroidBattery {
    fn level(&self) -> u8 {
        // In a real implementation, use JNI to call BatteryManager.BATTERY_PROPERTY_CAPACITY
        100
    }
}

pub struct AndroidLightSensor {}

impl AndroidLightSensor {
    pub fn new() -> Self {
        Self {}
    }
}

impl plato_core::lightsensor::LightSensor for AndroidLightSensor {
    fn value(&self) -> u16 {
        // In a real implementation, use JNI to call SensorManager.TYPE_LIGHT
        0
    }
}
