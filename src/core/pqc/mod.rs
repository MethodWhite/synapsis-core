use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Key error: {}", e))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::try_from(nonce_bytes.as_ref()).expect("nonce is exactly 12 bytes");

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| format!("Encryption error: {}", e))?;

    let mut result = Vec::new();
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

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

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut key);
    key
}

#[cfg(feature = "pqc")]
pub use prusia_vault::pqc::{
    pqc_encrypt, pqc_decrypt, pqc_generate_keypair,
    pqc_sign, pqc_verify, pqc_generate_signing_keypair,
};

#[cfg(feature = "pqc")]
pub mod mlkem {
    pub use prusia_vault::pqc::mlkem::MlKem;
}

#[cfg(feature = "pqc")]
pub mod mldsa {
    pub use prusia_vault::pqc::mldsa::MlDsa;
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
