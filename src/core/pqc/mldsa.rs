/// ML-DSA (FIPS 204) digital signature algorithm (formerly Dilithium).
///
/// Provides post-quantum secure digital signatures for message
/// authentication and non-repudiation. Uses **ML-DSA-87** for
/// NIST security level 5 (AES-256 equivalent).
use core::fmt;
use pqcrypto_traits::sign::{DetachedSignature, PublicKey as SignPublicKeyTrait, SecretKey as SignSecretKeyTrait};

#[derive(Clone)]
pub struct SecretKey(Vec<u8>);

#[derive(Clone)]
pub struct PublicKey(Vec<u8>);

#[derive(Clone)]
pub struct Signature(Vec<u8>);

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

impl fmt::Debug for Signature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Signature([redacted])")
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

impl Signature {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

pub struct MlDsa;

impl MlDsa {
    pub fn new() -> Self {
        Self
    }

    pub fn keypair() -> (SecretKey, PublicKey) {
        use pqcrypto_mldsa::mldsa87;
        let (pk, sk) = mldsa87::keypair();
        (
            SecretKey(sk.as_bytes().to_vec()),
            PublicKey(pk.as_bytes().to_vec()),
        )
    }

    pub fn sign(msg: &[u8], sk: &[u8]) -> Signature {
        use pqcrypto_mldsa::mldsa87;
        let sk_obj = mldsa87::SecretKey::from_bytes(sk)
            .expect("invalid ML-DSA-87 secret key");
        let sig = mldsa87::detached_sign(msg, &sk_obj);
        Signature(sig.as_bytes().to_vec())
    }

    pub fn verify(msg: &[u8], sig: &[u8], pk: &[u8]) -> bool {
        use pqcrypto_mldsa::mldsa87;
        let pk_obj = mldsa87::PublicKey::from_bytes(pk)
            .expect("invalid ML-DSA-87 public key");
        let sig_obj = mldsa87::DetachedSignature::from_bytes(sig)
            .expect("invalid ML-DSA-87 signature");
        mldsa87::verify_detached_signature(&sig_obj, msg, &pk_obj).is_ok()
    }

    pub const fn public_key_bytes() -> usize {
        pqcrypto_mldsa::mldsa87::public_key_bytes()
    }

    pub const fn secret_key_bytes() -> usize {
        pqcrypto_mldsa::mldsa87::secret_key_bytes()
    }

    pub const fn signature_bytes() -> usize {
        pqcrypto_mldsa::mldsa87::signature_bytes()
    }
}

impl Default for MlDsa {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypair() {
        let (sk, pk) = MlDsa::keypair();
        assert_eq!(sk.as_bytes().len(), MlDsa::secret_key_bytes());
        assert_eq!(pk.as_bytes().len(), MlDsa::public_key_bytes());
    }

    #[test]
    fn test_sign_verify() {
        let (sk, pk) = MlDsa::keypair();
        let msg = b"test message";
        let sig = MlDsa::sign(msg, sk.as_bytes());
        assert!(MlDsa::verify(msg, sig.as_bytes(), pk.as_bytes()));
    }

    #[test]
    fn test_wrong_key_fails() {
        let (sk, _pk) = MlDsa::keypair();
        let (_sk2, pk2) = MlDsa::keypair();
        let msg = b"test message";
        let sig = MlDsa::sign(msg, sk.as_bytes());
        assert!(!MlDsa::verify(msg, sig.as_bytes(), pk2.as_bytes()));
    }
}
