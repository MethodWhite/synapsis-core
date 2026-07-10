//! # Synapsis Core
//!
//! Core library for the Synapsis persistent memory engine.
//! Provides domain types, storage abstractions, and infrastructure
//! for building AI agent memory systems.
//!
//! ## Architecture
//!
//! - **domain** — Domain entities, types, ports (interfaces), crypto, and error types
//! - **core** — Business logic: anti-brick, audit, crypto provider, orchestrator, PQC, session, watchdog, zero-trust
//! - **infrastructure** — SQLite database, plugin system, agent registry, skills, auto-optimizer

pub use domain::ports::DbValue;

pub mod core;
pub mod domain;
pub mod infrastructure;
