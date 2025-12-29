//! Dynamic library loading for native plugins.

use libloading::{Library, Symbol};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::error::{PluginError, PluginResult};
use crate::traits::{Plugin, PluginFactory, PluginMetadata};

/// Type alias for the plugin entry point function.
pub type PluginCreate = unsafe fn() -> *mut dyn PluginFactory;

/// Dynamic library loader for plugins.
pub struct DynamicLoader {
    /// Search paths for plugins.
    search_paths: Vec<PathBuf>,

    /// Loaded libraries (kept alive to prevent unloading).
    loaded_libraries: Vec<Arc<Library>>,
}

impl DynamicLoader {
    /// Create a new dynamic loader.
    pub fn new() -> Self {
        Self {
            search_paths: vec![],
            loaded_libraries: vec![],
        }
    }

    /// Add a search path for plugins.
    pub fn add_search_path(&mut self, path: PathBuf) {
        if !self.search_paths.contains(&path) {
            self.search_paths.push(path);
        }
    }

    /// Load a plugin from a dynamic library.
    pub fn load_plugin(&mut self, library_path: &Path) -> PluginResult<Box<dyn Plugin>> {
        tracing::info!("Loading plugin from: {:?}", library_path);

        // Load the library
        let library = unsafe {
            Library::new(library_path).map_err(|e| PluginError::LibraryLoadError {
                path: library_path.to_path_buf(),
                source: e,
            })?
        };

        // Get the plugin creation function
        let plugin_create: Symbol<PluginCreate> = unsafe {
            library
                .get(b"_plugin_create")
                .map_err(|e| PluginError::LibraryLoadError {
                    path: library_path.to_path_buf(),
                    source: e,
                })?
        };

        // Create the plugin factory
        let factory_ptr = unsafe { plugin_create() };
        let factory: Box<dyn PluginFactory> = unsafe { Box::from_raw(factory_ptr) };

        // Create the plugin instance
        let plugin = factory.create()?;

        // Keep the library alive
        let library_arc = Arc::new(library);
        self.loaded_libraries.push(library_arc);

        tracing::info!(
            "Successfully loaded plugin: {}",
            plugin.metadata().id
        );

        Ok(plugin)
    }

    /// Search for a plugin by name in search paths.
    pub fn find_plugin(&self, plugin_name: &str) -> Option<PathBuf> {
        let lib_filename = Self::library_filename(plugin_name);

        for search_path in &self.search_paths {
            let plugin_path = search_path.join(&lib_filename);
            if plugin_path.exists() {
                return Some(plugin_path);
            }
        }

        None
    }

    /// Load a plugin by name (searches in search paths).
    pub fn load_plugin_by_name(&mut self, plugin_name: &str) -> PluginResult<Box<dyn Plugin>> {
        let plugin_path = self.find_plugin(plugin_name).ok_or_else(|| {
            PluginError::PluginNotFound {
                id: plugin_name.to_string(),
            }
        })?;

        self.load_plugin(&plugin_path)
    }

    /// Get platform-specific library filename.
    fn library_filename(plugin_name: &str) -> String {
        #[cfg(target_os = "windows")]
        {
            format!("{}.dll", plugin_name)
        }

        #[cfg(target_os = "macos")]
        {
            format!("lib{}.dylib", plugin_name)
        }

        #[cfg(target_os = "linux")]
        {
            format!("lib{}.so", plugin_name)
        }

        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            format!("{}.so", plugin_name)
        }
    }

    /// Get the number of loaded libraries.
    pub fn loaded_count(&self) -> usize {
        self.loaded_libraries.len()
    }

    /// Clear all loaded libraries (this will unload them).
    pub fn clear(&mut self) {
        self.loaded_libraries.clear();
    }
}

impl Default for DynamicLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper to verify plugin ABI compatibility.
pub struct AbiChecker {
    platform_version: semver::Version,
}

impl AbiChecker {
    /// Create a new ABI checker.
    pub fn new(platform_version: semver::Version) -> Self {
        Self { platform_version }
    }

    /// Check if a plugin is ABI compatible.
    pub fn check_compatibility(&self, metadata: &PluginMetadata) -> PluginResult<()> {
        // For now, just check version compatibility
        // In a real system, you might check:
        // - Rust compiler version
        // - Target triple
        // - ABI version number
        // - Required symbols

        if metadata.min_platform_version > self.platform_version {
            return Err(PluginError::VersionIncompatible(format!(
                "Plugin requires platform version >= {}, but current version is {}",
                metadata.min_platform_version, self.platform_version
            )));
        }

        Ok(())
    }
}

/// Plugin discovery helper.
pub struct PluginDiscovery {
    search_paths: Vec<PathBuf>,
}

impl PluginDiscovery {
    /// Create a new plugin discovery helper.
    pub fn new() -> Self {
        Self {
            search_paths: vec![],
        }
    }

    /// Add a search path.
    pub fn add_path(&mut self, path: PathBuf) {
        self.search_paths.push(path);
    }

    /// Discover all plugins in search paths.
    pub fn discover(&self) -> Vec<PathBuf> {
        let mut plugins = Vec::new();

        for search_path in &self.search_paths {
            if let Ok(entries) = std::fs::read_dir(search_path) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.is_file() && Self::is_plugin_library(&path) {
                        plugins.push(path);
                    }
                }
            }
        }

        plugins
    }

    /// Check if a file is a plugin library based on extension.
    fn is_plugin_library(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = ext.to_string_lossy().to_lowercase();

            #[cfg(target_os = "windows")]
            {
                ext_str == "dll"
            }

            #[cfg(target_os = "macos")]
            {
                ext_str == "dylib"
            }

            #[cfg(target_os = "linux")]
            {
                ext_str == "so"
            }

            #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
            {
                ext_str == "so"
            }
        } else {
            false
        }
    }
}

impl Default for PluginDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Safe wrapper for plugin loading with validation.
pub struct SafePluginLoader {
    loader: DynamicLoader,
    abi_checker: AbiChecker,
    allowed_paths: Vec<PathBuf>,
}

impl SafePluginLoader {
    /// Create a new safe plugin loader.
    pub fn new(platform_version: semver::Version) -> Self {
        Self {
            loader: DynamicLoader::new(),
            abi_checker: AbiChecker::new(platform_version),
            allowed_paths: vec![],
        }
    }

    /// Add an allowed plugin path (security restriction).
    pub fn allow_path(&mut self, path: PathBuf) {
        self.allowed_paths.push(path);
    }

    /// Add a search path.
    pub fn add_search_path(&mut self, path: PathBuf) {
        self.loader.add_search_path(path);
    }

    /// Load a plugin with security and compatibility checks.
    pub fn load_plugin(&mut self, library_path: &Path) -> PluginResult<Box<dyn Plugin>> {
        // Security check: verify path is allowed
        if !self.allowed_paths.is_empty() {
            let is_allowed = self
                .allowed_paths
                .iter()
                .any(|allowed| library_path.starts_with(allowed));

            if !is_allowed {
                return Err(PluginError::PermissionDenied {
                    id: library_path.to_string_lossy().to_string(),
                    action: "load from untrusted path".to_string(),
                });
            }
        }

        // Load the plugin
        let plugin = self.loader.load_plugin(library_path)?;

        // Check ABI compatibility
        self.abi_checker.check_compatibility(plugin.metadata())?;

        Ok(plugin)
    }

    /// Load plugin by name.
    pub fn load_plugin_by_name(&mut self, plugin_name: &str) -> PluginResult<Box<dyn Plugin>> {
        let plugin_path = self.loader.find_plugin(plugin_name).ok_or_else(|| {
            PluginError::PluginNotFound {
                id: plugin_name.to_string(),
            }
        })?;

        self.load_plugin(&plugin_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_filename() {
        #[cfg(target_os = "linux")]
        {
            let filename = DynamicLoader::library_filename("test_plugin");
            assert_eq!(filename, "libtest_plugin.so");
        }

        #[cfg(target_os = "windows")]
        {
            let filename = DynamicLoader::library_filename("test_plugin");
            assert_eq!(filename, "test_plugin.dll");
        }

        #[cfg(target_os = "macos")]
        {
            let filename = DynamicLoader::library_filename("test_plugin");
            assert_eq!(filename, "libtest_plugin.dylib");
        }
    }

    #[test]
    fn test_is_plugin_library() {
        #[cfg(target_os = "linux")]
        {
            assert!(PluginDiscovery::is_plugin_library(Path::new(
                "libtest.so"
            )));
            assert!(!PluginDiscovery::is_plugin_library(Path::new(
                "test.txt"
            )));
        }
    }
}
