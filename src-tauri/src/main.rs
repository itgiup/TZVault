// src-tauri/src/main.rs

mod crypto;
mod error;
mod models;
mod vault;
mod commands;

use vault::state::VaultState;
use vault::storage::Storage;
use commands::auth::StorageState;
use std::sync::Mutex;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
        .manage(VaultState::new())
        .setup(|app| {
            // QUAN TRỌNG: không lưu DB trong thư mục src-tauri (đường dẫn
            // tương đối như "vault.db" sẽ resolve vào đó khi chạy `cargo run`).
            // Tauri's dev watcher theo dõi toàn bộ src-tauri để tự rebuild khi
            // code Rust đổi — nếu DB cũng nằm trong đó, mỗi lần ghi dữ liệu
            // (thêm key, ghi audit log...) sẽ bị hiểu nhầm là code vừa sửa,
            // khiến app tự restart liên tục.
            //
            // Giải pháp đúng (và cũng là chuẩn cho production): lưu vào
            // app_data_dir của hệ điều hành, nơi Tauri không theo dõi và
            // luôn có quyền ghi.
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Không lấy được app_data_dir");

            std::fs::create_dir_all(&app_data_dir)
                .expect("Không tạo được thư mục app_data_dir");

            let db_path = app_data_dir.join("vault.db");
            let db_path_str = db_path.to_str().expect("Đường dẫn DB không hợp lệ");

            let storage = Storage::open(db_path_str).expect("Không mở được database");
            app.manage(StorageState(Mutex::new(storage)));

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::cmd_setup_vault,
            commands::auth::cmd_vault_exists,
            commands::auth::cmd_unlock_vault,
            commands::auth::cmd_lock_vault,
            commands::auth::cmd_is_unlocked,
            commands::auth::cmd_change_password,
            commands::auth::cmd_set_auto_lock_timeout,
            commands::keys::cmd_add_key,
            commands::keys::cmd_list_keys,
            commands::keys::cmd_get_key_secret,
            commands::keys::cmd_unlock_key_with_password,
            commands::keys::cmd_add_key_password,
            commands::keys::cmd_remove_key_password,
            commands::keys::cmd_change_key_password,
            commands::keys::cmd_delete_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
