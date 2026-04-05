use crate::anyhow::Error;
use crate::settings::Settings;

pub fn save_settings(settings: &Settings) -> Result<(), Error> {
    let mut path = std::env::current_dir().unwrap_or_default();
    path.push(crate::settings::SETTINGS_PATH);
    crate::helpers::save_toml(settings, &path)
}
