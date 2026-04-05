use crate::color::{BLACK, WHITE};
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, Pixmap, UpdateMode};
use crate::frontlight::LightLevels;
use crate::geom::{halves, Axis, CycleDir, DiagDir, Dir, LinearDir, Region};
use crate::geom::{BorderSpec, Boundary, CornerSpec, Point, Rectangle, Vec2};
use crate::gesture::GestureEvent;
use crate::helpers::AsciiExtension;
use crate::input::{ButtonCode, ButtonStatus, DeviceEvent, FingerStatus};
use crate::library::Library;
use crate::log_error;
use crate::log_warn;
use crate::metadata::{sort, BookQuery, Info, Metadata, SimpleStatus, SortMethod, TRASH_DIRNAME};
use crate::settings::{FirstColumn, Hook, LibraryMode, SecondColumn};
use crate::unit::{mm_to_px, scale_by_dpi};
use chrono::Local;
use fxhash::{FxHashMap, FxHashSet};
use rand_core::RngCore;
use serde_json::{json, Value as JsonValue};
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::mem;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;

use super::common::{
    locate, locate_by_id, rlocate, toggle_battery_menu, toggle_clock_menu, toggle_main_menu,
};
use super::filler::Filler;
use super::keyboard::Keyboard;
use super::menu::{Menu, MenuKind};
use super::menu_entry::MenuEntry;
use super::named_input::NamedInput;
use super::notification::Notification;
use super::search_bar::SearchBar;
use super::top_bar::TopBar;
use super::{
    AppCmd, Bus, EntryId, EntryKind, Event, Hub, Id, RenderData, RenderQueue, SliderId, View,
    ViewId, BIG_BAR_HEIGHT, ID_FEEDER, SMALL_BAR_HEIGHT, THICKNESS_MEDIUM,
};

use self::address_bar::AddressBar;
use self::book::Book;
use self::bottom_bar::BottomBar;
use self::directories_bar::DirectoriesBar;
use self::directory::Directory;
use self::library_label::LibraryLabel;
use self::navigation_bar::NavigationBar;
use self::shelf::Shelf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fetcher {
    pub path: String,
    pub full_path: PathBuf,
    pub process: Child,
    pub sort_method: Option<SortMethod>,
    pub first_column: Option<FirstColumn>,
    pub second_column: Option<SecondColumn>,
}

pub struct Home {
    id: Id,
    rect: Rectangle,
    children: Vec<Box<dyn View>>,
    background_fetchers: HashSet<u32, Fetcher>,
    query: Option<BookQuery>,
    current_directory: PathBuf,
    visible_books: Vec<Info>,
    sort_method: SortMethod,
    reverse_order: bool,
    batch_mode: bool,
    batch_selected: HashSet<usize>,
    shelf_index: usize,
    target_document: Option<PathBuf>,
    focus: Option<ViewId>,
    shelves: Vec<Shelf>,
}
