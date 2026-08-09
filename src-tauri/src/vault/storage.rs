// src-tauri/src/vault/storage.rs
//
// Lớp duy nhất được phép chạm vào SQLite. KHÔNG xử lý crypto ở đây —
// chỉ đọc/ghi dữ liệu đã mã hóa (ciphertext) do lớp crypto đưa xuống.
// Kể cả metadata (name/key_type/tags/notes) giờ cũng là ciphertext —
// storage.rs không biết và không cần biết ý nghĩa của dữ liệu nó lưu.

use rusqlite::{Connection, params};
use crate::models::{StoredKeyRow, StoredKeyMetaRow, LegacyKeyRow};

pub struct Storage {
    conn: Connection,
    db_path: String,
}

impl Storage {
    /// Mở (hoặc tạo mới) file database tại đường dẫn chỉ định và đảm bảo
    /// schema đã tồn tại.
    ///
    /// LƯU Ý PRODUCTION: dùng `Connection::open_with_flags` kết hợp
    /// SQLCipher (feature "bundled-sqlcipher" của rusqlite) để mã hóa
    /// toàn bộ file .db ở tầng đĩa, cộng thêm với mã hóa AES-GCM ở tầng
    /// ứng dụng đã có cho từng trường dữ liệu.
    pub fn open(db_path: &str) -> Result<Self, String> {
        let conn = Connection::open(db_path).map_err(|e| crate::error::internal_error("db_open", e))?;
        let storage = Storage { conn, db_path: db_path.to_string() };
        storage.init_schema()?;
        Ok(storage)
    }

    /// Dùng cho unit test: DB tồn tại hoàn toàn trong RAM, không ghi ra đĩa.
    /// clippy coi đây là "chưa dùng" khi build ở chế độ không-test vì chỉ
    /// được gọi trong `#[cfg(test)] mod tests` bên dưới — đó là dùng đúng
    /// mục đích thiết kế, không phải dead code thật.
    #[allow(dead_code)]
    pub fn open_in_memory() -> Result<Self, String> {
        let conn = Connection::open_in_memory().map_err(|e| crate::error::internal_error("db_open", e))?;
        let storage = Storage { conn, db_path: ":memory:".to_string() };
        storage.init_schema()?;
        Ok(storage)
    }

    pub fn db_path(&self) -> &str {
        &self.db_path
    }

    /// Xuất 1 bản sao ĐẦY ĐỦ, NHẤT QUÁN của DB hiện tại ra `dest_path`.
    /// Dùng `VACUUM INTO` thay vì copy file thô — an toàn hơn vì đảm bảo
    /// bản export không bị "xé" (torn) nếu có transaction đang dở dang,
    /// và loại luôn các trang đã xóa/rác trong file gốc.
    ///
    /// LƯU Ý: `VACUUM INTO` yêu cầu file đích CHƯA tồn tại, nên nếu người
    /// dùng chọn ghi đè lên 1 file export cũ (rất thường gặp), phải xóa
    /// file đó trước khi chạy VACUUM INTO.
    pub fn export_to(&self, dest_path: &str) -> Result<(), String> {
        if std::path::Path::new(dest_path).exists() {
            std::fs::remove_file(dest_path)
                .map_err(|e| crate::error::internal_error("db_export_remove_existing", e))?;
        }
        self.conn
            .execute("VACUUM INTO ?1", params![dest_path])
            .map_err(|e| crate::error::internal_error("db_export", e))?;
        Ok(())
    }

    /// Đóng kết nối hiện tại và mở lại TỪ ĐẦU tại cùng đường dẫn (self.db_path).
    /// Dùng sau khi ghi đè file .db bằng dữ liệu import — bắt buộc phải mở
    /// lại vì Connection cũ có thể còn cache schema/statement của file cũ.
    pub fn reload(&mut self) -> Result<(), String> {
        self.switch_path(&self.db_path.clone())
    }

    /// Đóng kết nối hiện tại và mở kết nối MỚI tới `new_path` — khác
    /// `reload()` ở chỗ đường dẫn thực sự thay đổi, dùng khi người dùng
    /// chọn "di chuyển" hoặc "liên kết" tới 1 file vault khác.
    pub fn switch_path(&mut self, new_path: &str) -> Result<(), String> {
        let conn = Connection::open(new_path).map_err(|e| crate::error::internal_error("db_reopen", e))?;
        self.conn = conn;
        self.db_path = new_path.to_string();
        self.init_schema()?;
        Ok(())
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
                    metadata_ciphertext BLOB,
                    metadata_nonce BLOB,
                    ciphertext BLOB NOT NULL,
                    nonce BLOB NOT NULL,
                    created_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL,
                    has_extra_password INTEGER NOT NULL DEFAULT 0,
                    extra_salt BLOB,
                    extra_nonce BLOB,
                    -- Cột CŨ (trước bản vá mã hóa metadata) - giữ lại tạm thời
                    -- chỉ để phục vụ migrate dữ liệu cũ, xem
                    -- list_legacy_plaintext_rows(). Key mới tạo sau bản vá
                    -- này không bao giờ ghi gì vào các cột dưới đây.
                    name TEXT NOT NULL DEFAULT '',
                    key_type TEXT NOT NULL DEFAULT '',
                    tags TEXT NOT NULL DEFAULT '[]',
                    notes TEXT
                );

                CREATE TABLE IF NOT EXISTS audit_log (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    action TEXT NOT NULL,
                    key_id TEXT,
                    timestamp INTEGER NOT NULL
                );
                "#,
            )
            .map_err(|e| crate::error::internal_error("db_init_schema", e))?;

        self.migrate_add_columns();
        Ok(())
    }

    /// Migration cho DB đã tồn tại từ trước — `CREATE TABLE IF NOT EXISTS`
    /// ở trên không tự thêm cột mới vào bảng đã có sẵn, nên cần ALTER
    /// TABLE riêng. Bỏ qua lỗi "duplicate column" một cách an toàn.
    fn migrate_add_columns(&self) {
        let statements = [
            "ALTER TABLE stored_keys ADD COLUMN has_extra_password INTEGER NOT NULL DEFAULT 0",
            "ALTER TABLE stored_keys ADD COLUMN extra_salt BLOB",
            "ALTER TABLE stored_keys ADD COLUMN extra_nonce BLOB",
            "ALTER TABLE stored_keys ADD COLUMN metadata_ciphertext BLOB",
            "ALTER TABLE stored_keys ADD COLUMN metadata_nonce BLOB",
        ];
        for stmt in statements {
            let _ = self.conn.execute(stmt, []);
        }
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
    ///
    /// clippy chê type tuple này phức tạp — đúng, nhưng tách thành struct
    /// riêng sẽ phải sửa theo ở mọi call site (commands/auth.rs), rủi ro
    /// hơn lợi ích ở quy mô hàm nội bộ, chỉ 1 nơi gọi. Cân nhắc refactor
    /// nếu sau này có thêm field hoặc thêm chỗ gọi.
    #[allow(clippy::type_complexity)]
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
                "INSERT INTO stored_keys
                 (id, metadata_ciphertext, metadata_nonce, ciphertext, nonce, created_at, updated_at,
                  has_extra_password, extra_salt, extra_nonce, name, key_type, tags, notes)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, '', '', '[]', NULL)",
                params![
                    row.id, row.metadata_ciphertext, row.metadata_nonce, row.ciphertext, row.nonce,
                    row.created_at, row.updated_at,
                    row.has_extra_password, row.extra_salt, row.extra_nonce
                ],
            )
            .map_err(|e| crate::error::internal_error("db_insert_key", e))?;
        Ok(())
    }

    pub fn get_key_row(&self, id: &str) -> Result<StoredKeyRow, String> {
        self.conn
            .query_row(
                "SELECT id, metadata_ciphertext, metadata_nonce, ciphertext, nonce, created_at, updated_at,
                        has_extra_password, extra_salt, extra_nonce
                 FROM stored_keys WHERE id = ?1",
                params![id],
                |row| {
                    Ok(StoredKeyRow {
                        id: row.get(0)?,
                        metadata_ciphertext: row.get(1)?,
                        metadata_nonce: row.get(2)?,
                        ciphertext: row.get(3)?,
                        nonce: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                        has_extra_password: row.get(7)?,
                        extra_salt: row.get(8)?,
                        extra_nonce: row.get(9)?,
                    })
                },
            )
            .map_err(|e| crate::error::internal_error("db_get_key", e))
    }

    /// Cập nhật lại phần mã hóa của NỘI DUNG key (dùng khi thêm/gỡ/đổi
    /// mật khẩu riêng — chỉ động vào ciphertext/nonce của secret, không
    /// đụng tới metadata).
    ///
    /// 8 tham số vượt ngưỡng mặc định của clippy (7) — đều là dữ liệu
    /// liên quan chặt tới nhau (kết quả của 1 lần mã hóa lại), gộp thành
    /// struct sẽ không rõ ràng hơn bao nhiêu ở quy mô hiện tại.
    #[allow(clippy::too_many_arguments)]
    pub fn update_key_encryption(
        &self,
        id: &str,
        ciphertext: &[u8],
        nonce: &[u8],
        has_extra_password: bool,
        extra_salt: Option<&[u8]>,
        extra_nonce: Option<&[u8]>,
        updated_at: i64,
    ) -> Result<(), String> {
        let affected = self
            .conn
            .execute(
                "UPDATE stored_keys
                 SET ciphertext = ?1, nonce = ?2, has_extra_password = ?3,
                     extra_salt = ?4, extra_nonce = ?5, updated_at = ?6
                 WHERE id = ?7",
                params![ciphertext, nonce, has_extra_password, extra_salt, extra_nonce, updated_at, id],
            )
            .map_err(|e| crate::error::internal_error("db_update_key_encryption", e))?;

        if affected == 0 {
            return Err("ERR_KEY_NOT_FOUND".to_string());
        }
        Ok(())
    }

    /// Danh sách rút gọn cho list view — chỉ lấy metadata đã mã hóa,
    /// KHÔNG lấy ciphertext của secret.
    pub fn list_key_meta_rows(&self) -> Result<Vec<StoredKeyMetaRow>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, metadata_ciphertext, metadata_nonce, created_at, updated_at, has_extra_password
                 FROM stored_keys
                 WHERE metadata_ciphertext IS NOT NULL
                 ORDER BY updated_at DESC",
            )
            .map_err(|e| crate::error::internal_error("db_prepare_query", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(StoredKeyMetaRow {
                    id: row.get(0)?,
                    metadata_ciphertext: row.get(1)?,
                    metadata_nonce: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    has_extra_password: row.get(5)?,
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

    // ---------- migration dữ liệu cũ (metadata plaintext -> mã hóa) ----------

    /// Các key được tạo trước bản vá mã hóa metadata sẽ có
    /// metadata_ciphertext = NULL. Trả về danh sách này kèm dữ liệu
    /// plaintext CŨ để lớp commands (có vault_key) mã hóa lại.
    pub fn list_legacy_plaintext_rows(&self) -> Result<Vec<LegacyKeyRow>, String> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, key_type, tags, notes FROM stored_keys WHERE metadata_ciphertext IS NULL")
            .map_err(|e| crate::error::internal_error("db_prepare_query", e))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(LegacyKeyRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    key_type: row.get(2)?,
                    tags: row.get(3)?,
                    notes: row.get(4)?,
                })
            })
            .map_err(|e| crate::error::internal_error("db_query_map", e))?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| crate::error::internal_error("db_collect_rows", e))
    }

    /// Ghi metadata đã mã hóa vào 1 row, đồng thời XÓA SẠCH plaintext cũ
    /// (ghi đè '' / NULL) trong cùng 1 câu UPDATE — không để lọt khoảnh
    /// khắc nào mà cả 2 bản (plaintext cũ + ciphertext mới) cùng tồn tại
    /// lâu hơn cần thiết.
    pub fn finish_migrate_key_metadata(
        &self,
        id: &str,
        metadata_ciphertext: &[u8],
        metadata_nonce: &[u8],
    ) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE stored_keys
                 SET metadata_ciphertext = ?1, metadata_nonce = ?2,
                     name = '', key_type = '', tags = '[]', notes = NULL
                 WHERE id = ?3",
                params![metadata_ciphertext, metadata_nonce, id],
            )
            .map_err(|e| crate::error::internal_error("db_migrate_metadata", e))?;
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
            metadata_ciphertext: vec![100, 101, 102],
            metadata_nonce: vec![103, 104],
            ciphertext: vec![1, 2, 3, 4],
            nonce: vec![5, 6, 7],
            created_at: 1000,
            updated_at: 1000,
            has_extra_password: false,
            extra_salt: None,
            extra_nonce: None,
        }
    }

    fn sample_row_with_extra_password(id: &str) -> StoredKeyRow {
        StoredKeyRow {
            has_extra_password: true,
            extra_salt: Some(vec![9, 9, 9]),
            extra_nonce: Some(vec![8, 8, 8]),
            ..sample_row(id)
        }
    }

    #[test]
    fn export_to_creates_readable_copy() {
        let dir = std::env::temp_dir().join(format!("vault_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let export_path = dir.join("exported.db");

        let storage = Storage::open_in_memory().unwrap();
        storage.insert_key(&sample_row("key-1")).unwrap();
        storage
            .save_vault_meta(b"salt", b"evk", b"nonce", "{}")
            .unwrap();

        storage.export_to(export_path.to_str().unwrap()).unwrap();

        // Mở lại file vừa export như 1 DB độc lập, phải đọc được đúng dữ liệu
        let reopened = Storage::open(export_path.to_str().unwrap()).unwrap();
        assert!(reopened.has_vault().unwrap());
        let fetched = reopened.get_key_row("key-1").unwrap();
        assert_eq!(fetched.ciphertext, vec![1, 2, 3, 4]);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn reload_picks_up_externally_replaced_file() {
        let dir = std::env::temp_dir().join(format!("vault_test_reload_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("reload.db");
        let path_str = path.to_str().unwrap();

        let storage = Storage::open(path_str).unwrap();
        storage.insert_key(&sample_row("key-a")).unwrap();
        drop(storage);

        // Giả lập "import": ghi đè file bằng 1 DB B hoàn toàn khác (tạo
        // riêng rồi export vào đúng path đó)
        let storage_b = Storage::open_in_memory().unwrap();
        storage_b.insert_key(&sample_row("key-b")).unwrap();
        storage_b.export_to(path_str).unwrap();

        // Mở lại storage A (đang trỏ vào path đó) và reload()
        let mut storage_a_again = Storage::open(path_str).unwrap();
        storage_a_again.reload().unwrap();

        // Phải thấy dữ liệu của DB B, không phải DB A cũ
        assert!(storage_a_again.get_key_row("key-b").is_ok());

        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn insert_key_works_on_legacy_table_without_column_defaults() {
        // Tái hiện chính xác lỗi thực tế: bảng được tạo bởi bản app CŨ,
        // khi `name`/`key_type` là NOT NULL nhưng KHÔNG có DEFAULT.
        // CREATE TABLE IF NOT EXISTS ở init_schema() không sửa được
        // constraint này trên bảng đã tồn tại — insert_key() phải tự ghi
        // rõ giá trị cho các cột đó, không được phụ thuộc vào DEFAULT.
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE stored_keys (
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
            "#,
        )
        .unwrap();

        let storage = Storage { conn, db_path: ":memory:".to_string() };
        // Chạy lại init_schema() để nó tự ALTER TABLE thêm các cột mới
        // (giống hệt luồng thật khi app mở 1 DB cũ đã tồn tại).
        storage.init_schema().unwrap();

        // Đây chính là thao tác trước đây bị lỗi "NOT NULL constraint
        // failed: stored_keys.name" — giờ phải chạy được bình thường.
        let result = storage.insert_key(&sample_row("key-1"));
        assert!(result.is_ok(), "insert_key phải thành công trên bảng cũ: {:?}", result);

        let fetched = storage.get_key_row("key-1").unwrap();
        assert_eq!(fetched.ciphertext, vec![1, 2, 3, 4]);
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
        assert_eq!(fetched.metadata_ciphertext, vec![100, 101, 102]);
        assert_eq!(fetched.ciphertext, vec![1, 2, 3, 4]);
    }

    #[test]
    fn insert_and_get_key_with_extra_password() {
        let storage = Storage::open_in_memory().unwrap();
        let row = sample_row_with_extra_password("key-extra");
        storage.insert_key(&row).unwrap();

        let fetched = storage.get_key_row("key-extra").unwrap();
        assert!(fetched.has_extra_password);
        assert_eq!(fetched.extra_salt, Some(vec![9, 9, 9]));
        assert_eq!(fetched.extra_nonce, Some(vec![8, 8, 8]));
    }

    #[test]
    fn list_meta_rows_excludes_unmigrated_legacy_keys() {
        let storage = Storage::open_in_memory().unwrap();
        storage.insert_key(&sample_row("key-normal")).unwrap();

        // Giả lập 1 key CŨ (metadata_ciphertext = NULL) chèn thẳng qua SQL,
        // bỏ qua insert_key (vốn luôn set metadata_ciphertext).
        storage
            .conn
            .execute(
                "INSERT INTO stored_keys (id, ciphertext, nonce, created_at, updated_at, name, key_type, tags)
                 VALUES ('key-legacy', X'0102', X'0304', 1000, 1000, 'Ten cu', 'ssh', '[\"prod\"]')",
                [],
            )
            .unwrap();

        let summaries = storage.list_key_meta_rows().unwrap();
        // Key legacy chưa migrate -> không xuất hiện trong list bình thường
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].id, "key-normal");
    }

    #[test]
    fn legacy_migration_flow() {
        let storage = Storage::open_in_memory().unwrap();
        storage
            .conn
            .execute(
                "INSERT INTO stored_keys (id, ciphertext, nonce, created_at, updated_at, name, key_type, tags, notes)
                 VALUES ('key-legacy', X'0102', X'0304', 1000, 1000, 'Ten cu bi lo', 'ssh', '[\"prod\"]', 'ghi chu cu')",
                [],
            )
            .unwrap();

        let legacy = storage.list_legacy_plaintext_rows().unwrap();
        assert_eq!(legacy.len(), 1);
        assert_eq!(legacy[0].name, "Ten cu bi lo");
        assert_eq!(legacy[0].notes, Some("ghi chu cu".to_string()));

        storage
            .finish_migrate_key_metadata("key-legacy", &[200, 201], &[202])
            .unwrap();

        // Sau khi migrate: không còn trong danh sách legacy nữa
        let legacy_after = storage.list_legacy_plaintext_rows().unwrap();
        assert_eq!(legacy_after.len(), 0);

        // Và đã xuất hiện trong list bình thường với metadata đã mã hóa
        let summaries = storage.list_key_meta_rows().unwrap();
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].metadata_ciphertext, vec![200, 201]);

        // Plaintext cũ đã bị xóa sạch (đọc thẳng qua SQL để chắc chắn)
        let (name, notes): (String, Option<String>) = storage
            .conn
            .query_row("SELECT name, notes FROM stored_keys WHERE id = 'key-legacy'", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!(name, "");
        assert_eq!(notes, None);
    }

    #[test]
    fn update_key_encryption_adds_extra_password() {
        let storage = Storage::open_in_memory().unwrap();
        storage.insert_key(&sample_row("key-1")).unwrap();

        storage
            .update_key_encryption("key-1", &[10, 11, 12], &[13, 14], true, Some(&[1, 2]), Some(&[3, 4]), 2000)
            .unwrap();

        let fetched = storage.get_key_row("key-1").unwrap();
        assert!(fetched.has_extra_password);
        assert_eq!(fetched.ciphertext, vec![10, 11, 12]);
        assert_eq!(fetched.extra_salt, Some(vec![1, 2]));
        assert_eq!(fetched.updated_at, 2000);
        // metadata không bị đụng tới khi chỉ update encryption của secret
        assert_eq!(fetched.metadata_ciphertext, vec![100, 101, 102]);
    }

    #[test]
    fn update_key_encryption_removes_extra_password() {
        let storage = Storage::open_in_memory().unwrap();
        storage.insert_key(&sample_row_with_extra_password("key-1")).unwrap();

        storage
            .update_key_encryption("key-1", &[20, 21], &[22, 23], false, None, None, 3000)
            .unwrap();

        let fetched = storage.get_key_row("key-1").unwrap();
        assert!(!fetched.has_extra_password);
        assert_eq!(fetched.extra_salt, None);
        assert_eq!(fetched.extra_nonce, None);
    }

    #[test]
    fn update_key_encryption_nonexistent_key_errors() {
        let storage = Storage::open_in_memory().unwrap();
        let result = storage.update_key_encryption("khong-ton-tai", &[1], &[2], false, None, None, 1000);
        assert_eq!(result, Err("ERR_KEY_NOT_FOUND".to_string()));
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
        assert_eq!(result, Err("ERR_KEY_NOT_FOUND".to_string()));
    }
}
