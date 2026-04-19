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

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::document::{BoundedText, Document, TextLocation, TocEntry};
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, Pixmap, UpdateMode};
use crate::geom::LinearDir;
use crate::geom::{BorderSpec, Boundary, CornerSpec, Point, Rectangle, Vec2};
use crate::input::{ButtonCode, ButtonStatus, DeviceEvent};
use crate::log_error;
use crate::metadata::{Annotation, Info, ZoomMode};
use crate::metadata::{DEFAULT_CONTRAST_EXPONENT, DEFAULT_CONTRAST_GRAY};
use crate::theme;
use crate::unit::{mm_to_px, scale_by_dpi};
use anyhow::{Context as AnyhowContext, Error};
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::{BTreeMap, VecDeque};
use std::sync::{atomic, Arc, Mutex};

use crate::view::{
    Bus, Event, Hub, Id, RenderData, RenderQueue, View, ViewId, ID_FEEDER, SMALL_BAR_HEIGHT,
};

use crate::view::reader::tool_bar::ToolBar;

use super::reader_core::{
    AnimState, Contrast, PageAnimKind, PageAnimation, RenderChunk, Resource, Search, Selection,
    State, ViewPort,
};
use super::reader_annotations_ext::ReaderAnnotationManager;
use super::reader_dialog_manager::ReaderDialogManager;
use super::reader_input::ReaderInputHandler;
use super::reader_rendering;
use super::reader_rendering_ext::{ReaderRenderCache, ReaderRenderEngine};
use super::reader_search_handler::ReaderSearchHandler;
use super::reader_settings_ui::ReaderSettingsManager;
use super::reader_state::ReaderStateManager;
use super::reader_toc::ReaderTocManager;

pub const HIGHLIGHT_DRIFT: f32 = 0.1;
pub const ANNOTATION_DRIFT: f32 = 0.05;

// ===========================================================================
// Type Definitions
// ===========================================================================

pub struct Reader {
    pub(crate) id: Id,
    pub(crate) rect: Rectangle,
    pub(crate) children: Vec<Box<dyn View>>,
    pub(crate) _doc: Arc<Mutex<Box<dyn Document>>>,
    pub(crate) cache: BTreeMap<usize, Resource>,
    pub(crate) chunks: Vec<RenderChunk>,
    pub(crate) text: FxHashMap<usize, Vec<BoundedText>>,
    pub(crate) annotation_manager: ReaderAnnotationManager,
    pub(crate) dialog_manager: ReaderDialogManager,
    pub(crate) input_handler: ReaderInputHandler,
    pub(crate) render_cache: ReaderRenderCache,
    pub(crate) render_engine: ReaderRenderEngine,
    pub(crate) search_handler: ReaderSearchHandler,
    pub(crate) settings_manager: ReaderSettingsManager,
    pub(crate) state_manager: ReaderStateManager,
    pub(crate) toc_manager: ReaderTocManager,
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
    pub(crate) bars_visible: bool,
    pub(crate) margin_cropper_visible: bool,
}

// ===========================================================================
// Constructors
// ===========================================================================

impl Reader {
    pub fn new(rect: Rectangle, info: Info, _hub: &Hub, context: &mut Context) -> Option<Reader> {
        let id = ID_FEEDER.next();
        let (doc, pages_count, reflowable) = Self::open_document(&info)?;
        let children = Self::create_toolbar(rect, reflowable, &info, context);

        Some(Self::create_reader(
            id,
            rect,
            children,
            doc,
            pages_count,
            reflowable,
            info,
        ))
    }

    fn open_document(info: &Info) -> Option<(Arc<Mutex<Box<dyn Document>>>, usize, bool)> {
        let doc = match crate::document::open(&info.file.path) {
            Some(d) => d,
            None => {
                log_error!("Failed to open document: {}", info.file.path.display());
                return None;
            }
        };
        let doc = Arc::new(Mutex::new(doc));
        let pages_count = (*doc.lock().expect("doc lock")).pages_count();
        let reflowable = (*doc.lock().expect("doc lock")).is_reflowable();
        Some((doc, pages_count, reflowable))
    }

    fn create_toolbar(
        rect: Rectangle,
        reflowable: bool,
        info: &Info,
        context: &mut Context,
    ) -> Vec<Box<dyn View>> {
        let dpi = crate::unit::get_device_dpi();
        let small_height = scale_by_dpi(SMALL_BAR_HEIGHT, dpi) as i32;

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
        vec![Box::new(tool_bar) as Box<dyn View>]
    }

    fn create_reader(
        id: Id,
        rect: Rectangle,
        children: Vec<Box<dyn View>>,
        doc: Arc<Mutex<Box<dyn Document>>>,
        pages_count: usize,
        reflowable: bool,
        info: Info,
    ) -> Reader {
        Reader {
            id,
            rect,
            children,
            _doc: doc,
            cache: BTreeMap::new(),
            chunks: Vec::new(),
            text: FxHashMap::default(),
            annotation_manager: ReaderAnnotationManager::new(),
            dialog_manager: ReaderDialogManager::new(id),
            input_handler: ReaderInputHandler::new(id),
            render_cache: ReaderRenderCache::new(50 * 1024 * 1024), // 50MB cache
            render_engine: ReaderRenderEngine::new(50 * 1024 * 1024),
            search_handler: ReaderSearchHandler::new(id),
            settings_manager: ReaderSettingsManager::new(id),
            state_manager: ReaderStateManager::new(info.clone(), 0, pages_count),
            toc_manager: ReaderTocManager::new(),
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
            bars_visible: true,
            margin_cropper_visible: false,
        }
    }

    pub fn from_html(
        rect: Rectangle,
        html: &str,
        _link_uri: Option<&str>,
        _hub: &Hub,
        context: &mut Context,
    ) -> Result<Reader, Error> {
        let id = ID_FEEDER.next();
        let (doc, pages_count, reflowable) = Self::open_html_document(html)?;
        let children = Self::create_toolbar(rect, reflowable, &Info::default(), context);
        let info = Self::create_html_info(html);

        Ok(Self::create_reader(
            id,
            rect,
            children,
            doc,
            pages_count,
            reflowable,
            info,
        ))
    }

    fn open_html_document(
        html: &str,
    ) -> Result<(Arc<Mutex<Box<dyn Document>>>, usize, bool), Error> {
        let doc = crate::document::open_html(html).context("Failed to open HTML document")?;
        let doc = Arc::new(Mutex::new(doc));
        let pages_count = (*doc.lock().expect("doc lock")).pages_count();
        let reflowable = (*doc.lock().expect("doc lock")).is_reflowable();
        Ok((doc, pages_count, reflowable))
    }

    fn create_html_info(html: &str) -> Info {
        Info {
            file: crate::metadata::FileInfo {
                path: std::path::PathBuf::from("memory.html"),
                kind: "html".to_string(),
                size: html.len() as u64,
            },
            reader: None,
            ..Default::default()
        }
    }

    /// Render page transition animation
    fn render_animation(&self, fb: &mut dyn Framebuffer, rect: Rectangle) {
        if let Some(ref anim) = self.animation {
            for chunk in &self.previous_chunks {
                self.render_chunk_animation(fb, rect, chunk, anim);
            }
        }
    }

    /// Render a single chunk of page animation
    fn render_chunk_animation(
        &self,
        fb: &mut dyn Framebuffer,
        rect: Rectangle,
        chunk: &RenderChunk,
        anim: &PageAnimation,
    ) {
        if let Some(resource) = self.cache.get(&chunk.location) {
            let chunk_rect = chunk.frame - chunk.frame.min + chunk.position;

            if let Some(region_rect) = rect.intersection(&chunk_rect) {
                let chunk_frame = region_rect - chunk.position + chunk.frame.min;
                let chunk_position = region_rect.min;
                let pixmap = &resource.pixmap;

                self.render_animation_kind(fb, pixmap, &chunk_frame, chunk_position, anim, rect);
            }
        }
    }

    /// Dispatch to specific animation type
    fn render_animation_kind(
        &self,
        fb: &mut dyn Framebuffer,
        pixmap: &Pixmap,
        chunk_frame: &Rectangle,
        chunk_position: Point,
        anim: &PageAnimation,
        rect: Rectangle,
    ) {
        match anim {
            PageAnimation::None => {}
            PageAnimation::Slide(kind) => {
                self.render_slide_animation(fb, pixmap, chunk_frame, chunk_position, kind, rect)
            }
            PageAnimation::Peel(state) => {
                self.render_peel_animation(fb, pixmap, chunk_frame, chunk_position, state, rect)
            }
        }
    }

    /// Render slide animation effect
    fn render_slide_animation(
        &self,
        fb: &mut dyn Framebuffer,
        pixmap: &Pixmap,
        chunk_frame: &Rectangle,
        chunk_position: Point,
        kind: &AnimState,
        rect: Rectangle,
    ) {
        let offset = (kind.progress * rect.width() as f32) as i32;
        let adjusted_position = if matches!(kind.direction, LinearDir::Forward) {
            pt!(chunk_position.x - offset, chunk_position.y)
        } else {
            pt!(chunk_position.x + offset, chunk_position.y)
        };
        let alpha = (1.0 - kind.progress) as u8;
        fb.draw_framed_pixmap_contrast_alpha(
            pixmap,
            chunk_frame,
            adjusted_position,
            self.contrast.exponent,
            self.contrast.gray,
            alpha,
        );
    }

    /// Render peel animation effect
    fn render_peel_animation(
        &self,
        fb: &mut dyn Framebuffer,
        pixmap: &Pixmap,
        chunk_frame: &Rectangle,
        chunk_position: Point,
        state: &AnimState,
        rect: Rectangle,
    ) {
        match state.kind {
            PageAnimKind::Fade => {
                self.render_fade_animation(fb, pixmap, chunk_frame, chunk_position, state)
            }
            PageAnimKind::Flip => {
                self.render_flip_animation(fb, pixmap, chunk_frame, chunk_position, state, rect)
            }
            _ => {}
        }
    }

    /// Render fade animation effect
    fn render_fade_animation(
        &self,
        fb: &mut dyn Framebuffer,
        pixmap: &Pixmap,
        chunk_frame: &Rectangle,
        chunk_position: Point,
        state: &AnimState,
    ) {
        let alpha = ((1.0 - state.progress) * 255.0) as u8;
        fb.draw_framed_pixmap_contrast_alpha(
            pixmap,
            chunk_frame,
            chunk_position,
            self.contrast.exponent,
            self.contrast.gray,
            alpha,
        );
    }

    /// Render flip animation effect
    fn render_flip_animation(
        &self,
        fb: &mut dyn Framebuffer,
        pixmap: &Pixmap,
        chunk_frame: &Rectangle,
        chunk_position: Point,
        state: &AnimState,
        rect: Rectangle,
    ) {
        let offset = (state.progress * rect.width() as f32) as i32;
        let adjusted_position = if matches!(state.direction, LinearDir::Forward) {
            pt!(chunk_position.x - offset, chunk_position.y)
        } else {
            pt!(chunk_position.x + offset, chunk_position.y)
        };
        let alpha = ((1.0 - state.progress * 0.5) * 255.0) as u8;
        fb.draw_framed_pixmap_contrast_alpha(
            pixmap,
            chunk_frame,
            adjusted_position,
            self.contrast.exponent,
            self.contrast.gray,
            alpha,
        );
    }

    // -----------------------------------------------------------------------
    // Table of Contents and Page Lookup
    // -----------------------------------------------------------------------

    /// Get table of contents for current document
    pub fn toc(&mut self) -> Option<Vec<TocEntry>> {
        self.toc_manager.build_toc(&self.info)
    }

    /// Find page index by name
    pub fn find_page_by_name(&self, name: &str) -> Option<usize> {
        self.toc_manager.find_page_by_name(&self.info, name)
    }

    // -----------------------------------------------------------------------
    // Text Excerpt and Selection Geometry
    // -----------------------------------------------------------------------

    /// Extract text excerpt from selection
    pub fn text_excerpt(&self, sel: [Point; 2]) -> Option<String> {
        reader_rendering::text_excerpt(&self.text, sel, &self.info.language)
    }

    // -----------------------------------------------------------------------
    // Annotation Lookup and UI
    // -----------------------------------------------------------------------

    /// Find mutable reference to annotation by selection
    pub fn find_annotation_mut(&mut self, sel: [TextLocation; 2]) -> Option<&mut Annotation> {
        super::reader_annotations::find_annotation_mut(&mut self.info, sel)
    }

    // -----------------------------------------------------------------------
    // Rendering Helpers (using render_cache and render_engine)
    // -----------------------------------------------------------------------

    /// Get render cache statistics
    pub fn get_render_cache_stats(&self) -> super::reader_rendering_ext::CacheStats {
        self.render_cache.stats()
    }

    /// Update render engine viewport
    pub fn update_render_viewport(&mut self, viewport: super::reader_core::ViewPort) {
        self.render_engine.viewport = viewport;
    }

    /// Start page transition animation
    pub fn start_page_animation(&mut self, kind: super::reader_core::PageAnimKind, _duration_ms: u32) {
        // Store current chunks for animation reference
        self.previous_chunks = self.chunks.clone();

        // Create animation state
        let anim_state = super::reader_core::AnimState {
            kind,
            direction: self.search_direction,
            progress: 0.0,
        };

        // Create animation based on kind
        self.animation = Some(match kind {
            super::reader_core::PageAnimKind::Slide => {
                super::reader_core::PageAnimation::Slide(anim_state)
            }
            _ => super::reader_core::PageAnimation::Peel(anim_state),
        });
    }

    /// Clear animation state
    pub fn clear_animation(&mut self) {
        self.animation = None;
        self.previous_chunks.clear();
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
                self.handle_device_event(*device_event, hub, rq, context);
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

    fn render(&self, fb: &mut dyn Framebuffer, rect: Rectangle, _fonts: &mut Fonts) {
        // Render page transition animation if active
        self.render_animation(fb, rect);
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
