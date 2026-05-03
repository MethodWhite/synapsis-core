//! Dynamic Plugin Loading
//!
//! Provides runtime loading of .so/.dylib/.dll plugins using libloading.
//!
//! # Example
//!
//! ```rust,no_run
//! use synapsis_core::domain::plugin::DynamicPluginLoader;
//!
//! let loader = DynamicPluginLoader::new();
//! let plugin = loader.load_plugin("/path/to/my_plugin.so").unwrap();
//! ```

use super::{PluginRegistry, SynapsisPlugin};
use crate::domain::Result;
use libloading::Library;
use std::path::Path;
use std::sync::Arc;

/// Type alias for plugin constructor function
/// Plugins must export a `plugin_create` function with this signature
#[allow(improper_ctypes_definitions)]
type PluginCreateFn = unsafe extern "C" fn() -> *mut dyn SynapsisPlugin;

/// Type alias for plugin destructor function
#[allow(improper_ctypes_definitions)]
type PluginDestroyFn = unsafe extern "C" fn(*mut dyn SynapsisPlugin);

/// Dynamic plugin loader for loading .so/.dylib/.dll files at runtime
pub struct DynamicPluginLoader {
    loaded_libraries: Vec<Arc<Library>>,
}

impl DynamicPluginLoader {
    /// Create a new dynamic plugin loader
    pub fn new() -> Self {
        Self {
            loaded_libraries: Vec::new(),
        }
    }

    /// Load a plugin from a .so/.dylib/.dll file
    ///
    /// The library must export a `plugin_create` function:
    /// ```c
    /// extern "C" fn plugin_create() -> *mut dyn SynapsisPlugin
    /// ```
    ///
    /// # Safety
    ///
    /// This function is unsafe because it loads dynamic code.
    /// Only load plugins from trusted sources.
    pub fn load_plugin<P: AsRef<Path>>(&mut self, path: P) -> Result<Arc<dyn SynapsisPlugin>> {
        let path = path.as_ref();

        // Load the library
        let library = unsafe { Library::new(path) }.map_err(|e| {
            crate::domain::SynapsisError::internal_bug(format!(
                "Failed to load plugin library: {}",
                e
            ))
        })?;

        // Get the plugin_create function
        let create_fn: libloading::Symbol<PluginCreateFn> =
            unsafe { library.get(b"plugin_create") }.map_err(|e| {
                crate::domain::SynapsisError::internal_bug(format!(
                    "Plugin missing plugin_create function: {}",
                    e
                ))
            })?;

        // Call the constructor
        let plugin_ptr = unsafe { create_fn() };

        if plugin_ptr.is_null() {
            return Err(crate::domain::SynapsisError::internal_bug(
                "plugin_create returned null pointer",
            ));
        }

        // Convert raw pointer to Arc
        // Safety: We trust the plugin to return a valid pointer
        let plugin: Arc<dyn SynapsisPlugin> = unsafe { Arc::from_raw(plugin_ptr) };

        // Store the library to keep it alive
        self.loaded_libraries.push(Arc::new(library));

        Ok(plugin)
    }

    /// Load a plugin and register it with the plugin registry
    pub fn load_and_register<P: AsRef<Path>>(
        &mut self,
        path: P,
        registry: &mut PluginRegistry,
    ) -> Result<()> {
        let plugin = self.load_plugin(path)?;
        let info = plugin.info();
        let _id = info.id;

        // Initialize plugin lifecycle
        plugin.on_lifecycle(super::PluginLifecycle::Load)?;

        // Register extensions
        plugin.register_extensions(registry)?;

        // Continue lifecycle
        plugin.on_lifecycle(super::PluginLifecycle::Initialize)?;

        // Store in registry (we need to clone the Arc, but plugin is already in loaded_libraries)
        // For now, we just register extensions without storing the plugin itself
        // TODO: Add plugin storage to registry

        Ok(())
    }

    /// Unload all plugins
    pub fn unload_all(&mut self) -> Result<()> {
        // Call cleanup on all plugins before unloading
        for library in &self.loaded_libraries {
            if let Ok(_destroy_fn) = unsafe { library.get::<PluginDestroyFn>(b"plugin_destroy") } {
                // If plugin has a destroy function, call it
                // Note: This is tricky because we need the plugin instance
                // For now, we just drop the libraries
            }
        }

        // Drop all libraries
        self.loaded_libraries.clear();

        Ok(())
    }

    /// Get the number of loaded plugins
    pub fn loaded_count(&self) -> usize {
        self.loaded_libraries.len()
    }
}

impl Default for DynamicPluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create a plugin instance from within a plugin library
///
/// This should be called from the plugin's `plugin_create` function
#[macro_export]
macro_rules! create_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn plugin_create() -> *mut dyn $crate::domain::plugin::SynapsisPlugin
        {
            Box::into_raw(Box::new(<$plugin_type>::new()))
        }
    };
}

/// Helper function to destroy a plugin instance
///
/// This should be exported by the plugin library as `plugin_destroy`
#[macro_export]
macro_rules! destroy_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub unsafe extern "C" fn plugin_destroy(
            plugin: *mut dyn $crate::domain::plugin::SynapsisPlugin,
        ) {
            if !plugin.is_null() {
                let _ = Box::from_raw(plugin);
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::plugin::*;

    #[test]
    fn test_loader_creation() {
        let loader = DynamicPluginLoader::new();
        assert_eq!(loader.loaded_count(), 0);
    }

    #[test]
    fn test_loader_unload_all() {
        let mut loader = DynamicPluginLoader::new();
        // No plugins loaded yet
        assert!(loader.unload_all().is_ok());
    }

    // Note: Full dynamic loading test requires a compiled .so file
    // Integration tests are in tests/dynamic_plugin_test.rs
}
