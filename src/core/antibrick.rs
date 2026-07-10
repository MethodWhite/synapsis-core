// TODO: Implement anti-brick command validation.
// All methods are currently stubs returning safe defaults.

use serde::{Deserialize, Serialize};

/// Risk levels for operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe = 0,
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
    Blocked = 5,
}

/// Types of destructive operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrickThreat {
    DiskWrite { target: String, tool: String },
    PartitionModify { disk: String, tool: String },
    BootloaderAccess { device: String, command: String },
    FilesystemDestroy { partition: String, tool: String },
    MountOperation { path: String, operation: String },
    FirmwareFlash { device: String, image_type: String },
    BootloaderLock { device: String, action: String },
    Suspicious { command: String, reason: String },
}

/// Audit log entry for anti-brick events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiBrickEvent {
    pub id: u64,
    pub timestamp: u64,
    pub event_type: String,
    pub threat: Option<BrickThreat>,
    pub risk_level: RiskLevel,
    pub command: String,
    pub args: Vec<String>,
    pub process_id: u32,
    pub user: String,
    pub blocked: bool,
    pub ai_validated: bool,
    pub ai_response: Option<String>,
    pub hash: String,
}

/// Legacy alias
pub type AntiBrickEngine = AntiBrick;
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct AntiBrick {
    #[allow(dead_code)]
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
    #[derive(Debug, Clone)]
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
