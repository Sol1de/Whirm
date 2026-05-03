use app_lib::crypto::{decrypt, derive_key, encrypt};

#[test]
fn encrypt_then_decrypt_roundtrip_recovers_plaintext() {
    let key = [42u8; 32];
    let plaintext = "super-secret-p@ssw0rd!";
    let encrypted = encrypt(plaintext, &key).expect("encrypt");
    let decrypted = decrypt(&encrypted, &key).expect("decrypt");
    assert_eq!(decrypted, plaintext);
}

#[test]
fn decrypt_short_ciphertext_errors() {
    let key = [42u8; 32];
    // "AAEC" is valid base64 decoding to 3 bytes, which is < 12 (nonce min)
    let result = decrypt("AAEC", &key);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Ciphertext too short");
}

#[test]
fn decrypt_invalid_base64_errors() {
    let key = [42u8; 32];
    let result = decrypt("not-valid-base64!!!", &key);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Invalid base64");
}

#[test]
fn decrypt_with_wrong_key_errors() {
    let key1 = [1u8; 32];
    let key2 = [2u8; 32];
    let encrypted = encrypt("password", &key1).expect("encrypt");
    let result = decrypt(&encrypted, &key2);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Decryption failed");
}

#[test]
fn derive_key_returns_32_bytes() {
    let key = derive_key().expect("derive key");
    assert_eq!(key.len(), 32);
}

#[test]
#[ignore = "relies on machine-uid; unreliable in ephemeral CI containers"]
fn derive_key_is_deterministic_on_same_machine() {
    let key1 = derive_key().expect("first derive");
    let key2 = derive_key().expect("second derive");
    assert_eq!(key1, key2);
}
