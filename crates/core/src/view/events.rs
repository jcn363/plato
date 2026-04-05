use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use crate::document::Location;
use crate::gesture::GestureEvent;
use crate::input::{DeviceEvent, FingerStatus};
use crate::metadata::{Margin, SortMethod};

use super::calculator::LineOrigin;
use super::entries::{EntryId, EntryKind, TextKind};
use super::identifiers::{PluginTriggerKind, SliderId, ViewId};
use super::key::KeyKind;
use crate::framebuffer::UpdateMode;
use crate::geom::{CycleDir, Rectangle};

pub type Bus = VecDeque<Event>;
pub type Hub = Sender<Event>;

#[derive(Debug, Clone)]
pub enum Event {
    Device(DeviceEvent),
    Gesture(GestureEvent),
    Keyboard(KeyboardEvent),
    Key(KeyKind),
    Open(Box<crate::metadata::Info>),
    OpenHtml(String, Option<String>),
    LoadPixmap(usize),
    Update(UpdateMode),
    RefreshBookPreview(PathBuf, Option<PathBuf>),
    Invalid(PathBuf),
    Notify(String),
    Page(CycleDir),
    ResultsPage(CycleDir),
    GoTo(usize),
    GoToLocation(Location),
    ResultsGoTo(usize),
    CropMargins(Box<Margin>),
    Chapter(CycleDir),
    SelectDirectory(PathBuf),
    ToggleSelectDirectory(PathBuf),
    NavigationBarResized(i32),
    Focus(Option<ViewId>),
    Select(EntryId),
    PropagateSelect(EntryId),
    EditLanguages,
    Define(String),
    Submit(ViewId, String),
    Slider(SliderId, f32, FingerStatus),
    ToggleNear(ViewId, Rectangle),
    ToggleInputHistoryMenu(ViewId, Rectangle),
    ToggleBookMenu(Rectangle, usize),
    ToggleBookSelection(usize),
    ToggleBookPreview(Rectangle, usize),
    TogglePresetMenu(Rectangle, usize),
    ToggleFontPicker,
    ToggleBatchMode,
    BatchSelect(usize),
    BatchDelete,
    BatchMove(PathBuf),
    CloudSyncStart,
    CloudSyncStatus(String),
    RefreshFonts,
    SubMenu(Rectangle, Vec<EntryKind>),
    ProcessLine(LineOrigin, String),
    History(CycleDir, bool),
    Toggle(ViewId),
    Show(ViewId),
    Close(ViewId),
    CloseSub(ViewId),
    Search(String),
    SearchResult(usize, Vec<crate::geom::Boundary>),
    FetcherAddDocument(u32, Box<crate::metadata::Info>),
    FetcherRemoveDocument(u32, PathBuf),
    FetcherSearch {
        id: u32,
        path: Option<PathBuf>,
        query: Option<String>,
        sort_by: Option<(SortMethod, bool)>,
    },
    CheckFetcher(u32),
    EndOfSearch,
    SearchReplace,
    Finished,
    ClockTick,
    BatteryTick,
    ToggleFrontlight,
    Load(PathBuf),
    LoadPreset(usize),
    Scroll(i32),
    Save,
    Guess,
    CheckBattery,
    SetWifi(bool),
    MightSuspend,
    ExternalStorageImport,
    ExternalStorageList,
    OpenCoverEditor(PathBuf),
    SaveCover(PathBuf, PathBuf),
    PluginTrigger(PluginTriggerKind),
    StartBackgroundSync,
    SyncComplete,
    WifiStateChanged(bool),
    PrepareSuspend,
    Suspend,
    Share,
    PrepareShare,
    Validate,
    Cancel,
    Reseed,
    Render(String),
    Back,
    Quit,
    WakeUp,
}

#[derive(Debug, Copy, Clone)]
pub enum KeyboardEvent {
    Append(char),
    Partial(char),
    Move {
        target: TextKind,
        dir: crate::geom::LinearDir,
    },
    Delete {
        target: TextKind,
        dir: crate::geom::LinearDir,
    },
    Submit,
}
