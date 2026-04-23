use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct Margin {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

impl Margin {
    pub fn new(top: f32, right: f32, bottom: f32, left: f32) -> Margin {
        let top = top.max(0.0);
        let right = right.max(0.0);
        let bottom = bottom.max(0.0);
        let left = left.max(0.0);
        Margin {
            top,
            right,
            bottom,
            left,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PageScheme {
    Any,
    EvenOdd,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CroppingMargins {
    Any(Margin),
    EvenOdd([Margin; 2]),
}

impl CroppingMargins {
    pub fn margin(&self, index: usize) -> &Margin {
        match *self {
            CroppingMargins::Any(ref margin) => margin,
            CroppingMargins::EvenOdd(ref pair) => &pair[index % 2],
        }
    }

    pub fn margin_mut(&mut self, index: usize) -> &mut Margin {
        match *self {
            CroppingMargins::Any(ref mut margin) => margin,
            CroppingMargins::EvenOdd(ref mut pair) => &mut pair[index % 2],
        }
    }

    pub fn apply(&mut self, index: usize, scheme: PageScheme) {
        let margin = self.margin(index).clone();

        match scheme {
            PageScheme::Any => *self = CroppingMargins::Any(margin),
            PageScheme::EvenOdd => *self = CroppingMargins::EvenOdd([margin.clone(), margin]),
        }
    }

    pub fn is_split(&self) -> bool {
        !matches!(*self, CroppingMargins::Any(..))
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TextAlign {
    Justify,
    Left,
    Right,
    Center,
}

impl TextAlign {
    pub fn icon_name(&self) -> &str {
        match self {
            TextAlign::Justify => "align-justify",
            TextAlign::Left => "align-left",
            TextAlign::Right => "align-right",
            TextAlign::Center => "align-center",
        }
    }
}

impl std::fmt::Display for TextAlign {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum ZoomMode {
    FitToPage,
    FitToWidth,
    Fit(f32),
    Custom(f32),
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
pub enum ScrollMode {
    Screen,
    Page,
    Vertical,
}

impl PartialEq for ZoomMode {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ZoomMode::FitToPage, ZoomMode::FitToPage) => true,
            (ZoomMode::FitToWidth, ZoomMode::FitToWidth) => true,
            (ZoomMode::Fit(z1), ZoomMode::Fit(z2)) => (z1 - z2).abs() < f32::EPSILON,
            (ZoomMode::Custom(z1), ZoomMode::Custom(z2)) => (z1 - z2).abs() < f32::EPSILON,
            _ => false,
        }
    }
}

impl Eq for ZoomMode {}
