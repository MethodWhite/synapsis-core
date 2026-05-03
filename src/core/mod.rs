//! Synapsis Core Module

pub mod agent;
pub mod antibrick;
pub mod auth;
pub mod auto_integrate;
pub mod crypto_plugin;
pub mod crypto_provider;
pub mod pqcrypto_provider;

pub mod discovery;
pub mod discovery_net;
pub mod orchestrator;
pub mod passive_capture;
pub mod pqc;
pub mod rate_limiter;
pub mod recycle;
pub mod resource_manager;
pub mod retry;
pub mod security;
pub mod session_cleanup;
pub mod sync;
pub mod task_queue;
pub mod tool_registry;
pub mod uuid;
#[cfg(feature = "security")]
pub mod vault;
pub mod watchdog;
pub mod worker; // NEW: Session cleanup job

pub use agent::*;
pub use auth::*;
pub use auto_integrate::*;
pub use crypto_plugin::*;
pub use crypto_provider::*;
pub use pqcrypto_provider::*;

pub use discovery::*;
pub use orchestrator::{
    Agent, AgentStatus, MessageType, Orchestrator, OrchestratorMessage, Task as OrchestratorTask,
    TaskStatus as OrchestratorTaskStatus,
};
pub use passive_capture::*;
pub use pqc::*;
pub use rate_limiter::*;
pub use recycle::*;
pub use retry::*;
pub use security::*;
pub use sync::*;
pub use task_queue::*;
pub use tool_registry::*;
pub use uuid::*;
#[cfg(feature = "security")]
pub use vault::*;
pub use worker::{
    CodeWorker, FileWorker, GitWorker, OpenCodeConnector, QwenConnector, SearchWorker, ShellWorker,
    Task as WorkerTask, TaskStatus as WorkerTaskStatus, WorkerAgent, WorkerRegistry,
};
pub mod agent_registry_ext;
pub mod audit_log;
pub mod chunk_query;
pub mod providers;
pub mod session_context;
pub mod session_id;
pub mod session_manager;
pub mod task_cleanup;
pub mod terminal_writer;
pub mod timeline_manager;
pub mod zero_trust;

pub mod json_rpc;
pub mod mcp_autoconfig;
pub mod port_pid_protection;
pub mod rpc_handlers;
pub mod rpc_methods;
