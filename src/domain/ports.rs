use crate::domain::entities::{MemoryEntry, SessionInfo};
use crate::domain::types::ObservationId;

/// Low-level storage backend abstraction supporting multiple database engines.
pub trait StorageBackend: Send + Sync {
    fn execute(&self, sql: &str, params: &[rusqlite::types::Value]) -> Result<u64, String>;
    fn query(&self, sql: &str, params: &[rusqlite::types::Value]) -> Result<Vec<Vec<rusqlite::types::Value>>, String>;
    fn execute_batch(&self, sql: &str) -> Result<(), String>;
    fn as_any(&self) -> &dyn std::any::Any;
}

pub trait StoragePort: Send + Sync {
    fn save_observation(&self, obs: &Observation) -> Result<ObservationId, String>;
    fn search_observations(&self, params: &SearchParams) -> Result<Vec<SearchResult>, String>;
    fn recent_observations(&self, limit: usize) -> Result<Vec<Observation>, String>;
    fn get_by_id(&self, id: i64) -> Result<Option<Observation>, String>;
    fn delete(&self, id: i64) -> Result<(), String>;
}

pub trait MemoryPort: Send + Sync {
    fn save_memory(&self, memory: &MemoryEntry) -> Result<(), String>;
    fn search_fts(&self, query: &str, project: Option<&str>, limit: i32, max_tokens: Option<u32>) -> Result<Vec<serde_json::Value>, String>;
    fn retain(&self, max_tokens: u64) -> Result<u64, String>;  // evict lowest-priority entries, return freed tokens
    fn stats(&self) -> Result<MemoryStats, String>;
}

use crate::domain::types::SessionId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_entries: u64,
    pub total_tokens: u64,
    pub avg_importance: f32,
    pub unique_sessions: u64,
}
use serde::{Deserialize, Serialize};

pub use crate::domain::types::ObservationType;
pub use crate::domain::entities::Observation;
pub use crate::domain::entities::SearchParams;
pub use crate::domain::entities::SearchResult;
pub use crate::domain::models::agent::AgentStatus;

// Legacy compatibility stubs
pub trait SessionPort: Send + Sync {}
impl SessionPort for crate::infrastructure::database::Database {}
