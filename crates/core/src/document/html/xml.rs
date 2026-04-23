use super::dom::{element, text, whitespace};
use super::dom::{Attributes, NodeId, XmlTree};
use rustc_hash::FxHashMap;

#[derive(Debug)]
pub struct XmlParser<'a> {
    pub input: &'a str,
    pub offset: usize,
}

impl<'a> XmlParser<'a> {
    pub fn new(input: &str) -> XmlParser<'_> {
        if input.is_empty() {
            return XmlParser { input: "", offset: 0 };
        }
        XmlParser { input, offset: 0 }
    }

    fn eof(&self) -> bool {
        self.offset >= self.input.len()
    }

    fn next(&self) -> Option<char> {
        self.input[self.offset..].chars().next()
    }

    fn starts_with(&self, s: &str) -> bool {
        self.input[self.offset..].starts_with(s)
    }

    fn advance(&mut self, n: usize) {
        for c in self.input[self.offset..].chars().take(n) {
            self.offset += c.len_utf8();
        }
    }

    fn advance_while<F>(&mut self, test: F)
    where
        F: FnMut(&char) -> bool,
    {
        for c in self.input[self.offset..].chars().take_while(test) {
            self.offset += c.len_utf8();
        }
    }

    fn advance_until(&mut self, target: &str) {
        while !self.eof() && !self.starts_with(target) {
            self.advance(1);
        }
        self.advance(target.chars().count());
    }

    fn parse_attributes(&mut self) -> Attributes {
        let mut attrs = FxHashMap::default();
        while !self.eof() {
            self.advance_while(|&c| c.is_xml_whitespace());
            match self.next() {
                Some('>' | '/') | None => break,
                _ => {
                    let offset = self.offset;
                    self.advance_while(|&c| c != '=');
                    let key = self.input[offset..self.offset].to_string();
                    self.advance_while(|&c| c != '"' && c != '\'');
                    let quote = self.next().unwrap_or('"');
                    self.advance(1);
                    let offset = self.offset;
                    self.advance_while(|&c| c != quote);
                    let value = self.input[offset..self.offset].to_string();
                    attrs.insert(key, value);
                    self.advance(1);
                }
            }
        }
        attrs
    }

    fn parse_element(&mut self, tree: &mut XmlTree, parent_id: NodeId) {
        let offset = self.offset;
        self.advance_while(|&c| c != '>' && c != '/' && !c.is_xml_whitespace());
        let name = &self.input[offset..self.offset];
        let attributes = self.parse_attributes();

        match self.next() {
            Some('/') => {
                self.advance(2);
                tree.get_mut(parent_id)
                    .append(element(name, offset - 1, attributes));
            }
            Some('>') => {
                self.advance(1);
                let id = tree
                    .get_mut(parent_id)
                    .append(element(name, offset - 1, attributes));
                self.parse_nodes(tree, id);
            }
            _ => (),
        }
    }

    fn parse_nodes(&mut self, tree: &mut XmlTree, parent_id: NodeId) {
        while !self.eof() {
            let offset = self.offset;
            self.advance_while(|&c| c.is_xml_whitespace());

            match self.next() {
                Some('<') => {
                    if self.offset > offset {
                        tree.get_mut(parent_id)
                            .append(whitespace(&self.input[offset..self.offset], offset));
                    }
                    if self.starts_with("</") {
                        self.advance(2);
                        self.advance_while(|&c| c != '>');
                        self.advance(1);
                        break;
                    }
                    self.advance(1);
                    match self.next() {
                        Some('?') => {
                            self.advance(1);
                            self.advance_until("?>");
                        }
                        Some('!') => {
                            self.advance(1);
                            match self.next() {
                                Some('-') => {
                                    self.advance(2);
                                    self.advance_until("-->");
                                }
                                Some('[') => {
                                    self.advance(1);
                                    self.advance_until("]]>");
                                }
                                _ => {
                                    self.advance_while(|&c| c != '>');
                                    self.advance(1);
                                }
                            }
                        }
                        _ => self.parse_element(tree, parent_id),
                    }
                }
                Some(..) => {
                    self.advance_while(|&c| c != '<');
                    tree.get_mut(parent_id)
                        .append(text(&self.input[offset..self.offset], offset));
                }
                None => break,
            }
        }
    }

    pub fn parse(&mut self) -> XmlTree {
        let mut tree = XmlTree::new();
        self.parse_nodes(&mut tree, NodeId::from_index(0));
        tree
    }
}

pub trait XmlExt {
    fn is_xml_whitespace(&self) -> bool;
}

impl XmlExt for char {
    fn is_xml_whitespace(&self) -> bool {
        matches!(self, ' ' | '\t' | '\n' | '\r')
    }
}

/// Enhanced XML utilities for general XML parsing beyond XFDF
/// Provides functions to extract data from XML documents for various formats
///
/// This module extends the basic XML parser with enhanced capabilities:
/// - Extract text content by tag name (extract_text_by_tag)
/// - Extract attributes by tag (extract_attribute_by_tag)
/// - Extract elements with attributes (extract_elements_with_attrs)
/// - XML validation (validate_xml)
/// - Pretty-printing (pretty_print_xml)
///
/// ## Dependencies
///
/// - `quick_xml` - For enhanced XML parsing and manipulation
use anyhow::{Context, Error};
use quick_xml::events::Event;
use quick_xml::Reader;

/// Extract all text content from an XML element by tag name
pub fn extract_text_by_tag(xml: &str, tag_name: &str) -> Result<Vec<String>, Error> {
    let mut reader = Reader::from_str(xml);

    let mut results = Vec::new();
    let mut in_target = false;

    let mut buf = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buf)
            .context("Failed to read XML event")?
        {
            Event::Start(ref e) if e.name().as_ref() == tag_name.as_bytes() => {
                in_target = true;
            }
            Event::Text(e) if in_target => {
                results.push(String::from_utf8_lossy(e.as_ref()).to_string());
            }
            Event::End(ref e) if e.name().as_ref() == tag_name.as_bytes() => {
                in_target = false;
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(results)
}

/// Extract attribute value from first matching XML element
pub fn extract_attribute_by_tag(
    xml: &str,
    tag_name: &str,
    attr_name: &str,
) -> Result<Option<String>, Error> {
    let mut reader = Reader::from_str(xml);

    let mut buf = Vec::new();
    loop {
        match reader
            .read_event_into(&mut buf)
            .context("Failed to read XML event")?
        {
            Event::Start(ref e) if e.name().as_ref() == tag_name.as_bytes() => {
                if let Some(attr) = e
                    .attributes()
                    .with_checks(false)
                    .filter_map(|a| a.ok())
                    .find(|a| a.key.as_ref() == attr_name.as_bytes())
                {
                    let value = attr
                        .unescape_value()
                        .context("Failed to unescape attribute value")?
                        .to_string();
                    return Ok(Some(value));
                }
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(None)
}

/// Extract all elements matching a tag name with their attributes
#[allow(clippy::type_complexity)]
pub fn extract_elements_with_attrs(
    xml: &str,
    tag_name: &str,
) -> Result<Vec<(String, FxHashMap<String, String>)>, Error> {
    let mut reader = Reader::from_str(xml);

    let mut results = Vec::new();
    let mut buf = Vec::new();

    loop {
        match reader
            .read_event_into(&mut buf)
            .context("Failed to read XML event")?
        {
            Event::Start(ref e) if e.name().as_ref() == tag_name.as_bytes() => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let mut attrs = FxHashMap::default();

                for attr in e.attributes().with_checks(false).filter_map(|a| a.ok()) {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = attr
                        .unescape_value()
                        .context("Failed to unescape attribute value")?
                        .to_string();
                    attrs.insert(key, value);
                }

                results.push((name, attrs));
            }
            Event::Eof => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(results)
}

/// Validate XML structure and return error if malformed
pub fn validate_xml(xml: &str) -> Result<(), Error> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    loop {
        if reader
            .read_event_into(&mut buf)
            .context("XML is malformed")?
            == Event::Eof
        {
            break;
        }
        buf.clear();
    }

    Ok(())
}

/// Pretty-print XML with proper indentation
pub fn pretty_print_xml(xml: &str, indent: usize) -> Result<String, Error> {
    let mut reader = Reader::from_str(xml);

    let mut writer = quick_xml::Writer::new_with_indent(Vec::new(), b' ', indent);
    let mut buf = Vec::new();

    loop {
        match reader
            .read_event_into(&mut buf)
            .context("Failed to read XML event")?
        {
            Event::Eof => break,
            e => writer.write_event(e).context("Failed to write XML event")?,
        }
        buf.clear();
    }

    let result = writer.into_inner();
    String::from_utf8(result).context("Generated XML is not valid UTF-8")
}
