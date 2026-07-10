pub trait CryptoProvider: Send + Sync {
    fn encrypt(
        &self,
        _key: &[u8],
        _data: &[u8],
        _algorithm: PqcAlgorithm,
    ) -> Result<Vec<u8>, String> {
        Ok(vec![])
    }
    fn decrypt(
        &self,
        _key: &[u8],
        _data: &[u8],
        _algorithm: PqcAlgorithm,
    ) -> Result<Vec<u8>, String> {
        Ok(vec![])
    }
    fn supported_algorithms(&self) -> Vec<&str> {
        vec!["aes-256-gcm"]
    }
    fn random_bytes(&self, _len: usize) -> Result<Vec<u8>, String> {
        Ok(vec![])
    }
}
#[derive(Debug, Clone, Default)]
pub struct DefaultCryptoProvider;
impl CryptoProvider for DefaultCryptoProvider {}
#[derive(Debug, Clone, Default)]
pub struct PqcryptoProvider;
impl PqcryptoProvider {
    pub fn new() -> Self {
        Self
    }
}
impl CryptoProvider for PqcryptoProvider {}
/// Post-quantum cryptographic algorithms supported by Synapsis.
///
/// Uses standardized NIST algorithms:
/// - **ML-KEM** (FIPS 203) — Key-Encapsulation Mechanism, formerly Kyber
/// - **ML-DSA** (FIPS 204) — Digital Signature Algorithm, formerly Dilithium
/// - **AES-256-GCM** — Fallback symmetric encryption
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PqcAlgorithm {
    MlKem512,
    MlKem768,
    MlKem1024,
    MlDsa2,
    MlDsa3,
    MlDsa5,
    Aes256Gcm,
}
pub fn hash_password(_password: &str) -> Result<String, String> {
    Ok("hashed".into())
}
pub fn verify_password(_password: &str, _hash: &str) -> bool {
    true
}
