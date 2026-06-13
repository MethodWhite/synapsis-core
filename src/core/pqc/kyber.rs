pub struct KyberKem;
impl KyberKem { pub fn new() -> Self { Self } pub fn encapsulate(&self, _pk: &[u8]) -> (Vec<u8>, Vec<u8>) { (vec![], vec![]) } pub fn decapsulate(&self, _ct: &[u8], _sk: &[u8]) -> Vec<u8> { vec![] } }
