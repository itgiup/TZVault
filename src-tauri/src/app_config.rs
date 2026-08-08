// src-tauri/src/app_config.rs
//
// Config nhỏ, KHÔNG nhạy cảm (chỉ 1 đường dẫn file) — lưu vault hiện tại
// đang được app trỏ tới đâu. Tách biệt hoàn toàn khỏi vault.db (không
// mã hóa, vì bản thân đường dẫn không phải bí mật cần bảo vệ).
//
// Nếu file config này không tồn tại (lần đầu mở app) -> dùng vị trí mặc
// định app_data_dir/vault.db như trước giờ.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Đường dẫn tuyệt đối tới file vault.db đang dùng. None = dùng mặc định.
    pub db_path: Option<String>,
}

fn config_file_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("app_config.json")
}

pub fn load_app_config(app_data_dir: &Path) -> AppConfig {
    let path = config_file_path(app_data_dir);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save_app_config(app_data_dir: &Path, config: &AppConfig) -> Result<(), String> {
    let path = config_file_path(app_data_dir);
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| crate::error::internal_error("encode_app_config", e))?;
    std::fs::write(&path, json).map_err(|e| crate::error::internal_error("write_app_config", e))?;
    Ok(())
}

/// Xác định vault.db thực sự cần mở khi app khởi động — ưu tiên đường
/// dẫn đã lưu trong config, fallback về vị trí mặc định.
pub fn resolve_db_path(app_data_dir: &Path) -> String {
    let config = load_app_config(app_data_dir);
    config
        .db_path
        .unwrap_or_else(|| app_data_dir.join("vault.db").to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_default_when_no_config_exists() {
        let dir = std::env::temp_dir().join(format!("app_config_test_default_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let resolved = resolve_db_path(&dir);
        assert_eq!(resolved, dir.join("vault.db").to_string_lossy());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn save_then_resolve_custom_path() {
        let dir = std::env::temp_dir().join(format!("app_config_test_custom_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        save_app_config(&dir, &AppConfig { db_path: Some("/custom/place/vault.db".to_string()) }).unwrap();

        let resolved = resolve_db_path(&dir);
        assert_eq!(resolved, "/custom/place/vault.db");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn corrupted_config_file_falls_back_to_default() {
        let dir = std::env::temp_dir().join(format!("app_config_test_corrupt_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(config_file_path(&dir), "khong-phai-json-hop-le{{{").unwrap();

        let resolved = resolve_db_path(&dir);
        assert_eq!(resolved, dir.join("vault.db").to_string_lossy());

        std::fs::remove_dir_all(&dir).ok();
    }
}
