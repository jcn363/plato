use super::*;

#[test]
fn test_simple_element() {
    let text = "<a/>";
    let xml = XmlParser::new(text).parse();
    let n = xml.root().first_child().expect("xml root has no children");
    assert_eq!(n.offset(), 0);
    assert_eq!(n.tag_name(), Some("a"));
}

#[test]
fn test_attributes() {
    let text = "<a b='c' d=\"e\"/>";
    let xml = XmlParser::new(text).parse();
    let n = xml.root().first_child().expect("xml root has no children");
    assert_eq!(n.attribute("b"), Some("c".to_string()));
    assert_eq!(n.attribute("d"), Some("e".to_string()));
}

#[test]
fn test_text() {
    let text = "<a>bcd</a>";
    let xml = XmlParser::new(text).parse();
    let child = xml
        .root()
        .first_child()
        .expect("xml root has no children")
        .children()
        .next();
    assert_eq!(child.map(|c| c.offset()), Some(3));
    assert_eq!(child.map(|c| c.text()), Some("bcd".to_string()));
}

#[test]
fn test_inbetween_space() {
    let text = "<a><b>x</b> <c>y</c></a>";
    let xml = XmlParser::new(text).parse();
    let child = xml
        .root()
        .first_child()
        .expect("xml root has no children")
        .children()
        .nth(1);
    assert_eq!(child.map(|c| c.text()), Some(" ".to_string()));
}

#[test]
fn test_central_space() {
    let text = "<a><b> </b></a>";
    let xml = XmlParser::new(text).parse();
    assert_eq!(xml.root().text(), " ");
}
