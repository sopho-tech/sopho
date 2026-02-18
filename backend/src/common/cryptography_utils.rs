use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key,
};
use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub const SECRET_KEY_PATH: &str = "./data/secret.key";

pub fn get_encryption_key_from_path() -> Result<Option<String>> {
    let path = Path::new(SECRET_KEY_PATH);
    if path.exists() {
        let key = fs::read_to_string(path)?.trim().to_string();
        Ok(Some(key))
    } else {
        Ok(None)
    }
}

pub fn generate_and_store_encryption_key() -> Result<(String, String)> {
    let encryption_key = generate_encryption_key();
    let path = Path::new(SECRET_KEY_PATH);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, &encryption_key)?;
    let full_path = path.canonicalize()?.display().to_string();
    Ok((encryption_key, full_path))
}

pub fn generate_encryption_key() -> String {
    let key = Aes256Gcm::generate_key(OsRng);
    STANDARD.encode(key.as_slice())
}

pub fn encrypt(encryption_key: String, data: String) -> Result<String, aes_gcm::Error> {
    let key_bytes = Sha256::digest(encryption_key.as_bytes());
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher.encrypt(&nonce, data.as_bytes())?;
    let mut combined = nonce.as_slice().to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(combined))
}

pub fn decrypt(encryption_key: String, data: String) -> Result<String> {
    let combined = STANDARD.decode(data.as_bytes())?;
    let (nonce_slice, ciphertext) = combined.split_at(12);
    let key_bytes = Sha256::digest(encryption_key.as_bytes());
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let plaintext = cipher
        .decrypt(aes_gcm::Nonce::from_slice(nonce_slice), ciphertext)
        .map_err(|e| anyhow::anyhow!("decryption failed: {}", e))?;
    String::from_utf8(plaintext).map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_encryption_key_returns_valid_base64() {
        let key = generate_encryption_key();
        let decoded = STANDARD.decode(&key).expect("should be valid base64");
        assert_eq!(decoded.len(), 32, "AES-256 key must be 32 bytes");
    }

    #[test]
    fn generate_encryption_key_produces_unique_keys() {
        let key1 = generate_encryption_key();
        let key2 = generate_encryption_key();
        assert_ne!(key1, key2);
    }

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let encryption_key = "test-secret-key-32-chars-long!!".to_string();
        let plaintext = "sensitive data".to_string();
        let ciphertext = encrypt(encryption_key.clone(), plaintext.clone()).unwrap();
        assert_ne!(ciphertext, plaintext);
        let decrypted = decrypt(encryption_key, ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_produces_different_ciphertext_for_same_plaintext() {
        let encryption_key = "test-secret-key-32-chars-long!!".to_string();
        let plaintext = "same plaintext".to_string();
        let ciphertext1 = encrypt(encryption_key.clone(), plaintext.clone()).unwrap();
        let ciphertext2 = encrypt(encryption_key, plaintext).unwrap();
        assert_ne!(ciphertext1, ciphertext2);
    }

    #[test]
    fn decrypt_with_wrong_key_fails() {
        let encryption_key = "test-secret-key-32-chars-long!!".to_string();
        let plaintext = "secret".to_string();
        let ciphertext = encrypt(encryption_key, plaintext).unwrap();
        let wrong_key = "wrong-secret-key-32-chars-long!!!".to_string();
        assert!(decrypt(wrong_key, ciphertext).is_err());
    }

    #[test]
    fn decrypt_invalid_base64_fails() {
        let encryption_key = "test-secret-key-32-chars-long!!".to_string();
        assert!(decrypt(encryption_key, "not-valid-base64!!!".to_string()).is_err());
    }

    #[test]
    fn encrypt_decrypt_empty_string() {
        let encryption_key = "test-secret-key-32-chars-long!!".to_string();
        let plaintext = String::new();
        let ciphertext = encrypt(encryption_key.clone(), plaintext.clone()).unwrap();
        let decrypted = decrypt(encryption_key, ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn encrypt_decrypt_unicode() {
        let encryption_key = "test-secret-key-32-chars-long!!".to_string();
        let plaintext = "日本語 🔐 café".to_string();
        let ciphertext = encrypt(encryption_key.clone(), plaintext.clone()).unwrap();
        let decrypted = decrypt(encryption_key, ciphertext).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}
