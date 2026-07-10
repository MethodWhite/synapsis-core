pub struct FilesystemWatchdog;
impl FilesystemWatchdog {
    pub fn new(_config: WatchdogConfig) -> Self {
        Self
    }
    pub fn start_monitoring(&self) {}
    pub fn stop_monitoring(&self) {}
}
pub struct WatchdogConfig;
impl Default for WatchdogConfig {
    fn default() -> Self {
        Self
    }
}
pub mod mcp_tools {
    use super::FilesystemWatchdog;
    use serde_json::{json, Value};
    pub struct McpWatchdogAdapter;
    impl McpWatchdogAdapter {
        pub fn new(_watchdog: FilesystemWatchdog) -> Self {
            Self
        }
        pub fn list_files(_path: &str) -> Vec<Value> {
            vec![]
        }
        pub fn read_file(_path: &str) -> Option<String> {
            None
        }
        pub fn watch(_path: &str, _callback: Box<dyn Fn(String)>) {}
    }
    // Legacy stub methods (matching wrapper's calling convention)
    use std::sync::Arc;
    pub fn handle_watchdog_stats(_wd: &Arc<super::FilesystemWatchdog>) -> Value {
        json!({"watched": 0})
    }
    pub fn handle_watchdog_verify(_wd: &Arc<super::FilesystemWatchdog>) -> Value {
        json!({"verified": true})
    }
    pub fn handle_watchdog_snapshot(_wd: &Arc<super::FilesystemWatchdog>, _path: String) -> Value {
        json!({"snapshot": "ok" })
    }
    pub fn handle_watchdog_events(_wd: &Arc<super::FilesystemWatchdog>, _limit: usize) -> Value {
        json!({"events": []})
    }
    pub fn handle_watchdog_check_path(
        _wd: &Arc<super::FilesystemWatchdog>,
        _path: String,
    ) -> Value {
        json!({"safe": true})
    }
}
