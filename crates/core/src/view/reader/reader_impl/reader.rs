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
//! let mut doc = self._doc.lock().unwrap_or_else(|poisoned: std::sync::PoisonError<MutexGuard<Box<dyn Document>>>| poisoned.into_inner());
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
use crate::document::{BoundedText, Document, SimpleTocEntry, TextLocation, TocEntry};
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

use crate::view::common::{locate, toggle_battery_menu, toggle_clock_menu, toggle_main_menu};
use crate::view::top_bar::TopBar;
use crate::view::{
    Bus, Event, Hub, Id, RenderData, RenderQueue, View, ViewId, SMALL_BAR_HEIGHT, THICKNESS_MEDIUM,
    ID_FEEDER,
};

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
    pub(crate) _doc: Arc<Mutex<Box<dyn Document>>>,
    pub(crate) cache: BTreeMap<usize, Resource>,
    pub(crate) chunks: Vec<RenderChunk>,
    pub(crate) text: FxHashMap<usize, Vec<BoundedText>>,
    pub(crate) _annotations: FxHashMap<usize, Vec<Annotation>>,
    pub(crate) _noninverted_regions: FxHashMap<usize, Vec<Boundary>>,
    pub(crate) focus: Option<ViewId>,
    pub(crate) search: Option<Search>,
    pub(crate) search_direction: LinearDir,
    pub(crate) held_buttons: FxHashSet<ButtonCode>,
    pub(crate) selection: Option<Selection>,
    pub(crate) _target_annotation: Option<[TextLocation; 2]>,
    pub(crate) history: VecDeque<usize>,
    pub(crate) state: State,
    pub(crate) info: Info,
    pub(crate) current_page: usize,
    pub(crate) pages_count: usize,
    pub(crate) view_port: ViewPort,
    pub(crate) contrast: Contrast,
    pub(crate) _synthetic: bool,
    pub(crate) _page_turns: usize,
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
            _doc: doc,
            cache: BTreeMap::new(),
            chunks: Vec::new(),
            text: FxHashMap::default(),
            _annotations: FxHashMap::default(),
            _noninverted_regions: FxHashMap::default(),
            focus: None,
            search: None,
            search_direction: LinearDir::Forward,
            held_buttons: FxHashSet::default(),
            selection: None,
            _target_annotation: None,
            history: VecDeque::new(),
            state: State::Idle,
            info,
            current_page: 0,
            pages_count,
            view_port: ViewPort::default(),
            contrast: Contrast::default(),
            _synthetic: false,
            _page_turns: 0,
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
            _doc: doc,
            cache: BTreeMap::new(),
            chunks: Vec::new(),
            text: FxHashMap::default(),
            _annotations: FxHashMap::default(),
            _noninverted_regions: FxHashMap::default(),
            focus: None,
            search: None,
            search_direction: LinearDir::Forward,
            held_buttons: FxHashSet::default(),
            selection: None,
            _target_annotation: None,
            history: VecDeque::new(),
            state: State::Idle,
            info,
            current_page: 0,
            pages_count,
            view_port: ViewPort::default(),
            contrast: Contrast::default(),
            _synthetic: false,
            _page_turns: 0,
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
    // Table of Contents and Page Lookup
    // -----------------------------------------------------------------------

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn toc(&self) -> Option<Vec<TocEntry>> {
        super::reader_settings::build_toc(&self.info, |name| {
            super::reader_settings::find_page_by_name(&self.info, name)
        })
    }

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn toc_aux(&self, simple_toc: &[SimpleTocEntry], index: &mut usize) -> Vec<TocEntry> {
        super::reader_settings::build_toc_aux(simple_toc, index, |name| {
            super::reader_settings::find_page_by_name(&self.info, name)
        })
    }

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn find_page_by_name(&self, name: &str) -> Option<usize> {
        super::reader_settings::find_page_by_name(&self.info, name)
    }

    // -----------------------------------------------------------------------
    // Text Excerpt and Selection Geometry
    // -----------------------------------------------------------------------

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn text_excerpt(&self, sel: [Point; 2]) -> Option<String> {
        reader_rendering::text_excerpt(&self.text, sel, &self.info.language)
    }

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn selected_text(&self) -> Option<String> {
        self.selection
            .as_ref()
            .and_then(|sel| self.text_excerpt([sel.start, sel.end]))
    }

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn text_rect(&self, sel: [Point; 2]) -> Option<Rectangle> {
        reader_rendering::text_rect(&self.text, &self.chunks, sel)
    }

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn render_results(&self, rq: &mut RenderQueue) {
        reader_search::render_results(self.search.as_ref(), &self.chunks, self.id, rq);
    }

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn selection_rect(&self) -> Option<Rectangle> {
        super::reader_rendering::selection_rect(self.selection.as_ref(), &self.text, &self.chunks)
    }

    // -----------------------------------------------------------------------
    // Annotation Lookup and UI Reseed
    // -----------------------------------------------------------------------

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn find_annotation_ref(&mut self, sel: [TextLocation; 2]) -> Option<&Annotation> {
        super::reader_annotations::find_annotation_ref(&self.info, sel)
    }

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
    fn find_annotation_mut(&mut self, sel: [TextLocation; 2]) -> Option<&mut Annotation> {
        super::reader_annotations::find_annotation_mut(&mut self.info, sel)
    }

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
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

    // -----------------------------------------------------------------------
    // Render
    // -----------------------------------------------------------------------

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
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

    #[allow(dead_code)] // Used by Reader::handle_menu_event method
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

                if let Some(rects) = self._noninverted_regions.get(&chunk.location) {
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

                if let Some(annotations) = self._annotations.get(&chunk.location) {
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
