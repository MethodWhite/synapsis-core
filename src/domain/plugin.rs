use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub enabled: bool,
}
pub struct PluginManager {
    plugins: HashMap<String, PluginInfo>,
}
impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }
    pub fn register(&mut self, info: PluginInfo) {
        self.plugins.insert(info.name.clone(), info);
    }
    pub fn get(&self, id: &str) -> Option<&PluginInfo> {
        self.plugins.get(id)
    }
    pub fn list(&self) -> Vec<&PluginInfo> {
        self.plugins.values().collect()
    }
    pub fn check_for_updates(&self) -> Vec<String> {
        vec![]
    }
    pub fn cleanup_unused_plugins(&self) -> usize {
        0
    }
}
