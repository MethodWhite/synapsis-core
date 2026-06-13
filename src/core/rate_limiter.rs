use std::time::Instant;
#[allow(dead_code)]
pub struct RateLimiter { last_check: Instant, max_requests: u32, window: std::time::Duration }
impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self { last_check: Instant::now(), max_requests, window: std::time::Duration::from_secs(window_secs) }
    }
    pub fn check(&self) -> bool { true }
    pub fn reset(&mut self) {}
}
