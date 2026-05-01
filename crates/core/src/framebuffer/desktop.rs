//! Desktop Framebuffer implementation using minifb
//!
//! This module provides a windowed framebuffer for Linux desktops (Wayland/X11).
//! It uses the minifb crate to create a window and display the pixel buffer.

use crate::color::Color;
use crate::geom::Rectangle;
use crate::framebuffer::{Framebuffer, UpdateMode};
use anyhow::{Result, Error, Context};
use minifb::{Window, WindowOptions, Key};
use image::{ImageBuffer, RgbImage};
use std::path::Path;

use crate::input::{DeviceEvent, ButtonCode, ButtonStatus, FingerStatus};
use crate::geom::Point;

/// Framebuffer that renders to a window on desktop Linux
pub struct DesktopFramebuffer {
    width: u32,
    height: u32,
    buffer: Vec<u32>,
    window: Window,
    title: String,
    debug_save_path: Option<String>,
    last_mouse_pos: Option<(f32, f32)>,
    last_mouse_buttons: (bool, bool, bool), // left, middle, right
}

impl DesktopFramebuffer {
    /// Create a new DesktopFramebuffer with given dimensions
    pub fn new(width: u32, height: u32, title: &str, debug_save_path: Option<String>) -> Result<Self> {
        let mut window = Window::new(
            title,
            width as usize,
            height as usize,
            WindowOptions::default(),
        ).map_err(|e| Error::msg(format!("failed to create window: {}", e)))?;

        // Limit to 60 fps
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        Ok(Self {
            width,
            height,
            buffer: vec![0xFFFFFFFF; (width * height) as usize],
            window,
            title: title.to_string(),
            debug_save_path,
            last_mouse_pos: None,
            last_mouse_buttons: (false, false, false),
        })
    }

    /// Poll and handle window events, returning a list of DeviceEvents
    pub fn handle_events(&mut self) -> Vec<DeviceEvent> {
        let mut events = Vec::new();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        // Handle mouse position and clicks
        if let Some(pos) = self.window.get_mouse_pos(minifb::MouseMode::Discard) {
            let current_pos = Point::new(pos.0 as i32, pos.1 as i32);
            let left_down = self.window.get_mouse_down(minifb::MouseButton::Left);
            
            if left_down {
                if !self.last_mouse_buttons.0 {
                    // Mouse down (Finger Down)
                    events.push(DeviceEvent::Finger {
                        id: 0,
                        time: now,
                        status: FingerStatus::Down,
                        position: current_pos,
                    });
                } else if self.last_mouse_pos != Some(pos) {
                    // Mouse move while down (Finger Motion)
                    events.push(DeviceEvent::Finger {
                        id: 0,
                        time: now,
                        status: FingerStatus::Motion,
                        position: current_pos,
                    });
                }
            } else if self.last_mouse_buttons.0 {
                // Mouse up (Finger Up)
                events.push(DeviceEvent::Finger {
                    id: 0,
                    time: now,
                    status: FingerStatus::Up,
                    position: current_pos,
                });
            }
            
            self.last_mouse_pos = Some(pos);
            self.last_mouse_buttons.0 = left_down;
        }

        // Handle keyboard events (mapping some keys to Kobo buttons)
        if self.window.is_key_pressed(Key::Escape, minifb::KeyRepeat::No) {
            events.push(DeviceEvent::Button {
                time: now,
                code: ButtonCode::Home,
                status: ButtonStatus::Pressed,
            });
        }
        
        if self.window.is_key_pressed(Key::Left, minifb::KeyRepeat::Yes) {
            events.push(DeviceEvent::Button {
                time: now,
                code: ButtonCode::Backward,
                status: ButtonStatus::Pressed,
            });
        }

        if self.window.is_key_pressed(Key::Right, minifb::KeyRepeat::Yes) {
            events.push(DeviceEvent::Button {
                time: now,
                code: ButtonCode::Forward,
                status: ButtonStatus::Pressed,
            });
        }

        events
    }
}

impl Framebuffer for DesktopFramebuffer {
    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            let rgb = color.rgb();
            // minifb expects 00RRGGBB format
            let u32_color = ((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | (rgb[2] as u32);
            self.buffer[(y * self.width + x) as usize] = u32_color;
        }
    }

    fn set_blended_pixel(&mut self, x: u32, y: u32, color: Color, alpha: f32) {
        if x < self.width && y < self.height {
            let idx = (y * self.width + x) as usize;
            let current_u32 = self.buffer[idx];
            let current = Color::Rgb(
                ((current_u32 >> 16) & 0xFF) as u8,
                ((current_u32 >> 8) & 0xFF) as u8,
                (current_u32 & 0xFF) as u8,
            );
            let blended = current.lerp(color, alpha);
            let rgb = blended.rgb();
            let blended_u32 = ((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | (rgb[2] as u32);
            self.buffer[idx] = blended_u32;
        }
    }

    fn invert_region(&mut self, rect: &Rectangle) {
        for y in rect.min.y..rect.max.y {
            for x in rect.min.x..rect.max.x {
                let ux = x as u32;
                let uy = y as u32;
                if ux < self.width && uy < self.height {
                    let idx = (uy * self.width + ux) as usize;
                    let current_u32 = self.buffer[idx];
                    let r = 255 - ((current_u32 >> 16) & 0xFF) as u8;
                    let g = 255 - ((current_u32 >> 8) & 0xFF) as u8;
                    let b = 255 - (current_u32 & 0xFF) as u8;
                    self.buffer[idx] = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
                }
            }
        }
        let _ = self.update_window();
    }

    fn shift_region(&mut self, rect: &Rectangle, drift: u8) {
        for y in rect.min.y..rect.max.y {
            for x in rect.min.x..rect.max.x {
                let ux = x as u32;
                let uy = y as u32;
                if ux < self.width && uy < self.height {
                    let idx = (uy * self.width + ux) as usize;
                    let current_u32 = self.buffer[idx];
                    let r = ((current_u32 >> 16) & 0xFF) as u8;
                    let g = ((current_u32 >> 8) & 0xFF) as u8;
                    let b = (current_u32 & 0xFF) as u8;
                    let mut color = Color::Rgb(r, g, b);
                    color.shift(drift);
                    let rgb = color.rgb();
                    self.buffer[idx] = ((rgb[0] as u32) << 16) | ((rgb[1] as u32) << 8) | (rgb[2] as u32);
                }
            }
        }
        let _ = self.update_window();
    }

    fn update(&mut self, _rect: &Rectangle, _mode: UpdateMode) -> Result<u32, Error> {
        self.update_window()?;
        Ok(0)
    }

    fn wait(&self, _token: u32) -> Result<i32, Error> {
        Ok(0)
    }

    fn is_active(&self) -> bool {
        self.window.is_open()
    }

    fn save(&self, path: &str) -> Result<(), Error> {
        let path = Path::new(path);
        let mut imgbuf: RgbImage = ImageBuffer::new(self.width, self.height);

        for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
            let u32_color = self.buffer[(y * self.width + x) as usize];
            let r = ((u32_color >> 16) & 0xFF) as u8;
            let g = ((u32_color >> 8) & 0xFF) as u8;
            let b = (u32_color & 0xFF) as u8;
            *pixel = image::Rgb([r, g, b]);
        }

        imgbuf
            .save(path)
            .with_context(|| format!("failed to save framebuffer to {}", path.display()))?;
        
        Ok(())
    }

    fn set_rotation(&mut self, _n: i8) -> Result<(u32, u32), Error> {
        Ok((self.width, self.height))
    }

    fn set_monochrome(&mut self, _enable: bool) {}
    fn set_dithered(&mut self, _enable: bool) {}
    fn set_inverted(&mut self, _enable: bool) {}

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}
