use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionId {
    pub cli_type: String,
    pub instance_uuid: String,
    pub hostname: String,
    pub pid: u32,
    pub created_at: i64,
}

impl SessionId {
    pub fn new(cli_type: &str) -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
        Self {
            cli_type: cli_type.to_string(),
            instance_uuid: uuid::Uuid::new_v4().to_string(),
            hostname: gethostname(),
            pid: std::process::id(),
            created_at: now,
        }
    }
    pub fn is_stale(&self, _max_age_secs: u64) -> bool { false }
    pub fn as_str(&self) -> &str { &self.instance_uuid }
}

fn gethostname() -> String {
    std::env::var("HOSTNAME").unwrap_or_else(|_| "localhost".to_string())
}

pub struct SessionRegistry {
    sessions: HashMap<String, SessionId>,
}
impl SessionRegistry {
    pub fn new() -> Self { Self { sessions: HashMap::new() } }
    pub fn register(&mut self, session: SessionId) {
        self.sessions.insert(session.instance_uuid.clone(), session);
    }
    pub fn count_by_cli_type(&self, cli_type: &str) -> usize {
        self.sessions.values().filter(|s| s.cli_type == cli_type).count()
    }
    pub fn get_active(&self, _max_age_secs: i64) -> Vec<SessionId> {
        self.sessions.values().cloned().collect()
    }
    pub fn cleanup_stale(&mut self, _max_age_secs: i64) -> usize { 0 }
    pub fn get_by_cli_type(&self, cli_type: &str) -> Vec<SessionId> {
        self.sessions.values().filter(|s| s.cli_type == cli_type).cloned().collect()
    }
    pub fn unregister(&mut self, uuid: &str) {
        self.sessions.remove(uuid);
    }
}
