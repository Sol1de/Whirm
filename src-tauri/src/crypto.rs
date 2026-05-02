use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use hkdf::Hkdf;
use sha2::Sha256;

pub fn derive_key() -> Result<[u8; 32], String> {
    let machine_id = machine_uid::get()
        .map_err(|e| format!("Failed to get machine ID: {e}"))?;

    let hk = Hkdf::<Sha256>::new(Some(b"whirm-field-v1"), machine_id.as_bytes());
    let mut key = [0u8; 32];
    hk.expand(b"password-field-key", &mut key)
        .map_err(|_| "HKDF expand failed".to_string())?;

    Ok(key)
}

pub fn encrypt(plaintext: &str, key: &[u8; 32]) -> Result<String, String> {
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext.as_bytes())
        .map_err(|_| "Encryption failed".to_string())?;

    let mut combined = nonce.to_vec();
    combined.extend_from_slice(&ciphertext);
    Ok(STANDARD.encode(&combined))
}

pub fn decrypt(encoded: &str, key: &[u8; 32]) -> Result<String, String> {
    let combined = STANDARD
        .decode(encoded)
        .map_err(|_| "Invalid base64".to_string())?;

    if combined.len() < 12 {
        return Err("Ciphertext too short".to_string());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key));

    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), ciphertext)
        .map_err(|_| "Decryption failed".to_string())?;

    String::from_utf8(plaintext).map_err(|_| "Invalid UTF-8 after decryption".to_string())
}
