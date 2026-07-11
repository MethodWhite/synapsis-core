//! Post-quantum cryptography implementations.
//!
//! - **mlkem** — ML-KEM (FIPS 203) key encapsulation, formerly Kyber
//! - **mldsa** — ML-DSA (FIPS 204) digital signatures, formerly Dilithium
//!
//! Also provides AES-256-GCM symmetric encryption as a fallback.

use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;

pub mod mldsa;
pub mod mlkem;

/// Encrypt data with AES-256-GCM using a key derived from SYNAPSIS_DB_KEY or a fixed app key.
/// The key is deterministic — same input always produces the same key.
pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Key error: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::try_from(nonce_bytes.as_ref())
        .expect("nonce is exactly 12 bytes");

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| format!("Encryption error: {}", e))?;

    let mut result = Vec::new();
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

/// Decrypt data with AES-256-GCM
pub fn decrypt(ciphertext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    if ciphertext.len() < 12 {
        return Err("Ciphertext too short".to_string());
    }

    let nonce_bytes = &ciphertext[..12];
    let data = &ciphertext[12..];

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Key error: {}", e))?;

    let nonce = Nonce::try_from(nonce_bytes)
        .map_err(|_| "Invalid nonce length".to_string())?;

    let plaintext = cipher
        .decrypt(&nonce, data)
        .map_err(|e| format!("Decryption error: {}", e))?;

    Ok(plaintext)
}

/// Generate a random key
pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = generate_key();
        let plaintext = b"Hello, World!";

        let ciphertext = encrypt(plaintext, &key).unwrap();
        let decrypted = decrypt(&ciphertext, &key).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }
}
