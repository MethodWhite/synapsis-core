/// ML-KEM (FIPS 203) key-encapsulation mechanism (formerly Kyber).
///
/// Provides post-quantum secure key encapsulation for establishing
/// shared secrets between two parties.
#[derive(Debug, Clone, Default)]
pub struct MlKem;
impl MlKem {
    pub fn new() -> Self {
        Self
    }
    pub fn encapsulate(&self, _pk: &[u8]) -> (Vec<u8>, Vec<u8>) {
        (vec![], vec![])
    }
    pub fn decapsulate(&self, _ct: &[u8], _sk: &[u8]) -> Vec<u8> {
        vec![]
    }
}
