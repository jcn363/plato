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
        let mut buf = Vec::new();
        let mut query = BookQuery::default();
        for word in text.rsplit(' ') {
            let mut chars = word.chars().peekable();
            match chars.next() {
                Some('\'') => {
                    let mut invert = false;
                    if chars.peek() == Some(&'!') {
                        invert = true;
                        chars.next();
                    }
                    match chars.next() {
                        Some('t') => {
                            buf.reverse();
                            query.title = make_query(&buf.join(" "));
                            buf.clear();
                        }
                        Some('u') => {
                            buf.reverse();
                            query.subtitle = make_query(&buf.join(" "));
                            buf.clear();
                        }
                        Some('a') => {
                            buf.reverse();
                            query.author = make_query(&buf.join(" "));
                            buf.clear();
                        }
                        Some('y') => {
                            buf.reverse();
                            query.year = make_query(&buf.join(" "));
                            buf.clear();
                        }
                        Some('l') => {
                            buf.reverse();
                            query.language = make_query(&buf.join(" "));
                            buf.clear();
                        }
                        Some('p') => {
                            buf.reverse();
                            query.publisher = make_query(&buf.join(" "));
                            buf.clear();
                        }
                        Some('s') => {
                            buf.reverse();
                            query.series = make_query(&buf.join(" "));
                            buf.clear();
                        }
                        Some('e') => {
                            buf.reverse();
                            query.edition = make_query(&buf.join(" "));
                            buf.clear();
                        }
                        Some('v') => {
                            buf.reverse();
                            query.volume = make_query(&buf.join(" "));
                            buf.clear();
                        }
                        Some('n') => {
                            buf.reverse();
                            query.number = make_query(&buf.join(" "));
                            buf.clear();
                        }
                        Some('R') => query.reading = Some(!invert),
                        Some('N') => query.new = Some(!invert),
                        Some('F') => query.finished = Some(!invert),
                        Some('A') => query.annotations = Some(!invert),
                        Some('B') => query.bookmarks = Some(!invert),
                        Some('O') => {
                            buf.reverse();
                            query.opened_after = NaiveDateTime::parse_from_str(
                                &buf.join(" "),
                                datetime_format::FORMAT,
                            )
                            .ok()
                            .map(|opened| (!invert, opened));
                            buf.clear();
                        }
                        Some('D') => {
                            buf.reverse();
                            query.added_after = NaiveDateTime::parse_from_str(
                                &buf.join(" "),
                                datetime_format::FORMAT,
                            )
                            .ok()
                            .map(|added| (!invert, added));
                            buf.clear();
                        }
                        Some('\'') => buf.push(&word[1..]),
                        _ => (),
                    }
                }
                _ => buf.push(word),
            }
        }
        buf.reverse();
        query.free = make_query(&buf.join(" "));
        if query.free.is_none()
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
        {
            None
        } else {
            Some(query)
        }
    }

    #[inline]
    pub fn is_match(&self, info: &Info) -> bool {
        self.free.as_ref().map(|re| {
            re.is_match(&info.title)
                || re.is_match(&info.subtitle)
                || re.is_match(&info.author)
                || re.is_match(&info.series)
                || info.file.path.to_str().map_or(false, |s| re.is_match(s))
        }) != Some(false)
            && self.title.as_ref().map(|re| re.is_match(&info.title)) != Some(false)
            && self.subtitle.as_ref().map(|re| re.is_match(&info.subtitle)) != Some(false)
            && self.author.as_ref().map(|re| re.is_match(&info.author)) != Some(false)
            && self.year.as_ref().map(|re| re.is_match(&info.year)) != Some(false)
            && self.language.as_ref().map(|re| re.is_match(&info.language)) != Some(false)
            && self
                .publisher
                .as_ref()
                .map(|re| re.is_match(&info.publisher))
                != Some(false)
            && self.series.as_ref().map(|re| re.is_match(&info.series)) != Some(false)
            && self.edition.as_ref().map(|re| re.is_match(&info.edition)) != Some(false)
            && self.volume.as_ref().map(|re| re.is_match(&info.volume)) != Some(false)
            && self.number.as_ref().map(|re| re.is_match(&info.number)) != Some(false)
            && self
                .reading
                .as_ref()
                .map(|eq| info.simple_status().eq(&SimpleStatus::Reading) == *eq)
                != Some(false)
            && self
                .new
                .as_ref()
                .map(|eq| info.simple_status().eq(&SimpleStatus::New) == *eq)
                != Some(false)
            && self
                .finished
                .as_ref()
                .map(|eq| info.simple_status().eq(&SimpleStatus::Finished) == *eq)
                != Some(false)
            && self.annotations.as_ref().map(|eq| {
                info.reader
                    .as_ref()
                    .map_or(false, |r| !r.annotations.is_empty())
                    == *eq
            }) != Some(false)
            && self.bookmarks.as_ref().map(|eq| {
                info.reader
                    .as_ref()
                    .map_or(false, |r| !r.bookmarks.is_empty())
                    == *eq
            }) != Some(false)
            && self.opened_after.as_ref().map(|(eq, opened)| {
                info.reader.as_ref().map_or(false, |r| r.opened.gt(opened)) == *eq
            }) != Some(false)
            && self
                .added_after
                .as_ref()
                .map(|(eq, added)| info.added.gt(added) == *eq)
                != Some(false)
    }

    #[inline]
    pub fn is_simple_match(&self, text: &str) -> bool {
        self.free.as_ref().map_or(true, |q| q.is_match(text))
    }
}
