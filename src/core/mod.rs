//! Core business logic layer.
//!
//! Provides domain services and security primitives:
//! - **session_id** — Session ID generation and registry
//! - **session_cleanup** — Stale session expiry (stub)
//! - **crypto_provider** — Cryptographic provider abstraction
//! - **pqc** — Post-quantum cryptography (ML-KEM, ML-DSA)
//! - **antibrick** — Command injection guard (stub)
//! - **audit_log** — Audit trail logging (stub)
//! - **orchestrator** — Agent orchestration (stub)
//! - **rate_limiter** — Request rate limiting (stub)
//! - **watchdog** — Filesystem integrity monitoring (stub)
//! - **zero_trust** — Zero-trust authorization (stub)
//!
//! > **Note:** Modules marked `(stub)` are placeholders and return safe defaults.

pub mod antibrick;
pub mod audit_log;
pub mod crypto_provider;
pub mod orchestrator;
pub mod pqc;
pub mod rate_limiter;
pub mod session_cleanup;
pub mod session_id;
pub mod watchdog;
pub mod zero_trust;

// Re-export legacy types at core level
pub use crate::domain::crypto::PqcAlgorithm;
pub use crypto_provider::CryptoProvider;
pub use crypto_provider::PqcryptoProvider;
