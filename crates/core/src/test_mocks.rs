//! Mock implementations for testing
//!
//! This module provides mock implementations of core traits for unit testing.
//! Using mocks allows tests to run without hardware dependencies.
//!
//! ## Available Mocks
//!
//! - **MockFramebuffer**: Simulates display for headless testing
//! - **MockFrontlight**: Simulates frontlight control
//! - **MockBattery**: Simulates battery status
//! - **MockLightSensor**: Simulates ambient light sensor
//! - **MockDocument**: Simulates document for testing document handlers
//!
//! ## Usage
//!
//! ```rust,ignore
//! use plato_core::test_mocks::MockFramebuffer;
//! use plato_core::framebuffer::Framebuffer;
//!
//! let mock = MockFramebuffer::new(1404, 1872);
//! let context = Context::with_framebuffer(Box::new(mock));
//! ```

use crate::battery::{Battery, BatteryError, Status};
use crate::color::Color;
use crate::device::{Device, FrontlightKind, Model, Orientation};
use crate::document::BoundedText;
use crate::document::{Document, Location, TocEntry};
use crate::framebuffer::{Framebuffer, Pixmap, UpdateMode};
use crate::frontlight::{Frontlight, LightLevels};
use crate::geom::{Boundary, Rectangle};
use crate::input::TouchProto;
use crate::lightsensor::LightSensor;
use crate::metadata::TextAlign;
use anyhow::Error;

/// Mock framebuffer for headless testing
pub struct MockFramebuffer {
    width: u32,
    height: u32,
    pixels: Vec<Color>,
    update_count: usize,
}

impl MockFramebuffer {
    /// Create a new mock framebuffer with given dimensions
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height) as usize;
        Self {
            width,
            height,
            pixels: vec![Color::Gray(255); size],
            update_count: 0,
        }
    }

    /// Get the number of update calls made
    pub fn update_count(&self) -> usize {
        self.update_count
    }

    /// Get pixel color at coordinates
    pub fn pixel(&self, x: u32, y: u32) -> Option<Color> {
        if x < self.width && y < self.height {
            Some(self.pixels[(y * self.width + x) as usize])
        } else {
            None
        }
    }
}

impl Framebuffer for MockFramebuffer {
    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x < self.width && y < self.height {
            self.pixels[(y * self.width + x) as usize] = color;
        }
    }

    fn set_blended_pixel(&mut self, x: u32, y: u32, color: Color, _alpha: f32) {
        self.set_pixel(x, y, color);
    }

    fn invert_region(&mut self, _rect: &Rectangle) {
        // No-op for mock
    }

    fn shift_region(&mut self, _rect: &Rectangle, _drift: u8) {
        // No-op for mock
    }

    fn update(&mut self, _rect: &Rectangle, _mode: UpdateMode) -> Result<u32, Error> {
        self.update_count += 1;
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

    fn set_monochrome(&mut self, _enable: bool) {
        // No-op for mock
    }

    fn set_dithered(&mut self, _enable: bool) {
        // No-op for mock
    }

    fn set_inverted(&mut self, _enable: bool) {
        // No-op for mock
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

/// Mock frontlight for testing
pub struct MockFrontlight {
    intensity: f32,
    warmth: f32,
}

impl MockFrontlight {
    /// Create a new mock frontlight
    pub fn new() -> Self {
        Self {
            intensity: 0.0,
            warmth: 0.0,
        }
    }

    /// Get current intensity
    pub fn intensity(&self) -> f32 {
        self.intensity
    }

    /// Get current warmth
    pub fn warmth(&self) -> f32 {
        self.warmth
    }
}

#[allow(clippy::derivable_impls)]
impl Default for MockFrontlight {
    fn default() -> Self {
        Self::new()
    }
}

impl Frontlight for MockFrontlight {
    fn set_intensity(&mut self, value: f32) {
        self.intensity = value.clamp(0.0, 100.0);
    }

    fn set_warmth(&mut self, value: f32) {
        self.warmth = value.clamp(0.0, 100.0);
    }

    fn levels(&self) -> LightLevels {
        LightLevels {
            intensity: self.intensity,
            warmth: self.warmth,
        }
    }
}

/// Mock battery for testing
pub struct MockBattery {
    capacities: Vec<f32>,
    statuses: Vec<Status>,
}

impl MockBattery {
    /// Create a new mock battery with given state
    pub fn new(capacity: f32, status: Status) -> Self {
        Self {
            capacities: vec![capacity],
            statuses: vec![status],
        }
    }

    /// Set battery state for next read
    pub fn set_state(&mut self, capacity: f32, status: Status) {
        self.capacities = vec![capacity];
        self.statuses = vec![status];
    }
}

impl Battery for MockBattery {
    fn capacity(&mut self) -> Result<Vec<f32>, BatteryError> {
        Ok(self.capacities.clone())
    }

    fn status(&mut self) -> Result<Vec<Status>, BatteryError> {
        Ok(self.statuses.clone())
    }
}

/// Mock device for testing
pub struct MockDevice {
    model: Model,
    proto: TouchProto,
    dims: (u32, u32),
    dpi: u16,
}

impl MockDevice {
    /// Create a new mock device with given model
    pub fn new(model: Model) -> Self {
        let (dims, dpi) = match model {
            Model::LibraColour => ((1264, 1680), 300),
            Model::ClaraColour => ((1072, 1448), 300),
            Model::ClaraBW => ((1072, 1448), 300),
            Model::Elipsa2E => ((1404, 1872), 227),
            Model::Clara2E => ((1072, 1448), 300),
            Model::Libra2 => ((1264, 1680), 300),
            Model::Sage => ((1440, 1920), 300),
            Model::Elipsa => ((1404, 1872), 227),
            Model::Nia => ((758, 1024), 212),
            Model::LibraH2O => ((1264, 1680), 300),
            Model::Forma32GB => ((1440, 1920), 300),
            Model::Forma => ((1440, 1920), 300),
            Model::ClaraHD => ((1072, 1448), 300),
            Model::AuraH2OEd2V2 => ((1080, 1440), 265),
            Model::AuraH2OEd2V1 => ((1080, 1440), 265),
            Model::AuraEd2V2 => ((758, 1024), 212),
            Model::AuraEd2V1 => ((758, 1024), 212),
            Model::AuraONELimEd => ((1404, 1872), 300),
            Model::AuraONE => ((1404, 1872), 300),
            Model::Touch2 => ((600, 800), 167),
            Model::GloHD => ((1072, 1448), 300),
            Model::AuraH2O => ((1080, 1440), 265),
            Model::Aura => ((758, 1024), 212),
            Model::AuraHD => ((1080, 1440), 265),
            Model::Mini => ((600, 800), 200),
            Model::Glo => ((758, 1024), 212),
            Model::TouchC => ((600, 800), 167),
            Model::TouchAB => ((600, 800), 167),
        };
        Self {
            model,
            proto: TouchProto::MultiB,
            dims,
            dpi,
        }
    }
}

impl Device for MockDevice {
    fn model(&self) -> Model {
        self.model
    }

    fn proto(&self) -> TouchProto {
        self.proto
    }

    fn dims(&self) -> (u32, u32) {
        self.dims
    }

    fn dpi(&self) -> u16 {
        self.dpi
    }

    fn color_samples(&self) -> usize {
        match self.model {
            Model::ClaraColour | Model::LibraColour => 3,
            _ => 1,
        }
    }

    fn frontlight_kind(&self) -> FrontlightKind {
        match self.model {
            Model::ClaraHD
            | Model::Forma
            | Model::Forma32GB
            | Model::LibraH2O
            | Model::Sage
            | Model::Libra2
            | Model::Clara2E
            | Model::Elipsa2E
            | Model::ClaraBW
            | Model::ClaraColour
            | Model::LibraColour => FrontlightKind::Premixed,
            Model::AuraONE | Model::AuraONELimEd | Model::AuraH2OEd2V1 | Model::AuraH2OEd2V2 => {
                FrontlightKind::Natural
            }
            _ => FrontlightKind::Standard,
        }
    }

    fn has_natural_light(&self) -> bool {
        self.frontlight_kind() != FrontlightKind::Standard
    }

    fn has_lightsensor(&self) -> bool {
        matches!(self.model, Model::AuraONE | Model::AuraONELimEd)
    }

    fn has_gyroscope(&self) -> bool {
        matches!(
            self.model,
            Model::Forma
                | Model::Forma32GB
                | Model::LibraH2O
                | Model::Elipsa
                | Model::Sage
                | Model::Libra2
                | Model::Elipsa2E
                | Model::LibraColour
        )
    }

    fn has_page_turn_buttons(&self) -> bool {
        matches!(
            self.model,
            Model::Forma
                | Model::Forma32GB
                | Model::LibraH2O
                | Model::Sage
                | Model::Libra2
                | Model::LibraColour
        )
    }

    fn has_power_cover(&self) -> bool {
        matches!(self.model, Model::Sage)
    }

    fn has_removable_storage(&self) -> bool {
        matches!(
            self.model,
            Model::AuraH2O
                | Model::Aura
                | Model::AuraHD
                | Model::Glo
                | Model::TouchAB
                | Model::TouchC
        )
    }

    fn should_invert_buttons(&self, rotation: i8) -> bool {
        let sr = self.startup_rotation();
        let (_, dir) = self.mirroring_scheme();

        rotation == (4 + sr - dir) % 4 || rotation == (4 + sr - 2 * dir) % 4
    }

    fn orientation(&self, rotation: i8) -> Orientation {
        if self.should_swap_axes(rotation) {
            Orientation::Portrait
        } else {
            Orientation::Landscape
        }
    }

    fn mark(&self) -> u8 {
        match self.model {
            Model::LibraColour => 13,
            Model::ClaraBW | Model::ClaraColour => 12,
            Model::Elipsa2E => 11,
            Model::Clara2E => 10,
            Model::Libra2 => 9,
            Model::Sage | Model::Elipsa => 8,
            Model::Nia
            | Model::LibraH2O
            | Model::Forma32GB
            | Model::Forma
            | Model::ClaraHD
            | Model::AuraH2OEd2V2
            | Model::AuraEd2V2 => 7,
            Model::AuraH2OEd2V1
            | Model::AuraEd2V1
            | Model::AuraONELimEd
            | Model::AuraONE
            | Model::Touch2
            | Model::GloHD => 6,
            Model::AuraH2O | Model::Aura => 5,
            Model::AuraHD | Model::Mini | Model::Glo | Model::TouchC => 4,
            Model::TouchAB => 3,
        }
    }

    fn should_mirror_axes(&self, rotation: i8) -> (bool, bool) {
        let (mxy, dir) = self.mirroring_scheme();
        let mx = (4 + (mxy + dir)) % 4;
        let my = (4 + (mxy - dir)) % 4;
        let mirror_x = mxy == rotation || mx == rotation;
        let mirror_y = mxy == rotation || my == rotation;
        (mirror_x, mirror_y)
    }

    fn mirroring_scheme(&self) -> (i8, i8) {
        match self.model {
            Model::AuraH2OEd2V1 | Model::LibraH2O | Model::Libra2 => (3, 1),
            Model::Sage => (0, 1),
            Model::AuraH2OEd2V2 => (0, -1),
            Model::Forma | Model::Forma32GB => (2, -1),
            _ => (2, 1),
        }
    }

    fn should_swap_axes(&self, rotation: i8) -> bool {
        rotation % 2 == self.swapping_scheme()
    }

    fn swapping_scheme(&self) -> i8 {
        match self.model {
            Model::LibraH2O => 0,
            _ => 1,
        }
    }

    fn startup_rotation(&self) -> i8 {
        match self.model {
            Model::LibraH2O => 0,
            Model::AuraH2OEd2V1
            | Model::Forma
            | Model::Forma32GB
            | Model::Sage
            | Model::Libra2
            | Model::Elipsa2E
            | Model::LibraColour => 1,
            _ => 3,
        }
    }

    fn to_canonical(&self, n: i8) -> i8 {
        let (_, dir) = self.mirroring_scheme();
        (4 + dir * (n - self.startup_rotation())) % 4
    }

    fn from_canonical(&self, n: i8) -> i8 {
        let (_, dir) = self.mirroring_scheme();
        (self.startup_rotation() + (4 + dir * n) % 4) % 4
    }

    fn transformed_rotation(&self, n: i8) -> i8 {
        match self.model {
            Model::AuraHD | Model::AuraH2O => n ^ 2,
            Model::AuraH2OEd2V2 | Model::Forma | Model::Forma32GB => (4 - n) % 4,
            _ => n,
        }
    }

    fn transformed_gyroscope_rotation(&self, n: i8) -> i8 {
        match self.model {
            Model::LibraH2O => n ^ 1,
            Model::Libra2 | Model::Sage | Model::Elipsa2E | Model::LibraColour => (6 - n) % 4,
            Model::Elipsa => (4 - n) % 4,
            _ => n,
        }
    }
}

/// Mock light sensor for testing
pub struct MockLightSensor {
    level: u16,
}

impl MockLightSensor {
    /// Create a new mock light sensor
    pub fn new(level: u16) -> Self {
        Self { level }
    }

    /// Set the light level
    pub fn set_level(&mut self, level: u16) {
        self.level = level;
    }
}

impl LightSensor for MockLightSensor {
    fn level(&mut self) -> Result<u16, Error> {
        Ok(self.level)
    }
}

/// Mock document for testing document handling
pub struct MockDocument {
    pages: usize,
    dims: (f32, f32),
    title: Option<String>,
    author: Option<String>,
}

impl MockDocument {
    /// Create a new mock document
    pub fn new(pages: usize, width: f32, height: f32) -> Self {
        Self {
            pages,
            dims: (width, height),
            title: None,
            author: None,
        }
    }

    /// Set document metadata
    pub fn with_metadata(mut self, title: &str, author: &str) -> Self {
        self.title = Some(title.to_string());
        self.author = Some(author.to_string());
        self
    }
}

impl Document for MockDocument {
    fn dims(&self, _index: usize) -> Option<(f32, f32)> {
        Some(self.dims)
    }

    fn pages_count(&self) -> usize {
        self.pages
    }

    fn toc(&mut self) -> Option<Vec<TocEntry>> {
        Some(Vec::new())
    }

    fn chapter<'a>(&mut self, _offset: usize, _toc: &'a [TocEntry]) -> Option<(&'a TocEntry, f32)> {
        None
    }

    fn chapter_relative<'a>(
        &mut self,
        _offset: usize,
        _dir: crate::geom::CycleDir,
        _toc: &'a [TocEntry],
    ) -> Option<&'a TocEntry> {
        None
    }

    fn words(&mut self, _loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        None
    }

    fn lines(&mut self, _loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        None
    }

    fn links(&mut self, _loc: Location) -> Option<(Vec<BoundedText>, usize)> {
        None
    }

    fn images(&mut self, _loc: Location) -> Option<(Vec<Boundary>, usize)> {
        None
    }

    fn pixmap(&mut self, _loc: Location, _scale: f32, _samples: usize) -> Option<(Pixmap, usize)> {
        None
    }

    fn layout(&mut self, _width: u32, _height: u32, _font_size: f32, _dpi: u16) {
        // No-op for mock
    }

    fn set_font_family(&mut self, _family_name: &str, _search_path: &str) {
        // No-op for mock
    }

    fn set_margin_width(&mut self, _width: i32) {
        // No-op for mock
    }

    fn set_text_align(&mut self, _text_align: TextAlign) {
        // No-op for mock
    }

    fn set_line_height(&mut self, _line_height: f32) {
        // No-op for mock
    }

    fn set_hyphen_penalty(&mut self, _hyphen_penalty: i32) {
        // No-op for mock
    }

    fn set_stretch_tolerance(&mut self, _stretch_tolerance: f32) {
        // No-op for mock
    }

    fn set_ignore_document_css(&mut self, _ignore: bool) {
        // No-op for mock
    }

    fn title(&self) -> Option<String> {
        self.title.clone()
    }

    fn author(&self) -> Option<String> {
        self.author.clone()
    }

    fn metadata(&self, _key: &str) -> Option<String> {
        None
    }

    fn is_reflowable(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::BLACK;

    #[test]
    fn test_mock_framebuffer() {
        let mut fb = MockFramebuffer::new(100, 100);
        fb.set_pixel(50, 50, BLACK);
        assert_eq!(fb.pixel(50, 50), Some(BLACK));
    }

    #[test]
    fn test_mock_frontlight() {
        let mut light = MockFrontlight::new();
        light.set_intensity(50.0);
        assert_eq!(light.intensity(), 50.0);
    }

    #[test]
    fn test_mock_battery() {
        let mut battery = MockBattery::new(80.0, Status::Discharging);
        let caps = battery.capacity().unwrap();
        assert_eq!(caps[0], 80.0);
    }

    #[test]
    fn test_mock_document() {
        let doc = MockDocument::new(100, 600.0, 800.0).with_metadata("Test Book", "Test Author");
        assert_eq!(doc.pages_count(), 100);
        assert_eq!(doc.title(), Some("Test Book".to_string()));
    }
}
