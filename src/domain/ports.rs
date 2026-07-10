use crate::domain::entities::MemoryEntry;
use crate::domain::types::ObservationId;

/// Database-agnostic value type used in StorageBackend trait.
///
/// Replaces direct dependency on `rusqlite::types::Value` so the domain
/// layer does not depend on any specific database engine.
#[derive(Debug, Clone)]
pub enum DbValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

impl From<String> for DbValue {
    fn from(s: String) -> Self { DbValue::Text(s) }
}
impl From<&str> for DbValue {
    fn from(s: &str) -> Self { DbValue::Text(s.to_string()) }
}
impl From<i64> for DbValue {
    fn from(n: i64) -> Self { DbValue::Integer(n) }
}
impl From<f64> for DbValue {
    fn from(f: f64) -> Self { DbValue::Real(f) }
}
impl From<Vec<u8>> for DbValue {
    fn from(b: Vec<u8>) -> Self { DbValue::Blob(b) }
}

/// Low-level storage backend abstraction supporting multiple database engines.
pub trait StorageBackend: Send + Sync {
    fn execute(&self, sql: &str, params: &[DbValue]) -> Result<u64, String>;
    fn query(&self, sql: &str, params: &[DbValue]) -> Result<Vec<Vec<DbValue>>, String>;
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
    fn search_fts(
        &self,
        query: &str,
        project: Option<&str>,
        limit: i32,
        max_tokens: Option<u32>,
    ) -> Result<Vec<serde_json::Value>, String>;
    fn retain(&self, max_tokens: u64) -> Result<u64, String>; // evict lowest-priority entries, return freed tokens
    fn stats(&self) -> Result<MemoryStats, String>;
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub total_entries: u64,
    pub total_tokens: u64,
    pub avg_importance: f32,
    pub unique_sessions: u64,
}
use serde::{Deserialize, Serialize};

pub use crate::domain::entities::Observation;
pub use crate::domain::entities::SearchParams;
pub use crate::domain::entities::SearchResult;
pub use crate::domain::models::agent::AgentStatus;
pub use crate::domain::types::ObservationType;

// Legacy compatibility stubs
pub trait SessionPort: Send + Sync {}
impl SessionPort for crate::infrastructure::database::Database {}
