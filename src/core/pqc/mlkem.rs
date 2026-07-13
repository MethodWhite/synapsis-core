/// ML-KEM (FIPS 203) key-encapsulation mechanism (formerly Kyber).
///
/// Provides post-quantum secure key encapsulation for establishing
/// shared secrets between two parties. Uses **ML-KEM-1024** for
/// NIST security level 5 (AES-256 equivalent).
use core::fmt;
use pqcrypto_traits::kem::{Ciphertext as KemCiphertextTrait, PublicKey as KemPublicKeyTrait, SecretKey as KemSecretKeyTrait, SharedSecret as KemSharedSecretTrait};

#[derive(Clone)]
pub struct SecretKey(Vec<u8>);

#[derive(Clone)]
pub struct PublicKey(Vec<u8>);

#[derive(Clone)]
pub struct Ciphertext(Vec<u8>);

#[derive(Clone)]
pub struct SharedSecret([u8; 32]);

impl fmt::Debug for SecretKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SecretKey([redacted])")
    }
}

impl fmt::Debug for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PublicKey([redacted])")
    }
}

impl fmt::Debug for Ciphertext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Ciphertext([redacted])")
    }
}

impl fmt::Debug for SharedSecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("SharedSecret([redacted])")
    }
}

impl SecretKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl PublicKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl Ciphertext {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl AsRef<[u8]> for SharedSecret {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

pub struct MlKem;

impl MlKem {
    pub fn new() -> Self {
        Self
    }

    pub fn keypair() -> (SecretKey, PublicKey) {
        let (pk, sk) = pqcrypto_mlkem::mlkem1024_keypair();
        (
            SecretKey(sk.as_bytes().to_vec()),
            PublicKey(pk.as_bytes().to_vec()),
        )
    }

    pub fn encapsulate(pk: &[u8]) -> (SharedSecret, Ciphertext) {
        use pqcrypto_mlkem::mlkem1024;
        let pk_obj = mlkem1024::PublicKey::from_bytes(pk)
            .expect("invalid ML-KEM-1024 public key");
        let (ss, ct) = mlkem1024::encapsulate(&pk_obj);
        (SharedSecret(ss.as_bytes().try_into().expect("ML-KEM-1024 shared secret must be 32 bytes")), Ciphertext(ct.as_bytes().to_vec()))
    }

    pub fn decapsulate(ct: &[u8], sk: &[u8]) -> SharedSecret {
        use pqcrypto_mlkem::mlkem1024;
        let ct_obj = mlkem1024::Ciphertext::from_bytes(ct)
            .expect("invalid ML-KEM-1024 ciphertext");
        let sk_obj = mlkem1024::SecretKey::from_bytes(sk)
            .expect("invalid ML-KEM-1024 secret key");
        let ss = mlkem1024::decapsulate(&ct_obj, &sk_obj);
        SharedSecret(ss.as_bytes().try_into().expect("ML-KEM-1024 shared secret must be 32 bytes"))
    }

    pub const fn public_key_bytes() -> usize {
        pqcrypto_mlkem::mlkem1024::public_key_bytes()
    }

    pub const fn secret_key_bytes() -> usize {
        pqcrypto_mlkem::mlkem1024::secret_key_bytes()
    }

    pub const fn ciphertext_bytes() -> usize {
        pqcrypto_mlkem::mlkem1024::ciphertext_bytes()
    }

    pub const fn shared_secret_bytes() -> usize {
        pqcrypto_mlkem::mlkem1024::shared_secret_bytes()
    }
}

impl Default for MlKem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair() {
        let (sk, pk) = MlKem::keypair();
        assert_eq!(sk.as_bytes().len(), MlKem::secret_key_bytes());
        assert_eq!(pk.as_bytes().len(), MlKem::public_key_bytes());
    }

    #[test]
    fn test_roundtrip() {
        let (sk, pk) = MlKem::keypair();
        let (ss1, ct) = MlKem::encapsulate(pk.as_bytes());
        let ss2 = MlKem::decapsulate(ct.as_bytes(), sk.as_bytes());
        assert_eq!(ss1.as_ref(), ss2.as_ref());
    }
}
