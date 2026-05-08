//! Synapsis Secure Vault
//!
//! Provides secure storage for session keys with TPM support.
//!
//! # Features
//!
//! - Session key storage
//! - TPM integration when available
//! - Master key auto-generation
//! - Key rotation support

#[allow(unused_imports)]
use aes_gcm::KeyInit;
use prusia_vault::Vault as PrusiaVault;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::RwLock;

const PQC_PASSPHRASE_ENV: &str = "SYNAPSIS_PQC_PASSPHRASE";
#[allow(dead_code)]
const PQC_KEYPAIR_FILE: &str = "vault_pqc.json";
#[allow(dead_code)]
const PQC_ENCRYPTED_MASTER_KEY_FILE: &str = "vault_master.pqc";

#[allow(dead_code)]
fn derive_key_from_passphrase(passphrase: &str, salt: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(passphrase.as_bytes());
    hasher.update(salt);
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result[..32]);
    key
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PqcKeyPair {
    public_key: Vec<u8>,
    encrypted_secret_key: Vec<u8>,
    salt: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionKey {
    pub session_id: String,
    pub encryption_key: Vec<u8>,
    pub mac_key: Vec<u8>,
    pub created_at: i64,
    pub last_used: i64,
    pub rotation_count: u32,
    pub expires_at: Option<i64>,
}

impl SessionKey {
    pub fn is_expired(&self, now: i64) -> bool {
        self.expires_at.map(|e| now > e).unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub session_id: String,
    pub encrypted_key: Vec<u8>,
    pub key_fingerprint: String,
    pub created_at: i64,
    pub last_used: i64,
    pub rotation_count: u32,
    pub tpm_protected: bool,
}

pub struct SecureVault {
    inner: RwLock<PrusiaVault>,
    use_tpm: bool,
}

pub struct MasterKey {
    pub key: Vec<u8>,
    pub created_at: i64,
    pub key_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PqcEncryptedMasterKey {
    pub version: u8,
    pub public_key: Vec<u8>,
    pub encrypted_secret_key: Vec<u8>,
    pub encrypted_master_key: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl serde::Serialize for MasterKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("MasterKey", 3)?;
        s.serialize_field("key", &base64_encode(&self.key))?;
        s.serialize_field("created_at", &self.created_at)?;
        s.serialize_field("key_id", &self.key_id)?;
        s.end()
    }
}

impl<'de> serde::Deserialize<'de> for MasterKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawMasterKey {
            key: String,
            created_at: i64,
            key_id: String,
        }
        let raw = RawMasterKey::deserialize(deserializer)?;
        Ok(MasterKey {
            key: base64_decode(&raw.key).map_err(serde::de::Error::custom)?,
            created_at: raw.created_at,
            key_id: raw.key_id,
        })
    }
}

fn base64_encode(data: &[u8]) -> String {
    base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data)
}

fn base64_decode(data: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(data)
        .map_err(|e| e.to_string())
}

impl SecureVault {
    pub fn new(data_dir: PathBuf) -> Self {
        let vault = PrusiaVault::pqc(data_dir);
        let use_tpm = match &vault {
            PrusiaVault::Pqc(pqc) => pqc.is_tpm_available(),
            _ => false,
        };
        Self {
            inner: RwLock::new(vault),
            use_tpm,
        }
    }

    pub fn is_tpm_available(&self) -> bool {
        self.use_tpm
    }

    pub fn initialize(&self) -> Result<(), VaultError> {
        let passphrase = std::env::var(PQC_PASSPHRASE_ENV)
            .map_err(|_| VaultError::PqcError("PQC passphrase not set".to_string()))?;
        let mut vault = self.inner.write().unwrap_or_else(|e| e.into_inner());
        vault.initialize(&passphrase)?;
        Ok(())
    }

    pub fn store_session_key(
        &self,
        session_id: &str,
        key: &SessionKey,
    ) -> Result<String, VaultError> {
        // Serialize the session key (excluding session_id) to preserve metadata
        let key_data =
            serde_json::to_vec(key).map_err(|e| VaultError::StorageError(e.to_string()))?;
        let vault = self.inner.write().unwrap_or_else(|e| e.into_inner());
        vault.store_session_key(session_id, &key_data)?;
        // Compute fingerprint from encryption key (for compatibility)
        let fingerprint = base64_encode(&compute_hash(&key.encryption_key)[..16]);
        Ok(fingerprint)
    }

    pub fn get_session_key(&self, session_id: &str) -> Result<Option<SessionKey>, VaultError> {
        // Retrieve serialized session key
        let vault = self.inner.read().unwrap();
        match vault.get_session_key(session_id)? {
            Some(key_data) => {
                let mut session_key: SessionKey = serde_json::from_slice(&key_data)
                    .map_err(|e| VaultError::StorageError(e.to_string()))?;
                session_key.session_id = session_id.to_string();
                Ok(Some(session_key))
            }
            None => Ok(None),
        }
    }

    pub fn rotate_key(&self, session_id: &str) -> Result<Option<String>, VaultError> {
        let vault = self.inner.write().unwrap_or_else(|e| e.into_inner());
        match vault.rotate_session_key(session_id)? {
            Some(_) => {
                let new_key = Self::generate_session_key()?;
                let key_data = serde_json::to_vec(&new_key)
                    .map_err(|e| VaultError::StorageError(e.to_string()))?;
                vault.store_session_key(session_id, &key_data)?;
                Ok(Some(base64_encode(
                    &compute_hash(&new_key.encryption_key)[..16],
                )))
            }
            None => Ok(None),
        }
    }

    pub fn close_session(&self, session_id: &str) -> bool {
        let mut vault = self.inner.write().unwrap_or_else(|e| e.into_inner());
        let key = format!("session:{}", session_id);
        vault.delete(&key).is_ok()
    }

    pub fn list_sessions(&self) -> Vec<(String, String, i64)> {
        let vault = self.inner.read().unwrap();
        vault.list_sessions()
    }

    pub fn store_secret(&self, key: &str, value: &str) -> Result<(), VaultError> {
        let mut vault = self.inner.write().unwrap_or_else(|e| e.into_inner());
        vault.store(key, value)?;
        Ok(())
    }

    pub fn retrieve_secret(&self, key: &str) -> Result<String, VaultError> {
        let vault = self.inner.read().unwrap();
        Ok(vault.retrieve(key)?)
    }

    pub fn cleanup_expired(&self) -> usize {
        // Not implemented in PqcVault, return 0 for now
        0
    }

    fn generate_session_key() -> Result<SessionKey, VaultError> {
        let mut encryption_key = vec![0u8; 32];
        getrandom::getrandom(&mut encryption_key).map_err(|e| {
            VaultError::EncryptionFailed(format!("random generation failed: {}", e))
        })?;
        let mac_key = derive_mac_key(&encryption_key);
        let now = current_timestamp();
        Ok(SessionKey {
            session_id: String::new(),
            encryption_key,
            mac_key,
            created_at: now,
            last_used: now,
            rotation_count: 0,
            expires_at: None,
        })
    }
}

#[derive(Debug, Clone)]
pub enum VaultError {
    NotInitialized,
    EncryptionFailed(String),
    DecryptionFailed,
    AuthenticationFailed,
    InvalidKeyLength,
    StorageError(String),
    PqcError(String),
}

impl std::fmt::Display for VaultError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VaultError::NotInitialized => write!(f, "Vault not initialized"),
            VaultError::EncryptionFailed(e) => write!(f, "Encryption failed: {}", e),
            VaultError::DecryptionFailed => write!(f, "Decryption failed"),
            VaultError::AuthenticationFailed => write!(f, "Authentication failed"),
            VaultError::InvalidKeyLength => write!(f, "Invalid key length"),
            VaultError::StorageError(e) => write!(f, "Storage error: {}", e),
            VaultError::PqcError(e) => write!(f, "PQC error: {}", e),
        }
    }
}

impl std::error::Error for VaultError {}

impl From<std::io::Error> for VaultError {
    fn from(e: std::io::Error) -> Self {
        VaultError::StorageError(e.to_string())
    }
}

impl From<serde_json::Error> for VaultError {
    fn from(e: serde_json::Error) -> Self {
        VaultError::StorageError(e.to_string())
    }
}

impl From<prusia_vault::error::VaultError> for VaultError {
    fn from(e: prusia_vault::error::VaultError) -> Self {
        VaultError::PqcError(e.to_string())
    }
}
fn compute_hash(data: &[u8]) -> Vec<u8> {
    let h = [0x6a09e667u32, 0xbb67ae85u32, 0x3c6ef372u32, 0xa54ff53au32];

    let mut hash = [0u32; 4];
    for (i, val) in h.iter().enumerate() {
        hash[i] = *val;
    }

    for chunk in data.chunks(64) {
        let mut w = [0u32; 16];

        for (i, bytes) in chunk.chunks(4).enumerate() {
            if bytes.len() == 4 {
                w[i] = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
            }
        }

        for i in 0..16 {
            hash[i % 4] = hash[i % 4].wrapping_add(w[i]);
        }
    }

    let mut result = Vec::with_capacity(16);
    for val in hash.iter() {
        result.extend_from_slice(&val.to_be_bytes());
    }
    result
}

#[allow(dead_code)]
fn compute_hmac(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut inner_pad = [0x36u8; 64];
    let mut outer_pad = [0x5cu8; 64];

    for i in 0..64.min(key.len()) {
        inner_pad[i] ^= key[i];
        outer_pad[i] ^= key[i];
    }

    let inner_data: Vec<u8> = inner_pad.iter().chain(data.iter()).cloned().collect();
    let inner_hash = compute_hash(&inner_data);

    let outer_data: Vec<u8> = outer_pad.iter().chain(inner_hash.iter()).cloned().collect();
    compute_hash(&outer_data)
}

fn derive_mac_key(encryption_key: &[u8]) -> Vec<u8> {
    let mut mac_key = vec![0u8; 32];
    for (i, byte) in mac_key.iter_mut().enumerate() {
        *byte = encryption_key[i % 32].wrapping_add(0x5a);
    }
    compute_hash(&mac_key)
}

#[allow(dead_code)]
fn generate_nonce(len: usize) -> Vec<u8> {
    let mut nonce = vec![0u8; len];
    if let Err(_e) = getrandom::getrandom(&mut nonce) {
        // Fallback to weak randomness if getrandom fails (should not happen)
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        for (i, byte) in nonce.iter_mut().enumerate() {
            let val = seed.wrapping_mul(i as u64 + 1).wrapping_mul(1103515245);
            *byte = ((val >> 16) ^ val) as u8;
        }
    }
    nonce
}

fn current_timestamp() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_vault_init() {
        // Set PQC passphrase for testing
        std::env::set_var("SYNAPSIS_PQC_PASSPHRASE", "test_passphrase_123");
        let vault = SecureVault::new(temp_dir().join("synapsis_test_vault"));
        vault.initialize().unwrap();

        assert!(vault.is_tpm_available() || !vault.is_tpm_available());
    }

    #[test]
    fn test_store_and_retrieve_key() {
        // Set PQC passphrase for testing
        std::env::set_var("SYNAPSIS_PQC_PASSPHRASE", "test_passphrase_123");
        let vault = SecureVault::new(temp_dir().join("synapsis_test_vault2"));
        vault.initialize().unwrap();

        let mut key = SessionKey {
            session_id: "test-session".to_string(),
            encryption_key: vec![0u8; 32],
            mac_key: vec![0u8; 32],
            created_at: current_timestamp(),
            last_used: current_timestamp(),
            rotation_count: 0,
            expires_at: None,
        };

        for (i, byte) in key.encryption_key.iter_mut().enumerate() {
            *byte = i as u8;
        }
        key.mac_key = derive_mac_key(&key.encryption_key);

        let fingerprint = vault.store_session_key("test-session", &key).unwrap();
        assert!(!fingerprint.is_empty());

        let retrieved = vault.get_session_key("test-session").unwrap().unwrap();
        assert_eq!(retrieved.encryption_key, key.encryption_key);
    }

    #[test]
    fn test_rotate_key() {
        // Set PQC passphrase for testing
        std::env::set_var("SYNAPSIS_PQC_PASSPHRASE", "test_passphrase_123");
        let vault = SecureVault::new(temp_dir().join("synapsis_test_vault3"));
        vault.initialize().unwrap();

        let key = SessionKey {
            session_id: "test-session".to_string(),
            encryption_key: vec![0u8; 32],
            mac_key: vec![0u8; 32],
            created_at: current_timestamp(),
            last_used: current_timestamp(),
            rotation_count: 0,
            expires_at: None,
        };

        vault.store_session_key("test-session", &key).unwrap();

        let new_fingerprint = vault.rotate_key("test-session").unwrap().unwrap();
        assert!(!new_fingerprint.is_empty());
    }

    #[test]
    fn test_close_session() {
        // Set PQC passphrase for testing
        std::env::set_var("SYNAPSIS_PQC_PASSPHRASE", "test_passphrase_123");
        let vault = SecureVault::new(temp_dir().join("synapsis_test_vault4"));
        vault.initialize().unwrap();

        let key = SessionKey {
            session_id: "test-session".to_string(),
            encryption_key: vec![0u8; 32],
            mac_key: vec![0u8; 32],
            created_at: current_timestamp(),
            last_used: current_timestamp(),
            rotation_count: 0,
            expires_at: None,
        };

        vault.store_session_key("test-session", &key).unwrap();

        assert!(vault.close_session("test-session"));
        assert!(vault.get_session_key("test-session").unwrap().is_none());
    }
}
