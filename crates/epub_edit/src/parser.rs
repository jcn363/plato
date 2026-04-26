//! EPUB parsing functions.

use anyhow::{format_err, Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::sync::LazyLock;
use zip::ZipArchive;

use crate::types::EpubChapter;

static TITLE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<dc:title[^>]*>([^<]+)</dc:title>").expect("invalid regex"));
static AUTHOR_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<dc:creator[^>]*>([^<]+)</dc:creator>").expect("invalid regex"));
static LANGUAGE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<dc:language[^>]*>([^<]+)</dc:language>").expect("invalid regex")
});
static IDENTIFIER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<dc:identifier[^>]*>([^<]+)</dc:identifier>").expect("invalid regex")
});
static PUBLISHER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<dc:publisher[^>]*>([^<]+)</dc:publisher>").expect("invalid regex")
});
static DATE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"<dc:date[^>]*>([^<]+)</dc:date>").expect("invalid regex"));
static DESCRIPTION_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"<dc:description[^>]*>([^<]+)</dc:description>").expect("invalid regex")
});
static ROOTFILE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"rootfile[^"]*"?([^"]+)"?"#).expect("invalid regex"));
static ITEM_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"<item[^>]+href="([^"]+)"[^>]+id="([^"]+)"[^>]*>"#).expect("invalid regex")
});
static SPINE_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r#"<itemref[^>]+idref="([^"]+)"[^>]*>"#).expect("invalid regex"));

/// Extracts the EPUB archive to the temporary directory.
///
/// # Errors
///
/// Returns an error if:
/// * Opening the EPUB file fails
/// * Reading the ZIP archive fails
/// * Creating directories for extracted files fails
/// * Writing extracted files fails
pub fn extract_epub(epub_path: &Path, temp_dir: &Path) -> Result<()> {
    let file = File::open(epub_path).context("Failed to open EPUB file")?;
    let mut archive = ZipArchive::new(file).context("Failed to read ZIP archive")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;

        // Prevent zip slip: reject entries containing path traversal components
        let name = file.name();
        if name.contains("..") || name.starts_with('/') || name.starts_with('\\') {
            return Err(format_err!("Zip entry contains path traversal: {name}"));
        }

        let outpath = temp_dir.join(name);

        if name.ends_with('/') {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

/// Parses metadata from the EPUB's OPF file.
///
/// # Errors
///
/// Returns an error if:
/// * The container.xml file is missing
/// * Reading the container.xml file fails
/// * The rootfile path cannot be found in container.xml
/// * The OPF file is missing at the specified path
/// * Reading the OPF file fails
pub fn parse_metadata(temp_dir: &Path) -> Result<(String, String)> {
    let container_path = temp_dir.join("META-INF/container.xml");
    if !container_path.exists() {
        return Err(format_err!("META-INF/container.xml not found"));
    }

    let container_content = fs::read_to_string(&container_path)?;

    if let Some(caps) = ROOTFILE_RE.captures(&container_content) {
        let opf_path = caps.get(1).map_or("OEBPS/content.opf", |m| m.as_str());
        let opf_full_path = temp_dir.join(opf_path);

        if opf_full_path.exists() {
            Ok((opf_path.to_string(), fs::read_to_string(&opf_full_path)?))
        } else {
            Err(format_err!("OPF file not found at {opf_path}"))
        }
    } else {
        Err(format_err!("Could not find rootfile in container.xml"))
    }
}

/// Parses OPF metadata fields from the OPF content.
pub fn parse_opf_metadata(opf_content: &str) -> crate::types::EpubMetadata {
    let mut metadata = crate::types::EpubMetadata::default();

    if let Some(caps) = TITLE_RE.captures(opf_content) {
        metadata.title = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
    }
    if let Some(caps) = AUTHOR_RE.captures(opf_content) {
        metadata.author = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
    }
    if let Some(caps) = LANGUAGE_RE.captures(opf_content) {
        metadata.language = caps
            .get(1)
            .map_or_else(|| "en".to_string(), |m| m.as_str().to_string());
    }
    if let Some(caps) = IDENTIFIER_RE.captures(opf_content) {
        metadata.identifier = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
    }
    if let Some(caps) = PUBLISHER_RE.captures(opf_content) {
        metadata.publisher = caps.get(1).map(|m| m.as_str().to_string());
    }
    if let Some(caps) = DATE_RE.captures(opf_content) {
        metadata.date = caps.get(1).map(|m| m.as_str().to_string());
    }
    if let Some(caps) = DESCRIPTION_RE.captures(opf_content) {
        metadata.description = caps.get(1).map(|m| m.as_str().to_string());
    }

    metadata
}

/// Parses the content structure from the EPUB's OPF file.
///
/// # Errors
///
/// Returns an error if:
/// * The container.xml file is missing
/// * Reading the container.xml file fails
/// * The rootfile path cannot be found in container.xml
/// * The OPF file is missing at the specified path
/// * Reading the OPF file fails
pub fn parse_content(temp_dir: &Path) -> Result<(String, String)> {
    let container_path = temp_dir.join("META-INF/container.xml");
    let container_content = fs::read_to_string(&container_path)?;

    if let Some(caps) = ROOTFILE_RE.captures(&container_content) {
        let opf_path = caps.get(1).map_or("OEBPS/content.opf", |m| m.as_str());
        let opf_full_path = temp_dir.join(opf_path);

        if opf_full_path.exists() {
            Ok((opf_path.to_string(), fs::read_to_string(&opf_full_path)?))
        } else {
            Err(format_err!("OPF file not found at {opf_path}"))
        }
    } else {
        Err(format_err!("Could not find rootfile in container.xml"))
    }
}

/// Parses OPF content to extract chapter list.
pub fn parse_opf_content(opf_content: &str, opf_dir: &str, temp_dir: &Path) -> Vec<EpubChapter> {
    let mut item_map: HashMap<String, String> = HashMap::new();
    for caps in ITEM_RE.captures_iter(opf_content) {
        let href = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        let id = caps
            .get(2)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        item_map.insert(id, href);
    }

    let mut order: Vec<String> = Vec::new();
    for caps in SPINE_RE.captures_iter(opf_content) {
        let idref = caps
            .get(1)
            .map(|m| m.as_str().to_string())
            .unwrap_or_default();
        order.push(idref);
    }

    let opf_parent = Path::new(opf_dir).parent().unwrap_or(Path::new(""));
    let mut chapters = Vec::new();

    for idref in order {
        if let Some(href) = item_map.get(&idref) {
            let full_path = temp_dir.join(opf_parent).join(href);
            if full_path.exists() {
                if let Ok(content) = fs::read_to_string(&full_path) {
                    chapters.push(EpubChapter {
                        id: idref.clone(),
                        href: href.clone(),
                        title: extract_title(&content).unwrap_or_else(|| href.clone()),
                        content,
                    });
                }
            }
        }
    }

    chapters
}

/// Extracts title from HTML content.
#[must_use]
pub fn extract_title(html: &str) -> Option<String> {
    let title_regex = Regex::new(r"(?i)<title[^>]*>([^<]+)</title>").ok()?;
    let h1_regex = Regex::new(r"(?i)<h1[^>]*>([^<]+)</h1>").ok()?;

    if let Some(caps) = title_regex.captures(html) {
        return Some(
            caps.get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default(),
        );
    }
    if let Some(caps) = h1_regex.captures(html) {
        return Some(
            caps.get(1)
                .map(|m| m.as_str().to_string())
                .unwrap_or_default(),
        );
    }
    None
}
