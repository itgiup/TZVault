// src-tauri/src/crypto/kdf.rs
//
// Chuyển Master Password (do người dùng nhập) thành một khóa mã hóa 32 byte
// bằng Argon2id — thuật toán KDF chống brute-force tốt nhất hiện nay,
// khuyến nghị bởi OWASP cho việc lưu trữ/derive key từ password.

use argon2::{Argon2, Algorithm, Version, Params};
use rand::RngCore;
use rand::rngs::OsRng;
use zeroize::Zeroize;

pub const SALT_LEN: usize = 16;
pub const KEY_LEN: usize = 32; // AES-256 cần key 32 byte

/// Tham số Argon2id. Các con số này ảnh hưởng trực tiếp tới độ an toàn
/// và thời gian unlock (nên test trên máy yếu nhất bạn định hỗ trợ).
///
/// - memory_cost: 19456 KB (~19 MB) — khuyến nghị OWASP 2023 tối thiểu
/// - time_cost: 2 iterations
/// - parallelism: 1 (an toàn cho desktop, không cần đa luồng)
///
/// Lưu các tham số này cùng dữ liệu vault (không phải bí mật) để có thể
/// nâng cấp độ khó trong tương lai mà vẫn giải mã được dữ liệu cũ.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct KdfParams {
    pub memory_cost_kb: u32,
    pub time_cost: u32,
    pub parallelism: u32,
}

impl Default for KdfParams {
    fn default() -> Self {
        Self {
            memory_cost_kb: 19_456,
            time_cost: 2,
            parallelism: 1,
        }
    }
}

/// Sinh salt ngẫu nhiên mới (dùng khi setup vault lần đầu).
/// Salt KHÔNG phải bí mật — có thể lưu plaintext cùng dữ liệu.
pub fn generate_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Derive một khóa 32 byte (KEK - Key Encryption Key) từ master password + salt.
///
/// Trả về Result vì Argon2 có thể fail nếu params không hợp lệ hoặc
/// hệ thống không đủ RAM cho memory_cost đã cấu hình.
pub fn derive_key(
    password: &str,
    salt: &[u8],
    params: &KdfParams,
) -> Result<[u8; KEY_LEN], String> {
    let argon2_params = Params::new(
        params.memory_cost_kb,
        params.time_cost,
        params.parallelism,
        Some(KEY_LEN),
    )
    .map_err(|e| crate::error::internal_error("kdf_invalid_params", e))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon2_params);

    let mut output_key = [0u8; KEY_LEN];
    argon2
        .hash_password_into(password.as_bytes(), salt, &mut output_key)
        .map_err(|e| crate::error::internal_error("kdf_derive_key", e))?;

    Ok(output_key)
}

/// Wrapper an toàn: xóa password khỏi memory ngay sau khi dùng xong.
/// Gọi hàm này thay vì derive_key trực tiếp khi có thể.
pub fn derive_key_and_wipe(
    mut password: String,
    salt: &[u8],
    params: &KdfParams,
) -> Result<[u8; KEY_LEN], String> {
    let result = derive_key(&password, salt, params);
    password.zeroize(); // xóa nội dung password khỏi RAM
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_password_same_salt_gives_same_key() {
        let salt = generate_salt();
        let params = KdfParams::default();
        let k1 = derive_key("mat_khau_test_123", &salt, &params).unwrap();
        let k2 = derive_key("mat_khau_test_123", &salt, &params).unwrap();
        assert_eq!(k1, k2);
    }

    #[test]
    fn different_salt_gives_different_key() {
        let params = KdfParams::default();
        let salt1 = generate_salt();
        let salt2 = generate_salt();
        let k1 = derive_key("cung_mat_khau", &salt1, &params).unwrap();
        let k2 = derive_key("cung_mat_khau", &salt2, &params).unwrap();
        assert_ne!(k1, k2);
    }
}
