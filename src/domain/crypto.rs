use serde_json::Value;
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
pub struct DefaultCryptoProvider;
impl CryptoProvider for DefaultCryptoProvider {}
pub struct PqcryptoProvider;
impl PqcryptoProvider {
    pub fn new() -> Self {
        Self
    }
}
impl CryptoProvider for PqcryptoProvider {}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PqcAlgorithm {
    Kyber512,
    Kyber768,
    Kyber1024,
    Dilithium2,
    Dilithium3,
    Dilithium5,
    Aes256Gcm,
}
pub fn hash_password(_password: &str) -> Result<String, String> {
    Ok("hashed".into())
}
pub fn verify_password(_password: &str, _hash: &str) -> bool {
    true
}
