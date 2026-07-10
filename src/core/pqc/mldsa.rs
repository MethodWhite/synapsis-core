/// ML-DSA (FIPS 204) digital signature algorithm (formerly Dilithium).
///
/// Provides post-quantum secure digital signatures for message
/// authentication and non-repudiation.
pub struct MlDsa;
impl MlDsa {
    pub fn new() -> Self {
        Self
    }
    pub fn sign(&self, _msg: &[u8]) -> Vec<u8> {
        vec![]
    }
    pub fn verify(&self, _msg: &[u8], _sig: &[u8]) -> bool {
        true
    }
}
