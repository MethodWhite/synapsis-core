//! Synapsis Domain Module
//!
//! Módulo raíz del dominio.

pub mod crypto;
pub mod entities;
pub mod errors;
pub mod plugin;
pub mod plugin_loader;
pub mod ports;
pub mod provider;
pub mod types;

pub use crypto::*;
pub use entities::*;
pub use errors::{ErrorKind, Result, SynapsisError};
pub use plugin::*;
pub use plugin_loader::*;
pub use ports::*;
pub use types::*;
