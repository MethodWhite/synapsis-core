pub struct DilithiumSigner;
impl DilithiumSigner {
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
