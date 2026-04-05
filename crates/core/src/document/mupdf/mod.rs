mod annotation;
mod context;
mod document;
mod image;
mod link;
mod outline;
mod page;
mod pixmap;
mod text;

pub use annotation::Annotation;
pub use context::{concat, invert_matrix, scale, MuPdfContext, IDENTITY};
pub use document::Document;
pub use image::Image;
pub use link::Link;
pub use outline::Outline;
pub use page::Page;
pub use pixmap::{
    close_device, create_annot, device_gray, device_rgb, drop_device, new_bbox_device,
    new_draw_device, new_pixmap, rect_from_quad, union_rect, Pixmap,
};
pub use text::{
    TextBlock, TextBlockIter, TextChar, TextCharIter, TextLine, TextLineIter, TextPage,
};

pub use crate::document::mupdf_sys::{
    fz_identity, FzLocation, FzMatrix, FzPoint, FzQuad, FzRect, FzTextOptions, FzWriteOptions,
    FZ_META_INFO_AUTHOR, FZ_META_INFO_TITLE, FZ_PAGE_BLOCK_IMAGE, FZ_PAGE_BLOCK_TEXT,
    FZ_TEXT_PRESERVE_IMAGES,
};
