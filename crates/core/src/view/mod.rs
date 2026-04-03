//! Views are organized as a tree. A view might receive / send events and render itself.
//!
//! The z-level of the n-th child of a view is less or equal to the z-level of its n+1-th child.
//!
//! Events travel from the root to the leaves, only the leaf views will handle the root events, but
//! any view can send events to its parent. From the events it receives from its children, a view
//! resends the ones it doesn't handle to its own parent. Hence an event sent from a child might
//! bubble up to the root. If it reaches the root without being captured by any view, then it will
//! be written to the main event channel and will be sent to every leaf in one of the next loop
//! iterations.

pub mod battery;
pub mod button;
pub mod calculator;
pub mod clock;
pub mod common;
pub mod cover_editor;
pub mod dialog;
pub mod dictionary;
pub mod epub_editor;
pub mod filler;
pub mod frontlight;
pub mod home;
pub mod icon;
pub mod image;
pub mod input_field;
pub mod intermission;
pub mod key;
pub mod keyboard;
pub mod label;
pub mod labeled_icon;
pub mod menu;
pub mod menu_entry;
pub mod named_input;
pub mod notification;
pub mod page_label;
pub mod pdf_manipulator;
pub mod preset;
pub mod presets_list;
pub mod reader;
pub mod rotation_values;
pub mod rounded_button;
pub mod search_bar;
pub mod search_replace;
pub mod settings;
pub mod sketch;
pub mod slider;
pub mod statistics;
pub mod top_bar;
pub mod touch_events;

use self::calculator::LineOrigin;
use self::key::KeyKind;
use crate::color::Color;
use crate::context::Context;
use crate::document::{Location, TextLocation};
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::{Boundary, CycleDir, LinearDir, Rectangle};
use crate::gesture::GestureEvent;
use crate::input::{DeviceEvent, FingerStatus};
use crate::log_error;
use crate::metadata::{
    Info, Margin, PageScheme, ScrollMode, SimpleStatus, SortMethod, TextAlign, ZoomMode,
};
use crate::settings::{ButtonScheme, FirstColumn, RotationLock, SecondColumn};
use downcast_rs::{impl_downcast, Downcast};
use fxhash::FxHashMap;
use std::collections::VecDeque;
use std::fmt::{self, Debug};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

#[macro_export]
macro_rules! impl_view_boilerplate {
    () => {
        fn rect(&self) -> &Rectangle {
            &self.rect
        }

        fn rect_mut(&mut self) -> &mut Rectangle {
            &mut self.rect
        }

        fn children(&self) -> &Vec<Box<dyn View>> {
            &self.children
        }

        fn children_mut(&mut self) -> &mut Vec<Box<dyn View>> {
            &mut self.children
        }

        fn id(&self) -> Id {
            self.id
        }
    };
}

// Border thicknesses in pixels, at 300 DPI.
pub const THICKNESS_SMALL: f32 = 1.5;
pub const THICKNESS_MEDIUM: f32 = 2.0;
pub const THICKNESS_LARGE: f32 = 3.0;

// Border radii in pixels, at 300 DPI.
pub const BORDER_RADIUS_SMALL: f32 = 6.0;
pub const BORDER_RADIUS_MEDIUM: f32 = 9.0;
pub const BORDER_RADIUS_LARGE: f32 = 12.0;

// Big and small bar heights in pixels, at 300 DPI.
// On the *Aura ONE*, the height is exactly `2 * sb + 10 * bb`.
pub const SMALL_BAR_HEIGHT: f32 = 121.0;
pub const BIG_BAR_HEIGHT: f32 = 163.0;

pub const CLOSE_IGNITION_DELAY: Duration = Duration::from_millis(150);

pub type Bus = VecDeque<Event>;
pub type Hub = Sender<Event>;

pub trait View: Downcast {
    fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool;
    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, fonts: &mut Fonts);
    fn rect(&self) -> &Rectangle;
    fn rect_mut(&mut self) -> &mut Rectangle;
    fn children(&self) -> &Vec<Box<dyn View>>;
    fn children_mut(&mut self) -> &mut Vec<Box<dyn View>>;
    fn id(&self) -> Id;

    fn render_rect(&self, _rect: &Rectangle) -> Rectangle {
        *self.rect()
    }

    fn resize(
        &mut self,
        rect: Rectangle,
        _hub: &Hub,
        _rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        *self.rect_mut() = rect;
    }

    fn child(&self, index: usize) -> &dyn View {
        self.children()[index].as_ref()
    }

    fn child_mut(&mut self, index: usize) -> &mut dyn View {
        self.children_mut()[index].as_mut()
    }

    fn len(&self) -> usize {
        self.children().len()
    }

    fn might_skip(&self, _evt: &Event) -> bool {
        false
    }

    fn might_rotate(&self) -> bool {
        true
    }

    fn is_background(&self) -> bool {
        false
    }

    fn view_id(&self) -> Option<ViewId> {
        None
    }
}

impl_downcast!(View);

impl Debug for Box<dyn View> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Box<dyn View>")
    }
}

// We start delivering events from the highest z-level to prevent views from capturing
// gestures that occurred in higher views.
// The consistency must also be ensured by the views: popups, for example, need to
// capture any tap gesture with a touch point inside their rectangle.
// A child can send events to the main channel through the *hub* or communicate with its parent through the *bus*.
// A view that wants to render can write to the rendering queue.
pub fn handle_event(
    view: &mut dyn View,
    evt: &Event,
    hub: &Hub,
    parent_bus: &mut Bus,
    rq: &mut RenderQueue,
    context: &mut Context,
) -> bool {
    if view.len() > 0 {
        let mut captured = false;

        if view.might_skip(evt) {
            return captured;
        }

        let mut child_bus: Bus = VecDeque::with_capacity(1);

        for i in (0..view.len()).rev() {
            if handle_event(view.child_mut(i), evt, hub, &mut child_bus, rq, context) {
                captured = true;
                break;
            }
        }

        let mut temp_bus: Bus = VecDeque::with_capacity(1);

        child_bus
            .retain(|child_evt| !view.handle_event(child_evt, hub, &mut temp_bus, rq, context));

        parent_bus.append(&mut child_bus);
        parent_bus.append(&mut temp_bus);

        captured || view.handle_event(evt, hub, parent_bus, rq, context)
    } else {
        view.handle_event(evt, hub, parent_bus, rq, context)
    }
}

// We render from bottom to top. For a view to render it has to either appear in `ids` or intersect
// one of the rectangles in `bgs`. When we're about to render a view, if `wait` is true, we'll wait
// for all the updates in `updating` that intersect with the view.
pub fn render(
    view: &dyn View,
    wait: bool,
    ids: &FxHashMap<Id, Vec<Rectangle>>,
    rects: &mut Vec<Rectangle>,
    bgs: &mut Vec<Rectangle>,
    fb: &mut dyn Framebuffer,
    fonts: &mut Fonts,
    updating: &mut Vec<UpdateData>,
) {
    let mut render_rects = Vec::new();

    if view.len() == 0 || view.is_background() {
        for rect in ids
            .get(&view.id())
            .cloned()
            .into_iter()
            .flatten()
            .chain(rects.iter().filter_map(|r| r.intersection(view.rect())))
            .chain(bgs.iter().filter_map(|r| r.intersection(view.rect())))
        {
            let render_rect = view.render_rect(&rect);

            if wait {
                updating.retain(|update| {
                    let overlaps = render_rect.overlaps(&update.rect);
                    if overlaps && !update.has_completed() {
                        fb.wait(update.token)
                            .map_err(|e| {
                                log_error!(
                                    "Can't wait for {}, {}: {:#}",
                                    update.token,
                                    update.rect,
                                    e
                                )
                            })
                            .ok();
                    }
                    !overlaps
                });
            }

            view.render(fb, rect, fonts);
            render_rects.push(render_rect);

            // Most views can't render a subrectangle of themselves.
            if *view.rect() == render_rect {
                break;
            }
        }
    } else {
        bgs.extend(ids.get(&view.id()).cloned().into_iter().flatten());
    }

    // Merge the contiguous zones to avoid having to schedule lots of small frambuffer updates.
    for rect in render_rects.into_iter() {
        if rects.is_empty() {
            rects.push(rect);
        } else {
            if let Some(last) = rects.last_mut() {
                if rect.extends(last) {
                    last.absorb(&rect);
                    let mut i = rects.len();
                    while i > 1 && rects[i - 1].extends(&rects[i - 2]) {
                        if let Some(rect) = rects.pop() {
                            if let Some(last) = rects.last_mut() {
                                last.absorb(&rect);
                            }
                        }
                        i -= 1;
                    }
                } else {
                    let mut i = rects.len();
                    while i > 0 && !rects[i - 1].contains(&rect) {
                        i -= 1;
                    }
                    if i == 0 {
                        rects.push(rect);
                    }
                }
            }
        }
    }

    for i in 0..view.len() {
        render(view.child(i), wait, ids, rects, bgs, fb, fonts, updating);
    }
}

#[inline]
pub fn process_render_queue(
    view: &dyn View,
    rq: &mut RenderQueue,
    context: &mut Context,
    updating: &mut Vec<UpdateData>,
) {
    for ((mode, wait), pairs) in rq.drain() {
        let mut ids = FxHashMap::default();
        let mut rects = Vec::new();
        let mut bgs = Vec::new();

        for (id, rect) in pairs.into_iter().rev() {
            if let Some(id) = id {
                ids.entry(id).or_insert_with(Vec::new).push(rect);
            } else {
                bgs.push(rect);
            }
        }

        render(
            view,
            wait,
            &ids,
            &mut rects,
            &mut bgs,
            context.fb.as_mut(),
            &mut context.fonts,
            updating,
        );

        for rect in rects {
            match context.fb.update(&rect, mode) {
                Ok(token) => {
                    updating.push(UpdateData {
                        token,
                        rect,
                        time: Instant::now(),
                    });
                }
                Err(err) => {
                    log_error!("Can't update {}: {:#}.", rect, err);
                }
            }
        }
    }
}

#[inline]
pub fn wait_for_all(updating: &mut Vec<UpdateData>, context: &mut Context) {
    for update in updating.drain(..) {
        if update.has_completed() {
            continue;
        }
        context
            .fb
            .wait(update.token)
            .map_err(|e| log_error!("Can't wait for {}, {}: {:#}", update.token, update.rect, e))
            .ok();
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Device(DeviceEvent),
    Gesture(GestureEvent),
    Keyboard(KeyboardEvent),
    Key(KeyKind),
    Open(Box<Info>),
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
    SearchResult(usize, Vec<Boundary>),
    FetcherAddDocument(u32, Box<Info>),
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AppCmd {
    Sketch,
    Calculator,
    Dictionary {
        query: String,
        language: String,
    },
    EpubEditor {
        path: String,
        chapter: Option<usize>,
    },
    CoverEditor,
    OpenCoverEditor(std::path::PathBuf),
    Statistics,
    PdfManipulator,
    TouchEvents,
    RotationValues,
}

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum ViewId {
    Home,
    Reader,
    SortMenu,
    MainMenu,
    TitleMenu,
    SelectionMenu,
    AnnotationMenu,
    BatteryMenu,
    ClockMenu,
    SearchTargetMenu,
    InputHistoryMenu,
    KeyboardLayoutMenu,
    Frontlight,
    Dictionary,
    FontSizeMenu,
    TextAlignMenu,
    FontFamilyMenu,
    MarginWidthMenu,
    ContrastExponentMenu,
    ContrastGrayMenu,
    LineHeightMenu,
    DirectoryMenu,
    BookMenu,
    LibraryMenu,
    PageMenu,
    PresetMenu,
    MarginCropperMenu,
    SearchMenu,
    SketchMenu,
    SettingsEditor,
    RenameDocument,
    RenameDocumentInput,
    GoToPage,
    GoToPageInput,
    GoToResultsPage,
    GoToResultsPageInput,
    NamePage,
    NamePageInput,
    EditNote,
    EditNoteInput,
    EditLanguages,
    EditLanguagesInput,
    HomeSearchInput,
    ReaderSearchInput,
    DictionarySearchInput,
    CalculatorInput,
    SearchBar,
    AddressBar,
    AddressBarInput,
    Keyboard,
    AboutDialog,
    ShareDialog,
    MarginCropper,
    TopBottomBars,
    TableOfContents,
    EpubEditor,
    EpubEditorSearchInput,
    EpubEditorReplaceInput,
    PdfManipulator,
    PdfManipulatorMenu,
    MessageNotif(Id),
    SubMenu(u8),
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SliderId {
    FontSize,
    LightIntensity,
    LightWarmth,
    ContrastExponent,
    ContrastGray,
    AutoSuspend,
    AutoPowerOff,
}

#[derive(Debug, Clone)]
pub enum PluginTriggerKind {
    OnBookImport,
    OnBookOpen,
    OnBookClose,
    OnSyncComplete,
    OnStartup,
    OnShutdown,
}

impl SliderId {
    pub fn label(self) -> String {
        match self {
            SliderId::LightIntensity => "Intensity".to_string(),
            SliderId::LightWarmth => "Warmth".to_string(),
            SliderId::FontSize => "Font Size".to_string(),
            SliderId::ContrastExponent => "Contrast Exponent".to_string(),
            SliderId::ContrastGray => "Contrast Gray".to_string(),
            SliderId::AutoSuspend => "Auto Suspend".to_string(),
            SliderId::AutoPowerOff => "Auto Power Off".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Align {
    Left(i32),
    Right(i32),
    Center,
}

impl Align {
    #[inline]
    pub fn offset(&self, width: i32, container_width: i32) -> i32 {
        match *self {
            Align::Left(dx) => dx,
            Align::Right(dx) => container_width - width - dx,
            Align::Center => (container_width - width) / 2,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum KeyboardEvent {
    Append(char),
    Partial(char),
    Move { target: TextKind, dir: LinearDir },
    Delete { target: TextKind, dir: LinearDir },
    Submit,
}

#[derive(Debug, Copy, Clone)]
pub enum TextKind {
    Char,
    Word,
    Extremum,
}

#[derive(Debug, Clone)]
pub enum EntryKind {
    Message(String, Option<EntryId>),
    Command(String, EntryId),
    CheckBox(String, EntryId, bool),
    RadioButton(String, EntryId, bool),
    SubMenu(String, Vec<EntryKind>),
    More(Vec<EntryKind>),
    Separator,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EntryId {
    About,
    SystemInfo,
    LoadLibrary(usize),
    Load(PathBuf),
    Flush,
    Save,
    Discard,
    Import,
    CleanUp,
    Sort(SortMethod),
    ReverseOrder,
    EmptyTrash,
    Rename(PathBuf),
    Remove(PathBuf),
    CopyTo(PathBuf, usize),
    MoveTo(PathBuf, usize),
    AddDirectory(PathBuf),
    SelectDirectory(PathBuf),
    ToggleSelectDirectory(PathBuf),
    SetStatus(PathBuf, SimpleStatus),
    SearchAuthor(String),
    RemovePreset(usize),
    FirstColumn(FirstColumn),
    SecondColumn(SecondColumn),
    ThumbnailPreviews,
    ApplyCroppings(usize, PageScheme),
    RemoveCroppings,
    SetZoomMode(ZoomMode),
    SetScrollMode(ScrollMode),
    SetPageName,
    RemovePageName,
    HighlightSelection,
    AnnotateSelection,
    DefineSelection,
    SearchForSelection,
    AdjustSelection,
    Annotations,
    Bookmarks,
    RemoveAnnotation([TextLocation; 2]),
    EditAnnotationNote([TextLocation; 2]),
    RemoveAnnotationNote([TextLocation; 2]),
    GoTo(usize),
    GoToSelectedPageName,
    SearchDirection(LinearDir),
    SetButtonScheme(ButtonScheme),
    SetFontFamily(String),
    SetFontSize(i32),
    SetTextAlign(TextAlign),
    SetMarginWidth(i32),
    SetLineHeight(i32),
    SetContrastExponent(i32),
    SetContrastGray(i32),
    SetRotationLock(Option<RotationLock>),
    SetSearchTarget(Option<String>),
    SetInputText(ViewId, String),
    SetKeyboardLayout(String),
    ToggleShowHidden,
    ToggleFuzzy,
    ToggleInverted,
    ToggleDithered,
    ToggleWifi,
    ToggleFrontlight,
    Rotate(i8),
    Launch(AppCmd),
    SelectChapter(usize),
    Undo,
    Redo,
    SetPenSize(i32),
    SetPenColor(Color),
    TogglePenDynamism,
    ReloadDictionaries,
    New,
    Refresh,
    TakeScreenshot,
    Reboot,
    Quit,
    SaveSettings,
    OpenSettingsEditor,
    ToggleDualPage,
    ToggleSleepCover,
    CycleFinishedAction,
    CycleLanguage,
    CycleUiFont,
    ToggleMangaMode,
    ToggleMupdfSearch,
    ToggleShowTime,
    ToggleShowBattery,
    ToggleExternalStorage,
    ToggleFastPageTurn,
    CyclePageTurnAnimation,
    Preview,
    SearchReplace,
    ReplaceInChapter,
    ReplaceInDocument,
    NextMatch,
    PrevMatch,
    CloseSearchReplace,
    PdfManipulate(PathBuf, String),
    Back,
    ToggleBatchMode,
    OpenFileBrowser,
}

impl EntryId {
    pub fn as_str(&self) -> &'static str {
        match self {
            EntryId::About => "About",
            EntryId::SystemInfo => "SystemInfo",
            EntryId::LoadLibrary(_) => "LoadLibrary",
            EntryId::Load(_) => "Load",
            EntryId::Flush => "Flush",
            EntryId::Save => "Save",
            EntryId::Discard => "Discard",
            EntryId::Import => "Import",
            EntryId::CleanUp => "CleanUp",
            EntryId::Sort(_) => "Sort",
            EntryId::ReverseOrder => "ReverseOrder",
            EntryId::EmptyTrash => "EmptyTrash",
            EntryId::Rename(_) => "Rename",
            EntryId::Remove(_) => "Remove",
            EntryId::CopyTo(_, _) => "CopyTo",
            EntryId::MoveTo(_, _) => "MoveTo",
            EntryId::AddDirectory(_) => "AddDirectory",
            EntryId::SelectDirectory(_) => "SelectDirectory",
            EntryId::ToggleSelectDirectory(_) => "ToggleSelectDirectory",
            EntryId::SetStatus(_, _) => "SetStatus",
            EntryId::SearchAuthor(_) => "SearchAuthor",
            EntryId::RemovePreset(_) => "RemovePreset",
            EntryId::FirstColumn(_) => "FirstColumn",
            EntryId::SecondColumn(_) => "SecondColumn",
            EntryId::ThumbnailPreviews => "ThumbnailPreviews",
            EntryId::ApplyCroppings(_, _) => "ApplyCroppings",
            EntryId::RemoveCroppings => "RemoveCroppings",
            EntryId::SetZoomMode(_) => "SetZoomMode",
            EntryId::SetScrollMode(_) => "SetScrollMode",
            EntryId::SetPageName => "SetPageName",
            EntryId::RemovePageName => "RemovePageName",
            EntryId::HighlightSelection => "HighlightSelection",
            EntryId::AnnotateSelection => "AnnotateSelection",
            EntryId::DefineSelection => "DefineSelection",
            EntryId::SearchForSelection => "SearchForSelection",
            EntryId::AdjustSelection => "AdjustSelection",
            EntryId::Annotations => "Annotations",
            EntryId::Bookmarks => "Bookmarks",
            EntryId::RemoveAnnotation(_) => "RemoveAnnotation",
            EntryId::EditAnnotationNote(_) => "EditAnnotationNote",
            EntryId::RemoveAnnotationNote(_) => "RemoveAnnotationNote",
            EntryId::GoTo(_) => "GoTo",
            EntryId::GoToSelectedPageName => "GoToSelectedPageName",
            EntryId::SearchDirection(_) => "SearchDirection",
            EntryId::SetButtonScheme(_) => "SetButtonScheme",
            EntryId::SetFontFamily(_) => "SetFontFamily",
            EntryId::SetFontSize(_) => "SetFontSize",
            EntryId::SetTextAlign(_) => "SetTextAlign",
            EntryId::SetMarginWidth(_) => "SetMarginWidth",
            EntryId::SetLineHeight(_) => "SetLineHeight",
            EntryId::SetContrastExponent(_) => "SetContrastExponent",
            EntryId::SetContrastGray(_) => "SetContrastGray",
            EntryId::SetRotationLock(_) => "SetRotationLock",
            EntryId::SetSearchTarget(_) => "SetSearchTarget",
            EntryId::SetInputText(_, _) => "SetInputText",
            EntryId::SetKeyboardLayout(_) => "SetKeyboardLayout",
            EntryId::ToggleShowHidden => "ToggleShowHidden",
            EntryId::ToggleFuzzy => "ToggleFuzzy",
            EntryId::ToggleInverted => "ToggleInverted",
            EntryId::ToggleDithered => "ToggleDithered",
            EntryId::ToggleWifi => "ToggleWifi",
            EntryId::ToggleFrontlight => "ToggleFrontlight",
            EntryId::Rotate(_) => "Rotate",
            EntryId::Launch(_) => "Launch",
            EntryId::SelectChapter(_) => "SelectChapter",
            EntryId::Undo => "Undo",
            EntryId::Redo => "Redo",
            EntryId::SetPenSize(_) => "SetPenSize",
            EntryId::SetPenColor(_) => "SetPenColor",
            EntryId::TogglePenDynamism => "TogglePenDynamism",
            EntryId::ReloadDictionaries => "ReloadDictionaries",
            EntryId::New => "New",
            EntryId::Refresh => "Refresh",
            EntryId::TakeScreenshot => "TakeScreenshot",
            EntryId::Reboot => "Reboot",
            EntryId::Quit => "Quit",
            EntryId::SaveSettings => "SaveSettings",
            EntryId::OpenSettingsEditor => "OpenSettingsEditor",
            EntryId::ToggleDualPage => "ToggleDualPage",
            EntryId::ToggleSleepCover => "ToggleSleepCover",
            EntryId::CycleFinishedAction => "CycleFinishedAction",
            EntryId::CycleLanguage => "CycleLanguage",
            EntryId::CycleUiFont => "CycleUiFont",
            EntryId::ToggleMangaMode => "ToggleMangaMode",
            EntryId::ToggleMupdfSearch => "ToggleMupdfSearch",
            EntryId::ToggleShowTime => "ToggleShowTime",
            EntryId::ToggleShowBattery => "ToggleShowBattery",
            EntryId::ToggleExternalStorage => "ToggleExternalStorage",
            EntryId::ToggleFastPageTurn => "ToggleFastPageTurn",
            EntryId::CyclePageTurnAnimation => "CyclePageTurnAnimation",
            EntryId::Preview => "Preview",
            EntryId::SearchReplace => "SearchReplace",
            EntryId::ReplaceInChapter => "ReplaceInChapter",
            EntryId::ReplaceInDocument => "ReplaceInDocument",
            EntryId::NextMatch => "NextMatch",
            EntryId::PrevMatch => "PrevMatch",
            EntryId::CloseSearchReplace => "CloseSearchReplace",
            EntryId::PdfManipulate(_, _) => "PdfManipulate",
            EntryId::Back => "Back",
            EntryId::ToggleBatchMode => "ToggleBatchMode",
            EntryId::OpenFileBrowser => "OpenFileBrowser",
        }
    }
}

impl EntryKind {
    pub fn is_separator(&self) -> bool {
        matches!(*self, EntryKind::Separator)
    }

    pub fn text(&self) -> &str {
        match *self {
            EntryKind::Message(ref s, ..)
            | EntryKind::Command(ref s, ..)
            | EntryKind::CheckBox(ref s, ..)
            | EntryKind::RadioButton(ref s, ..)
            | EntryKind::SubMenu(ref s, ..) => s,
            EntryKind::More(..) => "More",
            _ => "",
        }
    }

    pub fn get(&self) -> Option<bool> {
        match *self {
            EntryKind::CheckBox(_, _, v) | EntryKind::RadioButton(_, _, v) => Some(v),
            _ => None,
        }
    }

    pub fn set(&mut self, value: bool) {
        match *self {
            EntryKind::CheckBox(_, _, ref mut v) | EntryKind::RadioButton(_, _, ref mut v) => {
                *v = value
            }
            _ => (),
        }
    }
}

pub struct RenderData {
    pub id: Option<Id>,
    pub rect: Rectangle,
    pub mode: UpdateMode,
    pub wait: bool,
}

impl RenderData {
    pub fn new(id: Id, rect: Rectangle, mode: UpdateMode) -> RenderData {
        RenderData {
            id: Some(id),
            rect,
            mode,
            wait: true,
        }
    }

    pub fn no_wait(id: Id, rect: Rectangle, mode: UpdateMode) -> RenderData {
        RenderData {
            id: Some(id),
            rect,
            mode,
            wait: false,
        }
    }

    pub fn expose(rect: Rectangle, mode: UpdateMode) -> RenderData {
        RenderData {
            id: None,
            rect,
            mode,
            wait: true,
        }
    }
}

pub struct UpdateData {
    pub token: u32,
    pub time: Instant,
    pub rect: Rectangle,
}

pub const MAX_UPDATE_DELAY: Duration = Duration::from_millis(600);

impl UpdateData {
    pub fn has_completed(&self) -> bool {
        self.time.elapsed() >= MAX_UPDATE_DELAY
    }
}

type RQ = FxHashMap<(UpdateMode, bool), Vec<(Option<Id>, Rectangle)>>;
pub struct RenderQueue(RQ);

impl RenderQueue {
    pub fn new() -> RenderQueue {
        RenderQueue(FxHashMap::default())
    }

    /// Add render data to queue, deduplicating same (id, rect) pairs.
    /// Only the first entry is kept; subsequent duplicates are ignored.
    pub fn add(&mut self, data: RenderData) {
        let key = (data.mode, data.wait);
        let entry = self.entry(key).or_insert_with(|| Vec::with_capacity(8));

        // Skip if this (id, rect) pair already exists
        let new_pair = (data.id, data.rect);
        if !entry.iter().any(|existing| existing == &new_pair) {
            entry.push(new_pair);
        }
    }
}

impl Default for RenderQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for RenderQueue {
    type Target = RQ;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for RenderQueue {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub static ID_FEEDER: IdFeeder = IdFeeder::new(1);
pub struct IdFeeder(AtomicU64);
pub type Id = u64;

impl IdFeeder {
    pub const fn new(id: Id) -> Self {
        IdFeeder(AtomicU64::new(id))
    }

    pub fn next(&self) -> Id {
        self.0.fetch_add(1, Ordering::Relaxed)
    }
}
