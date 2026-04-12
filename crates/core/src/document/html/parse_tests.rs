use super::*;

#[test]
fn parse_color_test() {
    let a = parse_color("#000");
    let b = parse_color("#f00");
    let c = parse_color("#0f0");
    let d = parse_color("#00f");
    let e = parse_color("#fff");
    assert_eq!(a, Some(Color::Rgb(0, 0, 0)));
    assert_eq!(b, Some(Color::Rgb(255, 0, 0)));
    assert_eq!(c, Some(Color::Rgb(0, 255, 0)));
    assert_eq!(d, Some(Color::Rgb(0, 0, 255)));
    assert_eq!(e, Some(Color::Rgb(255, 255, 255)));
}
