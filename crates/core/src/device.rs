use crate::input::TouchProto;
use lazy_static::lazy_static;
use std::env;
use std::fmt;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Model {
    LibraColour,
    ClaraColour,
    ClaraBW,
    Elipsa2E,
    Clara2E,
    Libra2,
    Sage,
    Elipsa,
    Nia,
    LibraH2O,
    Forma32GB,
    Forma,
    ClaraHD,
    AuraH2OEd2V2,
    AuraH2OEd2V1,
    AuraEd2V2,
    AuraEd2V1,
    AuraONELimEd,
    AuraONE,
    Touch2,
    GloHD,
    AuraH2O,
    Aura,
    AuraHD,
    Mini,
    Glo,
    TouchC,
    TouchAB,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Orientation {
    Portrait,
    Landscape,
}

/// Device trait for hardware abstraction.
/// Enables testability and hardware independence.
pub trait Device {
    fn model(&self) -> Model;
    fn proto(&self) -> TouchProto;
    fn dims(&self) -> (u32, u32);
    fn dpi(&self) -> u16;
    fn color_samples(&self) -> usize;
    fn frontlight_kind(&self) -> FrontlightKind;
    fn has_natural_light(&self) -> bool;
    fn has_lightsensor(&self) -> bool;
    fn has_gyroscope(&self) -> bool;
    fn has_page_turn_buttons(&self) -> bool;
    fn has_power_cover(&self) -> bool;
    fn has_removable_storage(&self) -> bool;
    fn should_invert_buttons(&self, rotation: i8) -> bool;
    fn orientation(&self, rotation: i8) -> Orientation;
    fn mark(&self) -> u8;
    fn should_mirror_axes(&self, rotation: i8) -> (bool, bool);
    fn mirroring_scheme(&self) -> (i8, i8);
    fn should_swap_axes(&self, rotation: i8) -> bool;
    fn swapping_scheme(&self) -> i8;
    fn startup_rotation(&self) -> i8;
    fn to_canonical(&self, n: i8) -> i8;
    fn canonical_to_device(&self, n: i8) -> i8;
    fn transformed_rotation(&self, n: i8) -> i8;
    fn transformed_gyroscope_rotation(&self, n: i8) -> i8;
}

impl fmt::Display for Model {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Model::LibraColour => write!(f, "Libra Colour"),
            Model::ClaraColour => write!(f, "Clara Colour"),
            Model::ClaraBW => write!(f, "Clara BW"),
            Model::Elipsa2E => write!(f, "Elipsa 2E"),
            Model::Clara2E => write!(f, "Clara 2E"),
            Model::Libra2 => write!(f, "Libra 2"),
            Model::Sage => write!(f, "Sage"),
            Model::Elipsa => write!(f, "Elipsa"),
            Model::Nia => write!(f, "Nia"),
            Model::LibraH2O => write!(f, "Libra H₂O"),
            Model::Forma32GB => write!(f, "Forma 32GB"),
            Model::Forma => write!(f, "Forma"),
            Model::ClaraHD => write!(f, "Clara HD"),
            Model::AuraH2OEd2V1 => write!(f, "Aura H₂O Edition 2 Version 1"),
            Model::AuraH2OEd2V2 => write!(f, "Aura H₂O Edition 2 Version 2"),
            Model::AuraEd2V1 => write!(f, "Aura Edition 2 Version 1"),
            Model::AuraEd2V2 => write!(f, "Aura Edition 2 Version 2"),
            Model::AuraONELimEd => write!(f, "Aura ONE Limited Edition"),
            Model::AuraONE => write!(f, "Aura ONE"),
            Model::Touch2 => write!(f, "Touch 2.0"),
            Model::GloHD => write!(f, "Glo HD"),
            Model::AuraH2O => write!(f, "Aura H₂O"),
            Model::Aura => write!(f, "Aura"),
            Model::AuraHD => write!(f, "Aura HD"),
            Model::Mini => write!(f, "Mini"),
            Model::Glo => write!(f, "Glo"),
            Model::TouchC => write!(f, "Touch C"),
            Model::TouchAB => write!(f, "Touch A/B"),
        }
    }
}

#[derive(Debug)]
pub struct KoboDevice {
    pub model: Model,
    pub proto: TouchProto,
    pub dims: (u32, u32),
    pub dpi: u16,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FrontlightKind {
    Standard,
    Natural,
    Premixed,
}

impl KoboDevice {
    pub fn new(product: &str, model_number: &str) -> KoboDevice {
        match product {
            "kraken" => Self::create_device(Model::Glo, TouchProto::Single, (758, 1024), 212),
            "pixie" => Self::create_device(Model::Mini, TouchProto::Single, (600, 800), 200),
            "dragon" => Self::create_device(Model::AuraHD, TouchProto::Single, (1080, 1440), 265),
            "phoenix" => Self::create_device(Model::Aura, TouchProto::MultiA, (758, 1024), 212),
            "dahlia" => Self::create_device(Model::AuraH2O, TouchProto::MultiA, (1080, 1440), 265),
            "alyssum" => Self::create_device(Model::GloHD, TouchProto::MultiA, (1072, 1448), 300),
            "pika" => Self::create_device(Model::Touch2, TouchProto::MultiA, (600, 800), 167),
            "daylight" => Self::create_device_with_model_check(
                model_number,
                "381",
                Model::AuraONELimEd,
                Model::AuraONE,
                TouchProto::MultiA,
                (1404, 1872),
                300,
            ),
            "star" => Self::create_device_with_model_check(
                model_number,
                "379",
                Model::AuraEd2V2,
                Model::AuraEd2V1,
                TouchProto::MultiA,
                (758, 1024),
                212,
            ),
            "snow" => Self::create_device_with_model_check(
                model_number,
                "378",
                Model::AuraH2OEd2V2,
                Model::AuraH2OEd2V1,
                TouchProto::MultiB,
                (1080, 1440),
                265,
            ),
            "nova" => Self::create_device(Model::ClaraHD, TouchProto::MultiB, (1072, 1448), 300),
            "frost" => Self::create_device_with_model_check(
                model_number,
                "380",
                Model::Forma32GB,
                Model::Forma,
                TouchProto::MultiB,
                (1440, 1920),
                300,
            ),
            "storm" => Self::create_device(Model::LibraH2O, TouchProto::MultiB, (1264, 1680), 300),
            "luna" => Self::create_device(Model::Nia, TouchProto::MultiA, (758, 1024), 212),
            "europa" => Self::create_device(Model::Elipsa, TouchProto::MultiC, (1404, 1872), 227),
            "cadmus" => Self::create_device(Model::Sage, TouchProto::MultiC, (1440, 1920), 300),
            "io" => Self::create_device(Model::Libra2, TouchProto::MultiC, (1264, 1680), 300),
            "goldfinch" => {
                Self::create_device(Model::Clara2E, TouchProto::MultiB, (1072, 1448), 300)
            }
            "condor" => Self::create_device(Model::Elipsa2E, TouchProto::MultiC, (1404, 1872), 227),
            "spaBW" | "spaBWTPV" => {
                Self::create_device(Model::ClaraBW, TouchProto::MultiB, (1072, 1448), 300)
            }
            "spaColour" => {
                Self::create_device(Model::ClaraColour, TouchProto::MultiB, (1072, 1448), 300)
            }
            "monza" => {
                Self::create_device(Model::LibraColour, TouchProto::MultiB, (1264, 1680), 300)
            }
            _ => Self::create_device_with_model_check(
                model_number,
                "320",
                Model::TouchC,
                Model::TouchAB,
                TouchProto::Single,
                (600, 800),
                167,
            ),
        }
    }

    fn create_device(model: Model, proto: TouchProto, dims: (u32, u32), dpi: u16) -> KoboDevice {
        KoboDevice {
            model,
            proto,
            dims,
            dpi,
        }
    }

    fn create_device_with_model_check(
        model_number: &str,
        check_value: &str,
        model_if_match: Model,
        model_if_not_match: Model,
        proto: TouchProto,
        dims: (u32, u32),
        dpi: u16,
    ) -> KoboDevice {
        let model = if model_number == check_value {
            model_if_match
        } else {
            model_if_not_match
        };
        KoboDevice {
            model,
            proto,
            dims,
            dpi,
        }
    }

    pub(crate) fn device_specs(model: Model) -> ((u32, u32), u16, TouchProto) {
        match model {
            Model::LibraColour => ((1264, 1680), 300, TouchProto::MultiB),
            Model::ClaraColour => ((1072, 1448), 300, TouchProto::MultiB),
            Model::ClaraBW => ((1072, 1448), 300, TouchProto::MultiB),
            Model::Elipsa2E => ((1404, 1872), 227, TouchProto::MultiC),
            Model::Clara2E => ((1072, 1448), 300, TouchProto::MultiB),
            Model::Libra2 => ((1264, 1680), 300, TouchProto::MultiC),
            Model::Sage => ((1440, 1920), 300, TouchProto::MultiC),
            Model::Elipsa => ((1404, 1872), 227, TouchProto::MultiC),
            Model::Nia => ((758, 1024), 212, TouchProto::MultiA),
            Model::LibraH2O => ((1264, 1680), 300, TouchProto::MultiB),
            Model::Forma32GB => ((1440, 1920), 300, TouchProto::MultiB),
            Model::Forma => ((1440, 1920), 300, TouchProto::MultiB),
            Model::ClaraHD => ((1072, 1448), 300, TouchProto::MultiB),
            Model::AuraH2OEd2V2 => ((1080, 1440), 265, TouchProto::MultiB),
            Model::AuraH2OEd2V1 => ((1080, 1440), 265, TouchProto::MultiB),
            Model::AuraEd2V2 => ((758, 1024), 212, TouchProto::MultiA),
            Model::AuraEd2V1 => ((758, 1024), 212, TouchProto::MultiA),
            Model::AuraONELimEd => ((1404, 1872), 300, TouchProto::MultiA),
            Model::AuraONE => ((1404, 1872), 300, TouchProto::MultiA),
            Model::Touch2 => ((600, 800), 167, TouchProto::MultiA),
            Model::GloHD => ((1072, 1448), 300, TouchProto::MultiA),
            Model::AuraH2O => ((1080, 1440), 265, TouchProto::MultiA),
            Model::Aura => ((758, 1024), 212, TouchProto::MultiA),
            Model::AuraHD => ((1080, 1440), 265, TouchProto::Single),
            Model::Mini => ((600, 800), 200, TouchProto::Single),
            Model::Glo => ((758, 1024), 212, TouchProto::Single),
            Model::TouchC => ((600, 800), 167, TouchProto::Single),
            Model::TouchAB => ((600, 800), 167, TouchProto::Single),
        }
    }

    #[inline]
    pub fn color_samples(&self) -> usize {
        match self.model {
            Model::ClaraColour | Model::LibraColour => 3,
            _ => 1,
        }
    }

    #[inline]
    pub fn frontlight_kind(&self) -> FrontlightKind {
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

    #[inline]
    pub fn has_natural_light(&self) -> bool {
        self.frontlight_kind() != FrontlightKind::Standard
    }

    #[inline]
    pub fn has_lightsensor(&self) -> bool {
        matches!(self.model, Model::AuraONE | Model::AuraONELimEd)
    }

    #[inline]
    pub fn has_gyroscope(&self) -> bool {
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

    #[inline]
    pub fn has_page_turn_buttons(&self) -> bool {
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

    #[inline]
    pub fn has_power_cover(&self) -> bool {
        matches!(self.model, Model::Sage)
    }

    #[inline]
    pub fn has_removable_storage(&self) -> bool {
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

    #[inline]
    pub fn should_invert_buttons(&self, rotation: i8) -> bool {
        let sr = self.startup_rotation();
        let (_, dir) = self.mirroring_scheme();

        rotation == (4 + sr - dir) % 4 || rotation == (4 + sr - 2 * dir) % 4
    }

    #[inline]
    pub fn orientation(&self, rotation: i8) -> Orientation {
        if self.should_swap_axes(rotation) {
            Orientation::Portrait
        } else {
            Orientation::Landscape
        }
    }

    #[inline]
    pub fn mark(&self) -> u8 {
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

    #[inline]
    pub fn should_mirror_axes(&self, rotation: i8) -> (bool, bool) {
        let (mxy, dir) = self.mirroring_scheme();
        let mx = (4 + (mxy + dir)) % 4;
        let my = (4 + (mxy - dir)) % 4;
        let mirror_x = mxy == rotation || mx == rotation;
        let mirror_y = mxy == rotation || my == rotation;
        (mirror_x, mirror_y)
    }

    // Returns the center and direction of the mirroring pattern.
    #[inline]
    pub fn mirroring_scheme(&self) -> (i8, i8) {
        match self.model {
            Model::AuraH2OEd2V1 | Model::LibraH2O | Model::Libra2 => (3, 1),
            Model::Sage => (0, 1),
            Model::AuraH2OEd2V2 => (0, -1),
            Model::Forma | Model::Forma32GB => (2, -1),
            _ => (2, 1),
        }
    }

    #[inline]
    pub fn should_swap_axes(&self, rotation: i8) -> bool {
        rotation % 2 == self.swapping_scheme()
    }

    #[inline]
    pub fn swapping_scheme(&self) -> i8 {
        match self.model {
            Model::LibraH2O => 0,
            _ => 1,
        }
    }

    // The written rotation that makes the screen be in portrait mode
    // with the Kobo logo at the bottom.
    #[inline]
    pub fn startup_rotation(&self) -> i8 {
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

    // Return a device independent rotation value given
    // the device dependent written rotation value *n*.
    pub fn to_canonical(&self, n: i8) -> i8 {
        let (_, dir) = self.mirroring_scheme();
        (4 + dir * (n - self.startup_rotation())) % 4
    }

    // Return a device dependent written rotation value given
    // the device independent rotation value *n*.
    pub fn canonical_to_device(&self, n: i8) -> i8 {
        let (_, dir) = self.mirroring_scheme();
        (self.startup_rotation() + (4 + dir * n) % 4) % 4
    }

    // Return a device dependent written rotation value given
    // the device dependent read rotation value *n*.
    pub fn transformed_rotation(&self, n: i8) -> i8 {
        match self.model {
            Model::AuraHD | Model::AuraH2O => n ^ 2,
            Model::AuraH2OEd2V2 | Model::Forma | Model::Forma32GB => (4 - n) % 4,
            _ => n,
        }
    }

    pub fn transformed_gyroscope_rotation(&self, n: i8) -> i8 {
        match self.model {
            Model::LibraH2O => n ^ 1,
            Model::Libra2 | Model::Sage | Model::Elipsa2E | Model::LibraColour => (6 - n) % 4,
            Model::Elipsa => (4 - n) % 4,
            _ => n,
        }
    }
}

// Device trait implementation for testing abstraction
impl Device for KoboDevice {
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
        self.color_samples()
    }

    fn frontlight_kind(&self) -> FrontlightKind {
        self.frontlight_kind()
    }

    fn has_natural_light(&self) -> bool {
        self.has_natural_light()
    }

    fn has_lightsensor(&self) -> bool {
        self.has_lightsensor()
    }

    fn has_gyroscope(&self) -> bool {
        self.has_gyroscope()
    }

    fn has_page_turn_buttons(&self) -> bool {
        self.has_page_turn_buttons()
    }

    fn has_power_cover(&self) -> bool {
        self.has_power_cover()
    }

    fn has_removable_storage(&self) -> bool {
        self.has_removable_storage()
    }

    fn should_invert_buttons(&self, rotation: i8) -> bool {
        self.should_invert_buttons(rotation)
    }

    fn orientation(&self, rotation: i8) -> Orientation {
        self.orientation(rotation)
    }

    fn mark(&self) -> u8 {
        self.mark()
    }

    fn should_mirror_axes(&self, rotation: i8) -> (bool, bool) {
        self.should_mirror_axes(rotation)
    }

    fn mirroring_scheme(&self) -> (i8, i8) {
        self.mirroring_scheme()
    }

    fn should_swap_axes(&self, rotation: i8) -> bool {
        self.should_swap_axes(rotation)
    }

    fn swapping_scheme(&self) -> i8 {
        self.swapping_scheme()
    }

    fn startup_rotation(&self) -> i8 {
        self.startup_rotation()
    }

    fn to_canonical(&self, n: i8) -> i8 {
        self.to_canonical(n)
    }

    fn canonical_to_device(&self, n: i8) -> i8 {
        self.canonical_to_device(n)
    }

    fn transformed_rotation(&self, n: i8) -> i8 {
        self.transformed_rotation(n)
    }

    fn transformed_gyroscope_rotation(&self, n: i8) -> i8 {
        self.transformed_gyroscope_rotation(n)
    }
}

lazy_static! {
    pub static ref CURRENT_DEVICE: KoboDevice = {
        let product = env::var("PRODUCT").unwrap_or_default();
        let model_number = env::var("MODEL_NUMBER").unwrap_or_default();

        KoboDevice::new(&product, &model_number)
    };
}
