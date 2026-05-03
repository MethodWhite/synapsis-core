//! Plugin System for Synapsis
//!
//! Defines the plugin architecture that allows external capabilities to be
//! "mounted" like Lego pieces to the core Synapsis MCP server.

use crate::domain::Result;

/// Lifecycle hooks for plugins
pub enum PluginLifecycle {
    /// Plugin is being loaded
    Load,
    /// Plugin is being initialized (dependencies available)
    Initialize,
    /// Plugin is being started (ready to handle requests)
    Start,
    /// Plugin is being stopped
    Stop,
    /// Plugin is being unloaded
    Unload,
}

/// Extension points where plugins can register capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtensionPoint {
    /// Cryptography providers (CryptoProvider trait)
    CryptoProvider,
    /// Authentication/Authorization systems
    AuthProvider,
    /// Storage backends (StoragePort trait)
    StorageBackend,
    /// LLM providers (LlmProvider trait)
    LlmProvider,
    /// Worker agents (WorkerAgent trait)
    WorkerAgent,
    /// RPC handlers (RpcHandler trait)
    RpcHandler,
    /// Task queue implementations
    TaskQueue,
    /// Database adapters
    DatabaseAdapter,
    /// Monitoring/Telemetry
    Monitoring,
    /// Audit logging
    AuditLogging,
}

/// Information about a plugin
#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub license: String,
    pub extension_points: Vec<ExtensionPoint>,
    pub dependencies: Vec<String>,
}

/// Core trait for Synapsis plugins
pub trait SynapsisPlugin: Send + Sync {
    /// Get plugin metadata
    fn info(&self) -> PluginInfo;

    /// Handle lifecycle events
    fn on_lifecycle(&self, lifecycle: PluginLifecycle) -> Result<()>;

    /// Get extension points this plugin provides
    fn extension_points(&self) -> Vec<ExtensionPoint>;

    /// Register extensions at the given extension point
    fn register_extensions(&self, registry: &mut PluginRegistry) -> Result<()>;
}

/// Plugin registry for managing all plugins
pub struct PluginRegistry {
    plugins: std::collections::HashMap<String, std::sync::Arc<dyn SynapsisPlugin>>,
    extensions: std::collections::HashMap<
        ExtensionPoint,
        Vec<std::sync::Arc<dyn std::any::Any + Send + Sync>>,
    >,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: std::collections::HashMap::new(),
            extensions: std::collections::HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register_plugin(&mut self, plugin: std::sync::Arc<dyn SynapsisPlugin>) -> Result<()> {
        let info = plugin.info();
        let id = info.id.clone();

        // Initialize plugin lifecycle
        plugin.on_lifecycle(PluginLifecycle::Load)?;

        // Store plugin
        self.plugins.insert(id.clone(), plugin.clone());

        // Register extensions
        plugin.register_extensions(self)?;

        // Continue lifecycle
        plugin.on_lifecycle(PluginLifecycle::Initialize)?;

        Ok(())
    }

    /// Register an extension at an extension point
    pub fn register_extension<T: Send + Sync + 'static>(
        &mut self,
        point: ExtensionPoint,
        extension: std::sync::Arc<T>,
    ) {
        self.extensions
            .entry(point)
            .or_insert_with(Vec::new)
            .push(extension as std::sync::Arc<dyn std::any::Any + Send + Sync>);
    }

    /// Get extensions for a specific extension point
    pub fn get_extensions<T: Send + Sync + 'static>(
        &self,
        point: ExtensionPoint,
    ) -> Vec<std::sync::Arc<T>> {
        self.extensions
            .get(&point)
            .map(|exts| {
                exts.iter()
                    .filter_map(|ext| ext.clone().downcast::<T>().ok())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get a plugin by ID
    pub fn get_plugin(&self, id: &str) -> Option<std::sync::Arc<dyn SynapsisPlugin>> {
        self.plugins.get(id).cloned()
    }

    /// List all registered plugins
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    /// Start all plugins
    pub fn start_all(&self) -> Result<()> {
        for plugin in self.plugins.values() {
            plugin.on_lifecycle(PluginLifecycle::Start)?;
        }
        Ok(())
    }

    /// Stop all plugins
    pub fn stop_all(&self) -> Result<()> {
        for plugin in self.plugins.values() {
            plugin.on_lifecycle(PluginLifecycle::Stop)?;
        }
        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
