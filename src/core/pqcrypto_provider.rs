//! Comprehensive PQC Provider using pqcrypto crate
//!
//! This provider supports all PQC algorithms using the pqcrypto crate:
//! - Kyber-512, Kyber-768, Kyber-1024 (KEM)
//! - Dilithium-2, Dilithium-3, Dilithium-5 (Signatures)
//! - AES-256-GCM (Symmetric encryption)

use crate::domain::crypto::{CryptoProvider, CryptoProviderInfo, PqcAlgorithm};
use crate::domain::Result;

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use pqcrypto_traits::kem::{Ciphertext, PublicKey, SecretKey, SharedSecret};
use pqcrypto_traits::sign::{
    DetachedSignature, PublicKey as SignPublicKey, SecretKey as SignSecretKey,
};

use super::security::SecureRng;

/// Comprehensive PQC provider using pqcrypto crate
pub struct PqcryptoProvider {
    info: CryptoProviderInfo,
}

impl PqcryptoProvider {
    pub fn new() -> Self {
        Self {
            info: CryptoProviderInfo {
                id: "pqcrypto".to_string(),
                name: "PQCrypto Crate Provider".to_string(),
                description: "Comprehensive PQC provider using pqcrypto crate (all Kyber and Dilithium variants)".to_string(),
                version: "1.0.0".to_string(),
                supported_algorithms: vec![
                    PqcAlgorithm::Kyber512,
                    PqcAlgorithm::Kyber768,
                    PqcAlgorithm::Kyber1024,
                    PqcAlgorithm::Dilithium2,
                    PqcAlgorithm::Dilithium3,
                    PqcAlgorithm::Dilithium5,
                    PqcAlgorithm::Aes256Gcm,
                ],
            },
        }
    }
}

impl Default for PqcryptoProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl CryptoProvider for PqcryptoProvider {
    fn id(&self) -> &str {
        "pqcrypto"
    }

    fn name(&self) -> &str {
        "PQCrypto Crate Provider"
    }

    fn description(&self) -> &str {
        "Comprehensive PQC provider using pqcrypto crate"
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn supported_algorithms(&self) -> Vec<PqcAlgorithm> {
        self.info.supported_algorithms.clone()
    }

    fn generate_keypair(&self, algorithm: PqcAlgorithm) -> Result<(Vec<u8>, Vec<u8>)> {
        match algorithm {
            // Kyber variants for KEM
            PqcAlgorithm::Kyber512 => {
                use pqcrypto_kyber::kyber512;
                let (pk, sk) = kyber512::keypair();
                Ok((pk.as_bytes().to_vec(), sk.as_bytes().to_vec()))
            }
            PqcAlgorithm::Kyber768 => {
                use pqcrypto_kyber::kyber768;
                let (pk, sk) = kyber768::keypair();
                Ok((pk.as_bytes().to_vec(), sk.as_bytes().to_vec()))
            }
            PqcAlgorithm::Kyber1024 => {
                use pqcrypto_kyber::kyber1024;
                let (pk, sk) = kyber1024::keypair();
                Ok((pk.as_bytes().to_vec(), sk.as_bytes().to_vec()))
            }
            // Dilithium variants for signatures
            PqcAlgorithm::Dilithium2 => {
                use pqcrypto_dilithium::dilithium2;
                let (pk, sk) = dilithium2::keypair();
                Ok((pk.as_bytes().to_vec(), sk.as_bytes().to_vec()))
            }
            PqcAlgorithm::Dilithium3 => {
                use pqcrypto_dilithium::dilithium3;
                let (pk, sk) = dilithium3::keypair();
                Ok((pk.as_bytes().to_vec(), sk.as_bytes().to_vec()))
            }
            PqcAlgorithm::Dilithium5 => {
                use pqcrypto_dilithium::dilithium5;
                let (pk, sk) = dilithium5::keypair();
                Ok((pk.as_bytes().to_vec(), sk.as_bytes().to_vec()))
            }
            _ => Err(crate::domain::errors::SynapsisError::crypto_pqc_not_supported()),
        }
    }

    fn encapsulate(
        &self,
        public_key: &[u8],
        algorithm: PqcAlgorithm,
    ) -> Result<(Vec<u8>, Vec<u8>)> {
        match algorithm {
            PqcAlgorithm::Kyber512 => {
                use pqcrypto_kyber::kyber512;
                let pk = kyber512::PublicKey::from_bytes(public_key)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let (ss, ct) = kyber512::encapsulate(&pk);
                Ok((ct.as_bytes().to_vec(), ss.as_bytes().to_vec()))
            }
            PqcAlgorithm::Kyber768 => {
                use pqcrypto_kyber::kyber768;
                let pk = kyber768::PublicKey::from_bytes(public_key)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let (ss, ct) = kyber768::encapsulate(&pk);
                Ok((ct.as_bytes().to_vec(), ss.as_bytes().to_vec()))
            }
            PqcAlgorithm::Kyber1024 => {
                use pqcrypto_kyber::kyber1024;
                let pk = kyber1024::PublicKey::from_bytes(public_key)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let (ss, ct) = kyber1024::encapsulate(&pk);
                Ok((ct.as_bytes().to_vec(), ss.as_bytes().to_vec()))
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
        match algorithm {
            PqcAlgorithm::Kyber512 => {
                use pqcrypto_kyber::kyber512;
                let sk = kyber512::SecretKey::from_bytes(secret_key)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let ct = kyber512::Ciphertext::from_bytes(ciphertext)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let ss = kyber512::decapsulate(&ct, &sk);
                Ok(ss.as_bytes().to_vec())
            }
            PqcAlgorithm::Kyber768 => {
                use pqcrypto_kyber::kyber768;
                let sk = kyber768::SecretKey::from_bytes(secret_key)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let ct = kyber768::Ciphertext::from_bytes(ciphertext)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let ss = kyber768::decapsulate(&ct, &sk);
                Ok(ss.as_bytes().to_vec())
            }
            PqcAlgorithm::Kyber1024 => {
                use pqcrypto_kyber::kyber1024;
                let sk = kyber1024::SecretKey::from_bytes(secret_key)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let ct = kyber1024::Ciphertext::from_bytes(ciphertext)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let ss = kyber1024::decapsulate(&ct, &sk);
                Ok(ss.as_bytes().to_vec())
            }
            _ => Err(crate::domain::errors::SynapsisError::crypto_pqc_not_supported()),
        }
    }

    fn sign(&self, secret_key: &[u8], message: &[u8], algorithm: PqcAlgorithm) -> Result<Vec<u8>> {
        match algorithm {
            PqcAlgorithm::Dilithium2 => {
                use pqcrypto_dilithium::dilithium2;
                let sk = dilithium2::SecretKey::from_bytes(secret_key)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let sig = dilithium2::detached_sign(message, &sk);
                Ok(sig.as_bytes().to_vec())
            }
            PqcAlgorithm::Dilithium3 => {
                use pqcrypto_dilithium::dilithium3;
                let sk = dilithium3::SecretKey::from_bytes(secret_key)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let sig = dilithium3::detached_sign(message, &sk);
                Ok(sig.as_bytes().to_vec())
            }
            PqcAlgorithm::Dilithium5 => {
                use pqcrypto_dilithium::dilithium5;
                let sk = dilithium5::SecretKey::from_bytes(secret_key)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let sig = dilithium5::detached_sign(message, &sk);
                Ok(sig.as_bytes().to_vec())
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
        match algorithm {
            PqcAlgorithm::Dilithium2 => {
                use pqcrypto_dilithium::dilithium2;
                let pk = dilithium2::PublicKey::from_bytes(public_key)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let sig = dilithium2::DetachedSignature::from_bytes(signature)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                Ok(dilithium2::verify_detached_signature(&sig, message, &pk).is_ok())
            }
            PqcAlgorithm::Dilithium3 => {
                use pqcrypto_dilithium::dilithium3;
                let pk = dilithium3::PublicKey::from_bytes(public_key)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let sig = dilithium3::DetachedSignature::from_bytes(signature)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                Ok(dilithium3::verify_detached_signature(&sig, message, &pk).is_ok())
            }
            PqcAlgorithm::Dilithium5 => {
                use pqcrypto_dilithium::dilithium5;
                let pk = dilithium5::PublicKey::from_bytes(public_key)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                let sig = dilithium5::DetachedSignature::from_bytes(signature)
                    .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;
                Ok(dilithium5::verify_detached_signature(&sig, message, &pk).is_ok())
            }
            _ => Err(crate::domain::errors::SynapsisError::crypto_pqc_not_supported()),
        }
    }

    fn encrypt(&self, key: &[u8], plaintext: &[u8], _algorithm: PqcAlgorithm) -> Result<Vec<u8>> {
        if key.len() != 32 {
            return Err(crate::domain::errors::SynapsisError::crypto_cipher());
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| crate::domain::errors::SynapsisError::crypto_cipher_msg(e.to_string()))?;

        let mut nonce_bytes = [0u8; 12];
        SecureRng::fill_random(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| crate::domain::errors::SynapsisError::crypto_cipher_msg(e.to_string()))?;

        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    fn decrypt(&self, key: &[u8], ciphertext: &[u8], _algorithm: PqcAlgorithm) -> Result<Vec<u8>> {
        if key.len() != 32 {
            return Err(crate::domain::errors::SynapsisError::crypto_cipher());
        }

        if ciphertext.len() < 12 {
            return Err(crate::domain::errors::SynapsisError::crypto_cipher());
        }

        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| crate::domain::errors::SynapsisError::crypto_cipher_msg(e.to_string()))?;

        let nonce = Nonce::from_slice(&ciphertext[..12]);
        let data = &ciphertext[12..];

        let plaintext = cipher
            .decrypt(nonce, data)
            .map_err(|e| crate::domain::errors::SynapsisError::crypto_cipher_msg(e.to_string()))?;

        Ok(plaintext)
    }

    fn random_bytes(&self, length: usize) -> Result<Vec<u8>> {
        let mut bytes = vec![0u8; length];
        SecureRng::fill_random(&mut bytes);
        Ok(bytes)
    }

    fn derive_key_from_password(&self, password: &str, salt: &[u8]) -> Result<Vec<u8>> {
        use argon2::password_hash::SaltString;
        use argon2::{Argon2, PasswordHasher};

        // Convert salt to base64 string for SaltString (without padding)
        let salt_b64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, salt);
        let salt_str = salt_b64.trim_end_matches('=');

        // Create SaltString from base64 (without padding)
        let salt = SaltString::from_b64(salt_str).map_err(|e| {
            crate::domain::errors::SynapsisError::crypto_pqc(format!("Invalid salt: {}", e))
        })?;

        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| crate::domain::errors::SynapsisError::crypto_pqc(e.to_string()))?;

        Ok(hash.hash.unwrap().as_bytes().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_info() {
        let provider = PqcryptoProvider::new();
        assert_eq!(provider.id(), "pqcrypto");
        assert_eq!(provider.name(), "PQCrypto Crate Provider");
        assert!(provider.supports_algorithm(PqcAlgorithm::Kyber512));
        assert!(provider.supports_algorithm(PqcAlgorithm::Kyber768));
        assert!(provider.supports_algorithm(PqcAlgorithm::Kyber1024));
        assert!(provider.supports_algorithm(PqcAlgorithm::Dilithium2));
        assert!(provider.supports_algorithm(PqcAlgorithm::Dilithium3));
        assert!(provider.supports_algorithm(PqcAlgorithm::Dilithium5));
        assert!(provider.supports_algorithm(PqcAlgorithm::Aes256Gcm));
    }

    #[test]
    fn test_kyber512_keypair() {
        let provider = PqcryptoProvider::new();
        let (pk, sk) = provider.generate_keypair(PqcAlgorithm::Kyber512).unwrap();
        assert!(!pk.is_empty());
        assert!(!sk.is_empty());
        assert_ne!(pk, sk);
    }

    #[test]
    fn test_kyber768_keypair() {
        let provider = PqcryptoProvider::new();
        let (pk, sk) = provider.generate_keypair(PqcAlgorithm::Kyber768).unwrap();
        assert!(!pk.is_empty());
        assert!(!sk.is_empty());
        assert_ne!(pk, sk);
    }

    #[test]
    fn test_kyber512_encapsulate_decapsulate() {
        let provider = PqcryptoProvider::new();

        // Generate keypair
        let (pk, sk) = provider.generate_keypair(PqcAlgorithm::Kyber512).unwrap();

        // Encapsulate
        let (ct, ss1) = provider.encapsulate(&pk, PqcAlgorithm::Kyber512).unwrap();
        assert!(!ct.is_empty());
        assert!(!ss1.is_empty());

        // Decapsulate
        let ss2 = provider
            .decapsulate(&ct, &sk, PqcAlgorithm::Kyber512)
            .unwrap();

        // Shared secrets should match
        assert_eq!(ss1, ss2);
    }

    #[test]
    fn test_kyber768_encapsulate_decapsulate() {
        let provider = PqcryptoProvider::new();

        // Generate keypair
        let (pk, sk) = provider.generate_keypair(PqcAlgorithm::Kyber768).unwrap();

        // Encapsulate
        let (ct, ss1) = provider.encapsulate(&pk, PqcAlgorithm::Kyber768).unwrap();
        assert!(!ct.is_empty());
        assert!(!ss1.is_empty());

        // Decapsulate
        let ss2 = provider
            .decapsulate(&ct, &sk, PqcAlgorithm::Kyber768)
            .unwrap();

        // Shared secrets should match
        assert_eq!(ss1, ss2);
    }

    #[test]
    fn test_dilithium5_sign_verify() {
        let provider = PqcryptoProvider::new();

        // Generate keypair
        let (pk, sk) = provider.generate_keypair(PqcAlgorithm::Dilithium5).unwrap();

        // Sign
        let message = b"Hello, Synapsis!";
        let signature = provider
            .sign(&sk, message, PqcAlgorithm::Dilithium5)
            .unwrap();
        assert!(!signature.is_empty());

        // Verify
        let valid = provider
            .verify(&pk, message, &signature, PqcAlgorithm::Dilithium5)
            .unwrap();
        assert!(valid);

        // Tampered message should fail
        let tampered = b"Hello, Synapsis?";
        let valid_tampered = provider
            .verify(&pk, tampered, &signature, PqcAlgorithm::Dilithium5)
            .unwrap();
        assert!(!valid_tampered);
    }

    #[test]
    fn test_dilithium3_sign_verify() {
        let provider = PqcryptoProvider::new();

        // Generate keypair
        let (pk, sk) = provider.generate_keypair(PqcAlgorithm::Dilithium3).unwrap();

        // Sign
        let message = b"Test message for Dilithium3";
        let signature = provider
            .sign(&sk, message, PqcAlgorithm::Dilithium3)
            .unwrap();
        assert!(!signature.is_empty());

        // Verify
        let valid = provider
            .verify(&pk, message, &signature, PqcAlgorithm::Dilithium3)
            .unwrap();
        assert!(valid);
    }

    #[test]
    fn test_aes_encrypt_decrypt() {
        let provider = PqcryptoProvider::new();

        // Generate random key
        let key = provider.random_bytes(32).unwrap();
        let plaintext = b"Hello, World!";

        // Encrypt
        let ciphertext = provider
            .encrypt(&key, plaintext, PqcAlgorithm::Aes256Gcm)
            .unwrap();
        assert!(!ciphertext.is_empty());
        assert_ne!(ciphertext[..], plaintext[..]);

        // Decrypt
        let decrypted = provider
            .decrypt(&key, &ciphertext, PqcAlgorithm::Aes256Gcm)
            .unwrap();

        // Should match original
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_random_bytes() {
        let provider = PqcryptoProvider::new();

        let random1 = provider.random_bytes(32).unwrap();
        let random2 = provider.random_bytes(32).unwrap();

        assert_eq!(random1.len(), 32);
        assert_eq!(random2.len(), 32);
        assert_ne!(random1, random2); // Should be different
    }

    #[test]
    fn test_derive_key_from_password() {
        let provider = PqcryptoProvider::new();

        let password = "my_secure_password";
        let salt = b"randomsalt123456"; // 16 bytes, sin padding de base64

        let key1 = provider.derive_key_from_password(password, salt).unwrap();
        let key2 = provider.derive_key_from_password(password, salt).unwrap();

        // Same password + salt should produce same key
        assert_eq!(key1, key2);
        assert!(!key1.is_empty());

        // Different salt should produce different key
        let key3 = provider
            .derive_key_from_password(password, b"diffsalt12345678")
            .unwrap();
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_unsupported_algorithm() {
        let provider = PqcryptoProvider::new();

        // Try to use an algorithm that doesn't exist
        let result = provider.generate_keypair(PqcAlgorithm::Aes256Gcm);
        assert!(result.is_err());
    }
}
