use std::sync::atomic::{AtomicU64, Ordering};

pub type Id = u64;

pub static ID_FEEDER: IdFeeder = IdFeeder::new(1);

pub struct IdFeeder(AtomicU64);

impl IdFeeder {
    pub const fn new(id: Id) -> Self {
        IdFeeder(AtomicU64::new(id))
    }

    pub fn next(&self) -> Id {
        self.0.fetch_add(1, Ordering::Relaxed)
    }
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
pub enum PluginTriggerKind {
    OnBookImport,
    OnBookOpen,
    OnBookClose,
    OnSyncComplete,
    OnStartup,
    OnShutdown,
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
    OpenPdfManipulator(std::path::PathBuf),
    TouchEvents,
    RotationValues,
}
