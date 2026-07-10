use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub action: String,
    pub actor: String,
    pub resource: String,
    pub timestamp: chrono::NaiveDateTime,
}
pub struct AuditLogger;
impl AuditLogger {
    pub fn new() -> Self {
        Self
    }
    pub fn log(&self, _entry: AuditEntry) {}
    pub fn query(&self, _limit: i64) -> Vec<AuditEntry> {
        vec![]
    }
}
