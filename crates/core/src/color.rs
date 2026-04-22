use crate::geom::lerp;
use serde::{Deserialize, Serialize};

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;
#[cfg(target_arch = "arm")]
use std::arch::arm::*;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Color {
    Gray(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    #[inline]
    pub fn gray(&self) -> u8 {
        match *self {
            Color::Gray(level) => level,
            Color::Rgb(red, green, blue) => {
                rgb_to_grayscale_scalar(red, green, blue)
            }
        }
    }

    #[inline]
    pub fn rgb(&self) -> [u8; 3] {
        match *self {
            Color::Gray(level) => [level; 3],
            Color::Rgb(red, green, blue) => [red, green, blue],
        }
    }

    #[inline]
    pub fn from_rgb(rgb: &[u8]) -> Color {
        Color::Rgb(rgb[0], rgb[1], rgb[2])
    }

    #[inline]
    pub fn apply<F>(&self, f: F) -> Color
    where
        F: Fn(u8) -> u8,
    {
        match *self {
            Color::Gray(level) => Color::Gray(f(level)),
            Color::Rgb(red, green, blue) => Color::Rgb(f(red), f(green), f(blue)),
        }
    }

    #[inline]
    pub fn lerp(&self, color: Color, alpha: f32) -> Color {
        match (*self, color) {
            (Color::Gray(l1), Color::Gray(l2)) => {
                Color::Gray(lerp(l1 as f32, l2 as f32, alpha) as u8)
            }
            (Color::Rgb(red, green, blue), Color::Gray(level)) => Color::Rgb(
                lerp(red as f32, level as f32, alpha) as u8,
                lerp(green as f32, level as f32, alpha) as u8,
                lerp(blue as f32, level as f32, alpha) as u8,
            ),
            (Color::Gray(level), Color::Rgb(red, green, blue)) => Color::Rgb(
                lerp(level as f32, red as f32, alpha) as u8,
                lerp(level as f32, green as f32, alpha) as u8,
                lerp(level as f32, blue as f32, alpha) as u8,
            ),
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => Color::Rgb(
                lerp(r1 as f32, r2 as f32, alpha) as u8,
                lerp(g1 as f32, g2 as f32, alpha) as u8,
                lerp(b1 as f32, b2 as f32, alpha) as u8,
            ),
        }
    }

    #[inline]
    pub fn invert(&mut self) {
        match self {
            Color::Gray(level) => *level = 255 - *level,
            Color::Rgb(red, green, blue) => {
                *red = 255 - *red;
                *green = 255 - *green;
                *blue = 255 - *blue;
            }
        }
    }

    #[inline]
    pub fn shift(&mut self, drift: u8) {
        match self {
            Color::Gray(level) => *level = level.saturating_sub(drift),
            Color::Rgb(red, green, blue) => {
                *red = red.saturating_sub(drift);
                *green = green.saturating_sub(drift);
                *blue = blue.saturating_sub(drift);
            }
        }
    }
}

/// Scalar RGB to grayscale conversion (fallback)
#[inline]
fn rgb_to_grayscale_scalar(red: u8, green: u8, blue: u8) -> u8 {
    (red as f32 * 0.2126 + green as f32 * 0.7152 + blue as f32 * 0.0722) as u8
}

/// SIMD-optimized RGB to grayscale conversion for ARM NEON
#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
#[inline]
unsafe fn rgb_to_grayscale_simd_neon(rgb_data: &[u8]) -> Vec<u8> {
    let len = rgb_data.len() / 3;
    let mut result = Vec::with_capacity(len);
    
    // Process 8 pixels (24 bytes) at a time
    let chunks = rgb_data.chunks_exact(24);
    let remainder = chunks.remainder();
    
    for chunk in chunks {
        #[cfg(target_arch = "aarch64")]
        {
            let rgb_pixels = std::mem::transmute::<[u8; 24], uint8x16x3_t>(*chunk.as_ptr());
            let r = vld1q_u8(rgb_pixels.as_ptr().add(0));
            let g = vld1q_u8(rgb_pixels.as_ptr().add(8));
            let b = vld1q_u8(rgb_pixels.as_ptr().add(16));
            
            // Convert to 16-bit for multiplication
            let r16 = vmovl_u8_vdupq_n_u8(r);
            let g16 = vmovl_u8_vdupq_n_u8(g);
            let b16 = vmovl_u8_vdupq_n_u8(b);
            
            // Fixed-point arithmetic: multiply by 65536 then divide
            // 0.2126 ≈ 13932/65536, 0.7152 ≈ 46869/65536, 0.0722 ≈ 4732/65536
            let gray = vmlaq_n_u16q(vmlsq_n_u16q(r16, 13932), 
                                   vmlaq_n_u16q(vmlsq_n_u16q(g16, 46869), 
                                                 vmlsq_n_u16q(b16, 4732)));
            
            // Extract low 8 bits and store
            let gray8 = vshrn_n_u16(gray, 8);
            vst1q_u8(result.as_mut_ptr().add(result.len()), gray8);
            result.set_len(result.len() + 16);
        }
        
        #[cfg(target_arch = "arm")]
        {
            let rgb_pixels = std::mem::transmute::<[u8; 24], uint8x8x3_t>(*chunk.as_ptr());
            let r = vld1_u8(rgb_pixels.as_ptr());
            let g = vld1_u8(rgb_pixels.as_ptr().add(8));
            let b = vld1_u8(rgb_pixels.as_ptr().add(16));
            
            // Convert to 16-bit for multiplication
            let r16 = vmovl_u8(r);
            let g16 = vmovl_u8(g);
            let b16 = vmovl_u8(b);
            
            // Fixed-point arithmetic
            let gray = vmlaq_n_u16(vmlsq_n_u16(r16, 13932), 
                                   vmlaq_n_u16(vmlsq_n_u16(g16, 46869), 
                                                 vmlsq_n_u16(b16, 4732)));
            
            // Extract low 8 bits and store
            let gray8 = vshrn_n_u16(gray, 8);
            vst1_u8(result.as_mut_ptr().add(result.len()), gray8);
            result.set_len(result.len() + 8);
        }
    }
    
    // Handle remaining pixels with scalar code
    for chunk in remainder.chunks(3) {
        if chunk.len() == 3 {
            result.push(rgb_to_grayscale_scalar(chunk[0], chunk[1], chunk[2]));
        }
    }
    
    result
}

/// Bulk RGB to grayscale conversion with SIMD acceleration
pub fn rgb_to_grayscale_bulk(rgb_data: &[u8]) -> Vec<u8> {
    #[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
    {
        if is_neon_available() {
            return unsafe { rgb_to_grayscale_simd_neon(rgb_data) };
        }
    }
    
    // Fallback to scalar implementation
    let len = rgb_data.len() / 3;
    let mut result = Vec::with_capacity(len);
    for chunk in rgb_data.chunks_exact(3) {
        if chunk.len() == 3 {
            result.push(rgb_to_grayscale_scalar(chunk[0], chunk[1], chunk[2]));
        }
    }
    result
}

/// Check if NEON instructions are available at runtime
#[cfg(any(target_arch = "aarch64", target_arch = "arm"))]
fn is_neon_available() -> bool {
    #[cfg(target_arch = "aarch64")]
    {
        std::arch::is_aarch64_feature_detected!("neon")
    }
    #[cfg(target_arch = "arm")]
    {
        std::arch::is_arm_feature_detected!("neon")
    }
}

macro_rules! gray {
    ($a:expr) => {
        $crate::color::Color::Gray($a)
    };
}

pub const GRAY00: Color = gray!(0x00);
pub const GRAY01: Color = gray!(0x11);
pub const GRAY02: Color = gray!(0x22);
pub const GRAY03: Color = gray!(0x33);
pub const GRAY04: Color = gray!(0x44);
pub const GRAY05: Color = gray!(0x55);
pub const GRAY06: Color = gray!(0x66);
pub const GRAY07: Color = gray!(0x77);
pub const GRAY08: Color = gray!(0x88);
pub const GRAY09: Color = gray!(0x99);
pub const GRAY10: Color = gray!(0xAA);
pub const GRAY11: Color = gray!(0xBB);
pub const GRAY12: Color = gray!(0xCC);
pub const GRAY13: Color = gray!(0xDD);
pub const GRAY14: Color = gray!(0xEE);
pub const GRAY15: Color = gray!(0xFF);
pub const GRAYF4: Color = gray!(244);
pub const GRAY5C: Color = gray!(92);

pub const BLACK: Color = GRAY00;
pub const WHITE: Color = GRAY15;

pub const TEXT_NORMAL: [Color; 3] = [WHITE, BLACK, GRAY05];
pub const TEXT_BUMP_SMALL: [Color; 3] = [GRAY13, BLACK, GRAY07];
pub const TEXT_BUMP_LARGE: [Color; 3] = [GRAY11, BLACK, BLACK];

pub const TEXT_INVERTED_SOFT: [Color; 3] = [GRAY05, WHITE, WHITE];
pub const TEXT_INVERTED_HARD: [Color; 3] = [BLACK, WHITE, GRAY09];

pub const SEPARATOR_NORMAL: Color = GRAY10;
pub const SEPARATOR_STRONG: Color = GRAY07;

pub const KEYBOARD_BG: Color = GRAY11;
pub const BATTERY_FILL: Color = GRAY12;
pub const READING_PROGRESS: Color = GRAY07;

pub const PROGRESS_FULL: Color = GRAY05;
pub const PROGRESS_EMPTY: Color = GRAY13;
pub const PROGRESS_VALUE: Color = GRAY06;

pub const DARK_BACKGROUND: Color = GRAY02;
pub const DARK_FOREGROUND: Color = GRAY13;
pub const DARK_TEXT_NORMAL: [Color; 3] = [GRAY13, GRAY02, GRAY08];
pub const DARK_TEXT_BUMP_SMALL: [Color; 3] = [GRAY09, GRAY13, GRAY07];
pub const DARK_TEXT_BUMP_LARGE: [Color; 3] = [GRAY09, GRAY13, GRAY08];
pub const DARK_TEXT_INVERTED_SOFT: [Color; 3] = [GRAY07, GRAY13, GRAY13];
pub const DARK_TEXT_INVERTED_HARD: [Color; 3] = [GRAY13, GRAY02, GRAY08];
pub const DARK_KEYBOARD_BG: Color = GRAY03;
pub const DARK_SEPARATOR: Color = GRAY05;
pub const DARK_SEPARATOR_STRONG: Color = GRAY02;
pub const DARK_READING_PROGRESS: Color = GRAY02;
pub const DARK_PROGRESS_FULL: Color = GRAY02;
pub const DARK_PROGRESS_EMPTY: Color = GRAY10;
pub const DARK_PROGRESS_VALUE: Color = GRAY04;
pub const DARK_BATTERY_FILL: Color = GRAY02;

pub const SEPIA_BACKGROUND: Color = GRAYF4;
pub const SEPIA_FOREGROUND: Color = GRAY5C;

// Highlight colors
pub const YELLOW: Color = Color::Rgb(255, 255, 0);
pub const GREEN: Color = Color::Rgb(0, 255, 0);
pub const BLUE: Color = Color::Rgb(0, 0, 255);
pub const RED: Color = Color::Rgb(255, 0, 0);
pub const ORANGE: Color = Color::Rgb(255, 165, 0);
pub const PURPLE: Color = Color::Rgb(128, 0, 128);

#[inline]
pub fn background(dark: bool) -> Color {
    if dark {
        DARK_BACKGROUND
    } else {
        WHITE
    }
}

#[inline]
pub fn foreground(dark: bool) -> Color {
    if dark {
        DARK_FOREGROUND
    } else {
        BLACK
    }
}

#[inline]
pub fn text_normal(dark: bool) -> [Color; 3] {
    if dark {
        DARK_TEXT_NORMAL
    } else {
        TEXT_NORMAL
    }
}

#[inline]
pub fn text_bump_small(dark: bool) -> [Color; 3] {
    if dark {
        DARK_TEXT_BUMP_SMALL
    } else {
        TEXT_BUMP_SMALL
    }
}

#[inline]
pub fn separator(dark: bool) -> Color {
    if dark {
        DARK_SEPARATOR
    } else {
        SEPARATOR_NORMAL
    }
}

#[inline]
pub fn keyboard_bg(dark: bool) -> Color {
    if dark {
        DARK_KEYBOARD_BG
    } else {
        KEYBOARD_BG
    }
}

#[inline]
pub fn text_inverted_hard(dark: bool) -> [Color; 3] {
    if dark {
        DARK_TEXT_INVERTED_HARD
    } else {
        TEXT_INVERTED_HARD
    }
}

#[inline]
pub fn text_inverted_soft(dark: bool) -> [Color; 3] {
    if dark {
        DARK_TEXT_INVERTED_SOFT
    } else {
        TEXT_INVERTED_SOFT
    }
}

#[inline]
pub fn text_bump_large(dark: bool) -> [Color; 3] {
    if dark {
        DARK_TEXT_BUMP_LARGE
    } else {
        TEXT_BUMP_LARGE
    }
}

#[inline]
pub fn separator_strong(dark: bool) -> Color {
    if dark {
        DARK_SEPARATOR_STRONG
    } else {
        SEPARATOR_STRONG
    }
}

#[inline]
pub fn reading_progress(dark: bool) -> Color {
    if dark {
        DARK_READING_PROGRESS
    } else {
        READING_PROGRESS
    }
}

#[inline]
pub fn progress_full(dark: bool) -> Color {
    if dark {
        DARK_PROGRESS_FULL
    } else {
        PROGRESS_FULL
    }
}

#[inline]
pub fn progress_empty(dark: bool) -> Color {
    if dark {
        DARK_PROGRESS_EMPTY
    } else {
        PROGRESS_EMPTY
    }
}

#[inline]
pub fn progress_value(dark: bool) -> Color {
    if dark {
        DARK_PROGRESS_VALUE
    } else {
        PROGRESS_VALUE
    }
}

#[inline]
pub fn battery_fill(dark: bool) -> Color {
    if dark {
        DARK_BATTERY_FILL
    } else {
        BATTERY_FILL
    }
}
