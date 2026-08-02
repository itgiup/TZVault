// src-tauri/src/vault/storage.rs
//
// Lớp duy nhất được phép chạm vào SQLite. Không xử lý crypto ở đây —
// chỉ đọc/ghi dữ liệu đã mã hóa (ciphertext) do lớp crypto đưa xuống.

use rusqlite::{Connection, params};
use crate::models::{StoredKeyRow, KeySummary};

pub struct Storage {
    conn: Connection,
}

impl Storage {
    /// Mở (hoặc tạo mới) file database tại đường dẫn chỉ định và đảm bảo
    /// schema đã tồn tại.
    ///
    /// LƯU Ý PRODUCTION: dùng `Connection::open_with_flags` kết hợp
    /// SQLCipher (feature "bundled-sqlcipher" của rusqlite) để mã hóa
    /// toàn bộ file .db ở tầng đĩa, cộng thêm với mã hóa AES-GCM ở tầng
    /// ứng dụng đã có. Ở đây dùng SQLite thường để dễ test trong mọi môi trường.
    pub fn open(db_path: &str) -> Result<Self, String> {
        let conn = Connection::open(db_path).map_err(|e| crate::error::internal_error("db_open", e))?;
        let storage = Storage { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    /// Dùng cho unit test: DB tồn tại hoàn toàn trong RAM, không ghi ra đĩa.
    pub fn open_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| crate::error::internal_error("db_open", e))?;
        let storage = Storage { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> Result<(), String> {
        self.conn
            .execute_batch(
                r#"
                CREATE TABLE IF NOT EXISTS vault_meta (
                    id INTEGER PRIMARY KEY CHECK (id = 1), -- chỉ cho phép 1 dòng
                    salt BLOB NOT NULL,
                    encrypted_vault_key BLOB NOT NULL,
                    vault_key_nonce BLOB NOT NULL,
                    kdf_params TEXT NOT NULL
                );

                CREATE TABLE IF NOT EXISTS stored_keys (
                    id TEXT PRIMARY KEY,
                    name TEXT NOT NULL,
                    key_type TEXT NOT NULL,
                    ciphertext BLOB NOT NULL,
                    nonce BLOB NOT NULL,
                    tags TEXT NOT NULL DEFAULT '[]',
                    notes TEXT,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );

                CREATE TABLE IF NOT EXISTS audit_log (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    action TEXT NOT NULL,
                    key_id TEXT,
                    timestamp INTEGER NOT NULL
                );
                "#,
            )
            .map_err(|e| crate::error::internal_error("db_init_schema", e))
    }

    // ---------- vault_meta ----------

    pub fn has_vault(&self) -> Result<bool, String> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM vault_meta", [], |row| row.get(0))
            .map_err(|e| crate::error::internal_error("db_check_vault_exists", e))?;
        Ok(count > 0)
    }

    pub fn save_vault_meta(
        &self,
        salt: &[u8],
        encrypted_vault_key: &[u8],
        vault_key_nonce: &[u8],
        kdf_params_json: &str,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO vault_meta (id, salt, encrypted_vault_key, vault_key_nonce, kdf_params)
                 VALUES (1, ?1, ?2, ?3, ?4)",
                params![salt, encrypted_vault_key, vault_key_nonce, kdf_params_json],
            )
            .map_err(|e| crate::error::internal_error("db_save_vault_meta", e))?;
        Ok(())
    }

    /// Trả về (salt, encrypted_vault_key, vault_key_nonce, kdf_params_json)
    pub fn load_vault_meta(&self) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>, String), String> {
        self.conn
            .query_row(
                "SELECT salt, encrypted_vault_key, vault_key_nonce, kdf_params FROM vault_meta WHERE id = 1",
                [],
                |row| {
                    Ok((
                        row.get::<_, Vec<u8>>(0)?,
                        row.get::<_, Vec<u8>>(1)?,
                        row.get::<_, Vec<u8>>(2)?,
                        row.get::<_, String>(3)?,
                    ))
                },
            )
            .map_err(|e| crate::error::internal_error("db_load_vault_meta", e))
    }

    // ---------- stored_keys ----------

    pub fn insert_key(&self, row: &StoredKeyRow) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO stored_keys (id, name, key_type, ciphertext, nonce, tags, notes, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    row.id, row.name, row.key_type, row.ciphertext, row.nonce,
                    row.tags, row.notes, row.created_at, row.updated_at
                ],
            )
            .map_err(|e| crate::error::internal_error("db_insert_key", e))?;
        Ok(())
    }

    pub fn get_key_row(&self, id: &str) -> Result<StoredKeyRow, String> {
        self.conn
            .query_row(
                "SELECT id, name, key_type, ciphertext, nonce, tags, notes, created_at, updated_at
                 FROM stored_keys WHERE id = ?1",
                params![id],
                |row| {
                    Ok(StoredKeyRow {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        key_type: row.get(2)?,
                        ciphertext: row.get(3)?,
                        nonce: row.get(4)?,
                        tags: row.get(5)?,
                        notes: row.get(6)?,
                        created_at: row.get(7)?,
                        updated_at: row.get(8)?,
                    })
                },
            )
            .map_err(|e| crate::error::internal_error("db_get_key", e))
    }

    /// Danh sách rút gọn — KHÔNG lấy ciphertext, tránh giữ dữ liệu mã hóa
    /// trong bộ nhớ khi không cần thiết.
    pub fn list_key_summaries(&self) -> Result<Vec<KeySummary>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, key_type, tags, notes, created_at, updated_at FROM stored_keys ORDER BY updated_at DESC")
            .map_err(|e| crate::error::internal_error("db_prepare_query", e))?;

        let rows = stmt
            .query_map([], |row| {
                let tags_json: String = row.get(3)?;
                let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
                Ok(KeySummary {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    key_type: row.get(2)?,
                    tags,
                    notes: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| crate::error::internal_error("db_query_map", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| crate::error::internal_error("db_collect_rows", e))
    }

    pub fn delete_key(&self, id: &str) -> Result<(), String> {
        let affected = self
            .conn
            .execute("DELETE FROM stored_keys WHERE id = ?1", params![id])
            .map_err(|e| crate::error::internal_error("db_delete_key", e))?;
        if affected == 0 {
            return Err("ERR_KEY_NOT_FOUND".to_string());
        }
        Ok(())
    }

    // ---------- audit_log ----------

    pub fn log_action(&self, action: &str, key_id: Option<&str>, timestamp: i64) -> Result<(), String> {
        self.conn
            .execute(
                "INSERT INTO audit_log (action, key_id, timestamp) VALUES (?1, ?2, ?3)",
                params![action, key_id, timestamp],
            )
            .map_err(|e| crate::error::internal_error("db_log_action", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_row(id: &str) -> StoredKeyRow {
        StoredKeyRow {
            id: id.to_string(),
            name: "Test Key".to_string(),
            key_type: "ssh".to_string(),
            ciphertext: vec![1, 2, 3, 4],
            nonce: vec![5, 6, 7],
            tags: "[\"prod\"]".to_string(),
            notes: Some("ghi chu".to_string()),
            created_at: 1000,
            updated_at: 1000,
        }
    }

    #[test]
    fn setup_and_load_vault_meta() {
        let storage = Storage::open_in_memory().unwrap();
        assert!(!storage.has_vault().unwrap());

        storage
            .save_vault_meta(b"salt123", b"enc_vault_key", b"nonce123", "{}")
            .unwrap();

        assert!(storage.has_vault().unwrap());

        let (salt, evk, nonce, params) = storage.load_vault_meta().unwrap();
        assert_eq!(salt, b"salt123");
        assert_eq!(evk, b"enc_vault_key");
        assert_eq!(nonce, b"nonce123");
        assert_eq!(params, "{}");
    }

    #[test]
    fn insert_and_get_key() {
        let storage = Storage::open_in_memory().unwrap();
        let row = sample_row("key-1");
        storage.insert_key(&row).unwrap();

        let fetched = storage.get_key_row("key-1").unwrap();
        assert_eq!(fetched.name, "Test Key");
        assert_eq!(fetched.ciphertext, vec![1, 2, 3, 4]);
    }

    #[test]
    fn list_summaries_does_not_error_and_hides_no_secret_fields() {
        let storage = Storage::open_in_memory().unwrap();
        storage.insert_key(&sample_row("key-1")).unwrap();
        storage.insert_key(&sample_row("key-2")).unwrap();

        let summaries = storage.list_key_summaries().unwrap();
        assert_eq!(summaries.len(), 2);
        assert_eq!(summaries[0].tags, vec!["prod".to_string()]);
    }

    #[test]
    fn delete_key_removes_it() {
        let storage = Storage::open_in_memory().unwrap();
        storage.insert_key(&sample_row("key-1")).unwrap();
        storage.delete_key("key-1").unwrap();

        let result = storage.get_key_row("key-1");
        assert!(result.is_err());
    }

    #[test]
    fn delete_nonexistent_key_errors() {
        let storage = Storage::open_in_memory().unwrap();
        let result = storage.delete_key("khong-ton-tai");
        assert!(result.is_err());
    }
}
