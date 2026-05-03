//! Session Cleanup Job - Automatic Stale Session Detection and Cleanup
//!
//! Implements:
//! - Heartbeat monitoring (agents must heartbeat every N seconds)
//! - Automatic cleanup of zombie sessions
//! - Session timeout enforcement
//!
//! # Usage
//!
//! ```rust
//! // Start background cleanup job (runs every 60 seconds)
//! start_session_cleanup_job(db, 300, 60); // 5 min timeout, 60 sec interval
//! ```

use crate::infrastructure::database::Database;
use log::{error, info, warn};
use rusqlite::OptionalExtension;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::interval; // For .optional() method

/// Session Cleanup Configuration
#[derive(Clone)]
pub struct SessionCleanupConfig {
    /// Session timeout in seconds (default: 300 = 5 minutes)
    /// Sessions without heartbeat for this long are considered stale
    pub session_timeout_secs: u64,

    /// Cleanup interval in seconds (default: 60)
    /// How often to run cleanup job
    pub cleanup_interval_secs: u64,

    /// Enable heartbeat requirement
    /// If true, agents must send heartbeats to stay active
    pub require_heartbeat: bool,

    /// Auto-end sessions flag
    /// If true, stale sessions are automatically ended (not just deleted)
    pub auto_end_sessions: bool,
}

impl Default for SessionCleanupConfig {
    fn default() -> Self {
        Self {
            session_timeout_secs: 300, // 5 minutes
            cleanup_interval_secs: 60, // 1 minute
            require_heartbeat: true,
            auto_end_sessions: true,
        }
    }
}

/// Session Cleanup Job Manager
pub struct SessionCleanupJob {
    db: Arc<Database>,
    config: SessionCleanupConfig,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl SessionCleanupJob {
    /// Create a new session cleanup job
    pub fn new(db: Arc<Database>, config: SessionCleanupConfig) -> Self {
        Self {
            db,
            config,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    /// Start the cleanup job in background
    pub fn start(&self) {
        let running = self.running.clone();
        running.store(true, std::sync::atomic::Ordering::SeqCst);

        let db = self.db.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(config.cleanup_interval_secs));

            info!(
                "[SessionCleanup] Started: timeout={}s, interval={}s",
                config.session_timeout_secs, config.cleanup_interval_secs
            );

            loop {
                interval.tick().await;

                if !running.load(std::sync::atomic::Ordering::SeqCst) {
                    info!("[SessionCleanup] Stopped");
                    break;
                }

                match cleanup_stale_sessions(&db, &config).await {
                    Ok(stats) => {
                        if stats.cleaned > 0 {
                            info!(
                                "[SessionCleanup] Cleaned {} sessions, {} tasks, {} locks",
                                stats.cleaned, stats.tasks_cancelled, stats.locks_released
                            );
                        }
                    }
                    Err(e) => {
                        error!("[SessionCleanup] Error: {}", e);
                    }
                }
            }
        });
    }

    /// Stop the cleanup job
    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        info!("[SessionCleanup] Stop requested");
    }

    /// Check if the job is running
    pub fn is_running(&self) -> bool {
        self.running.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Run cleanup once (for manual execution)
    pub async fn run_once(&self) -> Result<CleanupStats, String> {
        cleanup_stale_sessions(&self.db, &self.config)
            .await
            .map_err(|e| e.to_string())
    }
}

/// Statistics from cleanup operation
#[derive(Debug, Clone, Default)]
pub struct CleanupStats {
    pub cleaned: usize,
    pub tasks_cancelled: usize,
    pub locks_released: usize,
    pub contexts_archived: usize,
}

/// Perform cleanup of stale sessions
async fn cleanup_stale_sessions(
    db: &Arc<Database>,
    config: &SessionCleanupConfig,
) -> Result<CleanupStats, Box<dyn std::error::Error + Send + Sync>> {
    let conn = db.get_conn();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let threshold = now - config.session_timeout_secs as i64;

    let mut stats = CleanupStats::default();

    if config.require_heartbeat {
        // Find sessions without recent heartbeat
        let mut stmt = conn.prepare(
            "SELECT session_id, agent_type, last_heartbeat 
             FROM agent_sessions 
             WHERE is_active = 1 
             AND (last_heartbeat IS NULL OR last_heartbeat < ?1)",
        )?;

        let stale_sessions: Vec<(String, String, Option<i64>)> = stmt
            .query_map([threshold], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .filter_map(|r| r.ok())
            .collect();

        for (session_id, agent_type, last_heartbeat) in stale_sessions {
            let heartbeat_info = last_heartbeat
                .map(|t| format!("{}s ago", now - t))
                .unwrap_or_else(|| "never".to_string());

            warn!(
                "[SessionCleanup] Stale agent detected: {} (type: {}, heartbeat: {})",
                session_id, agent_type, heartbeat_info
            );

            // Mark session as inactive
            conn.execute(
                "UPDATE agent_sessions SET is_active = 0 WHERE session_id = ?1",
                [session_id.clone()],
            )?;

            if config.auto_end_sessions {
                // End the session properly
                conn.execute(
                    "UPDATE sessions SET ended_at = ?1 WHERE id = ?2 AND ended_at IS NULL",
                    rusqlite::params![now, &session_id],
                )?;
            }

            stats.cleaned += 1;
        }
    }

    // Cancel pending tasks for stale sessions
    let tasks_cancelled = conn.execute(
        "UPDATE task_queue 
         SET status = 'cancelled', 
             completed_at = ?1,
             error = 'Session terminated: agent unresponsive'
         WHERE status IN ('pending', 'running')
         AND agent_session_id IN (
             SELECT session_id FROM agent_sessions 
             WHERE is_active = 0 
             AND last_heartbeat < ?2
         )",
        [now, threshold],
    )?;
    stats.tasks_cancelled = tasks_cancelled;

    // Release locks held by stale sessions
    let locks_released = conn.execute(
        "DELETE FROM active_locks 
         WHERE expires_at < ?1 
         OR agent_session_id IN (
             SELECT session_id FROM agent_sessions 
             WHERE is_active = 0
         )",
        [now],
    )?;
    stats.locks_released = locks_released;

    // Archive old contexts (optional - can be configured)
    // This moves old context data to cold storage

    Ok(stats)
}

/// Update heartbeat for a session
pub fn update_heartbeat(db: &Arc<Database>, session_id: &str) -> Result<(), String> {
    let conn = db.get_conn();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    conn.execute(
        "UPDATE agent_sessions
         SET last_heartbeat = ?1, is_active = 1
         WHERE session_id = ?2",
        rusqlite::params![now, session_id],
    )
    .map_err(|e| format!("Failed to update heartbeat: {}", e))?;

    Ok(())
}

/// Check if a session is still active (not stale)
pub fn is_session_active(
    db: &Arc<Database>,
    session_id: &str,
    timeout_secs: u64,
) -> Result<bool, String> {
    let conn = db.get_conn();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;
    let threshold = now - timeout_secs as i64;

    let result: Option<bool> = conn
        .query_row(
            "SELECT is_active FROM agent_sessions
             WHERE session_id = ?1
             AND (last_heartbeat IS NULL OR last_heartbeat >= ?2)",
            rusqlite::params![session_id, threshold],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| format!("Failed to check session status: {}", e))?;

    Ok(result.unwrap_or(false))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = SessionCleanupConfig::default();
        assert_eq!(config.session_timeout_secs, 300);
        assert_eq!(config.cleanup_interval_secs, 60);
        assert!(config.require_heartbeat);
        assert!(config.auto_end_sessions);
    }

    #[test]
    fn test_cleanup_stats_default() {
        let stats = CleanupStats::default();
        assert_eq!(stats.cleaned, 0);
        assert_eq!(stats.tasks_cancelled, 0);
        assert_eq!(stats.locks_released, 0);
    }
}
