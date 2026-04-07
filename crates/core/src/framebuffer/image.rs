use super::{Framebuffer, UpdateMode};
use crate::color::{Color, WHITE};
use crate::geom::{lerp, Rectangle};
use anyhow::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Pixmap {
    pub width: u32,
    pub height: u32,
    pub samples: usize,
    pub data: Vec<u8>,
    pub update_flag: bool,
}

impl Pixmap {
    #[inline]
    pub fn new(width: u32, height: u32, samples: usize) -> Pixmap {
        let len = samples * (width * height) as usize;
        let mut data = Vec::new();
        // TODO: Consider returning Result<Pixmap, Error> instead of panicking
        // Currently: panics on allocation failure (OOM on device)
        // Better: Use try_new() in callers where OOM is possible
        // Note: try_new() method already exists as fallback
        data.try_reserve_exact(len).unwrap_or_else(|_| {
            panic!(
                "Failed to allocate {} bytes for pixmap ({}x{}x{})",
                len, width, height, samples
            );
        });
        data.resize(len, WHITE.gray());
        Pixmap {
            width,
            height,
            samples,
            data,
            update_flag: false,
        }
    }

    #[inline]
    pub fn try_new(width: u32, height: u32, samples: usize) -> Option<Pixmap> {
        let mut data = Vec::new();
        let len = samples * (width * height) as usize;
        data.try_reserve_exact(len).ok()?;
        data.resize(len, WHITE.gray());
        Some(Pixmap {
            width,
            height,
            samples,
            data,
            update_flag: false,
        })
    }

    pub fn empty(width: u32, height: u32, samples: usize) -> Pixmap {
        Pixmap {
            width,
            height,
            samples,
            data: Vec::new(),
            update_flag: false,
        }
    }

    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    #[inline]
    pub fn data_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn from_png<P: AsRef<Path>>(path: P) -> Result<Pixmap, Error> {
        let file = File::open(path.as_ref())?;
        let decoder = png::Decoder::new(BufReader::new(file));
        let mut reader = decoder.read_info()?;
        let info = reader.info();
        let mut pixmap = Pixmap::new(info.width, info.height, info.color_type.samples());
        reader.next_frame(pixmap.data_mut())?;
        Ok(pixmap)
    }

    pub fn from_webp<P: AsRef<Path>>(path: P) -> Result<Pixmap, Error> {
        let img = image::open(path.as_ref())?;
        let rgba = img.to_rgba8();
        let (width, height) = (rgba.width(), rgba.height());
        let samples = 4;
        let mut pixmap = Pixmap::new(width, height, samples);
        for (i, pixel) in rgba.pixels().enumerate() {
            let addr = i * samples;
            pixmap.data[addr] = pixel[0];
            pixmap.data[addr + 1] = pixel[1];
            pixmap.data[addr + 2] = pixel[2];
            pixmap.data[addr + 3] = pixel[3];
        }
        Ok(pixmap)
    }

    pub fn from_image<P: AsRef<Path>>(path: P) -> Result<Pixmap, Error> {
        let ext = path
            .as_ref()
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase());

        match ext.as_deref() {
            Some("webp") => Self::from_webp(path),
            Some("png") => Self::from_png(path),
            _ => {
                let img = image::open(path.as_ref())?;
                Ok(Self::from_dynamic_image(&img))
            }
        }
    }

    pub fn from_dynamic_image(img: &image::DynamicImage) -> Pixmap {
        let rgba = img.to_rgba8();
        let (width, height) = (rgba.width(), rgba.height());
        let samples = 4;
        let mut pixmap = Pixmap::new(width, height, samples);
        for (i, pixel) in rgba.pixels().enumerate() {
            let addr = i * samples;
            pixmap.data[addr] = pixel[0];
            pixmap.data[addr + 1] = pixel[1];
            pixmap.data[addr + 2] = pixel[2];
            pixmap.data[addr + 3] = pixel[3];
        }
        pixmap
    }

    #[inline]
    pub fn get_pixel(&self, x: u32, y: u32) -> Color {
        if self.data.is_empty() {
            return WHITE;
        }
        let addr = self.samples * (y * self.width + x) as usize;
        if self.samples == 1 {
            Color::Gray(self.data[addr])
        } else {
            Color::from_rgb(&self.data[addr..addr + 3])
        }
    }
}

impl Framebuffer for Pixmap {
    #[inline]
    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }
        if self.data.is_empty() {
            return;
        }
        let addr = self.samples * (y * self.width + x) as usize;
        if self.samples == 1 {
            self.data[addr] = color.gray();
        } else {
            let rgb = color.rgb();
            self.data[addr..addr + 3].copy_from_slice(&rgb);
        }
    }

    #[inline]
    fn set_blended_pixel(&mut self, x: u32, y: u32, color: Color, alpha: f32) {
        if alpha >= 1.0 {
            self.set_pixel(x, y, color);
            return;
        }
        if x >= self.width || y >= self.height {
            return;
        }
        if self.data.is_empty() {
            return;
        }
        let addr = self.samples * (y * self.width + x) as usize;
        if self.samples == 1 {
            self.data[addr] = lerp(self.data[addr] as f32, color.gray() as f32, alpha) as u8;
        } else {
            let rgb = color.rgb();
            for (i, c) in self.data[addr..addr + 3].iter_mut().enumerate() {
                *c = lerp(*c as f32, rgb[i] as f32, alpha) as u8;
            }
        }
    }

    #[inline]
    fn invert_region(&mut self, rect: &Rectangle) {
        if self.data.is_empty() {
            return;
        }

        let height = rect.height() as i32;
        let width = rect.width() as i32;

        if self.samples == 1 {
            for y in 0..height {
                let row_offset = ((rect.min.y + y) * self.width as i32 + rect.min.x) as usize;
                for x in 0..width {
                    let addr = row_offset + x as usize;
                    self.data[addr] = (255u8).wrapping_sub(self.data[addr]);
                }
            }
        } else {
            for y in 0..height {
                let row_offset = ((rect.min.y + y) * self.width as i32 + rect.min.x) as usize
                    * self.samples as usize;
                for x in 0..width as usize {
                    let addr = row_offset + x * self.samples;
                    for c in self.data[addr..addr + self.samples].iter_mut() {
                        *c = (255u8).wrapping_sub(*c);
                    }
                }
            }
        }
    }

    fn update(&mut self, _rect: &Rectangle, _mode: UpdateMode) -> Result<u32, Error> {
        Ok(0)
    }

    fn wait(&self, _token: u32) -> Result<i32, Error> {
        Ok(0)
    }

    fn save(&self, _path: &str) -> Result<(), Error> {
        Ok(())
    }

    fn set_rotation(&mut self, _n: i8) -> Result<(u32, u32), Error> {
        Ok((self.width, self.height))
    }

    #[inline]
    fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    fn height(&self) -> u32 {
        self.height
    }
}
