use crate::battery::Battery;
use crate::device::CURRENT_DEVICE;
use crate::dictionary::{load_dictionary_from_file, Dictionary};
use crate::font::Fonts;
use crate::framebuffer::{Display, Framebuffer};
use crate::frontlight::Frontlight;
use crate::geom::Rectangle;
use crate::helpers::{load_json, IsHidden};
use crate::library::Library;
use crate::lightsensor::LightSensor;
use crate::log_error;
use crate::rtc::Rtc;
use crate::settings::Settings;
use crate::view::keyboard::Layout;
use crate::view::ViewId;
use bitflags::bitflags;
use chrono::Local;
use globset::Glob;
use rand_core::SeedableRng;
use rand_xoshiro::Xoroshiro128Plus;
use rustc_hash::FxHashMap;
use std::collections::{BTreeMap, VecDeque};
use std::path::Path;
use walkdir::WalkDir;

const KEYBOARD_LAYOUTS_DIRNAME: &str = "keyboard-layouts";
const DICTIONARIES_DIRNAME: &str = "dictionaries";
const INPUT_HISTORY_SIZE: usize = 32;

bitflags! {
    /// Device state flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct DeviceFlags: u8 {
        const PLUGGED = 0b0000_0001;
        const COVERED = 0b0000_0010;
        const SHARED = 0b0000_0100;
        const ONLINE = 0b0000_1000;
    }
}

pub struct Context {
    pub fb: Box<dyn Framebuffer>,
    pub rtc: Option<Rtc>,
    pub display: Display,
    pub settings: Settings,
    pub library: Library,
    pub fonts: Fonts,
    pub dictionaries: BTreeMap<String, Dictionary>,
    pub keyboard_layouts: BTreeMap<String, Layout>,
    pub input_history: FxHashMap<ViewId, VecDeque<String>>,
    pub frontlight: Box<dyn Frontlight>,
    pub battery: Box<dyn Battery>,
    pub lightsensor: Box<dyn LightSensor>,
    pub notification_index: u8,
    pub kb_rect: Rectangle,
    pub rng: Xoroshiro128Plus,
    pub flags: DeviceFlags,
}

impl Context {
    /// Creates a new Context with the given runtime dependencies.
    ///
    /// # Arguments
    /// * `fb` - Framebuffer for display rendering
    /// * `rtc` - Real-time clock (optional)
    /// * `library` - Book library instance
    /// * `settings` - Application settings
    /// * `fonts` - Font collection
    /// * `battery` - Battery status provider
    /// * `frontlight` - Frontlight controller
    /// * `lightsensor` - Ambient light sensor
    pub fn new(
        fb: Box<dyn Framebuffer>,
        rtc: Option<Rtc>,
        library: Library,
        settings: Settings,
        fonts: Fonts,
        battery: Box<dyn Battery>,
        frontlight: Box<dyn Frontlight>,
        lightsensor: Box<dyn LightSensor>,
    ) -> Context {
        let dims = fb.dims();
        let rotation = CURRENT_DEVICE.transformed_rotation(fb.rotation());
        let rng = Xoroshiro128Plus::seed_from_u64(Local::now().timestamp_subsec_nanos() as u64);
        Context {
            fb,
            rtc,
            display: Display { dims, rotation },
            library,
            settings,
            fonts,
            dictionaries: BTreeMap::new(),
            keyboard_layouts: BTreeMap::new(),
            input_history: FxHashMap::default(),
            battery,
            frontlight,
            lightsensor,
            notification_index: 0,
            kb_rect: Rectangle::default(),
            rng,
            flags: DeviceFlags::empty(),
        }
    }

    /// Imports books from configured import directories to all libraries.
    ///
    /// # Errors
    /// Logs errors for individual library import failures but continues processing.
    pub fn batch_import(&mut self) {
        self.library.import(&self.settings.import);
        let selected_library = self.settings.selected_library;
        for (index, library_settings) in self.settings.libraries.iter().enumerate() {
            if index == selected_library {
                continue;
            }
            if let Ok(mut library) = Library::new(&library_settings.path, library_settings.mode)
                .map_err(|e| log_error!("{:#?}", e))
            {
                library.import(&self.settings.import);
                library.flush();
            }
        }
    }

    /// Loads keyboard layouts from the KEYBOARD_LAYOUTS_DIRNAME directory.
    ///
    /// # Errors
    /// Logs errors for individual layout file failures but continues processing.
    pub fn load_keyboard_layouts(&mut self) {
        let glob = Glob::new("**/*.json")
            .expect("invalid glob pattern")
            .compile_matcher();
        for entry in WalkDir::new(Path::new(KEYBOARD_LAYOUTS_DIRNAME))
            .min_depth(1)
            .into_iter()
            .filter_entry(|e| !e.is_hidden())
        {
            if entry.is_err() {
                continue;
            }
            let entry = entry.expect("dir entry read failed");
            let path = entry.path();
            if !glob.is_match(path) {
                continue;
            }
            if let Ok(layout) = load_json::<Layout, _>(path)
                .map_err(|e| log_error!("Can't load {}: {:#?}.", path.display(), e))
            {
                self.keyboard_layouts.insert(layout.name.clone(), layout);
            }
        }
    }

    /// Loads dictionary index files from the DICTIONARIES_DIRNAME directory.
    ///
    /// # Errors
    /// Logs errors for individual dictionary load failures but continues processing.
    pub fn load_dictionaries(&mut self) {
        let glob = Glob::new("**/*.index")
            .expect("invalid glob pattern")
            .compile_matcher();
        for entry in WalkDir::new(Path::new(DICTIONARIES_DIRNAME))
            .min_depth(1)
            .into_iter()
            .filter_entry(|e| !e.is_hidden())
        {
            if entry.is_err() {
                continue;
            }
            let entry = entry.expect("dir entry read failed");
            if !glob.is_match(entry.path()) {
                continue;
            }
            let index_path = entry.path().to_path_buf();
            let mut content_path = index_path.clone();
            content_path.set_extension("dict.dz");
            if !content_path.exists() {
                content_path.set_extension("");
            }
            if let Ok(mut dict) = load_dictionary_from_file(&content_path, &index_path) {
                let name = dict.short_name().ok().unwrap_or_else(|| {
                    index_path
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default()
                });
                self.dictionaries.insert(name, dict);
            }
        }
    }

    /// Records text input for the given view for auto-complete history.
    ///
    /// Deduplicates consecutive identical inputs and limits history size.
    pub fn record_input(&mut self, text: &str, id: ViewId) {
        if text.is_empty() {
            return;
        }

        let history = self.input_history.entry(id).or_insert_with(VecDeque::new);

        if history.front().map(String::as_str) != Some(text) {
            history.push_front(text.to_string());
        }

        if history.len() > INPUT_HISTORY_SIZE {
            history.pop_back();
        }
    }

    /// Enables or disables the frontlight.
    ///
    /// When enabling, restores previously saved warmth and intensity levels.
    /// When disabling, saves current levels and turns off frontlight.
    pub fn set_frontlight(&mut self, enable: bool) {
        self.settings.frontlight = enable;

        if enable {
            let levels = self.settings.frontlight_levels;
            self.frontlight.set_warmth(levels.warmth);
            self.frontlight.set_intensity(levels.intensity);
            self.flags.insert(DeviceFlags::ONLINE);
        } else {
            self.settings.frontlight_levels = self.frontlight.levels();
            self.frontlight.set_intensity(0.0);
            self.frontlight.set_warmth(0.0);
            self.flags.remove(DeviceFlags::ONLINE);
        }
    }
}
