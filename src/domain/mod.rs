//! Domain layer: entities, types, ports, and error types.
//!
//! - **entities** — Core data structures: `Observation`, `SearchParams`, `Entity`, `Relation`, etc.
//! - **types** — Primitive types: `ObservationId`, `SessionId`, `Timestamp`, `ObservationType`
//! - **ports** — Trait definitions: `StorageBackend`, `StoragePort`, `MemoryPort`
//! - **models** — Agent and task models for orchestration
//! - **crypto** — Cryptographic primitives and PQC algorithm definitions
//! - **errors** — Domain-level error types
//! - **plugin** — Plugin trait definitions

pub mod crypto;
pub mod entities;
pub mod errors;
pub mod models;
pub mod plugin;
pub mod ports;
pub mod types;
