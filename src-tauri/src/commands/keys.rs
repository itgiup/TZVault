// src-tauri/src/commands/keys.rs

use tauri::State;
use uuid::Uuid;
use crate::crypto;
use crate::crypto::KeyMetadata;
use crate::vault::state::VaultState;
use crate::models::{KeySummary, KeyWithSecret, NewKeyInput, StoredKeyRow};
use crate::commands::auth::StorageState;

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn row_to_nonce(bytes: &[u8]) -> Result<[u8; crypto::cipher::NONCE_LEN], String> {
    bytes
        .try_into()
        .map_err(|_| crate::error::internal_error_msg("nonce_length_mismatch"))
}

/// Giải mã metadata (name/key_type/tags/notes) của 1 row -> KeyMetadata.
fn decrypt_row_metadata(
    vault_key: &[u8; 32],
    id: &str,
    metadata_ciphertext: &[u8],
    metadata_nonce: &[u8],
) -> Result<KeyMetadata, String> {
    let nonce = row_to_nonce(metadata_nonce)?;
    crypto::decrypt_metadata(vault_key, id, metadata_ciphertext, &nonce)
}

/// Thêm 1 private key mới vào vault. Trả về id vừa tạo.
/// Cả nội dung key VÀ metadata (name/type/tags/notes) đều được mã hóa
/// bằng Vault Key trước khi lưu — không trường nào là plaintext trong DB.
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
    let ts = now_ts();

    let metadata = KeyMetadata {
        name: input.name,
        key_type: input.key_type,
        tags: input.tags,
        notes: input.notes,
    };
    let encrypted_metadata = crypto::encrypt_metadata(&vault_key, &id, &metadata)?;

    let (ciphertext, nonce, has_extra_password, extra_salt, extra_nonce) = match &input.extra_password {
        Some(extra_password) if !extra_password.is_empty() => {
            let (outer, extra_salt, extra_nonce) = crypto::encrypt_key_value_with_extra_password(
                &vault_key,
                &id,
                &input.secret_value,
                extra_password,
            )?;
            (outer.ciphertext, outer.nonce.to_vec(), true, Some(extra_salt.to_vec()), Some(extra_nonce.to_vec()))
        }
        _ => {
            let encrypted = crypto::encrypt_key_value(&vault_key, &id, &input.secret_value)?;
            (encrypted.ciphertext, encrypted.nonce.to_vec(), false, None, None)
        }
    };

    let row = StoredKeyRow {
        id: id.clone(),
        metadata_ciphertext: encrypted_metadata.ciphertext,
        metadata_nonce: encrypted_metadata.nonce.to_vec(),
        ciphertext,
        nonce,
        created_at: ts,
        updated_at: ts,
        has_extra_password,
        extra_salt,
        extra_nonce,
    };

    db.insert_key(&row)?;
    db.log_action("add_key", Some(&id), ts)?;

    Ok(id)
}

/// Lấy danh sách key (KHÔNG bao gồm giá trị bí mật) — dùng cho màn hình
/// list. Metadata được giải mã ở đây (cần vault_key), storage.rs chỉ trả
/// về ciphertext thô.
///
/// LƯU Ý: nếu 1 row cụ thể decrypt lỗi (VD dữ liệu từ vault khác lẫn vào,
/// hoặc bị hỏng), CHỦ ĐỘNG BỎ QUA row đó thay vì làm fail toàn bộ danh
/// sách — người dùng vẫn thấy được các key hợp lệ khác, thay vì màn hình
/// trắng không rõ lý do. Lỗi chi tiết vẫn được log ra stderr để debug.
#[tauri::command]
pub fn cmd_list_keys(
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<Vec<KeySummary>, String> {
    let vault_key = vault_state.get_vault_key()?;
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;

    let meta_rows = db.list_key_meta_rows()?;

    let summaries = meta_rows
        .into_iter()
        .filter_map(|row| {
            match decrypt_row_metadata(&vault_key, &row.id, &row.metadata_ciphertext, &row.metadata_nonce) {
                Ok(metadata) => Some(KeySummary {
                    id: row.id,
                    name: metadata.name,
                    key_type: metadata.key_type,
                    tags: metadata.tags,
                    notes: metadata.notes,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                    has_extra_password: row.has_extra_password,
                }),
                Err(_) => {
                    eprintln!(
                        "[internal error] cmd_list_keys: bỏ qua key id={} vì decrypt metadata thất bại \
                         (không khớp vault_key hiện tại, hoặc dữ liệu hỏng)",
                        row.id
                    );
                    None
                }
            }
        })
        .collect();

    Ok(summaries)
}

/// Lấy giá trị thật của 1 key — chỉ dùng cho key KHÔNG có mật khẩu riêng.
/// Nếu key có `has_extra_password = true`, trả về ERR_EXTRA_PASSWORD_REQUIRED
/// để frontend biết cần hỏi thêm mật khẩu riêng rồi gọi cmd_unlock_key_with_password.
#[tauri::command]
pub fn cmd_get_key_secret(
    id: String,
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<KeyWithSecret, String> {
    let vault_key = vault_state.get_vault_key()?;
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;

    let row = db.get_key_row(&id)?;

    if row.has_extra_password {
        return Err("ERR_EXTRA_PASSWORD_REQUIRED".to_string());
    }

    let metadata = decrypt_row_metadata(&vault_key, &row.id, &row.metadata_ciphertext, &row.metadata_nonce)?;
    let nonce = row_to_nonce(&row.nonce)?;
    let secret = crypto::decrypt_key_value(&vault_key, &row.id, &row.ciphertext, &nonce)?;

    db.log_action("view_key", Some(&id), now_ts())?;

    Ok(KeyWithSecret {
        id: row.id,
        name: metadata.name,
        key_type: metadata.key_type,
        secret_value: secret,
        tags: metadata.tags,
        notes: metadata.notes,
    })
}

/// Mở 1 key CÓ mật khẩu riêng — cần cả vault đang unlock (master password)
/// VÀ đúng mật khẩu riêng của key này.
#[tauri::command]
pub fn cmd_unlock_key_with_password(
    id: String,
    key_password: String,
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<KeyWithSecret, String> {
    let vault_key = vault_state.get_vault_key()?;
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;

    let row = db.get_key_row(&id)?;

    if !row.has_extra_password {
        return Err(crate::error::internal_error_msg("unlock_key_password_on_unprotected_key"));
    }

    let extra_salt = row.extra_salt.as_deref().ok_or_else(|| {
        crate::error::internal_error_msg("missing_extra_salt_for_protected_key")
    })?;
    let extra_nonce_bytes = row.extra_nonce.as_deref().ok_or_else(|| {
        crate::error::internal_error_msg("missing_extra_nonce_for_protected_key")
    })?;
    let extra_nonce = row_to_nonce(extra_nonce_bytes)?;
    let nonce = row_to_nonce(&row.nonce)?;

    let secret = crypto::decrypt_key_value_with_extra_password(
        &vault_key,
        &row.id,
        &row.ciphertext,
        &nonce,
        extra_salt,
        &extra_nonce,
        &key_password,
    )?;

    let metadata = decrypt_row_metadata(&vault_key, &row.id, &row.metadata_ciphertext, &row.metadata_nonce)?;

    db.log_action("view_key_extra_password", Some(&id), now_ts())?;

    Ok(KeyWithSecret {
        id: row.id,
        name: metadata.name,
        key_type: metadata.key_type,
        secret_value: secret,
        tags: metadata.tags,
        notes: metadata.notes,
    })
}

/// Thêm mật khẩu riêng cho 1 key HIỆN CHƯA có mật khẩu riêng.
#[tauri::command]
pub fn cmd_add_key_password(
    id: String,
    new_key_password: String,
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<(), String> {
    if new_key_password.len() < 8 {
        return Err("ERR_KEY_PASSWORD_TOO_SHORT".to_string());
    }

    let vault_key = vault_state.get_vault_key()?;
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;

    let row = db.get_key_row(&id)?;
    if row.has_extra_password {
        return Err("ERR_KEY_ALREADY_PROTECTED".to_string());
    }

    let nonce = row_to_nonce(&row.nonce)?;
    let plaintext = crypto::decrypt_key_value(&vault_key, &row.id, &row.ciphertext, &nonce)?;

    let (outer, extra_salt, extra_nonce) =
        crypto::encrypt_key_value_with_extra_password(&vault_key, &row.id, &plaintext, &new_key_password)?;

    db.update_key_encryption(
        &id,
        &outer.ciphertext,
        &outer.nonce,
        true,
        Some(&extra_salt),
        Some(&extra_nonce),
        now_ts(),
    )?;
    db.log_action("add_key_password", Some(&id), now_ts())?;

    Ok(())
}

/// Gỡ mật khẩu riêng khỏi 1 key — cần đúng mật khẩu riêng hiện tại.
#[tauri::command]
pub fn cmd_remove_key_password(
    id: String,
    current_key_password: String,
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<(), String> {
    let vault_key = vault_state.get_vault_key()?;
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;

    let row = db.get_key_row(&id)?;
    if !row.has_extra_password {
        return Err(crate::error::internal_error_msg("remove_password_on_unprotected_key"));
    }

    let extra_salt = row.extra_salt.as_deref().ok_or_else(|| {
        crate::error::internal_error_msg("missing_extra_salt_for_protected_key")
    })?;
    let extra_nonce_bytes = row.extra_nonce.as_deref().ok_or_else(|| {
        crate::error::internal_error_msg("missing_extra_nonce_for_protected_key")
    })?;
    let extra_nonce = row_to_nonce(extra_nonce_bytes)?;
    let nonce = row_to_nonce(&row.nonce)?;

    let plaintext = crypto::decrypt_key_value_with_extra_password(
        &vault_key,
        &row.id,
        &row.ciphertext,
        &nonce,
        extra_salt,
        &extra_nonce,
        &current_key_password,
    )?;

    let encrypted = crypto::encrypt_key_value(&vault_key, &row.id, &plaintext)?;

    db.update_key_encryption(&id, &encrypted.ciphertext, &encrypted.nonce, false, None, None, now_ts())?;
    db.log_action("remove_key_password", Some(&id), now_ts())?;

    Ok(())
}

/// Đổi mật khẩu riêng của 1 key — cần đúng mật khẩu riêng hiện tại.
#[tauri::command]
pub fn cmd_change_key_password(
    id: String,
    current_key_password: String,
    new_key_password: String,
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<(), String> {
    if new_key_password.len() < 8 {
        return Err("ERR_KEY_PASSWORD_TOO_SHORT".to_string());
    }

    let vault_key = vault_state.get_vault_key()?;
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;

    let row = db.get_key_row(&id)?;
    if !row.has_extra_password {
        return Err(crate::error::internal_error_msg("change_password_on_unprotected_key"));
    }

    let extra_salt = row.extra_salt.as_deref().ok_or_else(|| {
        crate::error::internal_error_msg("missing_extra_salt_for_protected_key")
    })?;
    let extra_nonce_bytes = row.extra_nonce.as_deref().ok_or_else(|| {
        crate::error::internal_error_msg("missing_extra_nonce_for_protected_key")
    })?;
    let extra_nonce = row_to_nonce(extra_nonce_bytes)?;
    let nonce = row_to_nonce(&row.nonce)?;

    let plaintext = crypto::decrypt_key_value_with_extra_password(
        &vault_key,
        &row.id,
        &row.ciphertext,
        &nonce,
        extra_salt,
        &extra_nonce,
        &current_key_password,
    )?;

    let (outer, new_extra_salt, new_extra_nonce) =
        crypto::encrypt_key_value_with_extra_password(&vault_key, &row.id, &plaintext, &new_key_password)?;

    db.update_key_encryption(
        &id,
        &outer.ciphertext,
        &outer.nonce,
        true,
        Some(&new_extra_salt),
        Some(&new_extra_nonce),
        now_ts(),
    )?;
    db.log_action("change_key_password", Some(&id), now_ts())?;

    Ok(())
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
