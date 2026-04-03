use crate::settings::CoverEditorSettings;
use anyhow::{format_err, Error};
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Write;
use std::path::Path;

pub struct CoverEditor {
    settings: CoverEditorSettings,
}

impl CoverEditor {
    pub fn new(settings: &CoverEditorSettings) -> CoverEditor {
        CoverEditor {
            settings: settings.clone(),
        }
    }

    pub fn load_cover<P: AsRef<Path>>(&self, path: P) -> Result<DynamicImage, Error> {
        let img =
            image::open(path.as_ref()).map_err(|e| format_err!("Failed to open image: {}", e))?;
        Ok(img)
    }

    pub fn crop(
        &self,
        img: &DynamicImage,
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    ) -> DynamicImage {
        img.crop_imm(x, y, width, height)
    }

    pub fn resize(&self, img: &DynamicImage, width: u32, height: u32) -> DynamicImage {
        img.resize(width, height, image::imageops::FilterType::Lanczos3)
    }

    pub fn resize_to_cover(
        &self,
        img: &DynamicImage,
        target_width: u32,
        target_height: u32,
    ) -> DynamicImage {
        let (src_w, src_h) = img.dimensions();
        let target_ratio = target_width as f32 / target_height as f32;
        let src_ratio = src_w as f32 / src_h as f32;

        let (scale_w, scale_h) = if src_ratio > target_ratio {
            let scale = target_height as f32 / src_h as f32;
            ((src_w as f32 * scale) as u32, target_height)
        } else {
            let scale = target_width as f32 / src_w as f32;
            (target_width, (src_h as f32 * scale) as u32)
        };

        let scaled = img.resize(scale_w, scale_h, image::imageops::FilterType::Lanczos3);

        let x = (scale_w.saturating_sub(target_width)) / 2;
        let y = (scale_h.saturating_sub(target_height)) / 2;

        scaled.crop_imm(x, y, target_width, target_height)
    }

    pub fn rotate_90(&self, img: &DynamicImage) -> DynamicImage {
        img.rotate90()
    }

    pub fn rotate_180(&self, img: &DynamicImage) -> DynamicImage {
        img.rotate180()
    }

    pub fn rotate_270(&self, img: &DynamicImage) -> DynamicImage {
        img.rotate270()
    }

    pub fn adjust_brightness(&self, img: &DynamicImage, value: i32) -> DynamicImage {
        img.brighten(value)
    }

    pub fn adjust_contrast(&self, img: &DynamicImage, value: f32) -> DynamicImage {
        img.adjust_contrast(value)
    }

    pub fn grayscale(&self, img: &DynamicImage) -> DynamicImage {
        img.grayscale()
    }

    pub fn save_as_cover<P: AsRef<Path>>(&self, img: &DynamicImage, path: P) -> Result<(), Error> {
        let rgb_img = img.to_rgb8();
        let (width, height) = rgb_img.dimensions();

        let resized =
            if width != self.settings.default_width || height != self.settings.default_height {
                img.resize(
                    self.settings.default_width,
                    self.settings.default_height,
                    image::imageops::FilterType::Lanczos3,
                )
            } else {
                img.clone()
            };

        resized
            .save_with_format(path.as_ref(), ImageFormat::Jpeg)
            .map_err(|e| format_err!("Failed to save cover: {}", e))?;

        Ok(())
    }

    pub fn create_thumbnail(&self, img: &DynamicImage, size: u32) -> DynamicImage {
        img.thumbnail(size, size)
    }

    pub fn get_cover_dimensions(&self) -> (u32, u32) {
        (self.settings.default_width, self.settings.default_height)
    }
}

impl Default for CoverEditor {
    fn default() -> Self {
        CoverEditor {
            settings: CoverEditorSettings::default(),
        }
    }
}

pub fn extract_cover_from_epub<P: AsRef<Path>>(epub_path: P) -> Result<DynamicImage, Error> {
    let path = epub_path.as_ref();
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let _cover_patterns = [
        "cover.jpg",
        "cover.jpeg",
        "cover.png",
        "Cover.jpg",
        "Cover.jpeg",
    ];

    let names: Vec<String> = archive.file_names().map(|n| n.to_string()).collect();

    for name in &names {
        let name_lower = name.to_lowercase();
        if name_lower.starts_with("cover.") {
            if let Ok(mut file) = archive.by_name(name) {
                let mut buffer = Vec::new();
                std::io::Read::read_to_end(&mut file, &mut buffer)?;
                return image::load_from_memory(&buffer)
                    .map_err(|e| format_err!("Failed to decode cover: {}", e));
            }
        }
    }

    for entry in &names {
        let entry_lower = entry.to_lowercase();
        if (entry_lower.contains("cover") || entry_lower.contains("image"))
            && (entry_lower.ends_with(".jpg")
                || entry_lower.ends_with(".jpeg")
                || entry_lower.ends_with(".png"))
        {
            if let Ok(mut file) = archive.by_name(entry) {
                let mut buffer = Vec::new();
                std::io::Read::read_to_end(&mut file, &mut buffer)?;
                if let Ok(img) = image::load_from_memory(&buffer) {
                    return Ok(img);
                }
            }
        }
    }

    Err(format_err!("No cover image found in EPUB"))
}

pub fn set_cover_in_epub<P: AsRef<Path>>(epub_path: P, cover_path: P) -> Result<(), Error> {
    let epub_path = epub_path.as_ref();
    let cover_path = cover_path.as_ref();

    let cover_img = image::open(cover_path)?;
    let resized = cover_img.resize(600, 800, image::imageops::FilterType::Lanczos3);

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    resized.write_to(&mut cursor, ImageFormat::Jpeg)?;

    let file = std::fs::File::open(epub_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let temp_path = epub_path.with_extension("epub.bak");
    std::fs::copy(epub_path, &temp_path)?;

    let temp_file = std::fs::File::create(&temp_path)?;
    let mut new_archive = zip::ZipWriter::new(temp_file);

    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let name = entry.name().to_string();

        if name.to_lowercase().starts_with("cover.") {
            continue;
        }

        new_archive.start_file(&name, options)?;
        std::io::copy(&mut entry, &mut new_archive)?;
    }

    new_archive.start_file("cover.jpg", options)?;
    new_archive.write_all(&buffer)?;

    new_archive.finish()?;
    std::fs::rename(&temp_path, epub_path)?;

    Ok(())
}
