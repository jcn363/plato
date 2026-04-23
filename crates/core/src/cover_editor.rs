use crate::settings::CoverEditorSettings;
use crate::validation::{validate_path, validate_range};
use anyhow::{format_err, Context, Error};
use image::{DynamicImage, GenericImageView, ImageFormat};
use std::io::Write;
use std::path::Path;

#[derive(Default)]
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
        // Validate path before attempting to load
        validate_path(&path, "cover image path")?;

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
    ) -> Result<DynamicImage, Error> {
        // Validate crop parameters
        let (img_width, img_height) = img.dimensions();

        validate_range(x, 0, img_width, "crop x")?;
        validate_range(y, 0, img_height, "crop y")?;

        if width == 0 {
            return Err(format_err!("crop width must be greater than 0"));
        }
        if height == 0 {
            return Err(format_err!("crop height must be greater than 0"));
        }

        if x + width > img_width {
            return Err(format_err!(
                "crop region extends beyond image width: x({}) + width({}) > img_width({})",
                x,
                width,
                img_width
            ));
        }
        if y + height > img_height {
            return Err(format_err!(
                "crop region extends beyond image height: y({}) + height({}) > img_height({})",
                y,
                height,
                img_height
            ));
        }

        Ok(img.crop_imm(x, y, width, height))
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
        // Validate path before attempting to save
        validate_path(&path, "cover save path")?;

        // Validate image dimensions
        let (width, height) = img.dimensions();
        if width == 0 || height == 0 {
            return Err(format_err!("cannot save cover with zero dimensions"));
        }

        // Validate settings
        self.settings.validate()?;

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

/// Extract the cover image from an EPUB file
///
/// Searches for cover images in the EPUB archive using common naming patterns
/// and case-insensitive matching. Returns the first valid cover image found.
pub fn extract_cover_from_epub<P: AsRef<Path>>(epub_path: P) -> Result<DynamicImage, Error> {
    let path = epub_path.as_ref();
    let file = std::fs::File::open(path)
        .with_context(|| format!("can't open EPUB file {}", path.display()))?;
    let mut archive = zip::ZipArchive::new(file)?;

    let names: Vec<String> = archive.file_names().map(|n: &str| n.to_string()).collect();

    // Helper for case-insensitive prefix check without allocation
    fn starts_with_case_insensitive(text: &str, prefix: &str) -> bool {
        text.len() >= prefix.len()
            && text
                .chars()
                .zip(prefix.chars())
                .all(|(a, b)| a.eq_ignore_ascii_case(&b))
    }

    // Helper for case-insensitive contains without allocation
    fn contains_case_insensitive(text: &str, pattern: &str) -> bool {
        if pattern.is_empty() {
            return true;
        }
        let pattern_lower: String = pattern.to_lowercase();
        text.to_lowercase().contains(&pattern_lower)
    }

    // Helper for case-insensitive suffix check without allocation
    fn ends_with_case_insensitive(text: &str, suffix: &str) -> bool {
        text.len() >= suffix.len()
            && text
                .chars()
                .rev()
                .zip(suffix.chars().rev())
                .all(|(a, b)| a.eq_ignore_ascii_case(&b))
    }

    for name in &names {
        if starts_with_case_insensitive(name, "cover.") {
            if let Ok(mut file) = archive.by_name(name) {
                let mut buffer = Vec::new();
                std::io::Read::read_to_end(&mut file, &mut buffer)?;
                return image::load_from_memory(&buffer)
                    .map_err(|e| format_err!("Failed to decode cover: {}", e));
            }
        }
    }

    for entry in &names {
        if (contains_case_insensitive(entry, "cover") || contains_case_insensitive(entry, "image"))
            && (ends_with_case_insensitive(entry, ".jpg")
                || ends_with_case_insensitive(entry, ".jpeg")
                || ends_with_case_insensitive(entry, ".png"))
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

/// Set a new cover image in an EPUB file
///
/// Replaces the existing cover image in the EPUB archive with the provided image.
/// The cover image is stored as a standard cover file in the EPUB.
pub fn set_cover_in_epub<P: AsRef<Path>>(epub_path: P, cover_path: P) -> Result<(), Error> {
    let epub_path = epub_path.as_ref();
    let cover_path = cover_path.as_ref();

    let cover_img = image::open(cover_path)?;
    let resized = cover_img.resize(600, 800, image::imageops::FilterType::Lanczos3);

    let mut buffer = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut buffer);
    resized.write_to(&mut cursor, ImageFormat::Jpeg)?;

    let file = std::fs::File::open(epub_path)
        .with_context(|| format!("can't open EPUB file {}", epub_path.display()))?;
    let mut archive = zip::ZipArchive::new(file)?;

    let temp_path = epub_path.with_extension("epub.bak");
    std::fs::copy(epub_path, &temp_path)?;

    let temp_file = std::fs::File::create(&temp_path)
        .with_context(|| format!("can't create temporary file {}", temp_path.display()))?;
    let mut new_archive = zip::ZipWriter::new(temp_file);

    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::DEFLATE);

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
