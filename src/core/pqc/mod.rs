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

#[cfg(feature = "pqc")]
pub mod mldsa;
#[cfg(feature = "pqc")]
pub mod mlkem;

/// Encrypt data with AES-256-GCM
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

/// Hybrid PQC encrypt: ML-KEM-1024 key exchange + AES-256-GCM data encryption.
///
/// Format: `[kem_ciphertext][aes_nonce][aes_ciphertext]`
#[cfg(feature = "pqc")]
pub fn pqc_encrypt(plaintext: &[u8], recipient_pk: &[u8]) -> Result<Vec<u8>, String> {
    let (shared_secret, kem_ct) = mlkem::MlKem::encapsulate(recipient_pk);

    let key: [u8; 32] = shared_secret.as_ref().try_into()
        .map_err(|_| "Invalid shared secret length".to_string())?;

    let aes_ct = encrypt(plaintext, &key)?;

    let mut result = Vec::new();
    result.extend_from_slice(kem_ct.as_bytes());
    result.extend_from_slice(&aes_ct);

    Ok(result)
}

/// Hybrid PQC decrypt: ML-KEM-1024 key decapsulation + AES-256-GCM data decryption.
#[cfg(feature = "pqc")]
pub fn pqc_decrypt(ciphertext: &[u8], recipient_sk: &[u8]) -> Result<Vec<u8>, String> {
    let ct_len = mlkem::MlKem::ciphertext_bytes();
    if ciphertext.len() < ct_len + 12 {
        return Err("Ciphertext too short for PQC decrypt".to_string());
    }

    let kem_ct = &ciphertext[..ct_len];
    let aes_ct = &ciphertext[ct_len..];

    let shared_secret = mlkem::MlKem::decapsulate(kem_ct, recipient_sk);

    let key: [u8; 32] = shared_secret.as_ref().try_into()
        .map_err(|_| "Invalid shared secret length".to_string())?;

    decrypt(aes_ct, &key)
}

/// Generate a PQC keypair (ML-KEM-1024)
#[cfg(feature = "pqc")]
pub fn pqc_generate_keypair() -> (Vec<u8>, Vec<u8>) {
    let (sk, pk) = mlkem::MlKem::keypair();
    (sk.as_bytes().to_vec(), pk.as_bytes().to_vec())
}

/// Sign a message using ML-DSA-87
#[cfg(feature = "pqc")]
pub fn pqc_sign(msg: &[u8], sk: &[u8]) -> Result<Vec<u8>, String> {
    Ok(mldsa::MlDsa::sign(msg, sk).as_bytes().to_vec())
}

/// Verify a signature using ML-DSA-87
#[cfg(feature = "pqc")]
pub fn pqc_verify(msg: &[u8], sig: &[u8], pk: &[u8]) -> bool {
    mldsa::MlDsa::verify(msg, sig, pk)
}

/// Generate a signing keypair (ML-DSA-87)
#[cfg(feature = "pqc")]
pub fn pqc_generate_signing_keypair() -> (Vec<u8>, Vec<u8>) {
    let (sk, pk) = mldsa::MlDsa::keypair();
    (sk.as_bytes().to_vec(), pk.as_bytes().to_vec())
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

    #[cfg(feature = "pqc")]
    #[test]
    fn test_pqc_encrypt_decrypt() {
        let (sk, pk) = pqc_generate_keypair();
        let plaintext = b"Post-quantum secret message";

        let ciphertext = pqc_encrypt(plaintext, &pk).unwrap();
        let decrypted = pqc_decrypt(&ciphertext, &sk).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }

    #[cfg(feature = "pqc")]
    #[test]
    fn test_pqc_sign_verify() {
        let (sk, pk) = pqc_generate_signing_keypair();
        let msg = b"Important document";

        let sig = pqc_sign(msg, &sk).unwrap();
        assert!(pqc_verify(msg, &sig, &pk));
        assert!(!pqc_verify(b"tampered", &sig, &pk));
    }
}
