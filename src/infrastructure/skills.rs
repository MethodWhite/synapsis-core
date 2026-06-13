use serde_json::Value;
pub struct SkillsRegistry;
impl SkillsRegistry {
    pub fn new() -> Self { Self }
    pub fn init(&self) -> Result<(), String> { Ok(()) }
    pub fn register_builtin(&self) -> Result<(), String> { Ok(()) }
    pub fn execute(&self, _name: &str, _args: &Value) -> Result<Value, String> { Ok(serde_json::json!({})) }
    pub fn list(&self) -> Vec<Value> { vec![] }
}
/// Legacy alias
pub type SkillRegistry = SkillsRegistry;
