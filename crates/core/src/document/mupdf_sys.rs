#![allow(unused)]

use std::ffi::CString;
use std::mem;
use std::ptr;

pub const FZ_MAX_COLORS: usize = 32;
pub const FZ_VERSION: &str = "1.27.0";

pub const FZ_META_INFO_AUTHOR: &str = "info:Author";
pub const FZ_META_INFO_TITLE: &str = "info:Title";

pub const FZ_TEXT_PRESERVE_LIGATURES: libc::c_int = 1;
pub const FZ_TEXT_PRESERVE_WHITESPACE: libc::c_int = 2;
pub const FZ_TEXT_PRESERVE_IMAGES: libc::c_int = 4;
pub const FZ_TEXT_INHIBIT_SPACES: libc::c_int = 8;
pub const FZ_TEXT_DEHYPHENATE: libc::c_int = 16;
pub const FZ_TEXT_PRESERVE_SPANS: libc::c_int = 32;
pub const FZ_TEXT_CLIP: libc::c_int = 64;
pub const FZ_TEXT_CLIP_RECT: libc::c_int = 1 << 17;
pub const FZ_TEXT_COLLECT_STRUCTURE: libc::c_int = 256;
pub const FZ_TEXT_COLLECT_VECTORS: libc::c_int = 1024;
pub const FZ_TEXT_ACCURATE_BBOXES: libc::c_int = 512;
pub const FZ_TEXT_ACCURATE_ASCENDERS: libc::c_int = 1 << 18;
pub const FZ_TEXT_ACCURATE_SIDE_BEARINGS: libc::c_int = 1 << 19;
pub const FZ_TEXT_IGNORE_ACTUALTEXT: libc::c_int = 2048;
pub const FZ_TEXT_SEGMENT: libc::c_int = 4096;
pub const FZ_TEXT_PARAGRAPH_BREAK: libc::c_int = 8192;
pub const FZ_TEXT_TABLE_HUNT: libc::c_int = 16384;
pub const FZ_TEXT_USE_CID_FOR_UNKNOWN_UNICODE: libc::c_int = 128;
pub const FZ_TEXT_USE_GID_FOR_UNKNOWN_UNICODE: libc::c_int = 65536;

pub const FZ_PAGE_BLOCK_TEXT: libc::c_int = 0;
pub const FZ_PAGE_BLOCK_IMAGE: libc::c_int = 1;
pub const FZ_PAGE_BLOCK_STRUCT: libc::c_int = 2;
pub const FZ_PAGE_BLOCK_VECTOR: libc::c_int = 3;
pub const FZ_PAGE_BLOCK_GRID: libc::c_int = 4;

pub const CACHE_SIZE: libc::size_t = 16 * 1024 * 1024;

pub enum FzContext {}
pub enum FzDocument {}
pub enum FzStream {}
pub enum FzPool {}
pub enum FzPage {}
pub enum FzDevice {}
pub enum FzFont {}
pub enum FzColorspace {}
pub enum FzAllocContext {}
pub enum FzLocksContext {}
pub enum FzCookie {}
pub enum FzStoreDropFn {}
pub enum FzStoreDroppableFn {}
pub enum FzLinkSetRectFn {}
pub enum FzLinkSetUriFn {}
pub enum FzLinkDropLinkFn {}
pub enum FzSeparations {}
pub enum FzTextStruct {}
pub enum FzGridPositions {}
pub enum FzGridInfo {}
pub enum FzPoolArray {}
pub enum FzImage {}
pub enum FzAnnot {}
pub enum FzQuad {}
pub enum FzPoint {}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct FzTextOptions {
    pub flags: libc::c_int,
    pub scale: libc::c_float,
    pub clip: FzRect,
}

impl Default for FzTextOptions {
    fn default() -> Self {
        FzTextOptions {
            flags: 0,
            scale: 1.0,
            clip: FzRect::default(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FzMatrix {
    pub a: libc::c_float,
    pub b: libc::c_float,
    pub c: libc::c_float,
    pub d: libc::c_float,
    pub e: libc::c_float,
    pub f: libc::c_float,
}

impl Default for FzMatrix {
    fn default() -> FzMatrix {
        FzMatrix {
            a: 0.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
            e: 0.0,
            f: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FzRect {
    pub x0: libc::c_float,
    pub y0: libc::c_float,
    pub x1: libc::c_float,
    pub y1: libc::c_float,
}

#[repr(C)]
pub struct FzPixmap {
    pub refs: libc::c_int,
    pub w: libc::c_int,
    pub h: libc::c_int,
    pub n: libc::c_int,
    pub alpha: libc::c_int,
    pub xres: libc::c_int,
    pub yres: libc::c_int,
    pub x: libc::c_int,
    pub y: libc::c_int,
    pub width: libc::c_int,
    pub height: libc::c_int,
    pub stride: libc::ptrdiff_t,
    pub samples: *mut libc::c_uchar,
}

#[repr(C)]
pub struct FzWriteOptions {
    pub what: libc::c_int,
    pub progress: libc::c_int,
    pub res: libc::c_int,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct FzLocation {
    pub chapter: libc::c_int,
    pub page: libc::c_int,
}

impl Default for FzRect {
    fn default() -> Self {
        // SAFETY: All-zero bit pattern is valid for this type.
        unsafe { mem::zeroed() }
    }
}

impl Default for FzWriteOptions {
    fn default() -> Self {
        // SAFETY: All-zero bit pattern is valid for this type.
        unsafe { mem::zeroed() }
    }
}

#[link(name = "mupdf")]
#[link(name = "mupdf_wrapper", kind = "static")]
extern "C" {
    pub fn mp_open_document(ctx: *mut FzContext, path: *const libc::c_char) -> *mut FzDocument;
    pub fn mp_open_document_with_stream(
        ctx: *mut FzContext,
        kind: *const libc::c_char,
        stream: *mut FzStream,
    ) -> *mut FzDocument;
    pub fn mp_load_page(
        ctx: *mut FzContext,
        doc: *mut FzDocument,
        page_num: libc::c_int,
    ) -> *mut FzPage;
    pub fn mp_load_outline(ctx: *mut FzContext, doc: *mut FzDocument) -> *mut FzOutline;
    pub fn mp_load_links(ctx: *mut FzContext, page: *mut FzPage) -> *mut FzLink;
    pub fn mp_count_pages(ctx: *mut FzContext, doc: *mut FzDocument) -> libc::c_int;
    pub fn mp_page_number_from_location(
        ctx: *mut FzContext,
        doc: *mut FzDocument,
        loc: FzLocation,
    ) -> libc::c_int;
    pub fn mp_new_pixmap_from_page(
        ctx: *mut FzContext,
        page: *mut FzPage,
        matrix: FzMatrix,
        colorspace: *mut FzColorspace,
        alpha: libc::c_int,
    ) -> *mut FzPixmap;
    pub fn mp_new_stext_page_from_page(
        ctx: *mut FzContext,
        page: *mut FzPage,
        options: *const FzTextOptions,
    ) -> *mut FzTextPage;

    pub fn fz_new_context_imp(
        alloc_ctx: *const FzAllocContext,
        locks_ctx: *const FzLocksContext,
        cache_size: libc::size_t,
        version: *const libc::c_char,
    ) -> *mut FzContext;
    pub fn fz_drop_context(ctx: *mut FzContext);
    pub fn fz_register_document_handlers(ctx: *mut FzContext);
    pub fn fz_open_document(ctx: *mut FzContext, path: *const libc::c_char) -> *mut FzDocument;
    pub fn fz_drop_document(ctx: *mut FzContext, doc: *mut FzDocument);
    pub fn fz_load_page(
        ctx: *mut FzContext,
        doc: *mut FzDocument,
        number: libc::c_int,
    ) -> *mut FzPage;
    pub fn fz_drop_page(ctx: *mut FzContext, page: *mut FzPage);
    pub fn fz_count_pages(ctx: *mut FzContext, doc: *mut FzDocument) -> libc::c_int;
    pub fn fz_needs_password(ctx: *mut FzContext, doc: *mut FzDocument) -> libc::c_int;
    pub fn fz_authenticate_password(
        ctx: *mut FzContext,
        doc: *mut FzDocument,
        password: *const libc::c_char,
    ) -> libc::c_int;
    pub fn fz_lookup_metadata(
        ctx: *mut FzContext,
        doc: *mut FzDocument,
        key: *const libc::c_char,
        buf: *mut libc::c_char,
        size: libc::c_int,
    ) -> libc::c_int;
    pub fn fz_set_user_css(ctx: *mut FzContext, css: *const libc::c_char);
    pub fn fz_open_memory(
        ctx: *mut FzContext,
        data: *const libc::c_uchar,
        len: libc::size_t,
    ) -> *mut FzStream;
    pub fn fz_drop_stream(ctx: *mut FzContext, stream: *mut FzStream);
    pub fn fz_is_document_reflowable(ctx: *mut FzContext, doc: *mut FzDocument) -> libc::c_int;
    pub fn fz_layout_document(
        ctx: *mut FzContext,
        doc: *mut FzDocument,
        width: libc::c_float,
        height: libc::c_float,
    );
    pub fn fz_resolve_link_dest(
        ctx: *mut FzContext,
        doc: *mut FzDocument,
        link: *mut FzLink,
        target: *mut FzMatrix,
    ) -> FzRect;
    pub fn fz_drop_link(ctx: *mut FzContext, link: *mut FzLink);
    pub fn fz_drop_outline(ctx: *mut FzContext, outline: *mut FzOutline);
    pub fn fz_load_links(ctx: *mut FzContext, page: *mut FzPage) -> *mut FzLink;
    pub fn fz_new_bbox_device(ctx: *mut FzContext, rect: FzRect) -> *mut FzDevice;
    pub fn fz_rect_from_quad(ctx: *mut FzContext, quad: FzQuad) -> FzRect;
    pub fn fz_union_rect(ctx: *mut FzContext, a: FzRect, b: FzRect) -> FzRect;
    pub fn fz_save_document(
        ctx: *mut FzContext,
        doc: *mut FzDocument,
        filename: *const libc::c_char,
        opts: *const FzWriteOptions,
        fmt: *const libc::c_char,
    );
    pub fn fz_scale(sx: libc::c_float, sy: libc::c_float) -> FzMatrix;
    pub fn fz_concat(ctx: *mut FzContext, a: FzMatrix, b: FzMatrix) -> FzMatrix;
    pub fn fz_invert_matrix(ctx: *mut FzContext, m: FzMatrix) -> FzMatrix;
    pub fn fz_new_pixmap(
        ctx: *mut FzContext,
        colorspace: *mut FzColorspace,
        w: libc::c_int,
        h: libc::c_int,
        alpha: libc::c_int,
    ) -> *mut FzPixmap;
    pub fn fz_new_pixmap_with_bbox(
        ctx: *mut FzContext,
        colorspace: *mut FzColorspace,
        w: libc::c_int,
        h: libc::c_int,
        bbox: FzRect,
        alpha: libc::c_int,
    ) -> *mut FzPixmap;
    pub fn fz_drop_pixmap(ctx: *mut FzContext, pix: *mut FzPixmap);
    pub fn fz_clear_pixmap(ctx: *mut FzContext, pix: *mut FzPixmap);
    pub fn fz_fill_pixmap(
        ctx: *mut FzContext,
        dest: *mut FzPixmap,
        color: *mut FzColorspace,
        color_vals: *const libc::c_float,
        alpha: libc::c_float,
    );
    pub fn fz_device_gray(ctx: *mut FzContext) -> *mut FzColorspace;
    pub fn fz_device_rgb(ctx: *mut FzContext) -> *mut FzColorspace;
    pub fn fz_new_draw_device(
        ctx: *mut FzContext,
        matrix: FzMatrix,
        pixmap: *mut FzPixmap,
    ) -> *mut FzDevice;
    pub fn fz_run_page(
        ctx: *mut FzContext,
        page: *mut FzPage,
        dev: *mut FzDevice,
        matrix: FzMatrix,
        r: *mut FzCookie,
    );
    pub fn fz_close_device(ctx: *mut FzContext, dev: *mut FzDevice);
    pub fn fz_drop_device(ctx: *mut FzContext, dev: *mut FzDevice);
    pub fn fz_new_pdf_document(ctx: *mut FzContext) -> *mut FzDocument;
    pub fn fz_pdf_insert_page(
        ctx: *mut FzContext,
        doc: *mut FzDocument,
        page: *mut FzPage,
        after: libc::c_int,
    ) -> libc::c_int;
    pub fn fz_pdf_count_pages(ctx: *mut FzContext, doc: *mut FzDocument) -> libc::c_int;
    pub fn fz_pdf_can_move_pages(ctx: *mut FzContext, doc: *mut FzDocument) -> libc::c_int;
    pub fn fz_pdf_move_page(
        ctx: *mut FzContext,
        doc: *mut FzDocument,
        src: libc::c_int,
        dst: libc::c_int,
    );
    pub fn fz_pdf_delete_page(ctx: *mut FzContext, doc: *mut FzDocument, number: libc::c_int);
    pub fn fz_pdf_rotate_page(
        ctx: *mut FzContext,
        doc: *mut FzDocument,
        number: libc::c_int,
        rotation: libc::c_int,
    );
    pub fn fz_pdf_output_intent(ctx: *mut FzContext, doc: *mut FzDocument) -> *mut libc::c_char;
    pub fn fz_set_use_document_css(ctx: *mut FzContext, use_doc_css: libc::c_int);
    pub fn fz_bound_page(ctx: *mut FzContext, page: *mut FzPage) -> FzRect;
    pub fn fz_count_page_images(ctx: *mut FzContext, page: *mut FzPage) -> libc::c_int;
    pub fn fz_load_page_image(
        ctx: *mut FzContext,
        page: *mut FzPage,
        index: libc::c_int,
    ) -> *mut FzImage;
    pub fn fz_image_width(ctx: *mut FzContext, image: *mut FzImage) -> libc::c_int;
    pub fn fz_image_height(ctx: *mut FzContext, image: *mut FzImage) -> libc::c_int;
    pub fn fz_drop_image(ctx: *mut FzContext, image: *mut FzImage);
    pub fn fz_count_page_fonts(ctx: *mut FzContext, page: *mut FzPage) -> libc::c_int;
    pub fn fz_first_annot(ctx: *mut FzContext, page: *mut FzPage) -> *mut FzAnnot;
    pub fn fz_next_annot(ctx: *mut FzContext, annot: *mut FzAnnot) -> *mut FzAnnot;
    pub fn fz_create_annot(
        ctx: *mut FzContext,
        page: *mut FzPage,
        type_: *const libc::c_char,
    ) -> *mut FzAnnot;
    pub fn fz_annot_contents(ctx: *mut FzContext, annot: *mut FzAnnot) -> *mut libc::c_char;
    pub fn fz_set_annot_contents(
        ctx: *mut FzContext,
        annot: *mut FzAnnot,
        contents: *const libc::c_char,
    );
    pub fn fz_annot_rect(ctx: *mut FzContext, annot: *mut FzAnnot) -> FzRect;
    pub fn fz_set_annot_rect(ctx: *mut FzContext, annot: *mut FzAnnot, rect: FzRect);
    pub fn fz_drop_annot(ctx: *mut FzContext, annot: *mut FzAnnot);
    pub fn fz_apply_redactions(ctx: *mut FzContext, page: *mut FzPage, flags: libc::c_int);
    pub fn fz_remove_redactions(ctx: *mut FzContext, page: *mut FzPage);
    pub fn fz_search_page(
        ctx: *mut FzContext,
        page: *mut FzPage,
        text: *const libc::c_char,
        hits: *mut FzRect,
        hit_count: libc::c_int,
    ) -> libc::c_int;
    pub fn fz_drop_stext_page(ctx: *mut FzContext, page: *mut FzTextPage);
    pub fn fz_is_document_linearized(ctx: *mut FzContext, doc: *mut FzDocument) -> libc::c_int;
}

/// The identity matrix in MuPDF.
#[allow(non_upper_case_globals)]
pub const fz_identity: FzMatrix = FzMatrix {
    a: 1.0,
    b: 0.0,
    c: 0.0,
    d: 1.0,
    e: 0.0,
    f: 0.0,
};

#[repr(C)]
pub struct FzLink {
    refs: libc::c_int,
    pub next: *mut FzLink,
    pub rect: FzRect,
    pub uri: *mut libc::c_char,
    set_rect_fn: *mut FzLinkSetRectFn,
    set_uri_fn: *mut FzLinkSetUriFn,
    drop: *mut FzLinkDropLinkFn,
}

#[repr(C)]
pub struct FzTextPage {
    refs: libc::c_int,
    pool: *mut FzPool,
    mediabox: FzRect,
    pub first_block: *mut FzTextBlock,
    last_block: *mut FzTextBlock,
    last_struct: *mut FzTextStruct,
    id_list: *mut FzPoolArray,
}

#[repr(C)]
pub struct FzTextBlock {
    pub kind: libc::c_int,
    id: libc::c_int,
    pub bbox: FzRect,
    pub u: FzTextBlockTextImage,
    prev: *mut FzTextBlock,
    pub next: *mut FzTextBlock,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FzTextBlockText {
    pub first_line: *mut FzTextLine,
    last_line: *mut FzTextLine,
    flags: libc::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FzTextBlockImage {
    transform: FzMatrix,
    image: *mut FzImage,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FzTextBlockStruct {
    down: *mut FzTextStruct,
    index: libc::c_int,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FzTextBlockVector {
    flags: u32,
    argb: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct FzTextBlockGrid {
    xs: *mut FzGridPositions,
    ys: *mut FzGridPositions,
    info: *mut FzGridInfo,
}

#[repr(C)]
pub union FzTextBlockTextImage {
    pub text: FzTextBlockText,
    pub image: FzTextBlockImage,
    pub stru: FzTextBlockStruct,
    pub vector: FzTextBlockVector,
    pub grid: FzTextBlockGrid,
}

#[repr(C)]
pub struct FzTextLine {
    wmode: u8,
    flags: u8,
    dir: FzPoint,
    pub bbox: FzRect,
    pub first_char: *mut FzTextChar,
    last_char: *mut FzTextChar,
    prev: *mut FzTextLine,
    pub next: *mut FzTextLine,
}

#[repr(C)]
pub struct FzTextChar {
    pub c: libc::c_int,
    bidi: u16,
    flags: u16,
    argb: u32,
    origin: FzPoint,
    pub quad: FzQuad,
    size: libc::c_float,
    font: *mut FzFont,
    pub next: *mut FzTextChar,
}

#[repr(C)]
pub struct FzOutline {
    pub refs: libc::c_int,
    pub title: *mut libc::c_char,
    pub uri: *mut libc::c_char,
    pub page: FzLocation,
    pub x: libc::c_float,
    pub y: libc::c_float,
    pub next: *mut FzOutline,
    pub down: *mut FzOutline,
    pub is_open: libc::c_uint,
    pub flags: libc::c_uint,
    pub r: libc::c_uint,
    pub g: libc::c_uint,
    pub b: libc::c_uint,
}

impl Default for FzOutline {
    fn default() -> Self {
        // SAFETY: All-zero bit pattern is valid for this type.
        unsafe { mem::zeroed() }
    }
}

/// Create a new MuPDF context with document handlers registered.
/// This is the single authoritative factory for MuPDF contexts.
pub fn new_mupdf_context() -> Option<*mut FzContext> {
    // SAFETY: FFI call to MuPDF library. CString is valid and null-terminated.
    unsafe {
        let version = CString::new(FZ_VERSION).ok()?;
        let ctx = fz_new_context_imp(ptr::null(), ptr::null(), CACHE_SIZE, version.as_ptr());
        if ctx.is_null() {
            None
        } else {
            fz_register_document_handlers(ctx);
            Some(ctx)
        }
    }
}
