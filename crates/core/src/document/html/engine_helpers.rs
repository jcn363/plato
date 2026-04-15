use super::layout::ListStyleType;
use septem::Roman;

pub(super) fn format_list_prefix(kind: ListStyleType, index: usize) -> Option<String> {
    match kind {
        ListStyleType::None => None,
        ListStyleType::Disc => Some("· ".to_string()),
        ListStyleType::Circle => Some("o ".to_string()),
        ListStyleType::Square => Some("· ".to_string()),
        ListStyleType::Decimal => Some(format!("{}. ", index + 1)),
        ListStyleType::LowerRoman => Some(format!(
            "{}. ",
            Roman::from_unchecked(index as u32 + 1).to_lowercase()
        )),
        ListStyleType::UpperRoman => Some(format!(
            "{}. ",
            Roman::from_unchecked(index as u32 + 1).to_uppercase()
        )),
        ListStyleType::LowerAlpha | ListStyleType::UpperAlpha => {
            let i = index as u32 % 26;
            let start = if kind == ListStyleType::LowerAlpha {
                0x61
            } else {
                0x41
            };
            Some(format!("{}. ", char::try_from(start + i).unwrap_or('?')))
        }
        ListStyleType::LowerGreek | ListStyleType::UpperGreek => {
            let mut i = index as u32 % 24;
            // Skip .
            if i >= 17 {
                i += 1;
            }
            let start = if kind == ListStyleType::LowerGreek {
                0x03B1
            } else {
                0x0391
            };
            Some(format!("{}. ", char::try_from(start + i).unwrap_or('?')))
        }
    }
}

pub(super) fn default_fonts() -> Result<super::layout::Fonts, anyhow::Error> {
    let opener = crate::font::FontOpener::new()?;
    let mut fonts = super::layout::Fonts {
        serif: crate::font::FontFamily {
            regular: opener.open("fonts/LibertinusSerif-Regular.otf")?,
            italic: opener.open("fonts/LibertinusSerif-Italic.otf")?,
            bold: opener.open("fonts/LibertinusSerif-Bold.otf")?,
            bold_italic: opener.open("fonts/LibertinusSerif-BoldItalic.otf")?,
        },
        sans_serif: crate::font::FontFamily {
            regular: opener.open("fonts/NotoSans-Regular.ttf")?,
            italic: opener.open("fonts/NotoSans-Italic.ttf")?,
            bold: opener.open("fonts/NotoSans-Bold.ttf")?,
            bold_italic: opener.open("fonts/NotoSans-BoldItalic.ttf")?,
        },
        monospace: crate::font::FontFamily {
            regular: opener.open("fonts/SourceCodeVariable-Roman.otf")?,
            italic: opener.open("fonts/SourceCodeVariable-Italic.otf")?,
            bold: opener.open("fonts/SourceCodeVariable-Roman.otf")?,
            bold_italic: opener.open("fonts/SourceCodeVariable-Italic.otf")?,
        },
        cursive: opener.open("fonts/Parisienne-Regular.ttf")?,
        fantasy: opener.open("fonts/Delius-Regular.ttf")?,
    };
    fonts.monospace.bold.set_variations(&["wght=600"]);
    fonts.monospace.bold_italic.set_variations(&["wght=600"]);
    Ok(fonts)
}
