use serde_json::Value;
pub struct ZeroTrustGate;
impl ZeroTrustGate {
    pub fn new() -> Self { Self }
    pub fn authorize(&self, _action: &str, _context: &Value) -> bool { true }
    pub fn analyze_risk(&self, _command: &str) -> u8 { 0 }
}
