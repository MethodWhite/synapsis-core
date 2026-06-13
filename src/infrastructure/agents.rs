use serde_json::Value;
pub struct AgentManager;
impl AgentManager {
    pub fn new() -> Self { Self }
    pub fn init(&self) -> Result<(), String> { Ok(()) }
    pub fn register(&self, _name: &str, _agent_type: &str) -> Result<String, String> { Ok(uuid::Uuid::new_v4().to_string()) }
    pub fn list(&self) -> Vec<Value> { vec![] }
    pub fn get(&self, _id: &str) -> Option<Value> { None }
    pub fn delete(&self, _id: &str) -> Result<(), String> { Ok(()) }
}
/// Legacy alias
pub type AgentRegistry = AgentManager;
