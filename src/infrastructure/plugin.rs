//! Synapsis Plugin System for MCP Server
//!
//! Dynamic plugin management for extending MCP server capabilities.
//! Supports intelligent lifecycle management: loading, unloading, updating, cleanup.

use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Plugin metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub repository: Option<String>,
    pub license: Option<String>,
    pub enabled: bool,
    pub loaded_at: i64,
    pub last_used: i64,
    pub usage_count: u64,
}

/// MCP Tool definition
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

/// Plugin trait for dynamic MCP tool providers
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Get list of tools provided by this plugin
    fn tools(&self) -> Vec<MCPTool>;

    /// Execute a tool
    fn execute_tool(&self, name: &str, arguments: &Value) -> Result<Value, String>;

    /// Plugin initialization
    fn initialize(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Plugin cleanup
    fn cleanup(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Check for updates
    fn check_for_updates(&self) -> Option<String> {
        None
    }

    /// Get plugin health status
    fn health_check(&self) -> Result<(), String> {
        Ok(())
    }
}

/// Plugin manager for loading, unloading, and managing plugins
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, Box<dyn Plugin>>>>,
    #[allow(dead_code)]
    plugin_dir: PathBuf,
}

impl PluginManager {
    /// Create new plugin manager with directory for plugin storage
    pub fn new(plugin_dir: PathBuf) -> Self {
        std::fs::create_dir_all(&plugin_dir).ok();
        Self {
            plugins: Arc::new(RwLock::new(HashMap::new())),
            plugin_dir,
        }
    }

    /// Load a plugin from a path (supports WASM, dynamic libs, or scripts)
    pub fn load_plugin(&self, path: &str) -> Result<String, String> {
        let plugin_path = PathBuf::from(path);
        if !plugin_path.exists() {
            return Err(format!("Plugin not found: {}", path));
        }

        // TODO: Implement actual plugin loading based on file type
        // For now, create a dummy plugin for demonstration
        let plugin_id = format!(
            "plugin_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let metadata = PluginMetadata {
            id: plugin_id.clone(),
            name: plugin_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string(),
            version: "1.0.0".to_string(),
            description: format!("Plugin loaded from {}", path),
            author: None,
            repository: None,
            license: None,
            enabled: true,
            loaded_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            last_used: 0,
            usage_count: 0,
        };

        let plugin = DummyPlugin { metadata };

        let mut plugins = self.plugins.write().unwrap();
        plugins.insert(plugin_id.clone(), Box::new(plugin));

        Ok(plugin_id)
    }

    /// Unload a plugin
    pub fn unload_plugin(&self, plugin_id: &str) -> Result<(), String> {
        let mut plugins = self.plugins.write().unwrap();
        if let Some(mut plugin) = plugins.remove(plugin_id) {
            plugin.cleanup().ok();
            Ok(())
        } else {
            Err(format!("Plugin not found: {}", plugin_id))
        }
    }

    /// Get all loaded plugins
    pub fn get_plugins(&self) -> Vec<PluginMetadata> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().map(|p| p.metadata()).collect()
    }

    /// Get plugin by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<PluginMetadata> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(plugin_id).map(|p| p.metadata())
    }

    /// Enable/disable a plugin
    pub fn set_plugin_enabled(&self, plugin_id: &str, _enabled: bool) -> Result<(), String> {
        let mut plugins = self.plugins.write().unwrap();
        if let Some(_plugin) = plugins.get_mut(plugin_id) {
            // In a real implementation, we'd update the plugin state
            // For now, we'll just return success
            Ok(())
        } else {
            Err(format!("Plugin not found: {}", plugin_id))
        }
    }

    /// Get all tools from all plugins
    pub fn get_all_tools(&self) -> Vec<MCPTool> {
        let plugins = self.plugins.read().unwrap();
        plugins.values().flat_map(|p| p.tools()).collect()
    }

    /// Execute a tool from any plugin
    pub fn execute_tool(&self, tool_name: &str, arguments: &Value) -> Result<Value, String> {
        let plugins = self.plugins.read().unwrap();
        for plugin in plugins.values() {
            for tool in plugin.tools() {
                if tool.name == tool_name {
                    return plugin.execute_tool(tool_name, arguments);
                }
            }
        }
        Err(format!("Tool not found: {}", tool_name))
    }

    /// Check for plugin updates
    pub fn check_for_updates(&self) -> HashMap<String, Option<String>> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .iter()
            .map(|(id, plugin)| (id.clone(), plugin.check_for_updates()))
            .collect()
    }

    /// Perform health checks on all plugins
    pub fn health_check(&self) -> HashMap<String, Result<(), String>> {
        let plugins = self.plugins.read().unwrap();
        plugins
            .iter()
            .map(|(id, plugin)| (id.clone(), plugin.health_check()))
            .collect()
    }

    /// Clean up unused plugins (based on last_used timestamp)
    pub fn cleanup_unused_plugins(&self, max_age_seconds: i64) -> Vec<String> {
        let mut removed = Vec::new();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut plugins = self.plugins.write().unwrap();
        let ids: Vec<String> = plugins.keys().cloned().collect();

        for id in ids {
            if let Some(plugin) = plugins.get(&id) {
                let metadata = plugin.metadata();
                if metadata.last_used > 0 && (now - metadata.last_used) > max_age_seconds {
                    if let Some(mut plugin) = plugins.remove(&id) {
                        plugin.cleanup().ok();
                        removed.push(id);
                    }
                }
            }
        }

        removed
    }

    /// Update plugin usage timestamp
    pub fn update_plugin_usage(&self, _plugin_id: &str) -> Result<(), String> {
        // In a real implementation, we'd update the metadata
        // For now, just acknowledge
        Ok(())
    }
}

/// Dummy plugin for demonstration
struct DummyPlugin {
    metadata: PluginMetadata,
}

impl Plugin for DummyPlugin {
    fn metadata(&self) -> PluginMetadata {
        self.metadata.clone()
    }

    fn tools(&self) -> Vec<MCPTool> {
        vec![MCPTool {
            name: "dummy_tool".to_string(),
            description: "A dummy tool from plugin".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "message": { "type": "string" }
                },
                "required": ["message"]
            }),
        }]
    }

    fn execute_tool(&self, name: &str, arguments: &Value) -> Result<Value, String> {
        match name {
            "dummy_tool" => {
                let message = arguments
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Hello from plugin!");
                Ok(serde_json::json!({
                    "result": format!("Plugin says: {}", message)
                }))
            }
            _ => Err(format!("Unknown tool: {}", name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let temp_dir = std::env::temp_dir().join("synapsis_test_plugins");
        let manager = PluginManager::new(temp_dir.clone());

        // Initially no plugins
        assert_eq!(manager.get_plugins().len(), 0);

        // Try to load non-existent plugin
        assert!(manager.load_plugin("/nonexistent/plugin.wasm").is_err());
    }

    #[test]
    fn test_dummy_plugin() {
        let metadata = PluginMetadata {
            id: "test_plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "Test plugin".to_string(),
            author: None,
            repository: None,
            license: None,
            enabled: true,
            loaded_at: 0,
            last_used: 0,
            usage_count: 0,
        };

        let plugin = DummyPlugin { metadata };
        let tools = plugin.tools();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "dummy_tool");

        let result = plugin.execute_tool(
            "dummy_tool",
            &serde_json::json!({
                "message": "test"
            }),
        );
        assert!(result.is_ok());
    }
}
