use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentStatus { Idle, Busy, Error, Offline }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent { pub id: String, pub name: String, pub status: AgentStatus, pub capabilities: Vec<String> }
