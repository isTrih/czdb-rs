use aes::Aes128;
use aes::cipher::{BlockDecrypt, KeyInit, generic_array::GenericArray};
use base64::{Engine as _, engine::general_purpose};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DecryptError {
    #[error("Base64 decode error")]
    Base64Error(#[from] base64::DecodeError),
    #[error("Invalid key length")]
    InvalidKeyLength,
    #[allow(dead_code)]
    #[error("Decryption error")]
    DecryptionError,
}

pub fn decrypt_aes_ecb(key: &str, data: &[u8]) -> Result<Vec<u8>, DecryptError> {
    let key_bytes = general_purpose::STANDARD.decode(key)?;
    
    if key_bytes.len() != 16 {
        return Err(DecryptError::InvalidKeyLength);
    }

    let key = GenericArray::from_slice(&key_bytes);
    let cipher = Aes128::new(key);

    let mut decrypted_data = data.to_vec();
    
    // AES block size is 16 bytes
    for chunk in decrypted_data.chunks_mut(16) {
        if chunk.len() == 16 {
            let block = GenericArray::from_mut_slice(chunk);
            cipher.decrypt_block(block);
        }
    }

    // Remove padding (PKCS#7)
    // The C code uses EVP_DecryptFinal_ex which handles padding.
    // We need to handle it manually or use a crate that handles it.
    // Let's check the last byte for padding length.
    if let Some(&pad_len) = decrypted_data.last() {
        let pad_len = pad_len as usize;
        if pad_len > 0 && pad_len <= 16 && pad_len <= decrypted_data.len() {
             // Verify padding bytes
            let len = decrypted_data.len();
            let valid_padding = decrypted_data[len - pad_len..].iter().all(|&b| b == pad_len as u8);
            if valid_padding {
                decrypted_data.truncate(len - pad_len);
            }
        }
    }

    Ok(decrypted_data)
}

pub fn decrypt_xor(key: &str, data: &mut [u8]) -> Result<(), DecryptError> {
    let key_bytes = general_purpose::STANDARD.decode(key)?;
    let key_len = key_bytes.len();
    
    if key_len == 0 {
        return Ok(());
    }

    for (i, byte) in data.iter_mut().enumerate() {
        *byte ^= key_bytes[i % 16]; // C code uses % 16, assuming key is 16 bytes (128 bits)
        // Wait, C code: `base64_decode(key, ..., keyBytes, 128);` and `keyBytes[i % 16]`.
        // It seems it assumes the key is at least 16 bytes or it uses the first 16 bytes.
        // The AES key is 128 bits (16 bytes).
    }
    
    Ok(())
}
