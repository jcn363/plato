use super::*;
use std::io::Empty;

const PATH_CASE_SENSITIVE_INDEX: &str = "src/dictionary/testdata/case_sensitive_dict.index";
const PATH_CASE_INSENSITIVE_INDEX: &str = "src/dictionary/testdata/case_insensitive_dict.index";

#[test]
fn test_index_find() {
    let words = vec![
        Entry {
            headword: String::from("bar"),
            offset: 0,
            size: 8,
            original: None,
        },
        Entry {
            headword: String::from("baz"),
            offset: 8,
            size: 4,
            original: None,
        },
        Entry {
            headword: String::from("foo"),
            offset: 12,
            size: 4,
            original: None,
        },
    ];

    let index: Index<Empty> = Index {
        entries: words,
        state: None,
    };

    let r = index.find("apples", false);
    assert!(r.is_empty());

    let r = index.find("baz", false);
    assert!(!r.is_empty());
    assert_eq!(r.len(), 1);
    assert_eq!(
        r.first().expect("result should have one entry").headword,
        "baz"
    );

    let r = index.find("bas", true);
    assert!(!r.is_empty());
    assert_eq!(r.len(), 2);
    assert_eq!(
        r.first().expect("result should have entries").headword,
        "bar"
    );
}

#[test]
// Make sure that a lazy load does not inadvertently skip a word when it returns to BufRead
fn test_index_load_and_find() {
    let r = parse_index_from_file(PATH_CASE_INSENSITIVE_INDEX, true);
    assert!(r.is_ok());

    let mut index = r.expect("parse should succeed");
    assert_eq!(index.entries[0].headword, "00-database-allchars");
    assert_eq!(
        index
            .entries
            .last()
            .expect("entries should not be empty")
            .headword,
        "bar"
    );

    let r = index.load_and_find(
        "bar",
        false,
        &Metadata {
            all_chars: true,
            case_sensitive: false,
        },
    );
    assert!(!r.is_empty());

    let r = index.load_and_find(
        "foo",
        false,
        &Metadata {
            all_chars: true,
            case_sensitive: false,
        },
    );
    assert!(!r.is_empty());
}

#[test]
fn test_parse_index_from_file() {
    let r = parse_index_from_file(PATH_CASE_INSENSITIVE_INDEX, false);
    assert!(r.is_ok());

    let index = r.expect("parse should succeed");
    assert_eq!(index.entries[0].headword, "00-database-allchars");
    assert_eq!(
        index
            .entries
            .last()
            .expect("entries should not be empty")
            .headword,
        "あいおい"
    );
}

#[test]
fn test_parse_index_from_file_lazy() {
    let r = parse_index_from_file(PATH_CASE_INSENSITIVE_INDEX, true);
    assert!(r.is_ok());

    let index = r.expect("parse should succeed");
    assert_eq!(index.entries[0].headword, "00-database-allchars");
    assert_eq!(
        index
            .entries
            .last()
            .expect("entries should not be empty")
            .headword,
        "bar"
    );
}

#[test]
fn test_parse_index_from_file_handles_case_insensitivity() {
    let r = parse_index_from_file(PATH_CASE_INSENSITIVE_INDEX, false);
    assert!(r.is_ok());

    let index = r.expect("parse should succeed");

    let r = index.find("bar", false);
    assert!(!r.is_empty());
    assert_eq!(
        r.first().expect("result should have entries").headword,
        "bar"
    );
}

#[test]
fn test_parse_index_from_file_handles_case_sensitivity() {
    let r = parse_index_from_file(PATH_CASE_SENSITIVE_INDEX, false);
    assert!(r.is_ok());

    let index = r.expect("parse should succeed");

    let r = index.find("Bar", false);
    assert!(!r.is_empty());
    assert_eq!(
        r.first().expect("result should have entries").headword,
        "Bar"
    );
}
