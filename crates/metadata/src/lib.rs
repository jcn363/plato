//! Plato Metadata Module
//!
//! This crate provides metadata management functionality for Plato.

pub use plato_core::metadata::{
    consolidate, export_annotations_json, export_annotations_markdown,
    extract_metadata_from_document, extract_metadata_from_filename, file_name_from_info,
    make_query, rename_from_info, sort, sort_added, sort_author, sort_filename, sort_filepath,
    sort_kind, sort_opened, sort_pages, sort_progress, sort_series, sort_size, sort_status,
    sort_title, sort_year, sorter, Annotation, BookQuery, Collection, Collections, CroppingMargins,
    FileInfo, Info, Margin, Metadata, PageScheme, PageTurnEvent, ReaderInfo, ReadingStatistics,
    SavedQueries, SavedQuery, ScrollMode, SearchIndex, SimpleStatus, SmartCollectionRules,
    SortMethod, Status, TextAlign, ZoomMode, DEFAULT_CONTRAST_EXPONENT, DEFAULT_CONTRAST_GRAY,
    TITLE_PREFIXES,
};
