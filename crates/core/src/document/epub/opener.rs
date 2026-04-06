use crate::document::html::dom::XmlTree;
use crate::document::html::engine::{Engine, Page, ResourceFetcher};
use crate::document::html::xml::XmlParser;
use crate::helpers::decode_entities;
use crate::log_error;
use anyhow::{format_err, Error};
use percent_encoding::percent_decode_str;
use rustc_hash::FxHashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

#[derive(Debug)]
pub struct Chunk {
    pub path: String,
    pub size: usize,
}

pub struct EpubDocument {
    pub archive: ZipArchive<File>,
    pub(super) info: XmlTree,
    pub(super) parent: PathBuf,
    pub engine: Engine,
    pub spine: Vec<Chunk>,
    pub cache: FxHashMap<usize, Vec<Page>>,
    pub ignore_document_css: bool,
}

unsafe impl Send for EpubDocument {}
unsafe impl Sync for EpubDocument {}

impl ResourceFetcher for ZipArchive<File> {
    fn fetch(&mut self, name: &str) -> Result<Vec<u8>, Error> {
        let mut file = self.by_name(name)?;
        let size = file.size() as usize;
        let mut buf = Vec::with_capacity(size);
        file.read_to_end(&mut buf)?;
        Ok(buf)
    }
}

impl EpubDocument {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<EpubDocument, Error> {
        let file = File::open(path)?;
        let mut archive = ZipArchive::new(file)?;

        let opf_path = {
            let mut zf = archive.by_name("META-INF/container.xml")?;
            let size = zf.size() as usize;
            let mut text = String::with_capacity(size);
            zf.read_to_string(&mut text)?;
            let root = XmlParser::new(&text).parse();
            root.root()
                .find("rootfile")
                .and_then(|e| e.attribute("full-path"))
                .map(String::from)
        }
        .ok_or_else(|| format_err!("can't get the OPF path"))?;

        let parent = Path::new(&opf_path)
            .parent()
            .unwrap_or_else(|| Path::new(""));

        let text = {
            let mut zf = archive.by_name(&opf_path)?;
            let size = zf.size() as usize;
            let mut text = String::with_capacity(size);
            zf.read_to_string(&mut text)?;
            text
        };

        let info = XmlParser::new(&text).parse();
        let mut spine = Vec::new();

        {
            let manifest = info
                .root()
                .find("manifest")
                .ok_or_else(|| format_err!("the manifest is missing"))?;

            let spn = info
                .root()
                .find("spine")
                .ok_or_else(|| format_err!("the spine is missing"))?;

            for child in spn.children() {
                let vertebra_opt = child
                    .attribute("idref")
                    .and_then(|idref| manifest.find_by_id(idref))
                    .and_then(|entry| entry.attribute("href"))
                    .and_then(|href| {
                        let href = decode_entities(href);
                        let href = percent_decode_str(&href).decode_utf8_lossy();
                        let href_path = parent.join::<&str>(href.as_ref());
                        href_path.to_str().and_then(|path| {
                            archive
                                .by_name(path)
                                .map_err(|e| {
                                    log_error!(
                                        "Can't retrieve '{}' from the archive: {:#}.",
                                        path,
                                        e
                                    )
                                })
                                .map(|zf| (zf.size() as usize, path.to_string()))
                                .ok()
                        })
                    });

                if let Some((size, path)) = vertebra_opt {
                    spine.push(Chunk { path, size });
                }
            }
        }

        if spine.is_empty() {
            return Err(format_err!("the spine is empty"));
        }

        Ok(EpubDocument {
            archive,
            info,
            parent: parent.to_path_buf(),
            engine: Engine::new(),
            spine,
            cache: FxHashMap::default(),
            ignore_document_css: false,
        })
    }

    pub fn offset(&self, index: usize) -> usize {
        self.spine.iter().take(index).map(|c| c.size).sum()
    }

    pub fn size(&self) -> usize {
        self.offset(self.spine.len())
    }

    fn vertebra_coordinates_with<F>(&self, test: F) -> Option<(usize, usize)>
    where
        F: Fn(usize, usize) -> bool,
    {
        let mut start_offset = 0;
        let mut end_offset = start_offset;
        let mut index = 0;

        while index < self.spine.len() {
            end_offset += self.spine[index].size;
            if test(index, end_offset) {
                return Some((index, start_offset));
            }
            start_offset = end_offset;
            index += 1;
        }

        None
    }

    pub fn vertebra_coordinates(&self, offset: usize) -> Option<(usize, usize)> {
        self.vertebra_coordinates_with(|_, end_offset| offset < end_offset)
    }

    pub fn vertebra_coordinates_from_name(&self, name: &str) -> Option<(usize, usize)> {
        self.vertebra_coordinates_with(|index, _| self.spine[index].path == name)
    }
}
