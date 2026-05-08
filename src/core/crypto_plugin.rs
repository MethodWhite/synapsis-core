//! Built-in Cryptography Plugin for Synapsis
//!
//! Provides a plugin wrapper for PQC providers.

use crate::core::crypto_provider::SynapsisPqcProvider;
use crate::core::pqcrypto_provider::PqcryptoProvider;
use crate::domain::plugin::{
    ExtensionPoint, PluginInfo, PluginLifecycle, PluginRegistry, SynapsisPlugin,
};
use crate::domain::Result;
use std::sync::Arc;

/// Built-in cryptography plugin with comprehensive PQC support
pub struct CryptoPlugin {
    info: PluginInfo,
    /// Primary provider with all PQC algorithms
    primary_provider: Arc<PqcryptoProvider>,
    /// Legacy provider for backward compatibility
    legacy_provider: Arc<SynapsisPqcProvider>,
}

impl Default for CryptoPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl CryptoPlugin {
    pub fn new() -> Self {
        Self {
            info: PluginInfo {
                id: "synapsis-crypto".to_string(),
                name: "Synapsis Cryptography Plugin".to_string(),
                description:
                    "Comprehensive PQC provider (Kyber-512/768/1024, Dilithium-2/3/5, AES-256-GCM)"
                        .to_string(),
                version: "2.0.0".to_string(),
                author: "MethodWhite".to_string(),
                license: "Apache-2.0".to_string(),
                extension_points: vec![ExtensionPoint::CryptoProvider],
                dependencies: vec![],
            },
            primary_provider: Arc::new(PqcryptoProvider::new()),
            legacy_provider: Arc::new(SynapsisPqcProvider::new()),
        }
    }

    /// Get the primary crypto provider (comprehensive PQC)
    pub fn get_primary_provider(&self) -> Arc<dyn crate::domain::crypto::CryptoProvider> {
        self.primary_provider.clone()
    }

    /// Get the legacy crypto provider (backward compatibility)
    pub fn get_legacy_provider(&self) -> Arc<dyn crate::domain::crypto::CryptoProvider> {
        self.legacy_provider.clone()
    }
}

impl SynapsisPlugin for CryptoPlugin {
    fn info(&self) -> PluginInfo {
        self.info.clone()
    }

    fn on_lifecycle(&self, lifecycle: PluginLifecycle) -> Result<()> {
        match lifecycle {
            PluginLifecycle::Load => {
                // log::info!("Loading cryptography plugin...");
            }
            PluginLifecycle::Initialize => {
                // log::info!("Initializing cryptography plugin...");
            }
            PluginLifecycle::Start => {
                // log::info!("Starting cryptography plugin...");
            }
            PluginLifecycle::Stop => {
                // log::info!("Stopping cryptography plugin...");
            }
            PluginLifecycle::Unload => {
                // log::info!("Unloading cryptography plugin...");
            }
        }
        Ok(())
    }

    fn extension_points(&self) -> Vec<ExtensionPoint> {
        self.info.extension_points.clone()
    }

    fn register_extensions(&self, registry: &mut PluginRegistry) -> Result<()> {
        // Register the primary crypto provider (comprehensive PQC)
        registry.register_extension(
            ExtensionPoint::CryptoProvider,
            self.primary_provider.clone(),
        );

        // Also register legacy provider for backward compatibility
        registry.register_extension(ExtensionPoint::CryptoProvider, self.legacy_provider.clone());

        Ok(())
    }
}
