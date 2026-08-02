// src-tauri/src/crypto/mod.rs
//
// Điểm vào của module crypto. Gộp kdf + cipher thành các hàm
// high-level dùng trực tiếp trong logic vault (setup, unlock, add key...).

pub mod kdf;
pub mod cipher;

use kdf::KdfParams;
use rand::RngCore;
use rand::rngs::OsRng;
use zeroize::Zeroize;

/// Kết quả khi setup vault lần đầu — đây là những gì sẽ được lưu
/// vào bảng `vault_meta` trong database.
pub struct VaultSetupResult {
    pub salt: [u8; kdf::SALT_LEN],
    pub encrypted_vault_key: Vec<u8>,
    pub vault_key_nonce: [u8; cipher::NONCE_LEN],
    pub kdf_params: KdfParams,
}

/// SETUP LẦN ĐẦU: tạo vault mới từ master password.
///
/// Luồng:
/// 1. Sinh Vault Key ngẫu nhiên (đây mới là key thực sự mã hóa dữ liệu)
/// 2. Derive KEK từ master password
/// 3. Mã hóa Vault Key bằng KEK -> lưu bản mã hóa này
///
/// Trả về VaultSetupResult để lưu vào DB, và trả riêng vault_key (plaintext)
/// để giữ trong session state (RAM) cho phiên làm việc hiện tại.
pub fn setup_vault(master_password: &str) -> Result<(VaultSetupResult, [u8; 32]), String> {
    let params = KdfParams::default();
    let salt = kdf::generate_salt();
    let kek = kdf::derive_key(master_password, &salt, &params)?;

    // Sinh Vault Key ngẫu nhiên - đây là key THẬT SỰ mã hóa từng private key
    let mut vault_key = [0u8; 32];
    OsRng.fill_bytes(&mut vault_key);

    // Mã hóa Vault Key bằng KEK, dùng "vault_key" làm associated data cố định
    let encrypted = cipher::encrypt(&kek, &vault_key, b"vault_key")?;

    let result = VaultSetupResult {
        salt,
        encrypted_vault_key: encrypted.ciphertext,
        vault_key_nonce: encrypted.nonce,
        kdf_params: params,
    };

    Ok((result, vault_key))
}

/// MỞ KHÓA: giải mã Vault Key từ dữ liệu đã lưu + master password nhập vào.
///
/// Trả về vault_key (giữ trong RAM session) nếu password đúng,
/// hoặc Err nếu password sai / dữ liệu bị hỏng.
pub fn unlock_vault(
    master_password: &str,
    salt: &[u8],
    encrypted_vault_key: &[u8],
    vault_key_nonce: &[u8; cipher::NONCE_LEN],
    params: &KdfParams,
) -> Result<[u8; 32], String> {
    let kek = kdf::derive_key(master_password, salt, params)?;

    let decrypted = cipher::decrypt(&kek, encrypted_vault_key, vault_key_nonce, b"vault_key")?;

    if decrypted.len() != 32 {
        return Err(crate::error::internal_error_msg("vault_key_wrong_length"));
    }

    let mut vault_key = [0u8; 32];
    vault_key.copy_from_slice(&decrypted);
    Ok(vault_key)
}

/// ĐỔI MASTER PASSWORD: chỉ cần re-encrypt Vault Key bằng KEK mới,
/// KHÔNG cần giải mã lại từng private key trong vault.
pub fn change_master_password(
    old_password: &str,
    new_password: &str,
    salt: &[u8],
    encrypted_vault_key: &[u8],
    vault_key_nonce: &[u8; cipher::NONCE_LEN],
    params: &KdfParams,
) -> Result<VaultSetupResult, String> {
    // Bước 1: giải mã vault key hiện tại bằng password cũ
    let vault_key = unlock_vault(old_password, salt, encrypted_vault_key, vault_key_nonce, params)?;

    // Bước 2: derive KEK mới từ password mới, dùng salt mới
    let new_salt = kdf::generate_salt();
    let new_kek = kdf::derive_key(new_password, &new_salt, params)?;

    // Bước 3: mã hóa lại vault key bằng KEK mới
    let encrypted = cipher::encrypt(&new_kek, &vault_key, b"vault_key")?;

    Ok(VaultSetupResult {
        salt: new_salt,
        encrypted_vault_key: encrypted.ciphertext,
        vault_key_nonce: encrypted.nonce,
        kdf_params: params.clone(),
    })
}

/// Mã hóa nội dung 1 private key trước khi lưu vào DB.
/// `key_id` dùng làm associated_data để chống tấn công "ciphertext swap"
/// (copy ciphertext của record này gán cho record khác).
pub fn encrypt_key_value(
    vault_key: &[u8; 32],
    key_id: &str,
    plaintext_key: &str,
) -> Result<cipher::EncryptedData, String> {
    cipher::encrypt(vault_key, plaintext_key.as_bytes(), key_id.as_bytes())
}

/// Giải mã nội dung 1 private key khi người dùng bấm "Show" / "Copy".
pub fn decrypt_key_value(
    vault_key: &[u8; 32],
    key_id: &str,
    ciphertext: &[u8],
    nonce: &[u8; cipher::NONCE_LEN],
) -> Result<String, String> {
    let bytes = cipher::decrypt(vault_key, ciphertext, nonce, key_id.as_bytes())?;
    String::from_utf8(bytes).map_err(|e| crate::error::internal_error("decrypted_data_not_utf8", e))
}

/// Xóa một buffer 32-byte khỏi RAM một cách an toàn.
/// Gọi hàm này khi lock vault hoặc app thoát.
pub fn wipe_key(key: &mut [u8; 32]) {
    key.zeroize();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_setup_and_unlock_flow() {
        let password = "master_password_manh_123!";

        let (setup_result, vault_key_from_setup) = setup_vault(password).unwrap();

        let vault_key_from_unlock = unlock_vault(
            password,
            &setup_result.salt,
            &setup_result.encrypted_vault_key,
            &setup_result.vault_key_nonce,
            &setup_result.kdf_params,
        )
        .unwrap();

        // Vault key giải mã ra phải giống hệt vault key lúc setup
        assert_eq!(vault_key_from_setup, vault_key_from_unlock);
    }

    #[test]
    fn unlock_with_wrong_password_fails() {
        let (setup_result, _) = setup_vault("password_dung").unwrap();

        let result = unlock_vault(
            "password_sai",
            &setup_result.salt,
            &setup_result.encrypted_vault_key,
            &setup_result.vault_key_nonce,
            &setup_result.kdf_params,
        );

        assert!(result.is_err());
    }

    #[test]
    fn encrypt_and_decrypt_a_stored_key() {
        let (_, vault_key) = setup_vault("master_pw").unwrap();

        let key_id = "uuid-abc-123";
        let secret = "-----BEGIN OPENSSH PRIVATE KEY-----\nfakecontent\n-----END-----";

        let encrypted = encrypt_key_value(&vault_key, key_id, secret).unwrap();
        let decrypted = decrypt_key_value(&vault_key, key_id, &encrypted.ciphertext, &encrypted.nonce).unwrap();

        assert_eq!(decrypted, secret);
    }

    #[test]
    fn change_password_then_unlock_with_new_password() {
        let (setup_result, _) = setup_vault("password_cu").unwrap();

        let new_setup = change_master_password(
            "password_cu",
            "password_moi_123!",
            &setup_result.salt,
            &setup_result.encrypted_vault_key,
            &setup_result.vault_key_nonce,
            &setup_result.kdf_params,
        )
        .unwrap();

        // Unlock bằng password mới phải thành công
        let result = unlock_vault(
            "password_moi_123!",
            &new_setup.salt,
            &new_setup.encrypted_vault_key,
            &new_setup.vault_key_nonce,
            &new_setup.kdf_params,
        );
        assert!(result.is_ok());

        // Unlock bằng password cũ phải thất bại
        let result_old = unlock_vault(
            "password_cu",
            &new_setup.salt,
            &new_setup.encrypted_vault_key,
            &new_setup.vault_key_nonce,
            &new_setup.kdf_params,
        );
        assert!(result_old.is_err());
    }
}
