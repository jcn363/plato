/// PDF image wrapper.
/// PDFPurr provides image extraction functionality.

/// Image extracted from PDF.
pub struct Image {
    pub data: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: ImageFormat,
}

/// Image format.
#[derive(Debug, Clone, Copy)]
pub enum ImageFormat {
    Jpeg,
    Png,
    Tiff,
    Unknown,
}

impl Image {
    pub fn new(data: Vec<u8>, width: u32, height: u32, format: ImageFormat) -> Self {
        Image {
            data,
            width,
            height,
            format,
        }
    }
}
