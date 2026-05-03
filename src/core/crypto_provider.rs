//! Cryptography Provider Implementations
//!
//! Provides adapter implementations of CryptoProvider trait for existing
//! PQC implementations in the MethodWhite ecosystem.

use crate::domain::crypto::{CryptoProvider, PqcAlgorithm};
use crate::domain::Result;

/// Adapter for the built-in PQC implementation in `synapsis::core::pqc`
pub struct SynapsisPqcProvider;

impl SynapsisPqcProvider {
    pub fn new() -> Self {
        Self
    }
}

impl CryptoProvider for SynapsisPqcProvider {
    fn id(&self) -> &str {
        "synapsis-pqc"
    }

    fn name(&self) -> &str {
        "Synapsis Built-in PQC"
    }

    fn description(&self) -> &str {
        "Built-in PQC implementation using pqcrypto crate (Kyber512, Dilithium5, AES-256-GCM)"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn supported_algorithms(&self) -> Vec<PqcAlgorithm> {
        vec![
            PqcAlgorithm::Kyber512,
            PqcAlgorithm::Dilithium5,
            PqcAlgorithm::Aes256Gcm,
        ]
    }

    fn generate_keypair(&self, algorithm: PqcAlgorithm) -> Result<(Vec<u8>, Vec<u8>)> {
        use PqcAlgorithm::*;
        match algorithm {
            Kyber512 => {
                use crate::core::pqc::generate_kyber_keypair;
                generate_kyber_keypair()
                    .map_err(|e| crate::domain::errors::SynapsisError::internal_bug(e))
            }
            Dilithium5 => {
                use crate::core::pqc::generate_dilithium_keypair;
                generate_dilithium_keypair()
                    .map_err(|e| crate::domain::errors::SynapsisError::internal_bug(e))
            }
            _ => Err(crate::domain::errors::SynapsisError::crypto_pqc_not_supported()),
        }
    }

    fn encapsulate(
        &self,
        public_key: &[u8],
        algorithm: PqcAlgorithm,
    ) -> Result<(Vec<u8>, Vec<u8>)> {
        use PqcAlgorithm::*;
        match algorithm {
            Kyber512 => {
                use crate::core::pqc::kyber_encapsulate;
                kyber_encapsulate(public_key)
                    .map_err(|e| crate::domain::errors::SynapsisError::internal_bug(e))
            }
            _ => Err(crate::domain::errors::SynapsisError::crypto_pqc_not_supported()),
        }
    }

    fn decapsulate(
        &self,
        ciphertext: &[u8],
        secret_key: &[u8],
        algorithm: PqcAlgorithm,
    ) -> Result<Vec<u8>> {
        use PqcAlgorithm::*;
        match algorithm {
            Kyber512 => {
                use crate::core::pqc::kyber_decapsulate;
                kyber_decapsulate(ciphertext, secret_key)
                    .map_err(|e| crate::domain::errors::SynapsisError::internal_bug(e))
            }
            _ => Err(crate::domain::errors::SynapsisError::crypto_pqc_not_supported()),
        }
    }

    fn sign(&self, secret_key: &[u8], message: &[u8], algorithm: PqcAlgorithm) -> Result<Vec<u8>> {
        use PqcAlgorithm::*;
        match algorithm {
            Dilithium5 => {
                use crate::core::pqc::dilithium_sign;
                dilithium_sign(secret_key, message)
                    .map_err(|e| crate::domain::errors::SynapsisError::internal_bug(e))
            }
            _ => Err(crate::domain::errors::SynapsisError::crypto_pqc_not_supported()),
        }
    }

    fn verify(
        &self,
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
        algorithm: PqcAlgorithm,
    ) -> Result<bool> {
        use PqcAlgorithm::*;
        match algorithm {
            Dilithium5 => {
                use crate::core::pqc::dilithium_verify;
                Ok(dilithium_verify(public_key, message, signature))
            }
            _ => Err(crate::domain::errors::SynapsisError::crypto_pqc_not_supported()),
        }
    }

    fn encrypt(&self, key: &[u8], plaintext: &[u8], algorithm: PqcAlgorithm) -> Result<Vec<u8>> {
        use PqcAlgorithm::*;
        match algorithm {
            Aes256Gcm => {
                use crate::core::pqc::encrypt;
                if key.len() != 32 {
                    return Err(crate::domain::errors::SynapsisError::crypto_cipher());
                }
                let mut key_array = [0u8; 32];
                key_array.copy_from_slice(key);
                encrypt(plaintext, &key_array)
                    .map_err(|e| crate::domain::errors::SynapsisError::internal_bug(e))
            }
            _ => Err(crate::domain::errors::SynapsisError::crypto_pqc_not_supported()),
        }
    }

    fn decrypt(&self, key: &[u8], ciphertext: &[u8], algorithm: PqcAlgorithm) -> Result<Vec<u8>> {
        use PqcAlgorithm::*;
        match algorithm {
            Aes256Gcm => {
                use crate::core::pqc::decrypt;
                if key.len() != 32 {
                    return Err(crate::domain::errors::SynapsisError::crypto_cipher());
                }
                let mut key_array = [0u8; 32];
                key_array.copy_from_slice(key);
                decrypt(ciphertext, &key_array)
                    .map_err(|e| crate::domain::errors::SynapsisError::internal_bug(e))
            }
            _ => Err(crate::domain::errors::SynapsisError::crypto_pqc_not_supported()),
        }
    }

    fn random_bytes(&self, length: usize) -> Result<Vec<u8>> {
        use crate::core::security::SecureRng;
        let mut bytes = vec![0u8; length];
        SecureRng::fill_random(&mut bytes);
        Ok(bytes)
    }

    fn derive_key_from_password(&self, password: &str, salt: &[u8]) -> Result<Vec<u8>> {
        // For now, use a simple SHA256 derivation
        // TODO: Replace with Argon2 or similar KDF
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        Ok(hasher.finalize().to_vec())
    }
}
