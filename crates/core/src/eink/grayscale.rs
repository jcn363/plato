//! Grayscale conversion with dithering for e-ink displays
//!
//! Converts RGBA buffers to 16-level grayscale optimized for e-ink displays.

use anyhow::Result;

/// Dithering algorithms for grayscale conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DitheringMode {
    None,
    FloydSteinberg,
    Ordered,
}

/// Converts RGBA to 16-level grayscale with dithering
#[derive(Debug)]
pub struct GrayscaleConverter {
    mode: DitheringMode,
    gamma: f32,
}

impl GrayscaleConverter {
    pub fn new(mode: DitheringMode) -> Self {
        Self {
            mode,
            gamma: 2.2,
        }
    }

    pub fn with_gamma(mode: DitheringMode, gamma: f32) -> Result<Self> {
        if gamma <= 0.0 || gamma > 10.0 {
            anyhow::bail!("Gamma must be between 0.0 and 10.0, got {}", gamma);
        }
        Ok(Self { mode, gamma })
    }

    pub fn convert(&self, rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
        if rgba.len() != (width * height * 4) as usize {
            anyhow::bail!(
                "RGBA buffer length {} does not match expected {} for {}x{}",
                rgba.len(),
                width * height * 4,
                width,
                height
            );
        }

        match self.mode {
            DitheringMode::None => self.convert_no_dither(rgba, width, height),
            DitheringMode::FloydSteinberg => self.convert_floyd_steinberg(rgba, width, height),
            DitheringMode::Ordered => self.convert_ordered(rgba, width, height),
        }
    }

    fn convert_no_dither(&self, rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
        let mut grayscale = vec![0u8; (width * height) as usize];

        for i in 0..grayscale.len() {
            let r = rgba[i * 4] as f32;
            let g = rgba[i * 4 + 1] as f32;
            let b = rgba[i * 4 + 2] as f32;

            let luminance = 0.299 * r + 0.587 * g + 0.114 * b;
            let corrected = self.gamma_correct(luminance);
            grayscale[i] = self.quantize_16_level(corrected);
        }

        Ok(grayscale)
    }

    fn convert_floyd_steinberg(&self, rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
        let mut pixels: Vec<f32> = rgba
            .chunks_exact(4)
            .map(|c| {
                let r = c[0] as f32;
                let g = c[1] as f32;
                let b = c[2] as f32;
                0.299 * r + 0.587 * g + 0.114 * b
            })
            .collect();

        let mut grayscale = vec![0u8; pixels.len()];

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                let old_pixel = self.gamma_correct(pixels[idx]);
                let new_pixel = (self.quantize_16_level(old_pixel) as f32) * 255.0 / 15.0;
                let quant_error = old_pixel - new_pixel;

                grayscale[idx] = self.quantize_16_level(new_pixel);

                if x + 1 < width {
                    pixels[idx + 1] += quant_error * 7.0 / 16.0;
                }
                if y + 1 < height {
                    if x > 0 {
                        pixels[idx - width as usize + 1] += quant_error * 3.0 / 16.0;
                    }
                    pixels[idx + width as usize] += quant_error * 5.0 / 16.0;
                    if x + 1 < width {
                        pixels[idx + width as usize + 1] += quant_error * 1.0 / 16.0;
                    }
                }
            }
        }

        Ok(grayscale)
    }

    fn convert_ordered(&self, rgba: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
        let mut grayscale = vec![0u8; (width * height) as usize];
        let bayer_matrix = [[0, 8, 2, 10], [12, 4, 14, 6], [3, 11, 1, 9], [15, 7, 13, 5]];

        for y in 0..height {
            for x in 0..width {
                let idx = (y * width + x) as usize;
                let r = rgba[idx * 4] as f32;
                let g = rgba[idx * 4 + 1] as f32;
                let b = rgba[idx * 4 + 2] as f32;

                let luminance = 0.299 * r + 0.587 * g + 0.114 * b;
                let corrected = self.gamma_correct(luminance);

                let matrix_x = (x % 4) as usize;
                let matrix_y = (y % 4) as usize;
                let threshold = bayer_matrix[matrix_y][matrix_x] as f32 * 16.0 / 15.0;

                let adjusted = corrected + threshold;
                grayscale[idx] = self.quantize_16_level(adjusted);
            }
        }

        Ok(grayscale)
    }

    #[inline]
    fn gamma_correct(&self, value: f32) -> f32 {
        if value <= 0.0 {
            0.0
        } else {
            (value / 255.0).powf(1.0 / self.gamma) * 255.0
        }
    }

    #[inline]
    fn quantize_16_level(&self, value: f32) -> u8 {
        let clamped = value.max(0.0).min(255.0);
        ((clamped / 255.0) * 15.0).round() as u8
    }
}

impl Default for GrayscaleConverter {
    fn default() -> Self {
        Self::new(DitheringMode::FloydSteinberg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converter_creation() {
        let conv = GrayscaleConverter::new(DitheringMode::None);
        assert_eq!(conv.mode, DitheringMode::None);
    }

    #[test]
    fn test_gamma_validation() {
        let result = GrayscaleConverter::with_gamma(DitheringMode::None, 2.2);
        assert!(result.is_ok());

        let result = GrayscaleConverter::with_gamma(DitheringMode::None, -1.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_convert_no_dither() {
        let conv = GrayscaleConverter::new(DitheringMode::None);
        let rgba = vec![255u8, 128, 64, 255; 100];
        let result = conv.convert(&rgba, 10, 10);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 100);
    }

    #[test]
    fn test_convert_invalid_length() {
        let conv = GrayscaleConverter::new(DitheringMode::None);
        let rgba = vec![255u8; 50];
        let result = conv.convert(&rgba, 10, 10);
        assert!(result.is_err());
    }
}
