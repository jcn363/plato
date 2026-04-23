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
//! - `reader.rs` - Main Reader struct and core methods
//! - `reader_settings.rs` - Settings menus and configuration helpers
//! - `reader_rendering.rs` - Text and selection rendering utilities
//! - `reader_search.rs` - Search functionality
//! - `reader_annotations.rs` - Annotation and bookmark helpers
//! - `reader_dialogs.rs` - Dialog and input handling
//! - `reader_gestures.rs` - Touch/gesture handling and input processing
//! - `reader_core.rs` - Shared type definitions
//! - `reader_animation.rs` - Page transition animation rendering
//! - `reader_constructors.rs` - Reader constructor functions

// ===========================================================================
// Imports and Constants
// ===========================================================================

use crate::context::Context;
use crate::device::CURRENT_DEVICE;
use crate::document::{BoundedText, Document, TextLocation, TocEntry};
use crate::font::Fonts;
use crate::framebuffer::{Framebuffer, UpdateMode};
use crate::geom::{Boundary, Point, Rectangle, Vec2};
use crate::geom::{CycleDir, LinearDir};
use crate::input::ButtonCode;
use crate::metadata::{Annotation, Info, ZoomMode};
use crate::metadata::{DEFAULT_CONTRAST_EXPONENT, DEFAULT_CONTRAST_GRAY};
use anyhow::Error;
use rustc_hash::{FxHashMap, FxHashSet};
use std::collections::{BTreeMap, VecDeque};
use std::sync::{atomic, Arc, Mutex};

use crate::view::{Bus, Event, Hub, Id, RenderData, RenderQueue, View, ViewId, ID_FEEDER};

use super::reader_animation;
use super::reader_constructors;
use super::reader_core::{
    Contrast, PageAnimation, RenderChunk, Resource, Search, Selection, State, ViewPort,
};
use super::reader_rendering;
use super::reader_toc::ReaderTocManager;

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
    pub(crate) toc_manager: ReaderTocManager,
    pub(crate) _noninverted_regions: FxHashMap<usize, Vec<Boundary>>,
    pub(crate) focus: Option<ViewId>,
    pub(crate) search: Option<Search>,
    pub(crate) search_direction: LinearDir,
    pub(crate) held_buttons: FxHashSet<ButtonCode>,
    pub(crate) selection: Option<Selection>,
    pub(crate) _target_annotation: Option<[TextLocation; 2]>,
    pub(crate) history: VecDeque<usize>,
    pub(crate) _state: State,
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
        let (doc, pages_count, reflowable) = reader_constructors::open_document(&info)?;
        let children = reader_constructors::create_toolbar(rect, reflowable, &info, context);

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
            toc_manager: ReaderTocManager::new(),
            _noninverted_regions: FxHashMap::default(),
            focus: None,
            search: None,
            search_direction: LinearDir::Forward,
            held_buttons: FxHashSet::default(),
            selection: None,
            _target_annotation: None,
            history: VecDeque::new(),
            _state: State::Idle,
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
        let (doc, pages_count, reflowable) = reader_constructors::open_html_document(html)?;
        let children =
            reader_constructors::create_toolbar(rect, reflowable, &Info::default(), context);
        let info = reader_constructors::create_html_info(html);

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

    /// Render page transition animation
    fn render_animation(&self, fb: &mut dyn Framebuffer, rect: Rectangle) {
        reader_animation::render_animation(
            &self.cache,
            &self.previous_chunks,
            &self.animation,
            fb,
            rect,
            self.contrast.exponent,
            self.contrast.gray,
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
    // Rendering Helpers
    // -----------------------------------------------------------------------

    /* NOTE: render_engine module removed
    /// Update render engine viewport
    pub fn update_render_viewport(&mut self, _viewport: super::reader_core::ViewPort) {
        // render_engine module removed during dead code cleanup
    }
    */

    /// Start page transition animation
    pub fn start_page_animation(
        &mut self,
        kind: super::reader_core::PageAnimKind,
        _duration_ms: u32,
    ) {
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

// Additional Reader methods outside View trait
impl Reader {
    /// Navigate to a specific search result page
    pub fn go_to_results_page(
        &mut self,
        index: usize,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_search::go_to_results_page(index, self, hub, rq, context);
    }

    /// Navigate to next or previous search result
    pub fn go_to_results_neighbor(
        &mut self,
        dir: CycleDir,
        hub: &Hub,
        rq: &mut RenderQueue,
        context: &mut Context,
    ) {
        super::reader_search::go_to_results_neighbor(dir, self, hub, rq, context);
    }

    /// Render search result highlights
    pub fn render_search_results(
        &self,
        chunks: &[super::reader_core::RenderChunk],
        rq: &mut RenderQueue,
    ) {
        super::reader_search::render_results(self.search.as_ref(), chunks, self.id, rq);
    }

    /// Get the bounding rectangle for the current text selection
    pub fn selection_rect(&self, chunks: &[super::reader_core::RenderChunk]) -> Option<Rectangle> {
        super::reader_rendering::selection_rect(self.selection.as_ref(), &self.text, chunks)
    }
}
