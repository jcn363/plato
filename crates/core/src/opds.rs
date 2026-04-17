use anyhow::Error;
use reqwest::blocking::Client;
use rustc_hash::FxHashMap;
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
        let mut entries = Vec::with_capacity(32);
        let mut state = AtomParserState::new();

        let parser = Self::create_parser(body);
        Self::parse_atom_loop(parser, &mut title, &mut entries, &mut state);

        Ok(OPDSCatalog {
            _id: base_url.to_string(),
            title,
            _url: base_url.to_string(),
            entries,
        })
    }

    fn create_parser(body: &str) -> quick_xml::Reader<&[u8]> {
        let parser = quick_xml::Reader::from_str(body);
        let mut parser = parser;
        parser.config_mut().trim_text(true);
        parser
    }

    fn parse_atom_loop(
        parser: quick_xml::Reader<&[u8]>,
        title: &mut String,
        entries: &mut Vec<OPDSEntry>,
        state: &mut AtomParserState,
    ) {
        let mut parser = parser;
        let mut buf = Vec::with_capacity(1024);

        loop {
            match parser.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(e)) => {
                    state.handle_start_event(quick_xml::events::Event::Start(e));
                }
                Ok(quick_xml::events::Event::Text(e)) => {
                    if let Ok(text) = e.decode() {
                        state.handle_text_event(title, &text);
                    }
                }
                Ok(quick_xml::events::Event::End(e)) => {
                    state.handle_end_event(entries, quick_xml::events::Event::End(e));
                }
                Ok(quick_xml::events::Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
    }

    fn parse_nav(body: &str, _base_url: &str) -> Result<Self, Error> {
        let parser = Self::create_parser(body);
        let mut state = NavParserState::new();

        Self::parse_nav_loop(parser, &mut state);

        Ok(OPDSCatalog {
            _id: String::new(),
            title: state.title,
            _url: String::new(),
            entries: state.entries,
        })
    }

    fn parse_nav_loop(parser: quick_xml::Reader<&[u8]>, state: &mut NavParserState) {
        let mut parser = parser;
        let mut buf = Vec::with_capacity(1024);

        loop {
            match parser.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(e)) => {
                    state.handle_start_event(quick_xml::events::Event::Start(e));
                }
                Ok(quick_xml::events::Event::Text(e)) => {
                    if let Ok(text) = e.decode() {
                        state.handle_text_event(&text);
                    }
                }
                Ok(quick_xml::events::Event::End(e)) => {
                    state.handle_end_event(quick_xml::events::Event::End(e));
                }
                Ok(quick_xml::events::Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }
    }
}

struct NavParserState {
    entries: Vec<OPDSEntry>,
    title: String,
    in_link: bool,
    href: String,
}

impl NavParserState {
    fn new() -> Self {
        NavParserState {
            entries: Vec::with_capacity(16),
            title: String::new(),
            in_link: false,
            href: String::new(),
        }
    }

    fn handle_start_event(&mut self, e: quick_xml::events::Event) {
        if let quick_xml::events::Event::Start(elem) = e {
            let name = String::from_utf8_lossy(elem.name().as_ref()).to_string();
            if name == "nav" {
                for attr in elem.attributes().flatten() {
                    if attr.key.as_ref() == b"href" {
                        self.href = String::from_utf8_lossy(&attr.value).to_string();
                    }
                }
            }
        }
    }

    fn handle_text_event(&mut self, text: &str) {
        if !text.is_empty() && self.in_link {
            self.entries.push(OPDSEntry {
                id: self.href.clone(),
                title: text.to_string(),
                links: FxHashMap::default(),
                summary: String::new(),
            });
        } else if !text.is_empty() {
            self.title.push_str(text);
        }
    }

    fn handle_end_event(&mut self, e: quick_xml::events::Event) {
        if let quick_xml::events::Event::End(elem) = e {
            let name = String::from_utf8_lossy(elem.name().as_ref()).to_string();
            if name == "nav" {
                self.in_link = false;
            }
        }
    }
}

struct AtomParserState {
    in_title: bool,
    in_entry: bool,
    current_entry: Option<OPDSEntry>,
    in_content: bool,
}

impl AtomParserState {
    fn new() -> Self {
        AtomParserState {
            in_title: false,
            in_entry: false,
            current_entry: None,
            in_content: false,
        }
    }

    fn handle_start_event(&mut self, e: quick_xml::events::Event) {
        if let quick_xml::events::Event::Start(elem) = e {
            let name = String::from_utf8_lossy(elem.name().as_ref()).to_string();
            if name == "title" && !self.in_entry {
                self.in_title = true;
            } else if name == "entry" {
                self.in_entry = true;
                self.current_entry = Some(OPDSEntry::default());
            } else if name == "content" && self.in_entry {
                self.in_content = true;
            }
        }
    }

    fn handle_text_event(&mut self, title: &mut String, text: &str) {
        if self.in_title {
            title.push_str(text);
        } else if self.in_entry {
            if let Some(ref mut entry) = self.current_entry {
                if self.in_content {
                    entry.summary = text.to_string();
                }
            }
        }
    }

    fn handle_end_event(&mut self, entries: &mut Vec<OPDSEntry>, e: quick_xml::events::Event) {
        if let quick_xml::events::Event::End(elem) = e {
            let name = String::from_utf8_lossy(elem.name().as_ref()).to_string();
            if name == "title" && !self.in_entry {
                self.in_title = false;
            } else if name == "entry" {
                if let Some(entry) = self.current_entry.take() {
                    entries.push(entry);
                }
                self.in_entry = false;
            } else if name == "content" {
                self.in_content = false;
            }
        }
    }
}

impl OPDSCatalog {
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
    pub links: FxHashMap<String, String>,
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
