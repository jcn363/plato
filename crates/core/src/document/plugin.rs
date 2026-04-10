//! Document Plugin System
//!
//! Provides extensibility for custom document types via a plugin architecture.

use crate::framebuffer::Pixmap;
use crate::geom::Boundary;
use crate::geom::CycleDir;
use anyhow::{format_err, Error};
use std::path::Path;

/// Metadata about a document plugin
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub supported_extensions: Vec<String>,
    pub description: String,
}

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, Error>;

/// Document plugin trait for custom document type support
pub trait DocumentPlugin: Send + Sync {
    /// Returns metadata about this plugin
    fn metadata(&self) -> &PluginMetadata;

    /// Check if this plugin can handle the given file
    fn can_open(&self, path: &Path) -> bool;

    /// Open and return a document instance
    fn open(&self, path: &Path) -> PluginResult<Box<dyn PluginDocument>>;

    /// Optional: Check if file is encrypted
    fn is_encrypted(&self, _path: &Path) -> bool {
        false
    }
}

/// Plugin document trait - subset of Document trait for plugins
pub trait PluginDocument: Send + Sync {
    fn dims(&self, index: usize) -> Option<(f32, f32)>;
    fn pages_count(&self) -> usize;

    fn toc(&mut self) -> Option<Vec<crate::document::TocEntry>>;
    fn chapter<'a>(
        &mut self,
        offset: usize,
        toc: &'a [crate::document::TocEntry],
    ) -> Option<(&'a crate::document::TocEntry, f32)>;
    fn words(
        &mut self,
        loc: crate::document::Location,
    ) -> Option<(Vec<crate::document::BoundedText>, usize)>;
    fn lines(
        &mut self,
        loc: crate::document::Location,
    ) -> Option<(Vec<crate::document::BoundedText>, usize)>;
    fn links(
        &mut self,
        loc: crate::document::Location,
    ) -> Option<(Vec<crate::document::BoundedText>, usize)>;
    fn images(&mut self, loc: crate::document::Location) -> Option<(Vec<Boundary>, usize)>;

    fn pixmap(
        &mut self,
        loc: crate::document::Location,
        scale: f32,
        samples: usize,
    ) -> Option<(Pixmap, usize)>;
    fn layout(&mut self, width: u32, height: u32, font_size: f32, dpi: u16);

    fn title(&self) -> Option<String>;
    fn author(&self) -> Option<String>;
    fn metadata(&self, key: &str) -> Option<String>;

    fn is_reflowable(&self) -> bool;
}

/// Plugin registry for managing document plugins
pub struct PluginRegistry {
    plugins: Vec<Box<dyn DocumentPlugin>>,
}

impl PluginRegistry {
    pub fn new() -> PluginRegistry {
        PluginRegistry {
            plugins: Vec::new(),
        }
    }

    /// Register a new plugin
    pub fn register(&mut self, plugin: Box<dyn DocumentPlugin>) {
        self.plugins.push(plugin);
    }

    /// Find a plugin that can handle the given file
    pub fn find_plugin(&self, path: &Path) -> Option<&dyn DocumentPlugin> {
        for plugin in &self.plugins {
            if plugin.can_open(path) {
                return Some(plugin.as_ref());
            }
        }
        None
    }

    /// Open a document using the appropriate plugin
    pub fn open(&self, path: &Path) -> Option<Box<dyn crate::document::Document>> {
        let plugin = self.find_plugin(path)?;
        let doc = plugin.open(path).ok()?;
        Some(Box::new(PluginDocumentAdapter::new(doc)) as Box<dyn crate::document::Document>)
    }

    /// Get all registered plugins
    pub fn plugins(&self) -> &[Box<dyn DocumentPlugin>] {
        &self.plugins
    }

    /// Get plugin by name
    pub fn get(&self, name: &str) -> Option<&dyn DocumentPlugin> {
        for plugin in &self.plugins {
            if plugin.metadata().name == name {
                return Some(plugin.as_ref());
            }
        }
        None
    }

    /// Check if any plugin supports the given file extension
    pub fn supports_extension(&self, extension: &str) -> bool {
        let ext_lower = extension.to_lowercase();
        for plugin in &self.plugins {
            for supported in &plugin.metadata().supported_extensions {
                if supported.to_lowercase() == ext_lower {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Adapter to convert PluginDocument to Document trait
pub struct PluginDocumentAdapter {
    inner: Box<dyn PluginDocument>,
}

impl PluginDocumentAdapter {
    pub fn new(inner: Box<dyn PluginDocument>) -> PluginDocumentAdapter {
        PluginDocumentAdapter { inner }
    }
}

impl crate::document::Document for PluginDocumentAdapter {
    fn dims(&self, index: usize) -> Option<(f32, f32)> {
        self.inner.dims(index)
    }

    fn pages_count(&self) -> usize {
        self.inner.pages_count()
    }

    fn toc(&mut self) -> Option<Vec<crate::document::TocEntry>> {
        self.inner.toc()
    }

    fn chapter<'a>(
        &mut self,
        offset: usize,
        toc: &'a [crate::document::TocEntry],
    ) -> Option<(&'a crate::document::TocEntry, f32)> {
        self.inner.chapter(offset, toc)
    }

    fn chapter_relative<'a>(
        &mut self,
        offset: usize,
        dir: CycleDir,
        toc: &'a [crate::document::TocEntry],
    ) -> Option<&'a crate::document::TocEntry> {
        crate::document::chapter_relative(offset, dir, toc)
    }

    fn words(
        &mut self,
        loc: crate::document::Location,
    ) -> Option<(Vec<crate::document::BoundedText>, usize)> {
        self.inner.words(loc)
    }

    fn lines(
        &mut self,
        loc: crate::document::Location,
    ) -> Option<(Vec<crate::document::BoundedText>, usize)> {
        self.inner.lines(loc)
    }

    fn links(
        &mut self,
        loc: crate::document::Location,
    ) -> Option<(Vec<crate::document::BoundedText>, usize)> {
        self.inner.links(loc)
    }

    fn images(&mut self, loc: crate::document::Location) -> Option<(Vec<Boundary>, usize)> {
        self.inner.images(loc)
    }

    fn pixmap(
        &mut self,
        loc: crate::document::Location,
        scale: f32,
        samples: usize,
    ) -> Option<(Pixmap, usize)> {
        self.inner.pixmap(loc, scale, samples)
    }

    fn layout(&mut self, width: u32, height: u32, font_size: f32, dpi: u16) {
        self.inner.layout(width, height, font_size, dpi)
    }

    fn set_ignore_document_css(&mut self, _ignore: bool) {}

    fn title(&self) -> Option<String> {
        self.inner.title()
    }

    fn author(&self) -> Option<String> {
        self.inner.author()
    }

    fn metadata(&self, key: &str) -> Option<String> {
        self.inner.metadata(key)
    }

    fn is_reflowable(&self) -> bool {
        self.inner.is_reflowable()
    }
}

/// Plugin loader function type - plugins must export this function
/// to create and return their plugin instance.
#[allow(improper_ctypes_definitions)]
pub type PluginLoaderFn = extern "C" fn() -> *mut dyn DocumentPlugin;

/// Loads a plugin from a dynamic library.
///
/// # Safety
/// The library must contain a valid `plato_plugin_entry` function that
/// returns a pointer to a boxed DocumentPlugin.
pub unsafe fn load_plugin_from_library(
    library: &libloading::Library,
) -> Result<Box<dyn DocumentPlugin>, Error> {
    #[allow(improper_ctypes_definitions)]
    type EntryFn = extern "C" fn() -> *mut dyn DocumentPlugin;

    let entry_fn: libloading::Symbol<EntryFn> = library
        .get(b"plato_plugin_entry")
        .map_err(|e| format_err!("Plugin entry point not found: {}", e))?;

    let plugin_ptr = entry_fn();
    if plugin_ptr.is_null() {
        return Err(format_err!("Plugin returned null pointer"));
    }

    Ok(Box::from_raw(plugin_ptr))
}

/// Helper macro to export a plugin entry point
/// Usage in plugin library:
/// `plato_plugin_export!(MyPluginStruct);`
#[macro_export]
macro_rules! plato_plugin_export {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn plato_plugin_entry() -> *mut dyn super::DocumentPlugin {
            let plugin = <$plugin_type>::new() as Box<dyn super::DocumentPlugin>;
            Box::into_raw(plugin)
        }
    };
}
