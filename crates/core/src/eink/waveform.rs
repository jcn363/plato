//! Waveform mode selection for e-ink displays
//!
//! Selects appropriate waveform modes based on content type and update type.

use anyhow::Result;

/// Waveform modes for e-ink refresh
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaveformMode {
    GC16,  // High quality, slow (grayscale 16-level)
    GL16,  // Grayscale, medium
    DU,    // Direct update, fast
    A2,    // Monochrome text, very fast
    AUTO,  // Let controller decide
}

impl WaveformMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GC16 => "GC16",
            Self::GL16 => "GL16",
            Self::DU => "DU",
            Self::A2 => "A2",
            Self::AUTO => "AUTO",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_uppercase().as_str() {
            "GC16" => Ok(Self::GC16),
            "GL16" => Ok(Self::GL16),
            "DU" => Ok(Self::DU),
            "A2" => Ok(Self::A2),
            "AUTO" => Ok(Self::AUTO),
            _ => anyhow::bail!("Invalid waveform mode: {}", s),
        }
    }
}

/// Content type for waveform selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Text,
    Image,
    Mixed,
    UI,
}

/// Update type for waveform selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateType {
    Full,
    Partial,
    Fast,
}

/// Selects appropriate waveform mode based on content and update type
pub fn select_waveform(content: ContentType, update: UpdateType) -> WaveformMode {
    match (content, update) {
        (ContentType::Text, UpdateType::Full) => WaveformMode::GC16,
        (ContentType::Text, UpdateType::Partial) => WaveformMode::A2,
        (ContentType::Text, UpdateType::Fast) => WaveformMode::A2,
        (ContentType::Image, UpdateType::Full) => WaveformMode::GC16,
        (ContentType::Image, UpdateType::Partial) => WaveformMode::GL16,
        (ContentType::Image, UpdateType::Fast) => WaveformMode::DU,
        (ContentType::Mixed, UpdateType::Full) => WaveformMode::GC16,
        (ContentType::Mixed, UpdateType::Partial) => WaveformMode::GL16,
        (ContentType::Mixed, UpdateType::Fast) => WaveformMode::DU,
        (ContentType::UI, UpdateType::Full) => WaveformMode::GC16,
        (ContentType::UI, UpdateType::Partial) => WaveformMode::A2,
        (ContentType::UI, UpdateType::Fast) => WaveformMode::A2,
    }
}

impl Default for WaveformMode {
    fn default() -> Self {
        Self::GC16
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_waveform_from_str() {
        assert_eq!(WaveformMode::from_str("GC16").unwrap(), WaveformMode::GC16);
        assert_eq!(WaveformMode::from_str("A2").unwrap(), WaveformMode::A2);
        assert!(WaveformMode::from_str("INVALID").is_err());
    }

    #[test]
    fn test_select_waveform_text() {
        assert_eq!(
            select_waveform(ContentType::Text, UpdateType::Full),
            WaveformMode::GC16
        );
        assert_eq!(
            select_waveform(ContentType::Text, UpdateType::Partial),
            WaveformMode::A2
        );
    }

    #[test]
    fn test_select_waveform_image() {
        assert_eq!(
            select_waveform(ContentType::Image, UpdateType::Full),
            WaveformMode::GC16
        );
        assert_eq!(
            select_waveform(ContentType::Image, UpdateType::Partial),
            WaveformMode::GL16
        );
    }

    #[test]
    fn test_select_waveform_ui() {
        assert_eq!(
            select_waveform(ContentType::UI, UpdateType::Fast),
            WaveformMode::A2
        );
    }
}
