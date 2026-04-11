use super::*;

const PATH_CASE_SENSITIVE_DICT: &str = "src/dictionary/testdata/case_sensitive_dict.dict";
const PATH_CASE_SENSITIVE_INDEX: &str = "src/dictionary/testdata/case_sensitive_dict.index";
const PATH_CASE_INSENSITIVE_DICT: &str = "src/dictionary/testdata/case_insensitive_dict.dict";
const PATH_CASE_INSENSITIVE_INDEX: &str = "src/dictionary/testdata/case_insensitive_dict.index";

fn assert_dict_word_exists(
    mut dict: Dictionary,
    headword: &str,
    definition: &str,
) -> Dictionary {
    let r = dict.lookup(headword, false);
    assert!(r.is_ok());
    let search = r.expect("lookup should succeed");
    assert_eq!(search.len(), 1);
    assert!(search[0][1].contains(definition));

    dict
}

#[test]
fn test_load_dictionary_from_file() {
    let r = load_dictionary_from_file(PATH_CASE_INSENSITIVE_DICT, PATH_CASE_INSENSITIVE_INDEX);
    assert!(r.is_ok());
}

#[test]
fn test_dictionary_lookup_case_insensitive() {
    let r = load_dictionary_from_file(PATH_CASE_INSENSITIVE_DICT, PATH_CASE_INSENSITIVE_INDEX);
    let mut dict = r.expect("load should succeed");

    dict = assert_dict_word_exists(dict, "bar", "test for case-sensitivity");
    dict = assert_dict_word_exists(dict, "Bar", "test for case-sensitivity");
    assert_dict_word_exists(dict, "straße", "test for non-latin case-sensitivity");
}

#[test]
fn test_dictionary_lookup_case_insensitive_fuzzy() {
    let r = load_dictionary_from_file(PATH_CASE_INSENSITIVE_DICT, PATH_CASE_INSENSITIVE_INDEX);
    let mut dict = r.expect("load should succeed");

    let r = dict.lookup("ba", true);
    assert!(r.is_ok());
    let search = r.expect("lookup should succeed");
    assert_eq!(search.len(), 1);
    assert_eq!(search[0][0], "bar");
    assert!(search[0][1].contains("test for case-sensitivity"));
}

#[test]
fn test_dictionary_lookup_case_sensitive() {
    let r = load_dictionary_from_file(PATH_CASE_SENSITIVE_DICT, PATH_CASE_SENSITIVE_INDEX);
    let mut dict = r.expect("load should succeed");

    dict = assert_dict_word_exists(dict, "Bar", "test for case-sensitivity");
    dict = assert_dict_word_exists(dict, "straße", "test for non-latin case-sensitivity");

    let r = dict.lookup("bar", false);
    assert!(r.expect("lookup should succeed").is_empty());

    let r = dict.lookup("strasse", false);
    assert!(r.expect("lookup should succeed").is_empty());
}

#[test]
fn test_dictionary_lookup_case_sensitive_fuzzy() {
    let r = load_dictionary_from_file(PATH_CASE_SENSITIVE_DICT, PATH_CASE_SENSITIVE_INDEX);
    let mut dict = r.expect("load should succeed");

    let r = dict.lookup("Ba", true);
    assert!(r.is_ok());
    let search = r.expect("lookup should succeed");
    assert_eq!(search.len(), 1);
    assert_eq!(search[0][0], "Bar");
    assert!(search[0][1].contains("test for case-sensitivity"));
}
