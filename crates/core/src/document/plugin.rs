//! Document Plugin System
//!
//! Provides extensibility for custom document types via a plugin architecture.

#![allow(dead_code)]

use crate::framebuffer::Pixmap;
use crate::geom::Boundary;
use crate::geom::CycleDir;
use anyhow::{format_err, Error};
use once_cell::sync::Lazy;
use std::path::Path;
use std::sync::RwLock;

macro_rules! plugin_log_info {
    ($($arg:tt)*) => { eprintln!("[INFO] {}", format!($($arg)*)) };
}

macro_rules! plugin_log_warn {
    ($($arg:tt)*) => { eprintln!("[WARN] {}", format!($($arg)*)) };
}

macro_rules! plugin_log_error {
    ($($arg:tt)*) => { eprintln!("[ERROR] {}", format!($($arg)*)) };
}

static PLUGIN_REGISTRY: Lazy<RwLock<PluginRegistry>> = Lazy::new(|| RwLock::new(PluginRegistry::new()));

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

    fn set_ignore_document_css(&mut self, ignore: bool);

    fn set_font_family(&mut self, family_name: &str, search_path: &str);
    fn set_margin_width(&mut self, width: i32);
    fn set_text_align(&mut self, text_align: crate::metadata::TextAlign);
    fn set_line_height(&mut self, line_height: f32);
    fn set_hyphen_penalty(&mut self, hyphen_penalty: i32);
    fn set_stretch_tolerance(&mut self, stretch_tolerance: f32);
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

    /// Load plugins from a directory
    pub fn load_from_directory(&mut self, plugins_dir: &Path) -> Result<(), Error> {
        if !plugins_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(plugins_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                continue;
            };

            if !matches!(ext.to_lowercase().as_str(), "so" | "dll" | "dylib") {
                continue;
            }

            if let Ok(library) = unsafe { libloading::Library::new(&path) } {
                // SAFETY: We just opened the library ourselves, and we'll check
                // the plugin entry point exists before calling it.
                match unsafe { load_plugin_from_library(&library) } {
                    Ok(plugin) => {
                        plugin_log_info!(
                            "Loaded document plugin: {}",
                            plugin.metadata().name
                        );
                        self.plugins.push(plugin);
                    }
                    Err(e) => {
                        plugin_log_warn!("Failed to load plugin from {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get the count of loaded plugins
    pub fn len(&self) -> usize {
        self.plugins.len()
    }

    /// Check if no plugins are loaded
    pub fn is_empty(&self) -> bool {
        self.plugins.is_empty()
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Initialize the global plugin registry by loading plugins from a directory
pub fn init_plugin_registry(plugins_dir: Option<&Path>) {
    if let Some(dir) = plugins_dir {
        if let Ok(mut registry) = PLUGIN_REGISTRY.write() {
            if let Err(e) = registry.load_from_directory(dir) {
                plugin_log_error!("Failed to load document plugins: {}", e);
            }
        }
    }
}

/// Get the global plugin registry (read-only access)
pub fn get_plugin_registry() -> std::sync::RwLockReadGuard<'static, PluginRegistry> {
    PLUGIN_REGISTRY.read().unwrap()
}

/// Check if any loaded plugin can handle the given file
pub fn plugin_can_open(path: &Path) -> bool {
    if let Ok(registry) = PLUGIN_REGISTRY.read() {
        registry.find_plugin(path).is_some()
    } else {
        false
    }
}

/// Open a document using the global plugin registry
pub fn open_with_plugins(path: &Path) -> Option<Box<dyn crate::document::Document>> {
    if let Ok(registry) = PLUGIN_REGISTRY.read() {
        registry.open(path)
    } else {
        None
    }
}

/// Get the number of loaded plugins
pub fn plugin_count() -> usize {
    PLUGIN_REGISTRY.read().map(|r| r.len()).unwrap_or(0)
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

    fn set_ignore_document_css(&mut self, ignore: bool) {
        self.inner.set_ignore_document_css(ignore);
    }

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

    fn set_font_family(&mut self, family_name: &str, search_path: &str) {
        self.inner.set_font_family(family_name, search_path);
    }

    fn set_margin_width(&mut self, width: i32) {
        self.inner.set_margin_width(width);
    }

    fn set_text_align(&mut self, text_align: crate::metadata::TextAlign) {
        self.inner.set_text_align(text_align);
    }

    fn set_line_height(&mut self, line_height: f32) {
        self.inner.set_line_height(line_height);
    }

    fn set_hyphen_penalty(&mut self, hyphen_penalty: i32) {
        self.inner.set_hyphen_penalty(hyphen_penalty);
    }

    fn set_stretch_tolerance(&mut self, stretch_tolerance: f32) {
        self.inner.set_stretch_tolerance(stretch_tolerance);
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
    // SAFETY: The library must contain a symbol named "plato_plugin_entry" that is a valid
    // function pointer matching the EntryFn type. The caller must ensure the library is
    // loaded from a trusted source.
    #[expect(improper_ctypes_definitions, reason = "FFI type definition requires extern \"C\" function pointer returning trait object pointer for plugin system")]
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
