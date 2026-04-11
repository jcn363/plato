use std::path::PathBuf;

use super::identifiers::{AppCmd, ViewId};
use crate::color::Color;
use crate::metadata::{PageScheme, ScrollMode, SimpleStatus, SortMethod, TextAlign, ZoomMode};
use crate::settings::{ButtonScheme, FirstColumn, RotationLock, SecondColumn};

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
    RemoveAnnotation([crate::document::TextLocation; 2]),
    EditAnnotationNote([crate::document::TextLocation; 2]),
    RemoveAnnotationNote([crate::document::TextLocation; 2]),
    GoTo(usize),
    GoToSelectedPageName,
    SearchDirection(crate::geom::LinearDir),
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
    ToggleDarkMode,
    SetAutoThemeThreshold,
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
            EntryId::ToggleDarkMode => "ToggleDarkMode",
            EntryId::SetAutoThemeThreshold => "SetAutoThemeThreshold",
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
pub enum TextKind {
    Char,
    Word,
    Extremum,
}
