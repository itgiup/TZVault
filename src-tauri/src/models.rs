// src-tauri/src/models.rs

use serde::{Deserialize, Serialize};

/// Loại private key được hỗ trợ.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum KeyType {
    Ssh,
    CryptoWallet,
    Pgp,
    ApiKey,
    Other,
}

impl KeyType {
    pub fn as_str(&self) -> &'static str {
        match self {
            KeyType::Ssh => "ssh",
            KeyType::CryptoWallet => "crypto_wallet",
            KeyType::Pgp => "pgp",
            KeyType::ApiKey => "api_key",
            KeyType::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "ssh" => KeyType::Ssh,
            "crypto_wallet" => KeyType::CryptoWallet,
            "pgp" => KeyType::Pgp,
            "api_key" => KeyType::ApiKey,
            _ => KeyType::Other,
        }
    }
}

/// Metadata của 1 key hiển thị ra ngoài UI — KHÔNG bao giờ chứa giá trị
/// key thật. Dùng cho danh sách (list view).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeySummary {
    pub id: String,
    pub name: String,
    pub key_type: String,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Dữ liệu đầy đủ (đã giải mã) trả về khi người dùng bấm "Show"/"Copy".
/// Chỉ tạo struct này ngay trước khi trả về frontend, không giữ lâu trong RAM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyWithSecret {
    pub id: String,
    pub name: String,
    pub key_type: String,
    pub secret_value: String,
    pub tags: Vec<String>,
    pub notes: Option<String>,
}

/// Dữ liệu đầu vào khi thêm key mới từ UI.
#[derive(Debug, Clone, Deserialize)]
pub struct NewKeyInput {
    pub name: String,
    pub key_type: String,
    pub secret_value: String,
    pub tags: Vec<String>,
    pub notes: Option<String>,
}

/// Bản ghi thô lấy từ DB (dữ liệu vẫn đang mã hóa).
pub struct StoredKeyRow {
    pub id: String,
    pub name: String,
    pub key_type: String,
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
    pub tags: String, // JSON string
    pub notes: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
