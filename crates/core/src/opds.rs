use anyhow::Error;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::fmt;

pub struct OPDSCatalog {
    _id: String,
    title: String,
    _url: String,
    entries: Vec<OPDSEntry>,
}

impl OPDSCatalog {
    pub fn new(url: &str) -> Result<Self, Error> {
        let client = Client::new();
        #[allow(unused_mut)]
        let mut response = client
            .get(url)
            .send()
            .map_err(|e| anyhow::anyhow!("Failed to fetch OPDS: {}", e))?;

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let body = response
            .text()
            .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))?;

        if content_type.contains("atom") || content_type.contains("xml") || body.contains("<feed") {
            Self::parse_atom(&body, url)
        } else if content_type.contains("navigation") || body.contains("index") {
            Self::parse_nav(&body, url)
        } else {
            Err(anyhow::anyhow!("Unknown OPDS format"))
        }
    }

    fn parse_atom(body: &str, base_url: &str) -> Result<Self, Error> {
        let mut title = String::new();
        let mut entries = Vec::new();

        let parser = quick_xml::Reader::from_str(body);
        let mut parser = parser;
        parser.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut in_title = false;
        let mut in_entry = false;
        let mut current_entry: Option<OPDSEntry> = None;
        let mut in_content = false;

        loop {
            match parser.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "title" && !in_entry {
                        in_title = true;
                    } else if name == "entry" {
                        in_entry = true;
                        current_entry = Some(OPDSEntry::default());
                    } else if name == "content" && in_entry {
                        in_content = true;
                    }
                }
                Ok(quick_xml::events::Event::Text(e)) => {
                    let text = e.unescape()?;
                    if in_title {
                        title.push_str(&text);
                    } else if in_entry {
                        if let Some(ref mut entry) = current_entry {
                            if in_content {
                                entry.summary = text.to_string();
                            }
                        }
                    }
                }
                Ok(quick_xml::events::Event::End(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "title" && !in_entry {
                        in_title = false;
                    } else if name == "entry" {
                        if let Some(entry) = current_entry.take() {
                            entries.push(entry);
                        }
                        in_entry = false;
                    } else if name == "content" {
                        in_content = false;
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }

        let id = base_url.to_string();
        Ok(OPDSCatalog {
            _id: id,
            title,
            _url: base_url.to_string(),
            entries,
        })
    }

    fn parse_nav(body: &str, _base_url: &str) -> Result<Self, Error> {
        let parser = quick_xml::Reader::from_str(body);
        let mut parser = parser;
        parser.config_mut().trim_text(true);

        let mut buf = Vec::new();
        let mut entries = Vec::new();
        let mut title = String::new();
        let mut in_link = false;
        let mut href = String::new();

        loop {
            match parser.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "nav" {
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"href" {
                                href = String::from_utf8_lossy(&attr.value).to_string();
                            }
                        }
                    }
                }
                Ok(quick_xml::events::Event::Text(e)) => {
                    let text = e.unescape()?;
                    if !text.is_empty() && in_link {
                        entries.push(OPDSEntry {
                            id: href.clone(),
                            title: text.to_string(),
                            links: HashMap::new(),
                            summary: String::new(),
                        });
                    } else if !text.is_empty() {
                        title.push_str(&text);
                    }
                }
                Ok(quick_xml::events::Event::End(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    if name == "nav" {
                        in_link = false;
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }

        Ok(OPDSCatalog {
            _id: String::new(),
            title,
            _url: String::new(),
            entries,
        })
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn entries(&self) -> &[OPDSEntry] {
        &self.entries
    }

    pub fn search(&self, query: &str) -> Vec<&OPDSEntry> {
        let query_lower = query.to_lowercase();
        self.entries
            .iter()
            .filter(|e| e.title.to_lowercase().contains(&query_lower))
            .collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct OPDSEntry {
    pub id: String,
    pub title: String,
    pub links: HashMap<String, String>,
    pub summary: String,
}

impl OPDSEntry {
    pub fn download_url(&self) -> Option<&String> {
        self.links
            .get("http://opds-spec.org/acquisition/open-access")
            .or(self.links.get("http://opds-spec.org/acquisition/borrow"))
            .or(self.links.get("http://opds-spec.org/image"))
    }
}

impl fmt::Display for OPDSCatalog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "# {}\n", self.title)?;
        for entry in &self.entries {
            writeln!(f, "- {}", entry.title)?;
        }
        Ok(())
    }
}
