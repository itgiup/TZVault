// src-tauri/src/commands/auth.rs

use tauri::State;
use std::sync::Mutex;
use crate::crypto;
use crate::vault::state::VaultState;
use crate::vault::storage::Storage;

/// Storage được quản lý qua Mutex vì rusqlite::Connection không tự Sync.
/// Khởi tạo 1 lần khi app start (xem main.rs) và inject qua app.manage().
pub struct StorageState(pub Mutex<Storage>);

/// Đường dẫn app_data_dir của OS — cần giữ lại để đọc/ghi app_config.json
/// (chứa lựa chọn "đang dùng vault.db ở đâu") mỗi khi người dùng đổi vị
/// trí vault. Quản lý qua app.manage() giống StorageState.
pub struct AppPaths {
    pub app_data_dir: std::path::PathBuf,
}

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

    // Tự động migrate metadata (name/tags/notes) của các key được tạo
    // trước bản vá mã hóa metadata — trong suốt với người dùng, chỉ chạy
    // đúng 1 lần cho mỗi key (sau khi migrate, cột plaintext cũ trống,
    // list_legacy_plaintext_rows sẽ không còn thấy nó lần sau).
    migrate_legacy_metadata(&db, &vault_key)?;

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
/// Xuất toàn bộ vault (đã mã hóa, kể cả metadata) ra 1 file người dùng
/// tự chọn nơi lưu (frontend dùng dialog:save để lấy dest_path). File
/// export ra là 1 bản SQLite độc lập, tự chứa mọi thứ cần thiết —
/// mang sang máy khác và import lại là dùng được ngay, không cần thêm gì.
#[tauri::command]
pub fn cmd_export_vault(dest_path: String, storage: State<StorageState>) -> Result<(), String> {
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;
    db.export_to(&dest_path)
}

/// Import 1 file vault đã export từ máy khác — CHỈ cho phép khi máy này
/// CHƯA có vault nào (an toàn: không có gì để mất nếu import lỗi/file
/// không hợp lệ). Sau khi import, frontend cần gọi lại cmd_vault_exists
/// / checkVaultStatus() để chuyển sang màn hình Unlock.
#[tauri::command]
pub fn cmd_import_vault(
    src_path: String,
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<(), String> {
    let mut db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;

    if db.has_vault()? {
        return Err("ERR_VAULT_EXISTS".to_string());
    }

    let dest_path = db.db_path().to_string();

    std::fs::copy(&src_path, &dest_path)
        .map_err(|e| crate::error::internal_error("import_copy_file", e))?;

    // Bắt buộc mở lại connection — connection cũ không tự thấy file vừa
    // bị ghi đè ở tầng hệ điều hành.
    db.reload()?;

    if !db.has_vault()? {
        // File người dùng chọn không phải file vault hợp lệ (hoặc rỗng).
        // Vault vốn dĩ chưa tồn tại trước đó nên không mất gì, chỉ cần
        // báo lỗi rõ ràng để người dùng chọn lại đúng file.
        return Err("ERR_INVALID_VAULT_FILE".to_string());
    }

    vault_state.lock(); // đảm bảo trạng thái sạch, chưa unlock gì cả

    Ok(())
}

/// Lấy đường dẫn file vault đang dùng — hiển thị trong Settings để người
/// dùng biết dữ liệu của mình thực sự nằm ở đâu.
#[tauri::command]
pub fn cmd_get_db_path(storage: State<StorageState>) -> Result<String, String> {
    let db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;
    Ok(db.db_path().to_string())
}

/// Đổi vault đang dùng sang 1 vị trí khác. 2 chế độ:
/// - "move": copy vault HIỆN TẠI sang `new_path` rồi dùng vị trí đó từ giờ
///           (file cũ ở vị trí cũ vẫn còn nguyên, không tự xóa).
/// - "link": KHÔNG copy gì cả — chỉ trỏ thẳng tới 1 file vault đã có sẵn
///           tại `new_path` (VD file đang nằm trong thư mục đồng bộ).
///
/// Cả 2 chế độ đều lưu lựa chọn vào app_config.json để lần mở app sau tự
/// nhớ đúng vị trí này, và đều khóa lại vault (bắt buộc unlock lại bằng
/// master password của vault MỚI — có thể khác với vault cũ).
#[tauri::command]
pub fn cmd_set_db_path(
    new_path: String,
    mode: String,
    app_paths: State<AppPaths>,
    storage: State<StorageState>,
    vault_state: State<VaultState>,
) -> Result<(), String> {
    let mut db = storage.0.lock().map_err(|_| crate::error::internal_error_msg("storage_mutex_lock"))?;

    match mode.as_str() {
        "move" => {
            // VACUUM INTO tạo bản sao nhất quán tại vị trí mới trước,
            // rồi mới chuyển Storage sang trỏ vào đó.
            db.export_to(&new_path)?;
            db.switch_path(&new_path)?;
        }
        "link" => {
            // Lưu lại vị trí CŨ trước khi thử switch — nếu file mới không
            // hợp lệ, phải trỏ lại đúng chỗ này, tránh app "lạc mất" vault gốc.
            let old_path = db.db_path().to_string();
            db.switch_path(&new_path)?;
            if !db.has_vault()? {
                db.switch_path(&old_path)?;
                return Err("ERR_INVALID_VAULT_FILE".to_string());
            }
        }
        _ => return Err(crate::error::internal_error_msg("invalid_set_db_path_mode")),
    }

    crate::app_config::save_app_config(
        &app_paths.app_data_dir,
        &crate::app_config::AppConfig { db_path: Some(new_path) },
    )?;

    vault_state.lock(); // vault mới (hoặc vị trí mới) -> bắt buộc unlock lại

    Ok(())
}

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

/// Migrate metadata (name/key_type/tags/notes) của các key được tạo
/// trước bản vá mã hóa metadata. Chạy ngay sau khi unlock thành công vì
/// đây là lúc DUY NHẤT có vault_key trong tay để mã hóa lại.
///
/// An toàn khi gọi mỗi lần unlock (kể cả vault không có key nào cần
/// migrate) — list_legacy_plaintext_rows() chỉ trả về key có
/// metadata_ciphertext = NULL, sau khi migrate xong sẽ không còn xuất
/// hiện ở lần gọi sau.
fn migrate_legacy_metadata(db: &crate::vault::storage::Storage, vault_key: &[u8; 32]) -> Result<(), String> {
    let legacy_rows = db.list_legacy_plaintext_rows()?;

    for row in legacy_rows {
        let tags: Vec<String> = serde_json::from_str(&row.tags).unwrap_or_default();
        let metadata = crate::crypto::KeyMetadata {
            name: row.name,
            key_type: row.key_type,
            tags,
            notes: row.notes,
        };

        let encrypted = crate::crypto::encrypt_metadata(vault_key, &row.id, &metadata)?;
        db.finish_migrate_key_metadata(&row.id, &encrypted.ciphertext, &encrypted.nonce)?;
    }

    Ok(())
}
