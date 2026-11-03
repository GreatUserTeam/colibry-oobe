use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::error::Error;

/// Hash a password with Argon2id and return a PHC string for /etc/shadow.
pub fn hash_password(password: &str) -> Result<String, Box<dyn Error>> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| -> Box<dyn Error> { e.to_string().into() })?;
    Ok(hash.to_string())
}

/// Encrypt the Wi-Fi password with AES-256-GCM.
///
/// The key is SHA-256(device_password_hash || salt): knowing only the config
/// file is not enough to decrypt, the original device password is required.
/// Returns (base64(nonce || ciphertext), base64(salt)); empty strings when the
/// password is empty.
pub fn encrypt_wifi_password(
    password: &str,
    device_password_hash: &str,
) -> Result<(String, String), Box<dyn Error>> {
    if password.is_empty() {
        return Ok((String::new(), String::new()));
    }

    let mut salt = [0u8; 16];
    OsRng.fill_bytes(&mut salt);

    let mut hasher = Sha256::new();
    hasher.update(device_password_hash.as_bytes());
    hasher.update(salt);
    let key_bytes = hasher.finalize();

    let cipher = Aes256Gcm::new_from_slice(&key_bytes)
        .map_err(|e| -> Box<dyn Error> { e.to_string().into() })?;

    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, password.as_bytes())
        .map_err(|e| -> Box<dyn Error> { e.to_string().into() })?;

    let mut blob = nonce_bytes.to_vec();
    blob.extend_from_slice(&ciphertext);

    Ok((B64.encode(blob), B64.encode(salt)))
}
