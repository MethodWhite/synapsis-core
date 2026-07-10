//! Infrastructure/persistence layer.
//!
//! - **database** — SQLite + SQLCipher storage with FTS5, embeddings, knowledge graph
//! - **optimizer** — Token budget auto-tuning and observation retention
//! - **plugin** — Plugin system wrapper
//! - **agents** — Agent registry (stub)
//! - **skills** — Skills registry (stub)

pub mod agents;
pub mod database;
pub mod optimizer;
pub mod plugin;
pub mod skills;
