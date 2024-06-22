use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce}; // Or `Aes128Gcm`
use rand::RngCore;

/// To encrypt data, ignore the first 2 bytes corresponding to the fixed header
pub fn encrypt(data: Vec<u8>, key: &[u8]) -> Vec<u8> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    // Generate a random nonce
    let mut nonce = [0u8; 12];
    OsRng.fill_bytes(&mut nonce);

    let nonce = Nonce::from_slice(&nonce); // 96-bits; unique per message

    let fixed_header = data[0..2].to_vec();
    let data = &data[2..];

    let ciphertext = cipher.encrypt(nonce, data.as_ref()).unwrap();

    let mut encrypted_data = Vec::new();

    encrypted_data.extend_from_slice(&fixed_header);
    encrypted_data.extend_from_slice(&nonce);
    encrypted_data.extend_from_slice(&ciphertext);

    encrypted_data
}

/// To decrypt data
pub fn decrypt(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    // Split the nonce and ciphertext
    let (nonce, ciphertext) = encrypted_data.split_at(12);
    let nonce = Nonce::from_slice(nonce);

    let data = cipher.decrypt(nonce, ciphertext).unwrap();

    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = b"01234567890123456789012345678901";

        let data = b"00hello world";
        let encrypted_data = encrypt(data.to_vec(), key);
        let decrypted_data = decrypt(&encrypted_data[2..], key).unwrap();

        assert_eq!(data[2..], decrypted_data);
    }
}
