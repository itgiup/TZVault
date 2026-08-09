// src-tauri/src/crypto/cipher.rs
//
// Mã hóa/giải mã dữ liệu bằng AES-256-GCM (authenticated encryption:
// vừa mã hóa vừa chống việc dữ liệu bị chỉnh sửa mà không phát hiện được).

use aes_gcm::{
    aead::{Aead, KeyInit, Payload},
    Aes256Gcm, Nonce, Key,
};
use rand::RngCore;
use rand::rngs::OsRng;
use zeroize::Zeroize;

pub const NONCE_LEN: usize = 12; // chuẩn cho AES-GCM

#[derive(Debug)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; NONCE_LEN],
}

/// Sinh nonce ngẫu nhiên mới. QUAN TRỌNG: mỗi lần mã hóa PHẢI dùng nonce
/// khác nhau với cùng 1 key — tái sử dụng nonce sẽ phá vỡ hoàn toàn
/// tính bảo mật của AES-GCM.
fn generate_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

/// Mã hóa dữ liệu (VD: nội dung 1 private key) bằng key 32 byte.
///
/// `associated_data`: dữ liệu không mã hóa nhưng được "gắn chặt" vào
/// ciphertext để chống tấn công thay thế (VD: dùng id của record làm AAD,
/// để ciphertext của key A không thể bị copy-paste gán cho key B).
pub fn encrypt(
    key: &[u8; 32],
    plaintext: &[u8],
    associated_data: &[u8],
) -> Result<EncryptedData, String> {
    let cipher_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(cipher_key);

    let nonce_bytes = generate_nonce();
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(
            nonce,
            Payload {
                msg: plaintext,
                aad: associated_data,
            },
        )
        .map_err(|e| crate::error::internal_error("cipher_encrypt", e))?;

    Ok(EncryptedData {
        ciphertext,
        nonce: nonce_bytes,
    })
}

/// Giải mã. Sẽ trả về Err nếu key sai, dữ liệu bị chỉnh sửa,
/// hoặc associated_data không khớp với lúc mã hóa.
pub fn decrypt(
    key: &[u8; 32],
    ciphertext: &[u8],
    nonce: &[u8; NONCE_LEN],
    associated_data: &[u8],
) -> Result<Vec<u8>, String> {
    let cipher_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(cipher_key);
    let nonce = Nonce::from_slice(nonce);

    cipher
        .decrypt(
            nonce,
            Payload {
                msg: ciphertext,
                aad: associated_data,
            },
        )
        .map_err(|_| crate::error::internal_error_msg("cipher_decrypt_failed"))
}

/// Helper: mã hóa một chuỗi String (VD: nội dung private key dạng text)
/// và tự động xóa plaintext khỏi RAM sau khi xong. Chưa được gọi ở đâu
/// trong luồng hiện tại — giữ lại làm phương án nâng cấp an toàn hơn.
#[allow(dead_code)]
pub fn encrypt_string_and_wipe(
    key: &[u8; 32],
    mut plaintext: String,
    associated_data: &[u8],
) -> Result<EncryptedData, String> {
    let result = encrypt(key, plaintext.as_bytes(), associated_data);
    plaintext.zeroize();
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        [42u8; 32]
    }

    #[test]
    fn encrypt_then_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"ssh-ed25519 AAAA... noi dung private key bi mat";
        let aad = b"key-id-12345";

        let encrypted = encrypt(&key, plaintext, aad).unwrap();
        let decrypted = decrypt(&key, &encrypted.ciphertext, &encrypted.nonce, aad).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn wrong_key_fails_to_decrypt() {
        let key = test_key();
        let wrong_key = [99u8; 32];
        let plaintext = b"bi mat quan trong";
        let aad = b"key-id-1";

        let encrypted = encrypt(&key, plaintext, aad).unwrap();
        let result = decrypt(&wrong_key, &encrypted.ciphertext, &encrypted.nonce, aad);

        assert!(result.is_err());
    }

    #[test]
    fn wrong_associated_data_fails_to_decrypt() {
        let key = test_key();
        let plaintext = b"noi dung bi mat";

        let encrypted = encrypt(&key, plaintext, b"key-id-A").unwrap();
        // Thử giải mã nhưng dùng AAD của key khác -> phải fail
        let result = decrypt(&key, &encrypted.ciphertext, &encrypted.nonce, b"key-id-B");

        assert!(result.is_err());
    }

    #[test]
    fn tampered_ciphertext_fails_to_decrypt() {
        let key = test_key();
        let plaintext = b"du lieu goc";
        let aad = b"key-id-1";

        let mut encrypted = encrypt(&key, plaintext, aad).unwrap();
        encrypted.ciphertext[0] ^= 0xFF; // giả lập bị chỉnh sửa

        let result = decrypt(&key, &encrypted.ciphertext, &encrypted.nonce, aad);
        assert!(result.is_err());
    }
}
