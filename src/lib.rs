//! # Synapsis Core
//!
//! Core library for Synapsis - Persistent Memory Engine with PQC Security
//!
//! This crate provides the foundational types, traits, and business logic
//! for building applications on top of Synapsis.
//!
//! # Features
//!
//! - `security` - Enable security features (encryption, HMAC, etc.)
//! - `pqc` - Enable post-quantum cryptography (Kyber, Dilithium)
//! - `full` - Enable all features
//!
//! # Example
//!
//! ```rust
//! use synapsis_core::domain::{CryptoProvider, PqcAlgorithm};
//! use synapsis_core::core::PqcryptoProvider;
//!
//! let provider = PqcryptoProvider::new();
//! let (pk, sk) = provider.generate_keypair(PqcAlgorithm::Kyber768).unwrap();
//! ```

pub mod core;
pub mod domain;
pub mod infrastructure;

// Re-export commonly used types
pub use domain::{
    crypto::{CryptoProvider, CryptoProviderInfo, CryptoProviderRegistry, PqcAlgorithm},
    errors::{Error, ErrorKind, Result, SynapsisError},
    plugin::{ExtensionPoint, PluginInfo, PluginLifecycle, PluginRegistry, SynapsisPlugin},
    types::*,
};

pub use core::{
    crypto_plugin::CryptoPlugin, crypto_provider::SynapsisPqcProvider,
    pqcrypto_provider::PqcryptoProvider,
};

// 2026 Sovereign Intelligence: Reasoning and Learning module
// Note: materia_engine is a separate crate that depends on synapsis-core
// pub use materia_engine::{Materia, MateriaEvent};
