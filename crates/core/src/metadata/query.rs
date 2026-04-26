use chrono::NaiveDateTime;
use regex::Regex;

use crate::helpers::datetime_format;
use crate::log_error;

use super::info::Info;
use super::info::SimpleStatus;

#[derive(Debug, Clone, Default)]
pub struct BookQuery {
    pub free: Option<Regex>,
    pub title: Option<Regex>,
    pub subtitle: Option<Regex>,
    pub author: Option<Regex>,
    pub year: Option<Regex>,
    pub language: Option<Regex>,
    pub publisher: Option<Regex>,
    pub series: Option<Regex>,
    pub edition: Option<Regex>,
    pub volume: Option<Regex>,
    pub number: Option<Regex>,
    pub reading: Option<bool>,
    pub new: Option<bool>,
    pub finished: Option<bool>,
    pub annotations: Option<bool>,
    pub bookmarks: Option<bool>,
    pub opened_after: Option<(bool, NaiveDateTime)>,
    pub added_after: Option<(bool, NaiveDateTime)>,
}

pub fn make_query(text: &str) -> Option<Regex> {
    let any = Regex::new(r"^(\.*|\s)$").expect("invalid regex pattern");

    if any.is_match(text) {
        return None;
    }

    let text = text
        .replace('a', "[aáàâä]")
        .replace('e', "[eéèêë]")
        .replace('i', "[iíìîï]")
        .replace('o', "[oóòôö]")
        .replace('u', "[uúùûü]")
        .replace('c', "[cç]")
        .replace("ae", "(ae|æ)")
        .replace("oe", "(oe|œ)");
    Regex::new(&format!("(?i){}", text))
        .map_err(|e| log_error!("Can't create query: {:#}.", e))
        .ok()
}

impl BookQuery {
    pub fn new(text: &str) -> Option<BookQuery> {
        if text.is_empty() {
            return None;
        }
        let mut buf = Vec::new();
        let mut query = BookQuery::default();
        for word in text.rsplit(' ') {
            let mut chars = word.chars().peekable();
            match chars.next() {
                Some('\'') => {
                    let invert = Self::parse_inverted_flag(&mut chars);
                    match chars.next() {
                        Some('t') => Self::set_text_field(&mut buf, &mut query.title),
                        Some('u') => Self::set_text_field(&mut buf, &mut query.subtitle),
                        Some('a') => Self::set_text_field(&mut buf, &mut query.author),
                        Some('y') => Self::set_text_field(&mut buf, &mut query.year),
                        Some('l') => Self::set_text_field(&mut buf, &mut query.language),
                        Some('p') => Self::set_text_field(&mut buf, &mut query.publisher),
                        Some('s') => Self::set_text_field(&mut buf, &mut query.series),
                        Some('e') => Self::set_text_field(&mut buf, &mut query.edition),
                        Some('v') => Self::set_text_field(&mut buf, &mut query.volume),
                        Some('n') => Self::set_text_field(&mut buf, &mut query.number),
                        Some('R') => query.reading = Some(!invert),
                        Some('N') => query.new = Some(!invert),
                        Some('F') => query.finished = Some(!invert),
                        Some('A') => query.annotations = Some(!invert),
                        Some('B') => query.bookmarks = Some(!invert),
                        Some('O') => {
                            Self::set_date_field(&mut buf, invert, &mut query.opened_after)
                        }
                        Some('D') => Self::set_date_field(&mut buf, invert, &mut query.added_after),
                        Some('\'') => buf.push(&word[1..]),
                        _ => (),
                    }
                }
                _ => buf.push(word),
            }
        }
        buf.reverse();
        query.free = make_query(&buf.join(" "));
        if Self::is_query_empty(&query) {
            None
        } else {
            Some(query)
        }
    }

    fn parse_inverted_flag(chars: &mut std::iter::Peekable<std::str::Chars>) -> bool {
        let mut invert = false;
        if chars.peek() == Some(&'!') {
            invert = true;
            chars.next();
        }
        invert
    }

    fn set_text_field(buf: &mut Vec<&str>, field: &mut Option<Regex>) {
        buf.reverse();
        *field = make_query(&buf.join(" "));
        buf.clear();
    }

    fn set_date_field(
        buf: &mut Vec<&str>,
        invert: bool,
        field: &mut Option<(bool, NaiveDateTime)>,
    ) {
        buf.reverse();
        *field = NaiveDateTime::parse_from_str(&buf.join(" "), datetime_format::FORMAT)
            .ok()
            .map(|d| (!invert, d));
        buf.clear();
    }

    fn is_query_empty(query: &BookQuery) -> bool {
        query.free.is_none()
            && query.title.is_none()
            && query.subtitle.is_none()
            && query.author.is_none()
            && query.year.is_none()
            && query.language.is_none()
            && query.publisher.is_none()
            && query.series.is_none()
            && query.edition.is_none()
            && query.volume.is_none()
            && query.number.is_none()
            && query.reading.is_none()
            && query.new.is_none()
            && query.finished.is_none()
            && query.annotations.is_none()
            && query.bookmarks.is_none()
            && query.opened_after.is_none()
            && query.added_after.is_none()
    }

    #[inline]
    pub fn is_match(&self, info: &Info) -> bool {
        self.matches_free_field(info)
            && self.matches_text_field(&self.title, &info.title)
            && self.matches_text_field(&self.subtitle, &info.subtitle)
            && self.matches_text_field(&self.author, &info.author)
            && self.matches_text_field(&self.year, &info.year)
            && self.matches_text_field(&self.language, &info.language)
            && self.matches_text_field(&self.publisher, &info.publisher)
            && self.matches_text_field(&self.series, &info.series)
            && self.matches_text_field(&self.edition, &info.edition)
            && self.matches_text_field(&self.volume, &info.volume)
            && self.matches_text_field(&self.number, &info.number)
            && self.matches_status_field(&self.reading, info, SimpleStatus::Reading)
            && self.matches_status_field(&self.new, info, SimpleStatus::New)
            && self.matches_status_field(&self.finished, info, SimpleStatus::Finished)
            && self.matches_reader_field(&self.annotations, info, |r| !r.annotations.is_empty())
            && self.matches_reader_field(&self.bookmarks, info, |r| !r.bookmarks.is_empty())
            && self.matches_date_field(&self.opened_after, info, |i| {
                i.reader_info.as_ref().map(|r| r.opened).unwrap_or_default()
            })
            && self.matches_date_field(&self.added_after, info, |_| info.added)
    }

    fn matches_free_field(&self, info: &Info) -> bool {
        self.free.as_ref().is_none_or(|re| {
            re.is_match(&info.title)
                || re.is_match(&info.subtitle)
                || re.is_match(&info.author)
                || re.is_match(&info.series)
                || info.file.path.to_str().is_some_and(|s| re.is_match(s))
        })
    }

    fn matches_text_field(&self, field: &Option<Regex>, value: &str) -> bool {
        field.as_ref().is_none_or(|re| re.is_match(value))
    }

    fn matches_status_field(
        &self,
        field: &Option<bool>,
        info: &Info,
        expected_status: SimpleStatus,
    ) -> bool {
        field
            .as_ref()
            .is_none_or(|eq| info.simple_status().eq(&expected_status) == *eq)
    }

    fn matches_reader_field<F>(&self, field: &Option<bool>, info: &Info, check: F) -> bool
    where
        F: FnOnce(&crate::metadata::info::ReaderInfo) -> bool,
    {
        field
            .as_ref()
            .is_none_or(|eq| info.reader.as_ref().is_some_and(|r| check(r) == *eq))
    }

    fn matches_date_field<F>(
        &self,
        field: &Option<(bool, NaiveDateTime)>,
        info: &Info,
        get_date: F,
    ) -> bool
    where
        F: FnOnce(&Info) -> NaiveDateTime,
    {
        field
            .as_ref()
            .is_none_or(|(eq, date)| get_date(info).gt(date) == *eq)
    }

    #[inline]
    pub fn is_simple_match(&self, text: &str) -> bool {
        self.free.as_ref().is_none_or(|q| q.is_match(text))
    }
}
