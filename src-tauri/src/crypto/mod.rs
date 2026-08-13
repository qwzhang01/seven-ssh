use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::Argon2;
use rand::RngCore;
use serde::{Deserialize, Serialize};

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

pub const ENC_PREFIX: &str = "enc:";

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub salt: Vec<u8>,
    pub nonce: Vec<u8>,
    pub ciphertext: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FieldEncrypted {
    nonce: String,
    ciphertext: String,
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn from_hex(s: &str) -> Result<Vec<u8>, String> {
    if s.len() % 2 != 0 {
        return Err("Invalid hex string length".to_string());
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| format!("Hex decode error: {}", e)))
        .collect()
}

pub fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; KEY_LEN], String> {
    let mut key = [0u8; KEY_LEN];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut key)
        .map_err(|e| format!("Key derivation failed: {}", e))?;
    Ok(key)
}

pub fn encrypt_field(plaintext: &str, master_key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new_from_slice(master_key)
        .map_err(|e| format!("Cipher init failed: {}", e))?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let encoded = FieldEncrypted {
        nonce: to_hex(&nonce_bytes),
        ciphertext: to_hex(&ciphertext),
    };

    serde_json::to_string(&encoded).map_err(|e| format!("Serialization failed: {}", e))
}

pub fn decrypt_field(encoded: &str, master_key: &[u8; 32]) -> Result<String, String> {
    let field: FieldEncrypted =
        serde_json::from_str(encoded).map_err(|e| format!("Deserialization failed: {}", e))?;

    let nonce_bytes = from_hex(&field.nonce)?;
    let ciphertext = from_hex(&field.ciphertext)?;

    if nonce_bytes.len() != NONCE_LEN {
        return Err("Invalid nonce length".to_string());
    }

    let cipher = Aes256Gcm::new_from_slice(master_key)
        .map_err(|e| format!("Cipher init failed: {}", e))?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("UTF-8 decode failed: {}", e))
}

/// Argon2id hash: returns hex(salt || hash) for storage
pub fn hash_master_password(password: &str) -> Result<String, String> {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);

    let mut hash = [0u8; KEY_LEN];
    Argon2::default()
        .hash_password_into(password.as_bytes(), &salt, &mut hash)
        .map_err(|e| format!("Hashing failed: {}", e))?;

    let mut combined = Vec::with_capacity(SALT_LEN + KEY_LEN);
    combined.extend_from_slice(&salt);
    combined.extend_from_slice(&hash);
    Ok(to_hex(&combined))
}

/// Verify password against stored hex(salt || hash)
pub fn verify_master_password(password: &str, stored_hash: &str) -> Result<bool, String> {
    let combined = from_hex(stored_hash)?;
    if combined.len() != SALT_LEN + KEY_LEN {
        return Err("Invalid stored hash length".to_string());
    }

    let salt = &combined[..SALT_LEN];
    let expected_hash = &combined[SALT_LEN..];

    let mut computed = [0u8; KEY_LEN];
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, &mut computed)
        .map_err(|e| format!("Hashing failed: {}", e))?;

    Ok(computed.as_slice() == expected_hash)
}

/// Derive a 32-byte master key from a password (for in-memory use)
pub fn derive_master_key(password: &str, stored_hash: &str) -> Result<[u8; KEY_LEN], String> {
    let combined = from_hex(stored_hash)?;
    if combined.len() != SALT_LEN + KEY_LEN {
        return Err("Invalid stored hash length".to_string());
    }
    let salt = &combined[..SALT_LEN];
    derive_key(password, salt)
}

#[allow(dead_code)]
pub fn encrypt(plaintext: &[u8], password: &str) -> Result<EncryptedData, String> {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);

    let key = derive_key(password, &salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Cipher init failed: {}", e))?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| format!("Encryption failed: {}", e))?;

    Ok(EncryptedData {
        salt: salt.to_vec(),
        nonce: nonce_bytes.to_vec(),
        ciphertext,
    })
}

#[allow(dead_code)]
pub fn decrypt(encrypted: &EncryptedData, password: &str) -> Result<Vec<u8>, String> {
    let key = derive_key(password, &encrypted.salt)?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Cipher init failed: {}", e))?;

    let nonce = Nonce::from_slice(&encrypted.nonce);
    cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| format!("Decryption failed: {}", e))
}
