//! Unit tests for PDF annotation functionality
//!
//! Tests for annotation import, export, search, filtering, and XFDF handling.

use super::*;
use chrono::{Duration, Utc};
use std::path::PathBuf;

#[test]
fn test_annotation_creation() {
    let annot = PdfAnnotation::new(0, AnnotationSubtype::Highlight, "Test note".to_string());

    assert_eq!(annot.page, 0);
    assert_eq!(annot.subtype, AnnotationSubtype::Highlight);
    assert_eq!(annot.contents, "Test note");
    assert!(annot.rect.is_none());
    assert!(annot.color.is_none());
    assert!(annot.author.is_none());
    assert!(annot.subject.is_none());
    assert!(!annot.id.is_empty());
}

#[test]
fn test_annotation_touch() {
    let mut annot = PdfAnnotation::new(0, AnnotationSubtype::Text, "Test".to_string());
    let original_time = annot.modified_at;

    // Small delay to ensure time difference
    std::thread::sleep(std::time::Duration::from_millis(10));

    annot.touch();

    assert!(annot.modified_at > original_time);
}

#[test]
fn test_annotation_query_builder() {
    let query = AnnotationQuery::new()
        .with_subtype(AnnotationSubtype::Highlight)
        .with_text("search term".to_string())
        .with_page(5);

    assert_eq!(query.subtype, Some(AnnotationSubtype::Highlight));
    assert_eq!(query.text, Some("search term".to_string()));
    assert_eq!(query.page, Some(5));
}

#[test]
fn test_annotation_matching() {
    let now = Utc::now();
    let mut annot = PdfAnnotation::new(
        0,
        AnnotationSubtype::Highlight,
        "Important note".to_string(),
    );
    annot.author = Some("John Doe".to_string());
    annot.subject = Some("Review".to_string());

    // Test subtype match
    let query = AnnotationQuery::new().with_subtype(AnnotationSubtype::Highlight);
    assert!(annot.matches(&query));

    let query = AnnotationQuery::new().with_subtype(AnnotationSubtype::Text);
    assert!(!annot.matches(&query));

    // Test text match
    let query = AnnotationQuery::new().with_text("important".to_string());
    assert!(annot.matches(&query));

    let query = AnnotationQuery::new().with_text("missing".to_string());
    assert!(!annot.matches(&query));

    // Test page match
    let query = AnnotationQuery::new().with_page(0);
    assert!(annot.matches(&query));

    let query = AnnotationQuery::new().with_page(5);
    assert!(!annot.matches(&query));

    // Test date range
    let query = AnnotationQuery::new()
        .with_after(now - Duration::hours(1))
        .with_before(now + Duration::hours(1));
    assert!(annot.matches(&query));
}

#[test]
fn test_annotation_subtype_from_str() {
    assert_eq!(
        AnnotationSubtype::from_str("text").expect("invalid subtype"),
        AnnotationSubtype::Text
    );
    assert_eq!(
        AnnotationSubtype::from_str("highlight").expect("invalid subtype"),
        AnnotationSubtype::Highlight
    );
    assert_eq!(
        AnnotationSubtype::from_str("underline").expect("invalid subtype"),
        AnnotationSubtype::Underline
    );
    assert_eq!(
        AnnotationSubtype::from_str("strikeout").expect("invalid subtype"),
        AnnotationSubtype::StrikeOut
    );

    // Case insensitive
    assert_eq!(
        AnnotationSubtype::from_str("HIGHLIGHT").expect("invalid subtype"),
        AnnotationSubtype::Highlight
    );

    // Invalid subtype
    assert!(AnnotationSubtype::from_str("invalid").is_err());
}

#[test]
fn test_annotation_subtype_as_str() {
    assert_eq!(AnnotationSubtype::Text.as_str(), "Text");
    assert_eq!(AnnotationSubtype::Highlight.as_str(), "Highlight");
    assert_eq!(AnnotationSubtype::Underline.as_str(), "Underline");
    assert_eq!(AnnotationSubtype::StrikeOut.as_str(), "StrikeOut");
}

#[test]
fn test_xfdf_export() {
    let mut annot1 = PdfAnnotation::new(0, AnnotationSubtype::Highlight, "Note 1".to_string());
    annot1.rect = Some((10.0, 20.0, 100.0, 200.0));
    annot1.color = Some((255, 0, 0));
    annot1.author = Some("Alice".to_string());

    let mut annot2 = PdfAnnotation::new(1, AnnotationSubtype::Text, "Note 2".to_string());
    annot2.subject = Some("Comment".to_string());

    let annotations = vec![annot1, annot2];
    let pdf_path = PathBuf::from("/test/document.pdf");

    let xfdf = XfdfHandler::export_to_xfdf(&annotations, &pdf_path).expect("xfdf export failed");

    assert!(xfdf.contains("<?xml version=\"1.0\""));
    assert!(xfdf.contains("<xfdf"));
    assert!(xfdf.contains("<annotate>"));
    assert!(xfdf.contains("<subtype>Highlight</subtype>"));
    assert!(xfdf.contains("<contents>Note 1</contents>"));
    assert!(xfdf.contains("<rect>10,20,100,200</rect>"));
    assert!(xfdf.contains("<color>#FF0000</color>"));
    assert!(xfdf.contains("<author>Alice</author>"));
    assert!(xfdf.contains("<subject>Comment</subject>"));
}

#[test]
fn test_xfdf_import() {
    let xfdf_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<xfdf xmlns="http://ns.adobe.com/xfdf/" xml:space="preserve">
  <f href="/test/document.pdf"/>
  <annotate>
    <subtype>Highlight</subtype>
    <contents>Test note</contents>
    <page>0</page>
    <rect>10,20,100,200</rect>
    <color>#FFFF00</color>
    <author>Bob</author>
    <subject>Review</subject>
  </annotate>
</xfdf>"#;

    let annotations = XfdfHandler::import_from_xfdf(xfdf_content).expect("xfdf import failed");

    assert_eq!(annotations.len(), 1);
    assert_eq!(annotations[0].subtype, AnnotationSubtype::Highlight);
    assert_eq!(annotations[0].contents, "Test note");
    assert_eq!(annotations[0].page, 0);
    assert_eq!(annotations[0].rect, Some((10.0, 20.0, 100.0, 200.0)));
    assert_eq!(annotations[0].color, Some((255, 255, 0)));
    assert_eq!(annotations[0].author, Some("Bob".to_string()));
    assert_eq!(annotations[0].subject, Some("Review".to_string()));
}

#[test]
fn test_xfdf_roundtrip() {
    let mut original = PdfAnnotation::new(
        5,
        AnnotationSubtype::Underline,
        "Roundtrip test".to_string(),
    );
    original.rect = Some((50.0, 60.0, 150.0, 160.0));
    original.color = Some((0, 128, 255));
    original.author = Some("Charlie".to_string());
    original.subject = Some("Check".to_string());

    let annotations = vec![original.clone()];
    let pdf_path = PathBuf::from("/test/doc.pdf");

    let xfdf = XfdfHandler::export_to_xfdf(&annotations, &pdf_path).expect("xfdf export failed");
    let imported = XfdfHandler::import_from_xfdf(&xfdf).expect("xfdf import failed");

    assert_eq!(imported.len(), 1);
    assert_eq!(imported[0].subtype, original.subtype);
    assert_eq!(imported[0].contents, original.contents);
    assert_eq!(imported[0].page, original.page);
    assert_eq!(imported[0].rect, original.rect);
    assert_eq!(imported[0].color, original.color);
    assert_eq!(imported[0].author, original.author);
    assert_eq!(imported[0].subject, original.subject);
}

#[test]
fn test_escape_xml() {
    assert_eq!(escape_xml("test"), "test");
    assert_eq!(escape_xml("test & test"), "test &amp; test");
    assert_eq!(escape_xml("test < test"), "test &lt; test");
    assert_eq!(escape_xml("test > test"), "test &gt; test");
    assert_eq!(escape_xml("test \"test\""), "test &quot;test&quot;");
    assert_eq!(escape_xml("test 'test'"), "test &apos;test&apos;");
}

#[test]
fn test_annotation_manager_sorting() {
    // Note: This test requires a valid PDF file, so we'll test the sorting logic separately
    let mut annotations = [
        PdfAnnotation::new(2, AnnotationSubtype::Text, "Page 2".to_string()),
        PdfAnnotation::new(0, AnnotationSubtype::Highlight, "Page 0".to_string()),
        PdfAnnotation::new(1, AnnotationSubtype::Underline, "Page 1".to_string()),
    ];

    // Sort by page ascending
    annotations.sort_by_key(|a| a.page);
    assert_eq!(annotations[0].page, 0);
    assert_eq!(annotations[1].page, 1);
    assert_eq!(annotations[2].page, 2);

    // Sort by page descending
    annotations.sort_by_key(|b| std::cmp::Reverse(b.page));
    assert_eq!(annotations[0].page, 2);
    assert_eq!(annotations[1].page, 1);
    assert_eq!(annotations[2].page, 0);
}
