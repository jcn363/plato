use super::css::CssParser;
use super::specified_values;
use super::xml::XmlParser;

#[test]
fn simple_style() {
    let xml1 = XmlParser::new("<a class='c x y' style='c: 7'/>").parse();
    let xml2 = XmlParser::new("<a id='e' class='x y'/>").parse();
    let mut css = CssParser::new(
        "a { b: 23 }\
         .c.x.y { b: 6; c: 3 }\
         #e { b: 5 }\
         .y { b: 2 }",
    )
    .parse();
    css.sort();
    let n1 = xml1.root().first_child().expect("xml root has no children");
    let n2 = xml2.root().first_child().expect("xml root has no children");
    assert_eq!(
        specified_values(n1, &css),
        [
            ("b".to_string(), "6".to_string()),
            ("c".to_string(), "7".to_string())
        ]
        .iter()
        .cloned()
        .collect()
    );
    assert_eq!(
        specified_values(n2, &css),
        [("b".to_string(), "5".to_string())]
            .iter()
            .cloned()
            .collect()
    );
}
