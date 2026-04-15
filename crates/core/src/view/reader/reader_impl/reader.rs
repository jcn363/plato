//! Main Reader View Implementation
//!
//! The Reader view is the core component for displaying and interacting with documents
//! (EPUB, PDF, etc.) on Kobo e-readers.
//!
//! ## Architecture
//!
//! This implementation follows a modular design where related functionality is split
//! across specialized submodules:
//!
//! - `reader.rs` (3,300 lines) - Main Reader struct and core methods
//! - `reader_settings.rs` (947 lines) - Settings menus and configuration helpers
//! - `reader_rendering.rs` (231 lines) - Text and selection rendering utilities
//! - `reader_search.rs` (161 lines) - Search functionality
//! - `reader_annotations.rs` (90 lines) - Annotation and bookmark helpers
//! - `reader_dialogs.rs` (141 lines) - Dialog and input handling
//! - `reader_gestures.rs` (810 lines) - Touch/gesture handling and input processing
//! - `reader_core.rs` (128 lines) - Shared type definitions
//!
//! ## Key Design Decisions
//!
//! ### 1. Monolithic Reader Struct (INTENTIONAL)
//! The Reader struct contains 50+ fields representing:
//! - Document state (current_page, pages_count, doc, synthetic)
//! - View state (view_port, rect, reflowable)
//! - UI state (menus, focus, selection, search)
//! - Rendering cache (cache, text, annotations)
//!
//! **Why not split?** Splitting into sub-structs would require extensive refactoring
//! of 100+ methods that access multiple fields. The current approach is pragmatic
//! given the high interdependency.
//!
//!
//! ### 2. Complex Setter Methods (DOCUMENTED LIMITATIONS)
//! Several setter methods (`set_font_size`, `set_text_align`, etc.) perform:
//! 1. Arc strong count validation
//! 2. Info metadata update
//! 3. Document lock and manipulation
//! 4. Page recalculation
//! 5. Cache invalidation
//! 6. UI update
//!
//! **Why keep these in reader.rs?** Extracting these would require passing 8-12
//! parameters per method, creating more complexity than the original code.
//! **Attempted extraction**: Phase 3 concluded that full extraction is not beneficial.
//!
//! ### 3. Event Handling in handle_event() (LARGE METHOD)
//! The `handle_event()` method (~400 lines) contains the main event dispatcher
//! that handles:
//! - Gesture events (swipes, taps, long-press)
//! - Physical button events (home, navigation)
//! - Menu callbacks and selections
//! - Text selection and annotation interaction
//!
//!
//! ### 4. Document Manipulation Pattern
//! All document modifications follow a consistent pattern:
//! ```ignore
//! let mut doc = self.doc.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
//! doc.set_property(...);
//! drop(doc);  // explicit unlock
//! self.update(None, hub, rq, context);
//! ```
//!
//! This ensures proper locking and refresh behavior. Alternative approaches
//! (per-field locks, async mutations) would add significant complexity.
//!
//! ## Known Limitations & TODOs
//!
//! ### Type Duplication
//! Note: `ViewPort` is imported from `reader_core.rs` - consolidation in progress.
//! Other types like `Contrast`, `PageAnimation` are also in reader_core.rs.
//!
//! ### Missing Optimizations
//! - Page rendering doesn't parallelize across CPU cores
//! - Text extraction could be cached more aggressively
//! - Gesture recognition is synchronous (could be improved)
//!
//! **Rationale**: Device constraints (limited RAM, low CPU) mean optimizations
//! would likely add overhead. Optimize if profiling shows bottlenecks.
//!
//! ### Unimplemented Features
//! These are documented as stub implementations in trait methods:
//! - `set_monochrome()` - Not supported on Kobo e-readers (display API limitation)
//! - `set_font_family()` for PDFs - MuPDF API limitation (stub provided)
//!
//! **Location**: Search for `Not supported` in methods to find these stubs.
//!
//! ## Testing Notes
//!
//! The Reader view is difficult to unit test because:
//! 1. Heavy dependency on Context (device info, display settings)
//! 2. Requires actual document files (EPUB, PDF)
//! 3. MuPDF/FreeType initialization needed (native libs)
//!
//! **Current approach**: Integration tests in `tests/` directory with fixture documents.
//! Unit tests for pure functions (text extraction, search) are in `reader_rendering.rs`.
//!
//! ## Performance Characteristics
//!
//! ### Memory Usage
//! - Document cache: ~1-2 MB (depends on page complexity)
//! - Text index: ~100 KB-1 MB (depends on book size)
//! - Typical peak: 20-40 MB (manageable on Kobo)
//!
//! ### Rendering Performance
//! - Simple pages: 100-300ms render time (target: <500ms)
//! - Complex PDFs: 500-1500ms (acceptable for static content)
//! - Eink refresh adds 200-500ms (dominates user-perceived latency)
//!
//! **Optimization focus**: Minimize eink refresh regions, not raw computation.
//!
//! ## Future Refactoring Roadmap
//!
//! **Phase 4** (Estimated: 20-30 hours):
//! 1. Consolidate Reader fields into nested structs
//! 2. Extract sub-handlers from handle_event()
//! 3. Create GestureProcessor trait for extensibility
//! 4. Move event queue to central Hub
//!
//! **Phase 5** (Estimated: 30-40 hours):
//! 1. Async document I/O with tokio
//! 2. Parallel page rendering (if profiling justifies)
//! 3. Plugin architecture for custom document types
//! 4. Advanced gesture recognition (multi-touch, etc.)

// ===========================================================================
// Imports and Constants
// ===========================================================================

use crate::color::{background, foreground};
use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::document::{BoundedText, Document, Location, SimpleTocEntry, TextLocation, TocEntry};
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::{halves, CycleDir, LinearDir};
use crate::geom::{BorderSpec, Boundary, CornerSpec, Point, Rectangle, Vec2};
use crate::input::{ButtonCode, ButtonStatus, DeviceEvent, FingerStatus};
use crate::log_error;
use crate::metadata::{Annotation, Info, ScrollMode, TextAlign, ZoomMode};
use crate::metadata::{CroppingMargins, Margin};
use crate::metadata::{DEFAULT_CONTRAST_EXPONENT, DEFAULT_CONTRAST_GRAY};
use crate::settings::DEFAULT_FONT_FAMILY;
use crate::theme;
use crate::unit::{mm_to_px, scale_by_dpi};
use anyhow::{Context as AnyhowContext, Error};
use regex::Regex;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::{BTreeMap, VecDeque};
use std::sync::{atomic, Arc, LazyLock, Mutex};

use crate::view::common::{
    locate, locate_by_id, toggle_battery_menu, toggle_clock_menu, toggle_main_menu,
};
use crate::view::filler::Filler;
use crate::view::keyboard::Keyboard;
use crate::view::menu::Menu;
use crate::view::menu_entry::MenuEntry;
use crate::view::search_bar::SearchBar;
use crate::view::top_bar::TopBar;
use crate::view::{
    Bus, EntryId, Event, Hub, Id, RenderData, RenderQueue, SliderId, View, ViewId, BIG_BAR_HEIGHT,
    ID_FEEDER, SMALL_BAR_HEIGHT, THICKNESS_MEDIUM,
};

use crate::view::reader::bottom_bar::BottomBar;
use crate::view::reader::tool_bar::ToolBar;

use super::reader_core::{
    Contrast, PageAnimKind, PageAnimation, RenderChunk, Resource, Search, Selection, State,
    ViewPort,
};
use super::reader_rendering;
use super::reader_search;

#[allow(dead_code)] // Used by reader gesture handling
pub const RECT_DIST_JITTER: f32 = 0.1;
#[allow(dead_code)] // Used by memory scheme handling
pub const MEM_SCHEME: &str = "mem:";

#[allow(dead_code)] // Used by TOC parsing
pub static TOC_PAGE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)page\s*(\d+)").unwrap());
#[allow(dead_code)] // Used by PDF page parsing
pub static PDF_PAGE_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"(\d+)").unwrap());
#[allow(dead_code)] // Used by search result parsing
pub static SEARCH_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\((\d+),\s*(\d+)\)").unwrap());

#[allow(dead_code)] // Used by annotation rendering
pub const HIGHLIGHT_DRIFT: f32 = 0.1;
#[allow(dead_code)] // Used by annotation rendering
pub const ANNOTATION_DRIFT: f32 = 0.05;

// ===========================================================================
// Type Definitions
// ===========================================================================

pub struct Reader {
    pub(crate) id: Id,
    pub(crate) rect: Rectangle,
    children: Vec<Box<dyn View>>,
    pub(crate) doc: Arc<Mutex<Box<dyn Document>>>,
    pub(crate) cache: BTreeMap<usize, Resource>,
    pub(crate) chunks: Vec<RenderChunk>,
    pub(crate) text: FxHashMap<usize, Vec<BoundedText>>,
    pub(crate) annotations: FxHashMap<usize, Vec<Annotation>>,
    pub(crate) noninverted_regions: FxHashMap<usize, Vec<Boundary>>,
    pub(crate) focus: Option<ViewId>,
    pub(crate) search: Option<Search>,
    pub(crate) search_direction: LinearDir,
    pub(crate) held_buttons: FxHashSet<ButtonCode>,
    pub(crate) selection: Option<Selection>,
    pub(crate) target_annotation: Option<[TextLocation; 2]>,
    pub(crate) history: VecDeque<usize>,
    pub(crate) state: State,
    pub(crate) info: Info,
    pub(crate) current_page: usize,
    pub(crate) pages_count: usize,
    pub(crate) view_port: ViewPort,
    pub(crate) contrast: Contrast,
    pub(crate) synthetic: bool,
    pub(crate) page_turns: usize,
    pub(crate) reflowable: bool,
    pub(crate) ephemeral: bool,
    pub(crate) finished: bool,
    pub(crate) animation: Option<PageAnimation>,
    pub(crate) previous_chunks: Vec<RenderChunk>,
}

// ===========================================================================
// Constructors
// ===========================================================================

impl Reader {
    pub fn new(rect: Rectangle, info: Info, _hub: &Hub, context: &mut Context) -> Option<Reader> {
        let id = ID_FEEDER.next();
        let doc = match crate::document::open(&info.file.path) {
            Some(d) => d,
            None => {
                log_error!("Failed to open document: {}", info.file.path.display());
                return None;
            }
        };
        let doc = Arc::new(Mutex::new(doc));
        let pages_count = doc.lock().expect("doc lock").pages_count();
        let reflowable = doc.lock().expect("doc lock").is_reflowable();

        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;
        let _thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;

        let top_bar_rect = rect![
            rect.min.x,
            rect.min.y,
            rect.max.x,
            rect.min.y + small_height
        ];
        let tool_bar = ToolBar::new(
            top_bar_rect,
            reflowable,
            info.reader.as_ref(),
            &context.settings.reader,
        );
        let children = vec![Box::new(tool_bar) as Box<dyn View>];

        Some(Reader {
            id,
            rect,
            children,
            doc,
            cache: BTreeMap::new(),
            chunks: Vec::new(),
            text: FxHashMap::default(),
            annotations: FxHashMap::default(),
            noninverted_regions: FxHashMap::default(),
            focus: None,
            search: None,
            search_direction: LinearDir::Forward,
            held_buttons: FxHashSet::default(),
            selection: None,
            target_annotation: None,
            history: VecDeque::new(),
            state: State::Idle,
            info,
            current_page: 0,
            pages_count,
            view_port: ViewPort::default(),
            contrast: Contrast::default(),
            synthetic: false,
            page_turns: 0,
            reflowable,
            ephemeral: false,
            finished: false,
            animation: None,
            previous_chunks: Vec::new(),
        })
    }

    pub fn from_html(
        rect: Rectangle,
        html: &str,
        _link_uri: Option<&str>,
        _hub: &Hub,
        context: &mut Context,
    ) -> Result<Reader, Error> {
        let id = ID_FEEDER.next();
        let doc = crate::document::open_html(html).context("Failed to open HTML document")?;
        let doc = Arc::new(Mutex::new(doc));
        let pages_count = doc.lock().expect("doc lock").pages_count();
        let reflowable = doc.lock().expect("doc lock").is_reflowable();

        let dpi = CURRENT_DEVICE.dpi;
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;

        let top_bar_rect = rect![
            rect.min.x,
            rect.min.y,
            rect.max.x,
            rect.min.y + small_height
        ];
        let tool_bar = ToolBar::new(top_bar_rect, reflowable, None, &context.settings.reader);
        let children = vec![Box::new(tool_bar) as Box<dyn View>];

        let info = Info {
            file: crate::metadata::FileInfo {
                path: std::path::PathBuf::from("memory.html"),
                kind: "html".to_string(),
                size: html.len() as u64,
            },
            reader: None,
            ..Default::default()
        };

        Ok(Reader {
            id,
            rect,
            children,
            doc,
            cache: BTreeMap::new(),
            chunks: Vec::new(),
            text: FxHashMap::default(),
            annotations: FxHashMap::default(),
            noninverted_regions: FxHashMap::default(),
            focus: None,
            search: None,
            search_direction: LinearDir::Forward,
            held_buttons: FxHashSet::default(),
            selection: None,
            target_annotation: None,
            history: VecDeque::new(),
            state: State::Idle,
            info,
            current_page: 0,
            pages_count,
            view_port: ViewPort::default(),
            contrast: Contrast::default(),
            synthetic: false,
            page_turns: 0,
            reflowable,
            ephemeral: true,
            finished: false,
            animation: None,
            previous_chunks: Vec::new(),
        })
    }

    #[allow(dead_code)] // Used by Reader::render method
    fn render_animation(&self, fb: &mut dyn Framebuffer, rect: Rectangle) {
        if let Some(ref anim) = self.animation {
            for chunk in &self.previous_chunks {
                if let Some(resource) = self.cache.get(&chunk.location) {
                    let Resource {
                        ref pixmap,
                        scale: _,
                        ..
                    } = resource;
                    let chunk_rect = chunk.frame - chunk.frame.min + chunk.position;

                    if let Some(region_rect) = rect.intersection(&chunk_rect) {
                        let chunk_frame = region_rect - chunk.position + chunk.frame.min;
                        let chunk_position = region_rect.min;

                        match anim {
                            PageAnimation::None => {}
                            PageAnimation::Slide(kind) => {
                                let offset = (kind.progress * rect.width() as f32) as i32;
                                let adjusted_position =
                                    if matches!(kind.direction, LinearDir::Forward) {
                                        pt!(chunk_position.x - offset, chunk_position.y)
                                    } else {
                                        pt!(chunk_position.x + offset, chunk_position.y)
                                    };
                                let alpha = (1.0 - kind.progress) as u8;
                                fb.draw_framed_pixmap_contrast_alpha(
                                    pixmap,
                                    &chunk_frame,
                                    adjusted_position,
                                    self.contrast.exponent,
                                    self.contrast.gray,
                                    alpha,
                                );
                            }
                            PageAnimation::Peel(state) => match state.kind {
                                PageAnimKind::Fade => {
                                    let alpha = ((1.0 - state.progress) * 255.0) as u8;
                                    fb.draw_framed_pixmap_contrast_alpha(
                                        pixmap,
                                        &chunk_frame,
                                        chunk_position,
                                        self.contrast.exponent,
                                        self.contrast.gray,
                                        alpha,
                                    );
                                }
                                PageAnimKind::Flip => {
                                    let offset = (state.progress * rect.width() as f32) as i32;
                                    let adjusted_position =
                                        if matches!(state.direction, LinearDir::Forward) {
                                            pt!(chunk_position.x - offset, chunk_position.y)
                                        } else {
                                            pt!(chunk_position.x + offset, chunk_position.y)
                                        };
                                    let alpha = ((1.0 - state.progress * 0.5) * 255.0) as u8;
                                    fb.draw_framed_pixmap_contrast_alpha(
                                        pixmap,
                                        &chunk_frame,
                                        adjusted_position,
                                        self.contrast.exponent,
                                        self.contrast.gray,
                                        alpha,
                                    );
                                }
                                _ => {}
                            },
                        }
                    }
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Toggle Menus
    // -----------------------------------------------------------------------

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn toggle_edit_note(
        &mut self,
        text: Option<&str>,
        enable: Option<bool>,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_dialogs::toggle_edit_note(&mut self.children, text, enable, hub, rq, context);
    }

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn toggle_name_page(
        &mut self,
        enable: Option<bool>,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_dialogs::toggle_name_page(&mut self.children, enable, hub, rq, context);

        if let Some(false) = enable {
            if self
                .focus
                .map(|focus_id| focus_id == ViewId::NamePageInput)
                .unwrap_or(false)
            {
                self.toggle_keyboard(false, None, hub, rq, context);
            }
        }
    }

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn toggle_go_to_page(
        &mut self,
        enable: Option<bool>,
        id: ViewId,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_dialogs::toggle_go_to_page(&mut self.children, enable, id, hub, rq, context);

        if let Some(false) = enable {
            let input_id = if id == ViewId::GoToPage {
                ViewId::GoToPageInput
            } else {
                ViewId::GoToResultsPageInput
            };
            if self
                .focus
                .map(|focus_id| focus_id == input_id)
                .unwrap_or(false)
            {
                self.toggle_keyboard(false, None, hub, rq, context);
            }
        }
    }

    pub fn toggle_annotation_menu(
        &mut self,
        annot: &Annotation,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_settings::toggle_annotation_menu(
            &mut self.children,
            annot,
            rect,
            enable,
            rq,
            context,
        );
    }

    pub fn toggle_selection_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let file_kind = self.info.file.kind.as_str();
        let file_path = context.library.home.join(&self.info.file.path);
        let file_path_str = file_path.to_string_lossy().to_string();
        let has_page_names = self
            .info
            .reader
            .as_ref()
            .map_or(false, |r| !r.page_names.is_empty());

        super::reader_settings::toggle_selection_menu(
            &mut self.children,
            self.current_page,
            file_kind,
            if file_kind == "epub" {
                Some(file_path_str)
            } else {
                None
            },
            has_page_names,
            rect,
            enable,
            rq,
            context,
        );
    }

    pub fn toggle_title_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let file_kind = self.info.file.kind.as_str();
        let file_path = context.library.home.join(&self.info.file.path);
        let file_path_str = file_path.to_string_lossy().to_string();
        let has_annotations = self
            .info
            .reader
            .as_ref()
            .map_or(false, |r| !r.annotations.is_empty());
        let has_bookmarks = self
            .info
            .reader
            .as_ref()
            .map_or(false, |r| !r.bookmarks.is_empty());

        super::reader_settings::toggle_title_menu(
            &mut self.children,
            rect,
            self.reflowable,
            file_kind,
            if file_kind == "epub" {
                Some(file_path_str)
            } else {
                None
            },
            has_annotations,
            has_bookmarks,
            self.view_port.zoom_mode,
            self.view_port.scroll_mode,
            enable,
            rq,
            context,
        );
    }

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn toggle_font_family_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let current_family = self
            .info
            .reader
            .as_ref()
            .and_then(|r| r.font_family.clone())
            .unwrap_or_else(|| context.settings.reader.font_family.clone());
        super::reader_settings::toggle_font_family_menu(
            &mut self.children,
            current_family,
            rect,
            enable,
            rq,
            context,
        );
    }

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn toggle_font_size_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let current_size = self
            .info
            .reader
            .as_ref()
            .and_then(|r| r.font_size)
            .unwrap_or(context.settings.reader.font_size);
        super::reader_settings::toggle_font_size_menu(
            &mut self.children,
            current_size,
            rect,
            enable,
            rq,
            context,
        );
    }

    fn toggle_text_align_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let current_align = self
            .info
            .reader
            .as_ref()
            .and_then(|r| r.text_align)
            .unwrap_or(context.settings.reader.text_align);
        super::reader_settings::toggle_text_align_menu(
            &mut self.children,
            current_align,
            rect,
            enable,
            rq,
            context,
        );
    }

    fn toggle_line_height_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let current_height = self
            .info
            .reader
            .as_ref()
            .and_then(|r| r.line_height)
            .unwrap_or(context.settings.reader.line_height);
        super::reader_settings::toggle_line_height_menu(
            &mut self.children,
            current_height,
            rect,
            enable,
            rq,
            context,
        );
    }

    fn toggle_contrast_exponent_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_settings::toggle_contrast_exponent_menu(
            &mut self.children,
            self.contrast.exponent,
            rect,
            enable,
            rq,
            context,
        );
    }

    fn toggle_contrast_gray_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_settings::toggle_contrast_gray_menu(
            &mut self.children,
            self.contrast.gray,
            rect,
            enable,
            rq,
            context,
        );
    }

    fn toggle_margin_width_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        let margin_width = self
            .info
            .reader
            .as_ref()
            .and_then(|r| {
                if self.reflowable {
                    r.margin_width
                } else {
                    r.screen_margin_width
                }
            })
            .unwrap_or_else(|| {
                if self.reflowable {
                    context.settings.reader.margin_width
                } else {
                    0
                }
            });
        super::reader_settings::toggle_margin_width_menu(
            &mut self.children,
            margin_width,
            rect,
            enable,
            rq,
            context,
        );
    }

    fn toggle_page_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_settings::toggle_page_menu(
            &mut self.children,
            self.current_page,
            &self.info,
            rect,
            enable,
            rq,
            context,
        );
    }

    fn toggle_margin_cropper_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_settings::toggle_margin_cropper_menu(
            &mut self.children,
            self.current_page,
            &self.info,
            rect,
            enable,
            rq,
            context,
        );
    }

    fn toggle_search_menu(
        &mut self,
        rect: Rectangle,
        enable: Option<bool>,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_search::toggle_search_menu(
            &mut self.children,
            self.search_direction,
            rect,
            enable,
            rq,
            context,
        );
    }

    // -----------------------------------------------------------------------
    // Settings Setters
    // -----------------------------------------------------------------------

    fn set_font_size(
        &mut self,
        font_size: f32,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if Arc::strong_count(&self.doc) > 1 {
            return;
        }

        if let Some(ref mut r) = self.info.reader {
            r.font_size = Some(font_size);
        }

        let (width, height) = context.display.dims;
        {
            let mut doc = self
                .doc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());

            doc.layout(width, height, font_size, CURRENT_DEVICE.dpi);

            if self.synthetic {
                let current_page = self.current_page.min(doc.pages_count() - 1);
                if let Some(location) = doc.resolve_location(Location::Exact(current_page)) {
                    self.current_page = location;
                }
            } else {
                let ratio = doc.pages_count() / self.pages_count;
                self.pages_count = doc.pages_count();
                self.current_page = (ratio * self.current_page).min(self.pages_count - 1);
            }
        }

        self.cache.clear();
        self.text.clear();
        self.update(None, hub, rq, context);
        self.update_tool_bar(rq, context);
        self.update_bottom_bar(rq);
    }

    fn set_text_align(
        &mut self,
        text_align: TextAlign,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if Arc::strong_count(&self.doc) > 1 {
            return;
        }

        if let Some(ref mut r) = self.info.reader {
            r.text_align = Some(text_align);
        }

        {
            let mut doc = self
                .doc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            doc.set_text_align(text_align);

            if self.synthetic {
                let current_page = self.current_page.min(doc.pages_count() - 1);
                if let Some(location) = doc.resolve_location(Location::Exact(current_page)) {
                    self.current_page = location;
                }
            } else {
                self.pages_count = doc.pages_count();
                self.current_page = self.current_page.min(self.pages_count - 1);
            }
        }

        self.cache.clear();
        self.text.clear();
        self.update(None, hub, rq, context);
        self.update_tool_bar(rq, context);
        self.update_bottom_bar(rq);
    }

    fn set_font_family(
        &mut self,
        font_family: &str,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if Arc::strong_count(&self.doc) > 1 {
            return;
        }

        if let Some(ref mut r) = self.info.reader {
            r.font_family = Some(font_family.to_string());
        }

        {
            let mut doc = self
                .doc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let font_path = if font_family == DEFAULT_FONT_FAMILY {
                "fonts"
            } else {
                &context.settings.reader.font_path
            };

            doc.set_font_family(font_family, font_path);

            if self.synthetic {
                let current_page = self.current_page.min(doc.pages_count() - 1);
                if let Some(location) = doc.resolve_location(Location::Exact(current_page)) {
                    self.current_page = location;
                }
            } else {
                self.pages_count = doc.pages_count();
                self.current_page = self.current_page.min(self.pages_count - 1);
            }
        }

        self.cache.clear();
        self.text.clear();
        self.update(None, hub, rq, context);
        self.update_tool_bar(rq, context);
        self.update_bottom_bar(rq);
    }

    fn set_line_height(
        &mut self,
        line_height: f32,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if Arc::strong_count(&self.doc) > 1 {
            return;
        }

        if let Some(ref mut r) = self.info.reader {
            r.line_height = Some(line_height);
        }

        {
            let mut doc = self
                .doc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            doc.set_line_height(line_height);

            if self.synthetic {
                let current_page = self.current_page.min(doc.pages_count() - 1);
                if let Some(location) = doc.resolve_location(Location::Exact(current_page)) {
                    self.current_page = location;
                }
            } else {
                self.pages_count = doc.pages_count();
                self.current_page = self.current_page.min(self.pages_count - 1);
            }
        }

        self.cache.clear();
        self.text.clear();
        self.update(None, hub, rq, context);
        self.update_tool_bar(rq, context);
        self.update_bottom_bar(rq);
    }

    fn set_margin_width(
        &mut self,
        width: i32,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if Arc::strong_count(&self.doc) > 1 {
            return;
        }

        if let Some(ref mut r) = self.info.reader {
            if self.reflowable {
                r.margin_width = Some(width);
            } else {
                if width == 0 {
                    r.screen_margin_width = None;
                } else {
                    r.screen_margin_width = Some(width);
                }
            }
        }

        if self.reflowable {
            let mut doc = self
                .doc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            doc.set_margin_width(width);

            if self.synthetic {
                let current_page = self.current_page.min(doc.pages_count() - 1);
                if let Some(location) = doc.resolve_location(Location::Exact(current_page)) {
                    self.current_page = location;
                }
            } else {
                self.pages_count = doc.pages_count();
                self.current_page = self.current_page.min(self.pages_count - 1);
            }
        } else {
            let next_margin_width = mm_to_px(width as f32, CURRENT_DEVICE.dpi) as i32;
            if self.view_port.zoom_mode == ZoomMode::FitToWidth {
                let ratio = (self.rect.width() as i32 - 2 * next_margin_width) as f32
                    / (self.rect.width() as i32 - 2 * self.view_port.margin_width) as f32;
                self.view_port.page_offset.y = (self.view_port.page_offset.y as f32 * ratio) as i32;
            } else {
                self.view_port.page_offset += pt!(next_margin_width - self.view_port.margin_width);
            }
            self.view_port.margin_width = next_margin_width;
        }

        self.text.clear();
        self.cache.clear();
        self.update(None, hub, rq, context);
        self.update_tool_bar(rq, context);
        self.update_bottom_bar(rq);
    }

    fn toggle_bookmark(&mut self, rq: &mut RenderQueue) {
        super::reader_annotations::toggle_bookmark(
            self.current_page,
            &mut self.info,
            self.id,
            self.rect,
            rq,
        );
    }

    fn set_contrast_exponent(
        &mut self,
        exponent: f32,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_settings::update_contrast_exponent(
            &mut self.info,
            &mut self.contrast,
            exponent,
        );
        self.update(None, hub, rq, context);
        self.update_tool_bar(rq, context);
    }

    fn set_contrast_gray(
        &mut self,
        gray: f32,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_settings::update_contrast_gray(&mut self.info, &mut self.contrast, gray);
        self.update(None, hub, rq, context);
        self.update_tool_bar(rq, context);
    }

    pub(crate) fn set_zoom_mode(
        &mut self,
        zoom_mode: ZoomMode,
        reset_page_offset: bool,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if self.view_port.zoom_mode == zoom_mode {
            return;
        }

        if let Some(index) = locate_by_id(self, ViewId::TitleMenu) {
            self.child_mut(index)
                .child_mut(1)
                .downcast_mut::<MenuEntry>()
                .map(|entry| entry.set_disabled(zoom_mode != ZoomMode::FitToWidth, rq));
        }

        super::reader_settings::update_zoom_mode(
            &mut self.view_port.zoom_mode,
            &mut self.view_port.page_offset,
            zoom_mode,
            reset_page_offset,
        );
        self.cache.clear();
        self.update(None, hub, rq, context);
    }

    pub(crate) fn set_scroll_mode(
        &mut self,
        scroll_mode: ScrollMode,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if self.view_port.scroll_mode == scroll_mode
            || self.view_port.zoom_mode != ZoomMode::FitToWidth
        {
            return;
        }
        super::reader_settings::update_scroll_mode(
            &mut self.view_port.scroll_mode,
            &mut self.view_port.page_offset,
            scroll_mode,
        );
        self.update(None, hub, rq, context);
    }

    fn scaling_factor(
        rect: &Rectangle,
        _margin: &Margin,
        margin_width: i32,
        dims: (f32, f32),
        zoom_mode: ZoomMode,
    ) -> f32 {
        match zoom_mode {
            ZoomMode::FitToPage => {
                let scale_x = (rect.width() as f32 - 2.0 * margin_width as f32) / dims.0;
                let scale_y = (rect.height() as f32 - 2.0 * margin_width as f32) / dims.1;
                scale_x.min(scale_y)
            }
            ZoomMode::FitToWidth => {
                let scale_x = (rect.width() as f32 - 2.0 * margin_width as f32) / dims.0;
                scale_x
            }
            _ => 1.0,
        }
    }

    fn crop_margins(
        &mut self,
        index: usize,
        margin: &Margin,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if self.view_port.zoom_mode != ZoomMode::FitToPage {
            let Some(Resource { pixmap, frame, .. }) = self.cache.get(&index) else {
                return;
            };
            let offset = frame.min + self.view_port.page_offset;
            let dims = {
                let doc = self
                    .doc
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                doc.dims(index).unwrap_or((0.0, 0.0))
            };
            let scale = reader_rendering::scaling_factor(
                &self.rect,
                margin,
                self.view_port.margin_width,
                dims,
                self.view_port.zoom_mode,
            );
            if let Some(new_offset) = reader_rendering::calculate_margin_offset(
                offset,
                pixmap.width,
                pixmap.height,
                margin.left,
                margin.right,
                margin.top,
                margin.bottom,
                scale,
                dims,
            ) {
                self.view_port.page_offset = new_offset;
            }
        }
        if let Some(r) = self.info.reader.as_mut() {
            if r.cropping_margins.is_none() {
                r.cropping_margins = Some(CroppingMargins::Any(Margin::default()));
            }
            for c in r.cropping_margins.iter_mut() {
                *c.margin_mut(index) = margin.clone();
            }
        }
        self.cache.clear();
        self.update(None, hub, rq, context);
    }

    // -----------------------------------------------------------------------
    // Table of Contents and Page Lookup
    // -----------------------------------------------------------------------

    fn toc(&self) -> Option<Vec<TocEntry>> {
        super::reader_settings::build_toc(&self.info, |name| {
            super::reader_settings::find_page_by_name(&self.info, name)
        })
    }

    fn toc_aux(&self, simple_toc: &[SimpleTocEntry], index: &mut usize) -> Vec<TocEntry> {
        super::reader_settings::build_toc_aux(simple_toc, index, |name| {
            super::reader_settings::find_page_by_name(&self.info, name)
        })
    }

    fn find_page_by_name(&self, name: &str) -> Option<usize> {
        super::reader_settings::find_page_by_name(&self.info, name)
    }

    // -----------------------------------------------------------------------
    // Text Excerpt and Selection Geometry
    // -----------------------------------------------------------------------

    fn text_excerpt(&self, sel: [Point; 2]) -> Option<String> {
        reader_rendering::text_excerpt(&self.text, sel, &self.info.language)
    }

    fn selected_text(&self) -> Option<String> {
        self.selection
            .as_ref()
            .and_then(|sel| self.text_excerpt([sel.start, sel.end]))
    }

    fn text_rect(&self, sel: [Point; 2]) -> Option<Rectangle> {
        reader_rendering::text_rect(&self.text, &self.chunks, sel)
    }

    fn render_results(&self, rq: &mut RenderQueue) {
        reader_search::render_results(self.search.as_ref(), &self.chunks, self.id, rq);
    }

    fn selection_rect(&self) -> Option<Rectangle> {
        super::reader_rendering::selection_rect(self.selection.as_ref(), &self.text, &self.chunks)
    }

    // -----------------------------------------------------------------------
    // Annotation Lookup and UI Reseed
    // -----------------------------------------------------------------------

    fn find_annotation_ref(&mut self, sel: [TextLocation; 2]) -> Option<&Annotation> {
        super::reader_annotations::find_annotation_ref(&self.info, sel)
    }

    fn find_annotation_mut(&mut self, sel: [TextLocation; 2]) -> Option<&mut Annotation> {
        super::reader_annotations::find_annotation_mut(&mut self.info, sel)
    }

    fn reseed(&mut self, rq: &mut RenderQueue, context: &mut Context) {
        if let Some(index) = locate::<TopBar>(self) {
            if let Some(top_bar) = self.child_mut(index).downcast_mut::<TopBar>() {
                top_bar.reseed(rq, context);
            }
        }

        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
    }

    // -----------------------------------------------------------------------
    // Quit and State Persistence
    // -----------------------------------------------------------------------

    pub(crate) fn quit(&mut self, context: &mut Context) {
        if let Some(ref mut s) = self.search {
            s.running.store(false, atomic::Ordering::Relaxed);
        }

        if self.ephemeral {
            return;
        }

        if let Some(ref mut r) = self.info.reader {
            r.current_page = self.current_page;
            r.pages_count = self.pages_count;
            r.finished = self.finished;
            r.dithered = context.fb.dithered();

            if self.view_port.zoom_mode == ZoomMode::FitToPage {
                r.zoom_mode = None;
                r.page_offset = None;
            } else {
                r.zoom_mode = Some(self.view_port.zoom_mode);
                r.page_offset = Some(self.view_port.page_offset);
            }

            if self.view_port.zoom_mode == ZoomMode::FitToWidth {
                r.scroll_mode = Some(self.view_port.scroll_mode);
            } else {
                r.scroll_mode = None;
            }

            r.rotation = Some(CURRENT_DEVICE.to_canonical(context.display.rotation));

            if (self.contrast.exponent - DEFAULT_CONTRAST_EXPONENT).abs() > f32::EPSILON {
                r.contrast_exponent = Some(self.contrast.exponent);
                if (self.contrast.gray - DEFAULT_CONTRAST_GRAY).abs() > f32::EPSILON {
                    r.contrast_gray = Some(self.contrast.gray);
                } else {
                    r.contrast_gray = None;
                }
            } else {
                r.contrast_exponent = None;
                r.contrast_gray = None;
            }

            context.library.sync_reader_info(&self.info.file.path, r);
        }
    }

    // -----------------------------------------------------------------------
    // Page Scaling (Pinch/Spread Zoom)
    // -----------------------------------------------------------------------

    pub(crate) fn scale_page(
        &mut self,
        center: Point,
        factor: f32,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        if self.cache.is_empty() {
            return;
        }

        let current_factor = if let ZoomMode::Custom(sf) = self.view_port.zoom_mode {
            sf
        } else {
            self.cache[&self.current_page].scale
        };

        if let Some(chunk) = self.chunks.iter().find(|chunk| {
            let chunk_rect = chunk.frame - chunk.frame.min + chunk.position;
            chunk_rect.includes(center)
        }) {
            let smw = self.view_port.margin_width;
            let frame = self.cache[&chunk.location].frame;
            self.current_page = chunk.location;
            self.view_port.page_offset = Point::from(
                factor * Vec2::from(center - chunk.position + chunk.frame.min - frame.min),
            ) - pt!(
                self.rect.width() as i32 / 2 - smw,
                self.rect.height() as i32 / 2 - smw
            );

            self.set_zoom_mode(
                ZoomMode::Custom(current_factor * factor),
                false,
                hub,
                rq,
                context,
            );
        }
    }

    // ===========================================================================
    // View Trait Implementation
    // ===========================================================================

    // -----------------------------------------------------------------------
    // Event Handling
    // -----------------------------------------------------------------------

    pub(crate) fn handle_menu_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match evt {
            Event::Update(mode) => {
                self.update(Some(*mode), hub, rq, context);
                true
            }
            Event::LoadPixmap(location) => {
                self.load_pixmap(*location, hub, rq, context);
                true
            }
            Event::Submit(ViewId::GoToPageInput, ref text) => {
                self.handle_go_to_page_submit(text.parse().unwrap_or(0), hub, rq, context);
                true
            }
            Event::Submit(ViewId::GoToResultsPageInput, ref text) => {
                if let Ok(index) = text.parse::<usize>() {
                    self.go_to_results_page(index.saturating_sub(1), hub, rq, context);
                }
                true
            }
            Event::Submit(ViewId::NamePageInput, ref text) => {
                if !text.is_empty() {
                    if let Some(ref mut r) = self.info.reader {
                        r.page_names.insert(self.current_page, text.to_string());
                    }
                }
                self.toggle_keyboard(false, None, hub, rq, context);
                true
            }
            Event::Submit(ViewId::EditNoteInput, ref note) => {
                self.handle_edit_note_submit(note, hub, rq, context);
                true
            }
            Event::Submit(ViewId::ReaderSearchInput, ref text) => {
                self.handle_search_submit(text, hub, rq, context);
                true
            }
            Event::Page(dir) => {
                self.go_to_neighbor(*dir, hub, rq, context);
                true
            }
            Event::GoTo(location) | Event::Select(EntryId::GoTo(location)) => {
                self.go_to_page(*location, true, hub, rq, context);
                true
            }
            Event::GoToLocation(ref location) => {
                self.handle_go_to_location(location, hub, rq, context);
                true
            }
            Event::Chapter(dir) => {
                self.go_to_chapter(*dir, hub, rq, context);
                true
            }
            Event::ResultsPage(dir) => {
                self.go_to_results_neighbor(*dir, hub, rq, context);
                true
            }
            Event::CropMargins(ref margin) => {
                let current_page = self.current_page;
                self.crop_margins(current_page, margin.as_ref(), hub, rq, context);
                true
            }
            Event::Toggle(ViewId::TopBottomBars) => {
                self.toggle_bars(None, hub, rq, context);
                true
            }
            Event::Toggle(ViewId::GoToPage) => {
                self.toggle_go_to_page(None, ViewId::GoToPage, hub, rq, context);
                true
            }
            Event::Toggle(ViewId::GoToResultsPage) => {
                self.toggle_go_to_page(None, ViewId::GoToResultsPage, hub, rq, context);
                true
            }
            Event::Slider(SliderId::FontSize, font_size, FingerStatus::Up) => {
                self.set_font_size(*font_size, hub, rq, context);
                true
            }
            Event::Slider(SliderId::ContrastExponent, exponent, FingerStatus::Up) => {
                self.set_contrast_exponent(*exponent, hub, rq, context);
                true
            }
            Event::Slider(SliderId::ContrastGray, gray, FingerStatus::Up) => {
                self.set_contrast_gray(*gray, hub, rq, context);
                true
            }
            Event::ToggleNear(ViewId::TitleMenu, rect) => {
                self.toggle_title_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::MainMenu, rect) => {
                toggle_main_menu(self, *rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::BatteryMenu, rect) => {
                toggle_battery_menu(self, *rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::ClockMenu, rect) => {
                toggle_clock_menu(self, *rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::MarginCropperMenu, rect) => {
                self.toggle_margin_cropper_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::SearchMenu, rect) => {
                self.toggle_search_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::FontFamilyMenu, rect) => {
                self.toggle_font_family_menu(*rect, None, hub, rq, context);
                true
            }
            Event::ToggleNear(ViewId::FontSizeMenu, rect) => {
                self.toggle_font_size_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::TextAlignMenu, rect) => {
                self.toggle_text_align_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::MarginWidthMenu, rect) => {
                self.toggle_margin_width_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::LineHeightMenu, rect) => {
                self.toggle_line_height_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::ContrastExponentMenu, rect) => {
                self.toggle_contrast_exponent_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::ContrastGrayMenu, rect) => {
                self.toggle_contrast_gray_menu(*rect, None, rq, context);
                true
            }
            Event::ToggleNear(ViewId::PageMenu, rect) => {
                self.toggle_page_menu(*rect, None, rq, context);
                true
            }
            Event::Close(ViewId::MainMenu) => {
                toggle_main_menu(self, Rectangle::default(), Some(false), rq, context);
                true
            }
            Event::Close(ViewId::SearchBar) => {
                self.handle_close_search_bar(hub, rq, context);
                true
            }
            Event::Close(ViewId::GoToPage) => {
                self.toggle_go_to_page(Some(false), ViewId::GoToPage, hub, rq, context);
                true
            }
            Event::Close(ViewId::GoToResultsPage) => {
                self.toggle_go_to_page(Some(false), ViewId::GoToResultsPage, hub, rq, context);
                true
            }
            Event::Close(ViewId::SelectionMenu) => {
                if self.state == State::Idle && self.target_annotation.is_none() {
                    if let Some(rect) = self.selection_rect() {
                        self.selection = None;
                        rq.add(RenderData::new(self.id, rect, UpdateMode::Gui));
                    }
                }
                false
            }
            Event::Close(ViewId::EditNote) => {
                self.handle_close_edit_note(hub, rq, context);
                false
            }
            Event::Close(ViewId::NamePage) => {
                self.toggle_keyboard(false, None, hub, rq, context);
                false
            }
            Event::Show(ViewId::TableOfContents) => {
                self.handle_show_table_of_contents(hub, rq, context);
                true
            }
            Event::Select(EntryId::Annotations) => {
                self.handle_show_annotations(hub, rq, context);
                true
            }
            Event::Select(EntryId::Bookmarks) => {
                self.handle_show_bookmarks(hub, rq, context);
                true
            }
            Event::Show(ViewId::SearchBar) => {
                self.toggle_search_bar(true, hub, rq, context);
                true
            }
            Event::Show(ViewId::MarginCropper) => {
                self.toggle_margin_cropper(hub, rq, context);
                true
            }
            Event::Close(ViewId::MarginCropper) => {
                self.toggle_margin_cropper(hub, rq, context);
                true
            }
            Event::SearchResult(location, ref _rects) => {
                self.handle_search_result(*location, hub, rq, context);
                true
            }
            Event::EndOfSearch => {
                self.handle_end_of_search(hub, rq, context);
                true
            }
            Event::Select(EntryId::AnnotateSelection) => {
                self.toggle_edit_note(None, Some(true), hub, rq, context);
                true
            }
            Event::Select(EntryId::HighlightSelection) => {
                self.handle_highlight_selection(hub, rq, context);
                true
            }
            Event::Select(EntryId::DefineSelection) => {
                self.handle_define_selection(hub, rq, context);
                true
            }
            Event::Select(EntryId::SearchForSelection) => {
                self.handle_search_for_selection(hub, rq, context);
                true
            }
            Event::Select(EntryId::GoToSelectedPageName) => {
                self.handle_go_to_selected_page_name(hub, rq, context);
                true
            }
            Event::Select(EntryId::AdjustSelection) => {
                self.state = State::AdjustSelection;
                true
            }
            Event::Select(EntryId::EditAnnotationNote(_sel)) => {
                self.handle_edit_annotation_note(hub, rq, context);
                true
            }
            Event::Select(EntryId::RemoveAnnotationNote(_sel)) => {
                self.handle_remove_annotation_note(hub, rq, context);
                true
            }
            Event::Select(EntryId::RemoveAnnotation(_sel)) => {
                self.handle_remove_annotation(hub, rq, context);
                true
            }
            Event::Select(EntryId::SetZoomMode(zoom_mode)) => {
                self.set_zoom_mode(*zoom_mode, true, hub, rq, context);
                true
            }
            Event::Select(EntryId::SetScrollMode(scroll_mode)) => {
                self.set_scroll_mode(*scroll_mode, hub, rq, context);
                true
            }
            Event::Select(EntryId::Save) => {
                self.handle_save(hub, rq, context);
                true
            }
            Event::Select(EntryId::ApplyCroppings(index, scheme)) => {
                self.info.reader.as_mut().map(|r| {
                    if r.cropping_margins.is_none() {
                        r.cropping_margins = Some(CroppingMargins::Any(Margin::default()));
                    }
                    r.cropping_margins.as_mut().map(|cm| {
                        cm.apply(*index, *scheme);
                    })
                });
                true
            }
            Event::Select(EntryId::RemoveCroppings) => {
                if let Some(r) = self.info.reader.as_mut() {
                    r.cropping_margins = None;
                }
                self.cache.clear();
                self.update(None, hub, rq, context);
                true
            }
            Event::Select(EntryId::SearchDirection(dir)) => {
                self.search_direction = *dir;
                true
            }
            Event::Select(EntryId::SetFontFamily(ref font_family)) => {
                self.set_font_family(font_family, hub, rq, context);
                true
            }
            Event::Select(EntryId::SetTextAlign(text_align)) => {
                self.set_text_align(*text_align, hub, rq, context);
                true
            }
            Event::Select(EntryId::SetFontSize(v)) => {
                let font_size = self
                    .info
                    .reader
                    .as_ref()
                    .and_then(|r| r.font_size)
                    .unwrap_or(context.settings.reader.font_size);
                let font_size = font_size - 1.0 + *v as f32 / 10.0;
                self.set_font_size(font_size, hub, rq, context);
                true
            }
            Event::Select(EntryId::SetMarginWidth(width)) => {
                self.set_margin_width(*width, hub, rq, context);
                true
            }
            Event::Select(EntryId::SetLineHeight(v)) => {
                let line_height = 1.0 + *v as f32 / 10.0;
                self.set_line_height(line_height, hub, rq, context);
                true
            }
            Event::Select(EntryId::SetContrastExponent(v)) => {
                let exponent = 1.0 + *v as f32 / 2.0;
                self.set_contrast_exponent(exponent, hub, rq, context);
                true
            }
            Event::Select(EntryId::SetContrastGray(v)) => {
                let gray = ((1 << 8) - (1 << (8 - *v))) as f32;
                self.set_contrast_gray(gray, hub, rq, context);
                true
            }
            Event::Select(EntryId::SetPageName) => {
                self.toggle_name_page(None, hub, rq, context);
                true
            }
            Event::Select(EntryId::RemovePageName) => {
                if let Some(ref mut r) = self.info.reader {
                    r.page_names.remove(&self.current_page);
                }
                true
            }
            Event::Select(EntryId::ToggleInverted) => {
                self.update_noninverted_regions(rq);
                false
            }
            Event::Reseed => {
                self.reseed(rq, context);
                true
            }
            Event::ToggleFrontlight => {
                if let Some(index) = locate::<TopBar>(self) {
                    self.child_mut(index)
                        .downcast_mut::<TopBar>()
                        .map(|tb: &mut TopBar| tb.update_frontlight_icon(rq, context));
                }
                true
            }
            Event::Select(EntryId::Quit)
            | Event::Select(EntryId::Reboot)
            | Event::Back
            | Event::Suspend => {
                self.quit(context);
                false
            }
            Event::Focus(v) => {
                self.handle_focus(v.is_some(), hub, rq, context);
                true
            }
            _ => false,
        }
    }

    pub(crate) fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        _bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        // Delegate to specialized handlers
        if let Event::Gesture(ref gesture_evt) = evt {
            if self.handle_gesture_event(gesture_evt, hub, rq, context) {
                return true;
            }
        }

        if let Event::Device(ref device_evt) = evt {
            if self.handle_button_event(device_evt, hub, rq, context) {
                return true;
            }
        }

        if self.handle_menu_event(evt, hub, rq, context) {
            return true;
        }

        // Handle remaining device events
        match *evt {
            Event::Device(DeviceEvent::Button {
                code: ButtonCode::Home,
                status: ButtonStatus::Pressed,
                ..
            }) => {
                self.quit(context);
                hub.send(Event::Back).ok();
                true
            }
            _ => false,
        }
    }

    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, _fonts: &mut Fonts) {
        fb.draw_rectangle(&rect, background(theme::is_dark_mode()));

        for chunk in &self.chunks {
            let Resource {
                ref pixmap, scale, ..
            } = self.cache[&chunk.location];
            let chunk_rect = chunk.frame - chunk.frame.min + chunk.position;

            if let Some(region_rect) = rect.intersection(&chunk_rect) {
                let chunk_frame = region_rect - chunk.position + chunk.frame.min;
                let chunk_position = region_rect.min;
                fb.draw_framed_pixmap_contrast(
                    pixmap,
                    &chunk_frame,
                    chunk_position,
                    self.contrast.exponent,
                    self.contrast.gray,
                );

                if let Some(rects) = self.noninverted_regions.get(&chunk.location) {
                    for r in rects {
                        let rect = (*r * scale).to_rect() - chunk.frame.min + chunk.position;
                        if let Some(ref image_rect) = rect.intersection(&region_rect) {
                            fb.invert_region(image_rect);
                        }
                    }
                }

                if let Some(groups) = self
                    .search
                    .as_ref()
                    .and_then(|s| s.highlights.get(&chunk.location))
                {
                    for rect_ref in groups {
                        let mut last_rect: Option<Rectangle> = None;
                        let rect = *rect_ref - chunk.frame.min + chunk.position;
                        if let Some(ref search_rect) = rect.intersection(&region_rect) {
                            fb.invert_region(search_rect);
                        }
                        if let Some(last) = last_rect {
                            if rect.max.y.min(last.max.y) - rect.min.y.max(last.min.y)
                                > rect.height().min(last.height()) as i32 / 2
                                && (last.max.x < rect.min.x || rect.max.x < last.min.x)
                            {
                                let space = if last.max.x < rect.min.x {
                                    rect![
                                        last.max.x,
                                        (last.min.y + rect.min.y) / 2,
                                        rect.min.x,
                                        (last.max.y + rect.max.y) / 2
                                    ]
                                } else {
                                    rect![
                                        rect.max.x,
                                        (last.min.y + rect.min.y) / 2,
                                        last.min.x,
                                        (last.max.y + rect.max.y) / 2
                                    ]
                                };
                                if let Some(ref res_rect) = space.intersection(&region_rect) {
                                    fb.invert_region(res_rect);
                                }
                            }
                        }
                        let _ = last_rect.replace(rect);
                    }
                }

                if let Some(annotations) = self.annotations.get(&chunk.location) {
                    for annot in annotations {
                        let drift = if annot.note.is_empty() {
                            HIGHLIGHT_DRIFT
                        } else {
                            ANNOTATION_DRIFT
                        };
                        let drift_u8 = (drift * 255.0).clamp(0.0, 255.0) as u8;
                        if let Some(text) = self.text.get(&chunk.location) {
                            let mut last_rect: Option<Rectangle> = None;
                            for word in text.iter() {
                                let rect = (word.rect * scale).to_rect() - chunk.frame.min
                                    + chunk.position;
                                if let Some(ref sel_rect) = rect.intersection(&region_rect) {
                                    fb.shift_region(sel_rect, drift_u8);
                                }
                                if let Some(last) = last_rect {
                                    if rect.max.y.min(last.max.y) - rect.min.y.max(last.min.y)
                                        > rect.height().min(last.height()) as i32 / 2
                                        && (last.max.x < rect.min.x || rect.max.x < last.min.x)
                                    {
                                        let space = if last.max.x < rect.min.x {
                                            rect![
                                                last.max.x,
                                                (last.min.y + rect.min.y) / 2,
                                                rect.min.x,
                                                (last.max.y + rect.max.y) / 2
                                            ]
                                        } else {
                                            rect![
                                                rect.max.x,
                                                (last.min.y + rect.min.y) / 2,
                                                last.min.x,
                                                (last.max.y + rect.max.y) / 2
                                            ]
                                        };
                                        if let Some(ref sel_rect) = space.intersection(&region_rect)
                                        {
                                            fb.shift_region(sel_rect, drift_u8);
                                        }
                                    }
                                }
                                let _ = last_rect.replace(rect);
                            }
                        }
                    }
                }

                if let Some(sel) = self.selection.as_ref() {
                    if let Some(text) = self.text.get(&chunk.location) {
                        let mut last_rect: Option<Rectangle> = None;
                        for word in text
                            .iter()
                            .filter(|w| w.location >= sel.start && w.location <= sel.end)
                        {
                            let rect =
                                (word.rect * scale).to_rect() - chunk.frame.min + chunk.position;
                            if let Some(ref sel_rect) = rect.intersection(&region_rect) {
                                fb.invert_region(sel_rect);
                            }
                            if let Some(last) = last_rect {
                                if rect.max.y.min(last.max.y) - rect.min.y.max(last.min.y)
                                    > rect.height().min(last.height()) as i32 / 2
                                    && (last.max.x < rect.min.x || rect.max.x < last.min.x)
                                {
                                    let space = if last.max.x < rect.min.x {
                                        rect![
                                            last.max.x,
                                            (last.min.y + rect.min.y) / 2,
                                            rect.min.x,
                                            (last.max.y + rect.max.y) / 2
                                        ]
                                    } else {
                                        rect![
                                            rect.max.x,
                                            (last.min.y + rect.min.y) / 2,
                                            last.min.x,
                                            (last.max.y + rect.max.y) / 2
                                        ]
                                    };
                                    if let Some(ref sel_rect) = space.intersection(&region_rect) {
                                        fb.invert_region(sel_rect);
                                    }
                                }
                            }
                            last_rect = Some(rect);
                        }
                    }
                }
            }
        }

        if self
            .info
            .reader
            .as_ref()
            .map_or(false, |r| r.bookmarks.contains(&self.current_page))
        {
            let dpi = CURRENT_DEVICE.dpi;
            let thickness = scale_by_dpi(3.0, dpi) as u16;
            let radius = mm_to_px(0.4, dpi) as i32 + thickness as i32;
            let center = pt!(self.rect.max.x - 5 * radius, self.rect.min.y + 5 * radius);
            fb.draw_rounded_rectangle_with_border(
                &Rectangle::from_disk(center, radius),
                &CornerSpec::Uniform(radius),
                &BorderSpec {
                    thickness,
                    color: background(theme::is_dark_mode()),
                },
                &foreground(theme::is_dark_mode()),
            );
        }
    }

    // -----------------------------------------------------------------------
    // Rendering
    // -----------------------------------------------------------------------

    fn render_rect(&self, rect: &Rectangle) -> Rectangle {
        rect.intersection(&self.rect).unwrap_or(self.rect)
    }

    fn resize(&mut self, rect: Rectangle, hub: &Hub, rq: &mut RenderQueue, context: &mut Context) {
        if !self.children.is_empty() {
            let dpi = CURRENT_DEVICE.dpi;
            let thickness = scale_by_dpi(THICKNESS_MEDIUM, dpi) as i32;
            let (small_thickness, big_thickness) = halves(thickness);
            let (small_height, big_height) = (
                scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32,
                scale_by_dpi(BIG_BAR_HEIGHT, dpi) as i32,
            );
            let mut floating_layer_start = 0;

            self.children.retain(|child| !child.is::<Menu>());

            if self.children[0].is::<TopBar>() {
                let top_bar_rect = rect![
                    rect.min.x,
                    rect.min.y,
                    rect.max.x,
                    small_height - small_thickness
                ];
                self.children[0].resize(top_bar_rect, hub, rq, context);
                let separator_rect = rect![
                    rect.min.x,
                    small_height - small_thickness,
                    rect.max.x,
                    small_height + big_thickness
                ];
                self.children[1].resize(separator_rect, hub, rq, context);
            } else if self.children[0].is::<Filler>() {
                let mut index = 1;
                if self.children[index].is::<SearchBar>() {
                    let sb_rect = rect![
                        rect.min.x,
                        rect.max.y - (3 * big_height + 2 * small_height) as i32 + big_thickness,
                        rect.max.x,
                        rect.max.y - (3 * big_height + small_height) as i32 - small_thickness
                    ];
                    self.children[index].resize(sb_rect, hub, rq, context);
                    self.children[index - 1].resize(
                        rect![
                            rect.min.x,
                            sb_rect.min.y - thickness,
                            rect.max.x,
                            sb_rect.min.y
                        ],
                        hub,
                        rq,
                        context,
                    );
                    index += 2;
                }
                if self.children[index].is::<Keyboard>() {
                    let kb_rect = rect![
                        rect.min.x,
                        rect.max.y - (small_height + 3 * big_height) as i32 + big_thickness,
                        rect.max.x,
                        rect.max.y - small_height - small_thickness
                    ];
                    self.children[index].resize(kb_rect, hub, rq, context);
                    self.children[index + 1].resize(
                        rect![
                            rect.min.x,
                            kb_rect.max.y,
                            rect.max.x,
                            kb_rect.max.y + thickness
                        ],
                        hub,
                        rq,
                        context,
                    );
                    let kb_rect = *self.children[index].rect();
                    self.children[index - 1].resize(
                        rect![
                            rect.min.x,
                            kb_rect.min.y - thickness,
                            rect.max.x,
                            kb_rect.min.y
                        ],
                        hub,
                        rq,
                        context,
                    );
                    index += 2;
                }
                floating_layer_start = index;
            }

            if let Some(mut index) = locate::<BottomBar>(self) {
                floating_layer_start = index + 1;
                let separator_rect = rect![
                    rect.min.x,
                    rect.max.y - small_height - small_thickness,
                    rect.max.x,
                    rect.max.y - small_height + big_thickness
                ];
                self.children[index - 1].resize(separator_rect, hub, rq, context);
                let bottom_bar_rect = rect![
                    rect.min.x,
                    rect.max.y - small_height + big_thickness,
                    rect.max.x,
                    rect.max.y
                ];
                self.children[index].resize(bottom_bar_rect, hub, rq, context);

                index -= 2;

                while index > 2 {
                    let bar_height = if self.children[index].is::<ToolBar>() {
                        2 * big_height
                    } else if self.children[index].is::<Keyboard>() {
                        3 * big_height
                    } else {
                        small_height
                    } as i32;

                    let y_max = self.children[index + 1].rect().min.y;
                    let bar_rect = rect![
                        rect.min.x,
                        y_max - bar_height + thickness,
                        rect.max.x,
                        y_max
                    ];
                    self.children[index].resize(bar_rect, hub, rq, context);
                    let y_max = self.children[index].rect().min.y;
                    let sp_rect = rect![rect.min.x, y_max - thickness, rect.max.x, y_max];
                    self.children[index - 1].resize(sp_rect, hub, rq, context);

                    index -= 2;
                }
            }

            for i in floating_layer_start..self.children.len() {
                self.children[i].resize(rect, hub, rq, context);
            }
        }

        match self.view_port.zoom_mode {
            ZoomMode::FitToWidth => {
                let ratio = (rect.width() as i32 - 2 * self.view_port.margin_width) as f32
                    / (self.rect.width() as i32 - 2 * self.view_port.margin_width) as f32;
                self.view_port.page_offset.y = (self.view_port.page_offset.y as f32 * ratio) as i32;
            }
            ZoomMode::Custom(_) => {
                self.view_port.page_offset += pt!(
                    self.rect.width() as i32 - rect.width() as i32,
                    self.rect.height() as i32 - rect.height() as i32
                ) / 2;
            }
            _ => (),
        }

        self.rect = rect;

        if self.reflowable {
            let font_size = self
                .info
                .reader
                .as_ref()
                .and_then(|r| r.font_size)
                .unwrap_or(context.settings.reader.font_size);
            let mut doc = self
                .doc
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            doc.layout(rect.width(), rect.height(), font_size, CURRENT_DEVICE.dpi);
            let current_page = self.current_page.min(doc.pages_count() - 1);
            if let Some(location) = doc.resolve_location(Location::Exact(current_page)) {
                self.current_page = location;
            }
            self.text.clear();
        }

        self.cache.clear();
        self.update(Some(UpdateMode::Full), hub, rq, context);
    }

    fn might_rotate(&self) -> bool {
        self.search.is_none()
    }

    fn is_background(&self) -> bool {
        true
    }

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
}

// ===========================================================================
// Stub Method Declarations (Reader trait interface)
// ===========================================================================

impl Reader {
    pub fn update(
        &mut self,
        _update: Option<UpdateMode>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn update_tool_bar(&mut self, rq: &mut RenderQueue, _context: &Context) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn update_bottom_bar(&mut self, rq: &mut RenderQueue) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_save(&mut self, _hub: &Hub, rq: &mut RenderQueue, _context: &mut Context) {
        // Save functionality would be implemented here
        // For now, just trigger a partial update
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_focus(
        &mut self,
        _v: bool,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        // Focus handling would be implemented here
        // For now, just trigger a partial update
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn update_annotations(&mut self, _hub: &Hub, rq: &mut RenderQueue, _context: &mut Context) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn update_noninverted_regions(&mut self, rq: &mut RenderQueue) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn go_to_chapter(
        &mut self,
        _dir: CycleDir,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn go_to_results_neighbor(
        &mut self,
        dir: CycleDir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_search::go_to_results_neighbor(dir, self, hub, rq, context);
    }

    pub fn go_to_results_page(
        &mut self,
        index: usize,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_search::go_to_results_page(index, self, hub, rq, context);
    }

    pub fn go_to_bookmark(
        &mut self,
        _dir: CycleDir,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn go_to_annotation(
        &mut self,
        _dir: CycleDir,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn go_to_last_page(&mut self, _hub: &Hub, rq: &mut RenderQueue, _context: &mut Context) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn directional_scroll(
        &mut self,
        _delta: Point,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn vertical_scroll(
        &mut self,
        _distance: i32,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn toggle_bars(
        &mut self,
        _show: Option<bool>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn toggle_keyboard(
        &mut self,
        _enable: bool,
        _update: Option<UpdateMode>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn toggle_search_bar(
        &mut self,
        _enable: bool,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn toggle_margin_cropper(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn search(
        &mut self,
        _query: &str,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn load_pixmap(
        &mut self,
        _page_index: usize,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_go_to_page_submit(
        &mut self,
        _page: usize,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_edit_note_submit(
        &mut self,
        _note: &str,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_search_submit(
        &mut self,
        _query: &str,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_go_to_location(
        &mut self,
        _location: &Location,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_close_search_bar(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_close_edit_note(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_show_table_of_contents(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_show_annotations(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_show_bookmarks(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_search_result(
        &mut self,
        _result: usize,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_end_of_search(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_highlight_selection(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_define_selection(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_search_for_selection(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_go_to_selected_page_name(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_edit_annotation_note(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_remove_annotation_note(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_remove_annotation(
        &mut self,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_device_event(
        &mut self,
        _device_event: &crate::input::DeviceEvent,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_keyboard(
        &mut self,
        _key_code: crate::view::key::KeyKind,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_shown(&mut self, _hub: &Hub, rq: &mut RenderQueue, _context: &mut Context) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_open(
        &mut self,
        _file: &Box<crate::metadata::Info>,
        _hub: &Hub,
        rq: &mut RenderQueue,
        _context: &mut Context,
    ) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }

    pub fn handle_back(&mut self, _hub: &Hub, rq: &mut RenderQueue, _context: &mut Context) {
        rq.add(RenderData::new(self.id, self.rect, UpdateMode::Partial));
    }
}

impl View for Reader {
    fn handle_event(
        &mut self,
        evt: &Event,
        hub: &Hub,
        _bus: &mut Bus,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) -> bool {
        match evt {
            Event::Gesture(gesture_event) => {
                self.handle_gesture_event(gesture_event, hub, rq, context);
                true
            }
            Event::Device(device_event) => {
                self.handle_device_event(device_event, hub, rq, context);
                true
            }
            Event::Key(key_code) => {
                self.handle_keyboard(*key_code, hub, rq, context);
                true
            }
            Event::Update(_update_mode) => {
                rq.add(RenderData::new(self.id, self.rect, UpdateMode::Gui));
                true
            }
            Event::Focus(_view_id) => {
                self.handle_shown(hub, rq, context);
                true
            }
            Event::Open(file) => {
                self.handle_open(file, hub, rq, context);
                true
            }
            Event::Save => {
                self.handle_save(hub, rq, context);
                true
            }
            Event::Back => {
                self.handle_back(hub, rq, context);
                true
            }
            _ => false,
        }
    }

    fn render(&self, _fb: &mut dyn Framebuffer, _rect: Rectangle, _fonts: &mut Fonts) {
        // Implementation would go here
    }

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
}
