// src-tauri/src/vault/state.rs
//
// Giữ Vault Key trong RAM trong suốt phiên làm việc (sau khi unlock),
// và tự động khóa lại nếu người dùng không thao tác trong X phút.
//
// Đây là struct sẽ được quản lý bởi Tauri qua `app.manage(VaultState::new())`
// và inject vào các command qua `tauri::State<VaultState>`.

use std::sync::Mutex;
use std::time::{Duration, Instant};
use crate::crypto::wipe_key;

pub struct VaultState {
    vault_key: Mutex<Option<[u8; 32]>>,
    last_activity: Mutex<Instant>,
    auto_lock_timeout: Mutex<Duration>,
}

impl VaultState {
    pub fn new() -> Self {
        Self {
            vault_key: Mutex::new(None),
            last_activity: Mutex::new(Instant::now()),
            // Mặc định 5 phút, có thể đổi qua set_auto_lock_timeout (Settings UI)
            auto_lock_timeout: Mutex::new(Duration::from_secs(5 * 60)),
        }
    }

    /// Gọi sau khi unlock_vault() ở module crypto thành công.
    pub fn set_vault_key(&self, key: [u8; 32]) {
        let mut guard = self.vault_key.lock().unwrap();
        *guard = Some(key);
        self.touch();
    }

    /// Lấy vault key hiện tại. Trả về Err nếu vault đang bị khóa hoặc
    /// đã hết hạn do idle quá lâu (tự động lock).
    pub fn get_vault_key(&self) -> Result<[u8; 32], String> {
        self.check_and_apply_auto_lock();

        let guard = self.vault_key.lock().unwrap();
        match *guard {
            Some(key) => {
                drop(guard);
                self.touch();
                Ok(key)
            }
            None => Err("ERR_VAULT_LOCKED".to_string()),
        }
    }

    /// Khóa vault: xóa vault key khỏi RAM một cách an toàn (zeroize).
    pub fn lock(&self) {
        let mut guard = self.vault_key.lock().unwrap();
        if let Some(mut key) = guard.take() {
            wipe_key(&mut key);
        }
    }

    pub fn is_unlocked(&self) -> bool {
        self.check_and_apply_auto_lock();
        self.vault_key.lock().unwrap().is_some()
    }

    /// Cập nhật thời điểm hoạt động gần nhất — gọi mỗi khi có command
    /// nào đó chạm vào vault (list key, view key, add key...).
    pub fn touch(&self) {
        let mut last = self.last_activity.lock().unwrap();
        *last = Instant::now();
    }

    pub fn set_auto_lock_timeout(&self, seconds: u64) {
        let mut timeout = self.auto_lock_timeout.lock().unwrap();
        *timeout = Duration::from_secs(seconds);
    }

    /// Nếu đã idle quá lâu, tự động khóa vault (xóa key khỏi RAM).
    fn check_and_apply_auto_lock(&self) {
        let last = *self.last_activity.lock().unwrap();
        let timeout = *self.auto_lock_timeout.lock().unwrap();

        if last.elapsed() > timeout {
            self.lock();
        }
    }
}

impl Default for VaultState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn set_and_get_vault_key() {
        let state = VaultState::new();
        assert!(!state.is_unlocked());

        state.set_vault_key([7u8; 32]);
        assert!(state.is_unlocked());

        let key = state.get_vault_key().unwrap();
        assert_eq!(key, [7u8; 32]);
    }

    #[test]
    fn lock_clears_key() {
        let state = VaultState::new();
        state.set_vault_key([1u8; 32]);
        assert!(state.is_unlocked());

        state.lock();
        assert!(!state.is_unlocked());
        assert!(state.get_vault_key().is_err());
    }

    #[test]
    fn auto_lock_after_timeout() {
        let state = VaultState::new();
        state.set_auto_lock_timeout(0); // hết hạn ngay lập tức để test nhanh
        state.set_vault_key([9u8; 32]);

        sleep(Duration::from_millis(50));

        // Vì timeout = 0s và đã "elapsed", is_unlocked() phải trigger auto-lock
        assert!(!state.is_unlocked());
    }

    #[test]
    fn touch_resets_idle_timer() {
        let state = VaultState::new();
        state.set_auto_lock_timeout(3600); // 1 tiếng, không nên hết hạn trong test
        state.set_vault_key([3u8; 32]);

        sleep(Duration::from_millis(20));
        state.touch();

        assert!(state.is_unlocked());
    }
}
