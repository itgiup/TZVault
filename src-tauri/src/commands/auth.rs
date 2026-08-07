// src-tauri/src/commands/auth.rs

use tauri::State;
use std::sync::Mutex;
use crate::crypto;
use crate::vault::state::VaultState;
use crate::vault::storage::Storage;

/// Storage được quản lý qua Mutex vì rusqlite::Connection không tự Sync.
/// Khởi tạo 1 lần khi app start (xem main.rs) và inject qua app.manage().
pub struct StorageState(pub Mutex<Storage>);

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Gọi lần đầu tiên khi người dùng mở app và chưa có vault nào.
#[tauri::command]
pub fn cmd_setup_vault(
    password: String,
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<(), String> {
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;

    if db.has_vault()? {
        return Err("ERR_VAULT_EXISTS".to_string());
    }

    // Yêu cầu tối thiểu về độ mạnh password ở tầng backend
    // (frontend cũng nên check để UX tốt hơn, nhưng backend là chốt chặn thật sự)
    if password.len() < 12 {
        return Err("ERR_PASSWORD_TOO_SHORT".to_string());
    }

    let (setup_result, vault_key) = crypto::setup_vault(&password)?;

    let kdf_params_json = serde_json::to_string(&setup_result.kdf_params)
        .map_err(|e| crate::error::internal_error("encode_kdf_params", e))?;

    db.save_vault_meta(
        &setup_result.salt,
        &setup_result.encrypted_vault_key,
        &setup_result.vault_key_nonce,
        &kdf_params_json,
    )?;

    vault_state.set_vault_key(vault_key);
    db.log_action("setup", None, now_ts())?;

    Ok(())
}

/// Kiểm tra xem vault đã được setup chưa — frontend dùng để quyết định
/// hiển thị màn hình Setup hay Unlock khi app khởi động.
#[tauri::command]
pub fn cmd_vault_exists(storage: State<StorageState>) -> Result<bool, String> {
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;
    db.has_vault()
}

#[tauri::command]
pub fn cmd_unlock_vault(
    password: String,
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<(), String> {
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;

    let (salt, encrypted_vault_key, nonce_vec, kdf_params_json) = db.load_vault_meta()?;

    let params: crypto::kdf::KdfParams = serde_json::from_str(&kdf_params_json)
        .map_err(|e| crate::error::internal_error("decode_kdf_params", e))?;

    let nonce: [u8; crypto::cipher::NONCE_LEN] = nonce_vec
        .try_into()
        .map_err(|_| crate::error::internal_error_msg("nonce_length_mismatch"))?;

    // unlock_vault trả về Err nếu password sai — KHÔNG lộ chi tiết gì thêm
    // (không nói "sai password" khác với "dữ liệu hỏng" để tránh dò thông tin)
    let vault_key = crypto::unlock_vault(&password, &salt, &encrypted_vault_key, &nonce, &params)
        .map_err(|_| "ERR_INVALID_PASSWORD".to_string())?;

    vault_state.set_vault_key(vault_key);
    db.log_action("unlock", None, now_ts())?;

    Ok(())
}

#[tauri::command]
pub fn cmd_lock_vault(vault_state: State<VaultState>) -> Result<(), String> {
    vault_state.lock();
    Ok(())
}

#[tauri::command]
pub fn cmd_is_unlocked(vault_state: State<VaultState>) -> Result<bool, String> {
    Ok(vault_state.is_unlocked())
}

#[tauri::command]
pub fn cmd_change_password(
    old_password: String,
    new_password: String,
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<(), String> {
    if new_password.len() < 12 {
        return Err("ERR_PASSWORD_TOO_SHORT".to_string());
    }

    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;
    let (salt, encrypted_vault_key, nonce_vec, kdf_params_json) = db.load_vault_meta()?;

    let params: crypto::kdf::KdfParams = serde_json::from_str(&kdf_params_json)
        .map_err(|e| crate::error::internal_error("decode_kdf_params", e))?;
    let nonce: [u8; crypto::cipher::NONCE_LEN] = nonce_vec
        .try_into()
        .map_err(|_| crate::error::internal_error_msg("nonce_length_mismatch"))?;

    let new_setup = crypto::change_master_password(
        &old_password,
        &new_password,
        &salt,
        &encrypted_vault_key,
        &nonce,
        &params,
    )
    .map_err(|_| "ERR_INVALID_PASSWORD".to_string())?;

    let new_params_json = serde_json::to_string(&new_setup.kdf_params)
        .map_err(|e| crate::error::internal_error("encode_kdf_params", e))?;

    db.save_vault_meta(
        &new_setup.salt,
        &new_setup.encrypted_vault_key,
        &new_setup.vault_key_nonce,
        &new_params_json,
    )?;

    // Bắt buộc unlock lại bằng password mới sau khi đổi, cho chắc chắn
    vault_state.lock();
    db.log_action("change_password", None, now_ts())?;

    Ok(())
}

/// Cho phép người dùng chỉnh thời gian auto-lock từ màn hình Settings.
/// Giá trị chỉ tồn tại trong RAM của phiên hiện tại (không cần lưu DB
/// vì không phải dữ liệu nhạy cảm — mặc định lại 5 phút mỗi lần mở app
/// là lựa chọn an toàn hơn).
#[tauri::command]
pub fn cmd_set_auto_lock_timeout(
    seconds: u64,
    vault_state: State<VaultState>,
) -> Result<(), String> {
    if seconds < 30 {
        return Err("ERR_TIMEOUT_TOO_SHORT".to_string());
    }
    vault_state.set_auto_lock_timeout(seconds);
    Ok(())
}
