use super::css::CssParser;

#[test]
fn simple_css() {
    let text = "a, .b { b: c; d: e }";
    let css = CssParser::new(text).parse();
    assert!(!css.rules.is_empty());
}

#[test]
fn combinators_css() {
    let text = "a#i.j.k > b { b: c } a + .l { u: v } a { x: y }";
    let css = CssParser::new(text).parse();
    assert!(!css.rules.is_empty());
}
