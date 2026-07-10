use serde_json::Value;
/// Legacy alias
pub type AntiBrickEngine = AntiBrick;
pub struct AntiBrickConfig {
    pub max_args: usize,
    pub blocked_patterns: Vec<String>,
}
impl Default for AntiBrickConfig {
    fn default() -> Self {
        Self {
            max_args: 64,
            blocked_patterns: vec![],
        }
    }
}
pub struct AntiBrick {
    enabled: bool,
}
impl AntiBrick {
    pub fn new(_config: AntiBrickConfig) -> Self {
        Self { enabled: true }
    }
    pub fn check(&self, _args: &[std::ffi::OsString]) -> Result<(), String> {
        Ok(())
    }
}
pub mod mcp_tools {
    use super::AntiBrick;
    use serde_json::{json, Value};
    use std::sync::Arc;
    pub struct McpAntiBrickAdapter;
    impl McpAntiBrickAdapter {
        pub fn new(_anti_brick: AntiBrick) -> Self {
            Self
        }
        pub fn validate_command(_cmd: &str, _args: &[String]) -> Result<(), String> {
            Ok(())
        }
        pub fn block(_reason: &str) -> Value {
            serde_json::json!({"blocked": true})
        }
    }
    // Legacy stub functions
    pub fn handle_antibrick_scan(
        _ab: &Arc<super::AntiBrick>,
        _cmd: &str,
        _args: Vec<String>,
    ) -> Value {
        json!({"scanned": true})
    }
    pub fn handle_antibrick_stats(_ab: &Arc<super::AntiBrick>) -> Value {
        json!({"blocked": 0})
    }
    pub fn handle_antibrick_enable(_ab: &Arc<super::AntiBrick>, _enable: bool) -> Value {
        json!({"enabled": true})
    }
}
