//! Synapsis Authentication Module
//!
//! Provides intelligent agent authentication, classification, and permission management.
//!
//! # Components
//!
//! - [`permissions`] - Permission system and trust levels
//! - [`classifier`] - Agent classification based on device and connection type
//! - [`tpm`] - TPM and MFA integration
//! - [`challenge`] - Challenge-response authentication

pub mod challenge;
pub mod classifier;
pub mod permissions;
pub mod tpm;

pub use challenge::*;
pub use classifier::*;
pub use permissions::*;
pub use tpm::*;
