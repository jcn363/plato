use super::*;

#[test]
fn test_entities() {
    assert_eq!(decode_entities("a &amp b"), "a &amp b");
    assert_eq!(decode_entities("a &zZz; b"), "a &zZz; b");
    assert_eq!(decode_entities("a &amp; b"), "a & b");
    assert_eq!(decode_entities("a &#x003E; b"), "a > b");
    assert_eq!(decode_entities("a &#38; b"), "a & b");
    assert_eq!(decode_entities("a &lt; b &gt; c"), "a < b > c");
}
