use crate::domain::plugin::{PluginInfo, PluginManager as DomainPluginManager};
pub struct PluginManager(pub DomainPluginManager);
impl PluginManager {
    pub fn new() -> Self {
        Self(DomainPluginManager::new())
    }
    pub fn with_path(_path: std::path::PathBuf) -> Self {
        Self(DomainPluginManager::new())
    }
    pub fn register(&mut self, name: &str, version: &str, enabled: bool) {
        self.0.register(PluginInfo {
            id: name.into(),
            name: name.into(),
            version: version.into(),
            description: String::new(),
            enabled,
        });
    }
    pub fn list(&self) -> Vec<&PluginInfo> {
        self.0.list()
    }
    pub fn get(&self, id: &str) -> Option<&PluginInfo> {
        self.0.get(id)
    }
    pub fn check_for_updates(&self) -> Vec<String> {
        self.0.check_for_updates()
    }
    pub fn cleanup_unused_plugins(&self, _max_age: i64) -> usize {
        self.0.cleanup_unused_plugins()
    }
    pub fn health_check(&self) -> std::collections::HashMap<String, serde_json::Value> {
        std::collections::HashMap::new()
    }
    pub fn load_plugin(&self, _name: &str) -> Result<(), String> {
        Ok(())
    }
    pub fn unload_plugin(&self, _name: &str) -> Result<(), String> {
        Ok(())
    }
    pub fn get_plugins(&self) -> Vec<&PluginInfo> {
        self.0.list()
    }
    pub fn get_plugin(&self, id: &str) -> Option<&PluginInfo> {
        self.0.get(id)
    }
    pub fn set_plugin_enabled(&self, _name: &str, _enabled: bool) -> Result<(), String> {
        Ok(())
    }
}
