use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Audit Log Entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: i64,
    pub observation_id: i64,
    pub action: String, // "update", "delete", "restore"
    pub agent_id: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub reason: Option<String>,
    pub timestamp: i64,
}

/// Audit Log Manager
pub struct AuditLog;

impl AuditLog {
    pub fn new() -> Self {
        Self
    }

    /// Log an update action
    pub fn log_update(
        &self,
        obs_id: i64,
        agent_id: &str,
        old_content: &str,
        new_content: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        let _entry = AuditEntry {
            id: current_timestamp(),
            observation_id: obs_id,
            action: "update".to_string(),
            agent_id: agent_id.to_string(),
            old_value: Some(old_content.to_string()),
            new_value: Some(new_content.to_string()),
            reason: reason.map(String::from),
            timestamp: current_timestamp(),
        };

        eprintln!("[Audit] UPDATE obs={} by agent={}", obs_id, agent_id);
        Ok(())
    }

    /// Log a soft delete action
    pub fn log_delete(&self, obs_id: i64, agent_id: &str, reason: Option<&str>) -> Result<()> {
        let _entry = AuditEntry {
            id: current_timestamp(),
            observation_id: obs_id,
            action: "delete".to_string(),
            agent_id: agent_id.to_string(),
            old_value: None,
            new_value: None,
            reason: reason.map(String::from),
            timestamp: current_timestamp(),
        };

        eprintln!("[Audit] DELETE obs={} by agent={}", obs_id, agent_id);
        Ok(())
    }

    /// Log a restore action
    pub fn log_restore(&self, obs_id: i64, agent_id: &str) -> Result<()> {
        let _entry = AuditEntry {
            id: current_timestamp(),
            observation_id: obs_id,
            action: "restore".to_string(),
            agent_id: agent_id.to_string(),
            old_value: None,
            new_value: None,
            reason: Some("Restored from soft delete".to_string()),
            timestamp: current_timestamp(),
        };

        eprintln!("[Audit] RESTORE obs={} by agent={}", obs_id, agent_id);
        Ok(())
    }

    /// Get audit trail for an observation
    pub fn get_audit_trail(&self, _obs_id: i64) -> Result<Vec<AuditEntry>> {
        Ok(vec![])
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_creation() {
        let log = AuditLog::new();
        assert!(log.log_update(1, "agent1", "old", "new", None).is_ok());
    }
}
