use super::layout::{Display, Float, ListStyleType, TextAlign, TextOverflow};
use super::layout::{FontKind, FontStyle, FontWeight, WordSpacing};
use super::layout::{GlueMaterial, InlineMaterial, PenaltyMaterial};
use crate::color::{Color, BLACK, WHITE};
use crate::geom::Edge;
use crate::unit::{in_to_px, mm_to_px, pc_to_px, pt_to_px};
use crate::unit::{CENTIMETERS_PER_INCH, MILLIMETERS_PER_INCH, PICAS_PER_INCH, POINTS_PER_INCH};
use fxhash::FxHashSet;
use regex::Regex;

const SIZE_FACTOR: f32 = 1.26;
const ABSOLUTE_SIZE_KEYWORDS: [&str; 7] = [
    "xx-small", "x-small", "small", "medium", "large", "x-large", "xx-large",
];
const RELATIVE_SIZE_KEYWORDS: [&str; 2] = ["smaller", "larger"];

pub fn parse_length(value: &str, em: f32, rem: f32, dpi: u16) -> Option<i32> {
    if let Some(index) = value.find(|c: char| c.is_ascii_alphabetic()) {
        value[..index]
            .parse()
            .ok()
            .and_then(|v| match &value[index..] {
                "em" => Some(pt_to_px(v * em, dpi).round() as i32),
                "rem" => Some(pt_to_px(v * rem, dpi).round() as i32),
                "pt" => Some(pt_to_px(v, dpi).round() as i32),
                "pc" => Some(pc_to_px(v, dpi).round() as i32),
                "cm" => Some(mm_to_px(10.0 * v, dpi).round() as i32),
                "mm" => Some(mm_to_px(v, dpi).round() as i32),
                "in" => Some(in_to_px(v, dpi).round() as i32),
                "px" => Some(pt_to_px(v * 0.75, dpi).round() as i32),
                _ => None,
            })
    } else if value == "0" {
        Some(0)
    } else {
        None
    }
}

pub fn parse_viewport_length(
    value: &str,
    viewport_width: i32,
    viewport_height: i32,
    dpi: u16,
) -> Option<i32> {
    let vmin = viewport_width.min(viewport_height) as f32;
    let vmax = viewport_width.max(viewport_height) as f32;

    if let Some(index) = value.find(|c: char| c.is_ascii_alphabetic()) {
        value[..index]
            .parse()
            .ok()
            .and_then(|v| match &value[index..] {
                "vw" => Some((v / 100.0 * viewport_width as f32) as i32),
                "vh" => Some((v / 100.0 * viewport_height as f32) as i32),
                "vmin" => Some((v / 100.0 * vmin) as i32),
                "vmax" => Some((v / 100.0 * vmax) as i32),
                "em" => Some(pt_to_px(v, dpi).round() as i32),
                "rem" => Some(pt_to_px(v, dpi).round() as i32),
                "pt" => Some(pt_to_px(v, dpi).round() as i32),
                "px" => Some(pt_to_px(v * 0.75, dpi).round() as i32),
                _ => None,
            })
    } else if value == "0" {
        Some(0)
    } else {
        None
    }
}

// Input and output sizes are in points.
pub fn parse_font_size(value: &str, em: f32, rem: f32) -> Option<f32> {
    if value.find(|c: char| c.is_ascii_digit()).is_some() {
        if let Some(index) = value.find(|c: char| c.is_ascii_alphabetic()) {
            value[..index].parse().ok().and_then(|v| {
                if v <= 0.0 {
                    return None;
                }
                match &value[index..] {
                    "em" => Some(v * em),
                    "rem" => Some(v * rem),
                    "pt" => Some(v),
                    "pc" => Some(v * POINTS_PER_INCH / PICAS_PER_INCH),
                    "cm" => Some(v * POINTS_PER_INCH / CENTIMETERS_PER_INCH),
                    "mm" => Some(v * POINTS_PER_INCH / MILLIMETERS_PER_INCH),
                    "in" => Some(v * POINTS_PER_INCH),
                    "px" => Some(v * 0.75),
                    _ => None,
                }
            })
        } else if let Some(percent) = value.strip_suffix('%') {
            percent.parse::<f32>().ok().map(|v| v / 100.0 * em)
        } else {
            None
        }
    } else if let Some(index) = ABSOLUTE_SIZE_KEYWORDS.iter().position(|&v| v == value) {
        let e = index as i32 - 3;
        Some(SIZE_FACTOR.powi(e) * rem)
    } else if let Some(index) = RELATIVE_SIZE_KEYWORDS.iter().position(|&v| v == value) {
        let e = (2 * index) as i32 - 1;
        Some(SIZE_FACTOR.powi(e) * em)
    } else {
        None
    }
}

pub fn parse_inline_material(value: &str, em: f32, rem: f32, dpi: u16) -> Vec<InlineMaterial> {
    let split = value.split(',');
    let count = split.clone().count();
    let mut inlines = Vec::with_capacity(count);
    for decl in split {
        let tokens: Vec<&str> = decl.trim().split_whitespace().collect();
        match tokens.get(0).cloned() {
            Some("glue") => {
                let width = tokens
                    .get(1)
                    .and_then(|s| parse_length(s, em, rem, dpi))
                    .unwrap_or(0);
                let stretch = tokens
                    .get(2)
                    .and_then(|s| parse_length(s, em, rem, dpi))
                    .unwrap_or(0);
                let shrink = tokens
                    .get(3)
                    .and_then(|s| parse_length(s, em, rem, dpi))
                    .unwrap_or(0);
                inlines.push(InlineMaterial::Glue(GlueMaterial {
                    width,
                    stretch,
                    shrink,
                }));
            }
            Some("penalty") => {
                let penalty = tokens.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let flagged = tokens.get(2).and_then(|s| s.parse().ok()).unwrap_or(false);
                let width = tokens
                    .get(3)
                    .and_then(|s| parse_length(s, em, rem, dpi))
                    .unwrap_or(0);
                inlines.push(InlineMaterial::Penalty(PenaltyMaterial {
                    width,
                    penalty,
                    flagged,
                }));
            }
            Some("box") => {
                let width = tokens
                    .get(1)
                    .and_then(|s| parse_length(s, em, rem, dpi))
                    .unwrap_or(0);
                inlines.push(InlineMaterial::Box(width));
            }
            _ => (),
        }
    }
    inlines
}

pub fn parse_font_kind(value: &str) -> Option<FontKind> {
    value
        .split(',')
        .last()
        .map(str::trim)
        .and_then(|v| match v {
            "serif" => Some(FontKind::Serif),
            "sans-serif" => Some(FontKind::SansSerif),
            "monospace" => Some(FontKind::Monospace),
            "cursive" => Some(FontKind::Cursive),
            "fantasy" => Some(FontKind::Fantasy),
            _ => None,
        })
}

pub fn parse_letter_spacing(value: &str, em: f32, rem: f32, dpi: u16) -> Option<i32> {
    if value == "normal" {
        Some(0)
    } else {
        parse_length(value, em, rem, dpi)
    }
}

pub fn parse_word_spacing(value: &str, em: f32, rem: f32, dpi: u16) -> Option<WordSpacing> {
    if value == "normal" {
        Some(WordSpacing::Normal)
    } else if let Some(percent) = value.strip_suffix('%') {
        percent
            .parse::<f32>()
            .ok()
            .map(|v| WordSpacing::Ratio(v / 100.0))
    } else {
        parse_length(value, em, rem, dpi).map(WordSpacing::Length)
    }
}

pub fn parse_vertical_align(
    value: &str,
    em: f32,
    rem: f32,
    line_height: i32,
    dpi: u16,
) -> Option<i32> {
    if value == "baseline" {
        Some(0)
    } else if value == "super" || value == "top" {
        Some(pt_to_px(0.4 * em, dpi).round() as i32)
    } else if value == "sub" || value == "bottom" {
        Some(pt_to_px(-0.2 * em, dpi).round() as i32)
    } else if let Some(percent) = value.strip_suffix('%') {
        percent
            .parse::<f32>()
            .ok()
            .map(|v| (v / 100.0 * line_height as f32) as i32)
    } else {
        parse_length(value, em, rem, dpi)
    }
}

pub fn parse_font_weight(value: &str) -> Option<FontWeight> {
    if value == "normal" {
        Some(FontWeight::Normal)
    } else if value == "bold" {
        Some(FontWeight::Bold)
    } else {
        None
    }
}

pub fn parse_font_style(value: &str) -> Option<FontStyle> {
    if value == "normal" {
        Some(FontStyle::Normal)
    } else if value == "italic" {
        Some(FontStyle::Italic)
    } else {
        None
    }
}

pub fn parse_display(value: &str) -> Option<Display> {
    match value {
        "block" => Some(Display::Block),
        "inline" => Some(Display::Inline),
        "inline-block" => Some(Display::InlineBlock),
        "inline-table" => Some(Display::InlineTable),
        "table" => Some(Display::Table),
        "table-cell" => Some(Display::TableCell),
        "table-row" => Some(Display::TableRow),
        "table-row-group" => Some(Display::TableRowGroup),
        "table-header-group" => Some(Display::TableHeaderGroup),
        "table-footer-group" => Some(Display::TableFooterGroup),
        "list-item" => Some(Display::ListItem),
        "none" => Some(Display::None),
        _ => None,
    }
}

pub fn parse_white_space(value: &str) -> Option<(bool, bool)> {
    match value {
        "normal" => Some((false, false)),
        "nowrap" => Some((false, true)),
        "pre" => Some((true, true)),
        "pre-wrap" => Some((true, false)),
        "pre-line" => Some((true, true)),
        _ => None,
    }
}

pub fn parse_float(value: &str) -> Option<Float> {
    match value {
        "left" => Some(Float::Left),
        "right" => Some(Float::Right),
        _ => None,
    }
}

pub fn parse_list_style_type(value: &str) -> Option<ListStyleType> {
    match value {
        "none" => Some(ListStyleType::None),
        "disc" => Some(ListStyleType::Disc),
        "circle" => Some(ListStyleType::Circle),
        "square" => Some(ListStyleType::Square),
        "decimal" => Some(ListStyleType::Decimal),
        "lower-roman" => Some(ListStyleType::LowerRoman),
        "upper-roman" => Some(ListStyleType::UpperRoman),
        "lower-alpha" | "lower-latin" => Some(ListStyleType::LowerAlpha),
        "upper-alpha" | "upper-latin" => Some(ListStyleType::UpperAlpha),
        "lower-greek" => Some(ListStyleType::LowerGreek),
        "upper-greek" => Some(ListStyleType::UpperGreek),
        _ => None,
    }
}

pub fn parse_width(value: &str, em: f32, rem: f32, width: i32, dpi: u16) -> Option<i32> {
    if value == "auto" {
        Some(0)
    } else if let Some(percent) = value.strip_suffix('%') {
        percent
            .parse::<f32>()
            .ok()
            .map(|v| (v / 100.0 * width as f32) as i32)
    } else {
        parse_length(value, em, rem, dpi)
    }
}

pub fn parse_height(value: &str, em: f32, rem: f32, width: i32, dpi: u16) -> Option<i32> {
    if value == "auto" {
        Some(0)
    } else if let Some(percent) = value.strip_suffix('%') {
        percent
            .parse::<f32>()
            .ok()
            .map(|v| (v / 100.0 * width as f32) as i32)
    } else {
        parse_length(value, em, rem, dpi)
    }
}

pub fn parse_min_width(value: &str, em: f32, rem: f32, width: i32, dpi: u16) -> Option<i32> {
    if value == "auto" || value == "none" {
        Some(0)
    } else if let Some(percent) = value.strip_suffix('%') {
        percent
            .parse::<f32>()
            .ok()
            .map(|v| (v / 100.0 * width as f32) as i32)
    } else {
        parse_length(value, em, rem, dpi)
    }
}

pub fn parse_max_width(value: &str, em: f32, rem: f32, width: i32, dpi: u16) -> Option<i32> {
    if value == "none" {
        Some(i32::MAX)
    } else if let Some(percent) = value.strip_suffix('%') {
        percent
            .parse::<f32>()
            .ok()
            .map(|v| (v / 100.0 * width as f32) as i32)
    } else {
        parse_length(value, em, rem, dpi)
    }
}

pub fn parse_min_height(value: &str, em: f32, rem: f32, width: i32, dpi: u16) -> Option<i32> {
    if value == "auto" || value == "none" {
        Some(0)
    } else if let Some(percent) = value.strip_suffix('%') {
        percent
            .parse::<f32>()
            .ok()
            .map(|v| (v / 100.0 * width as f32) as i32)
    } else {
        parse_length(value, em, rem, dpi)
    }
}

pub fn parse_max_height(value: &str, em: f32, rem: f32, width: i32, dpi: u16) -> Option<i32> {
    if value == "none" {
        Some(i32::MAX)
    } else if let Some(percent) = value.strip_suffix('%') {
        percent
            .parse::<f32>()
            .ok()
            .map(|v| (v / 100.0 * width as f32) as i32)
    } else {
        parse_length(value, em, rem, dpi)
    }
}

fn parse_edge_length(value: &str, em: f32, rem: f32, width: i32, auto_value: i32, dpi: u16) -> i32 {
    if value == "auto" {
        auto_value
    } else if value == "0" {
        0
    } else if let Some(percent) = value.strip_suffix('%') {
        percent
            .parse::<f32>()
            .ok()
            .map(|v| (v / 100.0 * width as f32) as i32)
            .unwrap_or_default()
    } else {
        parse_length(value, em, rem, dpi).unwrap_or_default()
    }
}

pub fn parse_edge(
    top_edge: Option<&str>,
    right_edge: Option<&str>,
    bottom_edge: Option<&str>,
    left_edge: Option<&str>,
    em: f32,
    rem: f32,
    width: i32,
    dpi: u16,
) -> Edge {
    let mut e = Edge::default();

    if let Some(v) = top_edge {
        e.top = parse_edge_length(v, em, rem, width, 0, dpi);
    }

    if let Some(v) = right_edge {
        e.right = parse_edge_length(v, em, rem, width, width, dpi);
    }

    if let Some(v) = bottom_edge {
        e.bottom = parse_edge_length(v, em, rem, width, 0, dpi);
    }

    if let Some(v) = left_edge {
        e.left = parse_edge_length(v, em, rem, width, width, dpi);
    }

    e
}

pub fn parse_text_align(value: &str) -> Option<TextAlign> {
    match value {
        "justify" => Some(TextAlign::Justify),
        "left" => Some(TextAlign::Left),
        "right" => Some(TextAlign::Right),
        "center" => Some(TextAlign::Center),
        _ => None,
    }
}

pub fn parse_text_transform(value: &str) -> Option<super::layout::TextTransform> {
    match value {
        "none" => Some(super::layout::TextTransform::None),
        "uppercase" => Some(super::layout::TextTransform::Uppercase),
        "lowercase" => Some(super::layout::TextTransform::Lowercase),
        "capitalize" => Some(super::layout::TextTransform::Capitalize),
        _ => None,
    }
}

pub fn parse_text_decoration(value: &str) -> Option<super::layout::TextDecoration> {
    match value {
        "none" => Some(super::layout::TextDecoration::None),
        "underline" => Some(super::layout::TextDecoration::Underline),
        "overline" => Some(super::layout::TextDecoration::Overline),
        "line-through" => Some(super::layout::TextDecoration::LineThrough),
        _ => None,
    }
}

pub fn parse_tab_size(value: &str, em: f32, dpi: u16) -> Option<super::layout::TabSize> {
    if let Ok(n) = value.parse::<i32>() {
        Some(super::layout::TabSize::Number(n))
    } else if let Some(n) = parse_length(value, em, em, dpi) {
        Some(super::layout::TabSize::Length(n))
    } else {
        None
    }
}

pub fn parse_direction(value: &str) -> Option<super::layout::Direction> {
    match value {
        "ltr" => Some(super::layout::Direction::Ltr),
        "rtl" => Some(super::layout::Direction::Rtl),
        "auto" => Some(super::layout::Direction::Auto),
        _ => None,
    }
}

pub fn parse_line_height(value: &str, em: f32, rem: f32, dpi: u16) -> Option<i32> {
    if value == "normal" {
        Some(pt_to_px(1.2 * em, dpi).round() as i32)
    } else if let Some(percent) = value.strip_suffix('%') {
        percent
            .parse::<f32>()
            .ok()
            .map(|v| pt_to_px(v / 100.0 * em as f32, dpi).round() as i32)
    } else if value.ends_with(|c: char| !c.is_ascii_alphabetic()) {
        value
            .parse::<f32>()
            .ok()
            .map(|v| pt_to_px(v * em, dpi).round() as i32)
    } else {
        parse_length(value, em, rem, dpi)
    }
}

pub fn parse_text_indent(value: &str, em: f32, rem: f32, width: i32, dpi: u16) -> Option<i32> {
    if let Some(percent) = value.strip_suffix('%') {
        percent
            .parse::<f32>()
            .ok()
            .map(|v| (v / 100.0 * width as f32) as i32)
    } else {
        parse_length(value, em, rem, dpi)
    }
}

pub fn parse_text_overflow(value: &str) -> Option<TextOverflow> {
    match value {
        "clip" => Some(TextOverflow::Clip),
        "ellipsis" => Some(TextOverflow::Ellipsis),
        _ => None,
    }
}

pub fn parse_font_features(value: &str) -> Vec<String> {
    let re = Regex::new(r#""([^"]+)"\s*(on|off|\d+)?"#).unwrap();
    let mut features = Vec::new();

    for cap in re.captures_iter(value) {
        let mut name = cap[1].to_string();
        let value = cap.get(2).map_or("", |m| m.as_str());
        match value {
            "off" | "0" => name = format!("-{}", name),
            "on" | "1" | "" => (),
            _ => name = format!("{}={}", name, value),
        }
        features.push(name);
    }

    features
}

pub fn parse_font_variant(value: &str) -> Vec<String> {
    let mut features = FxHashSet::default();

    for name in value.split_whitespace() {
        match name {
            "small-caps" => {
                features.insert("smcp");
            }
            "all-small-caps" => {
                features.insert("smcp");
                features.insert("c2sc");
            }
            "oldstyle-nums" => {
                features.insert("onum");
            }
            "lining-nums" => {
                features.insert("lnum");
            }
            "tabular-nums" => {
                features.insert("tnum");
            }
            "proportional-nums" => {
                features.insert("pnum");
            }
            "contextual" => {
                features.insert("clig");
            }
            "discretionary-ligatures" => {
                features.insert("clig");
                features.insert("dlig");
            }
            "slashed-zero" => {
                features.insert("zero");
            }
            _ => (),
        }
    }

    features.into_iter().map(String::from).collect()
}

pub fn parse_color(value: &str) -> Option<Color> {
    if value.starts_with('#') {
        if value.len() < 4 {
            return None;
        }
        let chunk_size = if value.len() < 7 { 1 } else { 2 };
        let red = u8::from_str_radix(&value[1..=chunk_size].repeat(3 - chunk_size), 16).ok()?;
        let green = u8::from_str_radix(
            &value[chunk_size + 1..=2 * chunk_size].repeat(3 - chunk_size),
            16,
        )
        .ok()?;
        let blue = u8::from_str_radix(
            &value[2 * chunk_size + 1..=3 * chunk_size].repeat(3 - chunk_size),
            16,
        )
        .ok()?;
        let color = Color::from_rgb(&[red, green, blue]);
        Some(color)
    } else {
        match value {
            "black" => Some(BLACK),
            "white" => Some(WHITE),
            "gray" | "grey" => parse_color("#888"),
            "silver" => parse_color("#c0c0c0"),
            "red" => parse_color("#f00"),
            "maroon" => parse_color("#800000"),
            "orange" => parse_color("#ffA500"),
            "yellow" => parse_color("#ff0"),
            "olive" => parse_color("#808000"),
            "lime" => parse_color("#0f0"),
            "green" => parse_color("#008000"),
            "aqua" | "cyan" => parse_color("#0ff"),
            "teal" => parse_color("#008080"),
            "blue" => parse_color("#00f"),
            "navy" => parse_color("#000080"),
            "fuchsia" | "magenta" => parse_color("#f0f"),
            "purple" => parse_color("#800080"),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color() {
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
}
