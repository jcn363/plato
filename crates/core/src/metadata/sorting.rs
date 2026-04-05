use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

use super::info::{Info, SimpleStatus, Status};

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum SortMethod {
    Opened,
    Added,
    Status,
    Progress,
    Title,
    Year,
    Author,
    Series,
    Pages,
    Size,
    Kind,
    FileName,
    FilePath,
}

impl SortMethod {
    pub fn reverse_order(self) -> bool {
        !matches!(
            self,
            SortMethod::Author
                | SortMethod::Title
                | SortMethod::Series
                | SortMethod::Kind
                | SortMethod::FileName
                | SortMethod::FilePath
        )
    }

    pub fn is_status_related(self) -> bool {
        matches!(
            self,
            SortMethod::Opened | SortMethod::Status | SortMethod::Progress
        )
    }

    pub fn label(&self) -> &str {
        match *self {
            SortMethod::Opened => "Date Opened",
            SortMethod::Added => "Date Added",
            SortMethod::Status => "Status",
            SortMethod::Progress => "Progress",
            SortMethod::Author => "Author",
            SortMethod::Title => "Title",
            SortMethod::Year => "Year",
            SortMethod::Series => "Series",
            SortMethod::Size => "File Size",
            SortMethod::Kind => "File Type",
            SortMethod::Pages => "Pages Count",
            SortMethod::FileName => "File Name",
            SortMethod::FilePath => "File Path",
        }
    }

    pub fn title(self) -> String {
        format!("Sort by: {}", self.label())
    }
}

pub fn sort(md: &mut super::info::Metadata, sort_method: SortMethod, reverse_order: bool) {
    let sort_fn = sorter(sort_method);

    if reverse_order {
        md.sort_by(|a, b| sort_fn(a, b).reverse());
    } else {
        md.sort_by(sort_fn);
    }
}

#[inline]
pub fn sorter(sort_method: SortMethod) -> fn(&Info, &Info) -> Ordering {
    match sort_method {
        SortMethod::Opened => sort_opened,
        SortMethod::Added => sort_added,
        SortMethod::Status => sort_status,
        SortMethod::Progress => sort_progress,
        SortMethod::Author => sort_author,
        SortMethod::Title => sort_title,
        SortMethod::Year => sort_year,
        SortMethod::Series => sort_series,
        SortMethod::Size => sort_size,
        SortMethod::Kind => sort_kind,
        SortMethod::Pages => sort_pages,
        SortMethod::FileName => sort_filename,
        SortMethod::FilePath => sort_filepath,
    }
}

pub fn sort_opened(i1: &Info, i2: &Info) -> Ordering {
    i1.reader
        .as_ref()
        .map(|r1| r1.opened)
        .cmp(&i2.reader.as_ref().map(|r2| r2.opened))
}

pub fn sort_added(i1: &Info, i2: &Info) -> Ordering {
    i1.added.cmp(&i2.added)
}

pub fn sort_pages(i1: &Info, i2: &Info) -> Ordering {
    i1.reader
        .as_ref()
        .map(|r1| r1.pages_count)
        .cmp(&i2.reader.as_ref().map(|r2| r2.pages_count))
}

pub fn sort_author(i1: &Info, i2: &Info) -> Ordering {
    let a1 = i1.alphabetic_author().to_lowercase();
    let a2 = i2.alphabetic_author().to_lowercase();
    a1.cmp(&a2)
}

pub fn sort_title(i1: &Info, i2: &Info) -> Ordering {
    let t1 = i1.alphabetic_title().to_lowercase();
    let t2 = i2.alphabetic_title().to_lowercase();
    t1.cmp(&t2)
}

pub fn sort_status(i1: &Info, i2: &Info) -> Ordering {
    match (i1.simple_status(), i2.simple_status()) {
        (SimpleStatus::Reading, SimpleStatus::Reading)
        | (SimpleStatus::Finished, SimpleStatus::Finished) => sort_opened(i1, i2),
        (SimpleStatus::New, SimpleStatus::New) => sort_added(i1, i2),
        (SimpleStatus::New, SimpleStatus::Finished) => Ordering::Greater,
        (SimpleStatus::Finished, SimpleStatus::New) => Ordering::Less,
        (SimpleStatus::New, SimpleStatus::Reading) => Ordering::Less,
        (SimpleStatus::Reading, SimpleStatus::New) => Ordering::Greater,
        (SimpleStatus::Finished, SimpleStatus::Reading) => Ordering::Less,
        (SimpleStatus::Reading, SimpleStatus::Finished) => Ordering::Greater,
    }
}

pub fn sort_progress(i1: &Info, i2: &Info) -> Ordering {
    match (i1.status(), i2.status()) {
        (Status::Finished, Status::Finished) => Ordering::Equal,
        (Status::New, Status::New) => Ordering::Equal,
        (Status::New, Status::Finished) => Ordering::Greater,
        (Status::Finished, Status::New) => Ordering::Less,
        (Status::New, Status::Reading(..)) => Ordering::Less,
        (Status::Reading(..), Status::New) => Ordering::Greater,
        (Status::Finished, Status::Reading(..)) => Ordering::Less,
        (Status::Reading(..), Status::Finished) => Ordering::Greater,
        (Status::Reading(p1), Status::Reading(p2)) => {
            p1.partial_cmp(&p2).unwrap_or(Ordering::Equal)
        }
    }
}

pub fn sort_size(i1: &Info, i2: &Info) -> Ordering {
    i1.file.size.cmp(&i2.file.size)
}

pub fn sort_kind(i1: &Info, i2: &Info) -> Ordering {
    i1.file.kind.cmp(&i2.file.kind)
}

pub fn sort_year(i1: &Info, i2: &Info) -> Ordering {
    i1.year.cmp(&i2.year)
}

pub fn sort_series(i1: &Info, i2: &Info) -> Ordering {
    i1.series.cmp(&i2.series).then_with(|| {
        usize::from_str_radix(&i1.number, 10)
            .ok()
            .zip(usize::from_str_radix(&i2.number, 10).ok())
            .map_or_else(|| i1.number.cmp(&i2.number), |(a, b)| a.cmp(&b))
    })
}

pub fn sort_filename(i1: &Info, i2: &Info) -> Ordering {
    let name1 = i1
        .file
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let name2 = i2
        .file
        .path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    natural_cmp(name1, name2)
}

pub fn sort_filepath(i1: &Info, i2: &Info) -> Ordering {
    let path1 = i1.file.path.to_str().unwrap_or("");
    let path2 = i2.file.path.to_str().unwrap_or("");
    natural_cmp(path1, path2)
}

fn natural_cmp(a: &str, b: &str) -> Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();

    loop {
        match (a_chars.peek(), b_chars.peek()) {
            (Some(&a_digit), Some(&b_digit))
                if a_digit.is_ascii_digit() && b_digit.is_ascii_digit() =>
            {
                let mut a_num = String::new();
                let mut b_num = String::new();

                while let Some(&c) = a_chars.peek() {
                    if c.is_ascii_digit() {
                        a_num.push(c);
                        a_chars.next();
                    } else {
                        break;
                    }
                }

                while let Some(&c) = b_chars.peek() {
                    if c.is_ascii_digit() {
                        b_num.push(c);
                        b_chars.next();
                    } else {
                        break;
                    }
                }

                let a_int: u64 = a_num.parse().unwrap_or(0);
                let b_int: u64 = b_num.parse().unwrap_or(0);

                match a_int.cmp(&b_int) {
                    Ordering::Equal => continue,
                    other => return other,
                }
            }
            (Some(a), Some(b)) => {
                let a_lower: String = a.to_lowercase().to_string();
                let b_lower: String = b.to_lowercase().to_string();
                let cmp = a_lower.as_str().cmp(b_lower.as_str());
                if cmp != Ordering::Equal {
                    return cmp;
                }
                a_chars.next();
                b_chars.next();
            }
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
        }
    }
}
