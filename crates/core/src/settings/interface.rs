use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RotationLock {
    Landscape,
    Portrait,
    Current,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ButtonScheme {
    Natural,
    Inverted,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum UiFont {
    #[default]
    SansSerif,
    Serif,
}

impl std::fmt::Display for ButtonScheme {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(self, f)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntermKind {
    Suspend,
    PowerOff,
    Share,
}

impl IntermKind {
    pub fn text(&self) -> &str {
        match self {
            IntermKind::Suspend => "Sleeping",
            IntermKind::PowerOff => "Powered off",
            IntermKind::Share => "Shared",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Intermissions {
    pub suspend: PathBuf,
    pub power_off: PathBuf,
    pub share: PathBuf,
}

impl std::ops::Index<IntermKind> for Intermissions {
    type Output = PathBuf;

    fn index(&self, key: IntermKind) -> &Self::Output {
        match key {
            IntermKind::Suspend => &self.suspend,
            IntermKind::PowerOff => &self.power_off,
            IntermKind::Share => &self.share,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct HomeSettings {
    pub address_bar: bool,
    pub navigation_bar: bool,
    pub max_levels: usize,
    pub max_trash_size: u64,
}

impl Default for HomeSettings {
    fn default() -> Self {
        HomeSettings {
            address_bar: false,
            navigation_bar: true,
            max_levels: 3,
            max_trash_size: 32 * (1 << 20),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FirstColumn {
    TitleAndAuthor,
    FileName,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SecondColumn {
    Progress,
    Year,
}
