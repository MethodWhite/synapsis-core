use serde_json::Value;
pub struct Orchestrator;
impl Orchestrator {
    pub fn new() -> Self { Self }
    pub fn register_agent(&self, _id: &str) {}
    pub fn get_agent_status(&self, _id: &str) -> AgentStatus { AgentStatus::Idle }
    pub fn list_agents(&self) -> Vec<Value> { vec![] }
    pub fn assign_task(&self, _agent_id: &str, _task: Value) {}
    pub fn find_best_agent(&self, _skills: &[String]) -> Option<String> { None }
    pub fn create_task(&self, _description: &str, _capabilities: Vec<String>, _priority: i32, _parent: Option<String>) -> String {
        uuid::Uuid::new_v4().to_string()
    }
    pub fn complete_task(&self, _task_id: &str, _success: bool) -> Result<(), String> { Ok(()) }
    pub fn delegate_task(&self, _agent_id: &str, _task: Value) -> Option<String> { None }
    pub fn heartbeat(&self, _id: &str, _status: Option<crate::domain::models::agent::AgentStatus>, _task: Option<&str>) {}
}
pub use crate::domain::models::agent::AgentStatus;
