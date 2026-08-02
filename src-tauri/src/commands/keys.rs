// src-tauri/src/commands/keys.rs

use tauri::State;
use uuid::Uuid;
use crate::crypto;
use crate::vault::state::VaultState;
use crate::models::{KeySummary, KeyWithSecret, NewKeyInput, StoredKeyRow};
use crate::commands::auth::StorageState;

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Thêm 1 private key mới vào vault. Trả về id vừa tạo.
#[tauri::command]
pub fn cmd_add_key(
    input: NewKeyInput,
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<String, String> {
    let vault_key = vault_state.get_vault_key()?;
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;

    if input.name.trim().is_empty() {
        return Err("ERR_NAME_EMPTY".to_string());
    }
    if input.secret_value.is_empty() {
        return Err("ERR_SECRET_EMPTY".to_string());
    }

    let id = Uuid::new_v4().to_string();

    let encrypted = crypto::encrypt_key_value(&vault_key, &id, &input.secret_value)?;

    let tags_json = serde_json::to_string(&input.tags).map_err(|e| e.to_string())?;
    let ts = now_ts();

    let row = StoredKeyRow {
        id: id.clone(),
        name: input.name,
        key_type: input.key_type,
        ciphertext: encrypted.ciphertext,
        nonce: encrypted.nonce.to_vec(),
        tags: tags_json,
        notes: input.notes,
        created_at: ts,
        updated_at: ts,
    };

    db.insert_key(&row)?;
    db.log_action("add_key", Some(&id), ts)?;

    Ok(id)
}

/// Lấy danh sách key (KHÔNG bao gồm giá trị bí mật) — dùng cho màn hình list.
#[tauri::command]
pub fn cmd_list_keys(
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<Vec<KeySummary>, String> {
    // Vẫn cần vault unlocked để list, dù không giải mã gì —
    // tránh lộ cả metadata (tên key) khi vault đang khóa.
    vault_state.get_vault_key()?;
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;
    db.list_key_summaries()
}

/// Lấy giá trị thật của 1 key — chỉ gọi khi người dùng chủ động bấm
/// "Show" hoặc "Copy". Ghi audit log mỗi lần gọi.
#[tauri::command]
pub fn cmd_get_key_secret(
    id: String,
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<KeyWithSecret, String> {
    let vault_key = vault_state.get_vault_key()?;
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;

    let row = db.get_key_row(&id)?;

    let nonce: [u8; crypto::cipher::NONCE_LEN] = row
        .nonce
        .clone()
        .try_into()
        .map_err(|_| crate::error::internal_error_msg("nonce_length_mismatch"))?;

    let secret = crypto::decrypt_key_value(&vault_key, &row.id, &row.ciphertext, &nonce)?;

    let tags: Vec<String> = serde_json::from_str(&row.tags).unwrap_or_default();

    db.log_action("view_key", Some(&id), now_ts())?;

    Ok(KeyWithSecret {
        id: row.id,
        name: row.name,
        key_type: row.key_type,
        secret_value: secret,
        tags,
        notes: row.notes,
    })
}

#[tauri::command]
pub fn cmd_delete_key(
    id: String,
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<(), String> {
    vault_state.get_vault_key()?; // đảm bảo vault đang unlock
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;

    db.delete_key(&id)?;
    db.log_action("delete_key", Some(&id), now_ts())?;

    Ok(())
}
