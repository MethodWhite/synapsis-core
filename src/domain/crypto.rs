//! Cryptography Provider Abstractions
//!
//! Defines the core traits for cryptography providers, enabling pluggable
//! post-quantum cryptography implementations.

use crate::domain::Result;

/// Algorithm variants for post-quantum cryptography
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PqcAlgorithm {
    /// Kyber-512 (ML-KEM-512)
    Kyber512,
    /// Kyber-768 (ML-KEM-768)
    Kyber768,
    /// Kyber-1024 (ML-KEM-1024)
    Kyber1024,
    /// Dilithium-2
    Dilithium2,
    /// Dilithium-3
    Dilithium3,
    /// Dilithium-5
    Dilithium5,
    /// AES-256-GCM (symmetric fallback)
    Aes256Gcm,
}

/// Information about a cryptography provider
#[derive(Debug, Clone)]
pub struct CryptoProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub supported_algorithms: Vec<PqcAlgorithm>,
    pub version: String,
}

/// Core trait for cryptography providers
pub trait CryptoProvider: Send + Sync {
    /// Unique identifier for this provider (e.g., "pqcrypto", "liboqs", "openssl")
    fn id(&self) -> &str;

    /// User-friendly name for this provider
    fn name(&self) -> &str;

    /// Description of the provider implementation
    fn description(&self) -> &str;

    /// Version string for the provider
    fn version(&self) -> &str;

    /// List algorithms supported by this provider
    fn supported_algorithms(&self) -> Vec<PqcAlgorithm>;

    /// Check if a specific algorithm is supported
    fn supports_algorithm(&self, algorithm: PqcAlgorithm) -> bool {
        self.supported_algorithms().contains(&algorithm)
    }

    /// Generate a keypair for the specified algorithm
    fn generate_keypair(&self, algorithm: PqcAlgorithm) -> Result<(Vec<u8>, Vec<u8>)>;

    /// Encapsulate a shared secret (for KEM algorithms like Kyber)
    fn encapsulate(&self, public_key: &[u8], algorithm: PqcAlgorithm)
        -> Result<(Vec<u8>, Vec<u8>)>;

    /// Decapsulate a shared secret (for KEM algorithms like Kyber)
    fn decapsulate(
        &self,
        ciphertext: &[u8],
        secret_key: &[u8],
        algorithm: PqcAlgorithm,
    ) -> Result<Vec<u8>>;

    /// Sign a message (for signature algorithms like Dilithium)
    fn sign(&self, secret_key: &[u8], message: &[u8], algorithm: PqcAlgorithm) -> Result<Vec<u8>>;

    /// Verify a signature (for signature algorithms like Dilithium)
    fn verify(
        &self,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
        algorithm: PqcAlgorithm,
    ) -> Result<bool>;

    /// Encrypt data with symmetric algorithm (like AES-256-GCM)
    fn encrypt(&self, key: &[u8], plaintext: &[u8], algorithm: PqcAlgorithm) -> Result<Vec<u8>>;

    /// Decrypt data with symmetric algorithm (like AES-256-GCM)
    fn decrypt(&self, key: &[u8], ciphertext: &[u8], algorithm: PqcAlgorithm) -> Result<Vec<u8>>;

    /// Generate random bytes of specified length
    fn random_bytes(&self, length: usize) -> Result<Vec<u8>>;

    /// Derive key from password using Argon2 or similar KDF
    fn derive_key_from_password(&self, password: &str, salt: &[u8]) -> Result<Vec<u8>>;
}

/// Registry of available cryptography providers
pub struct CryptoProviderRegistry {
    providers: std::collections::HashMap<String, std::sync::Arc<dyn CryptoProvider>>,
    default_provider: Option<String>,
}

impl CryptoProviderRegistry {
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
            default_provider: None,
        }
    }

    pub fn register(&mut self, provider: std::sync::Arc<dyn CryptoProvider>) {
        let id = provider.id().to_string();
        if self.default_provider.is_none() {
            self.default_provider = Some(id.clone());
        }
        self.providers.insert(id, provider);
    }

    pub fn get(&self, id: &str) -> Option<std::sync::Arc<dyn CryptoProvider>> {
        self.providers.get(id).cloned()
    }

    pub fn get_default(&self) -> Option<std::sync::Arc<dyn CryptoProvider>> {
        self.default_provider.as_ref().and_then(|id| self.get(id))
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub fn find_provider_for_algorithm(
        &self,
        algorithm: PqcAlgorithm,
    ) -> Option<std::sync::Arc<dyn CryptoProvider>> {
        self.providers
            .values()
            .find(|provider| provider.supports_algorithm(algorithm))
            .cloned()
    }
}

impl Default for CryptoProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}
