use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ObservationId(pub i64);
impl ObservationId {
    pub fn new(id: i64) -> Self {
        Self(id)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(pub i64);
impl Timestamp {
    pub fn now() -> Self {
        let dur = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        Self(dur.as_secs() as i64)
    }
}
impl std::fmt::Display for Timestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ObservationType {
    Note,
    Memory,
    Event,
    Log,
    Manual,
    ToolUse,
    Search,
    FileChange,
    Decision,
    Command,
    Pattern,
    Learning,
    Discovery,
    Config,
    Bugfix,
    Architecture,
}
impl std::fmt::Display for ObservationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
pub use crate::core::session_id::SessionId;
