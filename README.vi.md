# TZVault

Ứng dụng desktop quản lý private key (SSH key, ví crypto, PGP key, API key...) — mã hóa toàn bộ dữ liệu ở tầng ứng dụng trước khi lưu xuống đĩa, xây trên **Tauri 2 + Rust + React + TypeScript**.

[English](./README.md)

![platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)
![status](https://img.shields.io/badge/status-development-orange)

---

## Mục lục

- [Tính năng](#tính-năng)
- [Kiến trúc bảo mật](#kiến-trúc-bảo-mật)
- [Tech stack](#tech-stack)
- [Cấu trúc project](#cấu-trúc-project)
- [Bắt đầu](#bắt-đầu)
- [Vault location — di chuyển / liên kết / export / import](#vault-location--di-chuyển--liên-kết--export--import)
- [Danh sách Tauri commands](#danh-sách-tauri-commands)
- [Đa ngôn ngữ (i18n)](#đa-ngôn-ngữ-i18n)
- [Giới hạn hiện tại & việc cần làm trước khi phát hành](#giới-hạn-hiện-tại--việc-cần-làm-trước-khi-phát-hành)
- [Câu hỏi thường gặp khi dev](#câu-hỏi-thường-gặp-khi-dev)

---

## Tính năng

- **Mã hóa toàn bộ**, kể cả metadata — không chỉ nội dung key mà cả tên, tag, ghi chú đều được mã hóa AES-256-GCM trước khi lưu DB. File `vault.db` không chứa bất kỳ plaintext nào.
- **Master password** mở khóa toàn bộ vault (Argon2id → AES-256-GCM), với auto-lock sau X phút không hoạt động (mặc định 5 phút, chỉnh được trong Settings).
- **Mật khẩu riêng cho từng key** (tùy chọn) — mã hóa 2 lớp độc lập, dùng cho các key đặc biệt nhạy cảm (VD ví crypto giá trị lớn).
- **Vault location linh hoạt** — Export/Import, Move (di chuyển) sang thư mục đồng bộ, hoặc Link (liên kết) trực tiếp tới 1 file vault khác mà không cần copy.
- **Giao diện sáng/tối**, đa ngôn ngữ (English mặc định, Tiếng Việt), responsive cho cả cửa sổ hẹp.
- **Auto-lock, clipboard tự xóa sau 20s, xác nhận 2 bước khi xóa key** — các thói quen bảo mật chuẩn của trình quản lý mật khẩu.

## Kiến trúc bảo mật

```
Master Password (người dùng nhập, KHÔNG BAO GIỜ lưu lại)
      │
      ▼ Argon2id (salt ngẫu nhiên, lưu cùng vault)
Key Encryption Key (KEK)
      │
      ▼ AES-256-GCM giải mã
Vault Key (sinh ngẫu nhiên lúc setup, giữ trong RAM khi đang unlock)
      │
      ├─▶ mã hóa NỘI DUNG từng key      (AES-256-GCM, nonce riêng/key)
      └─▶ mã hóa METADATA từng key      (name/type/tags/notes, AAD riêng)
              │
              ▼ (tùy chọn, nếu key có "mật khẩu riêng")
        Lớp mã hóa THỨ 2 bằng Key Password riêng của key đó
        (độc lập hoàn toàn với Vault Key — mất Vault Key cũng
         không đọc được nội dung key đã bảo vệ thêm)
```

**Nguyên tắc thiết kế:**

- **Zero-knowledge trong tầm ứng dụng**: mọi thao tác giải mã chỉ xảy ra trong tiến trình Rust, dữ liệu trả ra frontend chỉ khi người dùng chủ động "Show"/"Copy".
- **Tách biệt 2 lớp mã hóa** bằng AAD (Associated Data) khác nhau cho metadata vs. nội dung key vs. lớp mật khẩu riêng — chống ciphertext-swap attack (không thể lấy ciphertext của trường/key này gán cho trường/key khác).
- **Lỗi không bao giờ lộ chi tiết nội bộ ra UI**: mọi lỗi hệ thống (DB hỏng, mutex lock...) log chi tiết ra `stderr` (`eprintln!`), chỉ trả về UI đúng 1 mã `ERR_INTERNAL` chung — tránh rò rỉ thông tin, đồng thời giữ đúng kiến trúc đa ngôn ngữ (frontend tự dịch mã lỗi, backend không bao giờ trả câu prose).
- **Auto-lock, rate-limit unlock sai** (tăng dần delay), **audit log** nội bộ (không ghi nội dung key, chỉ ghi hành động + thời gian).

## Tech stack

| Layer | Công nghệ |
|---|---|
| Desktop shell | [Tauri 2](https://tauri.app) |
| Backend | Rust — `argon2`, `aes-gcm`, `rand`, `zeroize`, `rusqlite` (SQLite), `uuid`, `serde`/`serde_json`, `tauri-plugin-dialog` |
| Frontend | React + TypeScript + Vite |
| Styling | CSS thuần, design token riêng (theme "két thép, khóa đồng thau") |

## Cấu trúc project

```
src-tauri/
├── src/
│   ├── main.rs                 # entrypoint, đăng ký state + commands + plugin
│   ├── app_config.rs           # config nhỏ (không nhạy cảm) lưu đường dẫn vault đang dùng
│   ├── error.rs                # chuẩn hóa lỗi nội bộ -> ERR_INTERNAL + log server-side
│   ├── models.rs                # struct dữ liệu dùng chung (KeySummary, StoredKeyRow...)
│   ├── crypto/
│   │   ├── kdf.rs               # Argon2id: derive key từ password
│   │   ├── cipher.rs            # AES-256-GCM: encrypt/decrypt cấp thấp
│   │   └── mod.rs               # hàm high-level: setup/unlock vault, mã hóa metadata,
│   │                             #   mã hóa 2 lớp cho key có mật khẩu riêng
│   ├── vault/
│   │   ├── storage.rs           # lớp DUY NHẤT chạm SQLite — không biết ý nghĩa dữ liệu
│   │   └── state.rs             # giữ Vault Key trong RAM, auto-lock theo thời gian
│   └── commands/
│       ├── auth.rs              # setup/unlock/lock, đổi password, export/import,
│       │                         #   move/link vault location
│       └── keys.rs              # CRUD key, quản lý mật khẩu riêng từng key
└── capabilities/
    └── default.json             # permission Tauri (core, opener, dialog)

src/
├── App.tsx                      # điều phối luồng Setup -> Unlock -> Vault
├── types.ts                     # type dùng chung, khớp 1-1 với struct Rust
├── api/vault.ts                 # lớp DUY NHẤT gọi invoke() — mọi command đi qua đây
├── i18n/
│   ├── translations.ts          # EN/VI + hàm dịch mã lỗi từ backend
│   └── LanguageContext.tsx
├── hooks/useTheme.ts
├── styles/vault.css             # design token (dark/light), animation
└── components/
    ├── SetupScreen.tsx          # tạo vault mới / import vault có sẵn
    ├── UnlockScreen.tsx         # mở khóa / đổi sang vault khác (link)
    ├── VaultScreen.tsx          # danh sách + chi tiết key (layout 2 cột)
    ├── KeyDetail.tsx            # xem/copy/xóa key, quản lý mật khẩu riêng
    ├── AddKeyModal.tsx
    ├── SettingsModal.tsx        # auto-lock, đổi password, export, move/link vault
    ├── KeyPasswordModal.tsx     # dùng chung cho unlock/add/remove/change mật khẩu riêng
    ├── Modal.tsx                 # wrapper animation mở/đóng dùng chung cho mọi modal
    ├── ThemeToggle.tsx / LanguageToggle.tsx / Dial.tsx
```

## Bắt đầu

### Yêu cầu

- [Rust](https://rustup.rs) (bản ổn định mới nhất)
- [Node.js](https://nodejs.org) + [pnpm](https://pnpm.io)
- Dependency hệ thống cho Tauri trên Linux: `libwebkit2gtk-4.1-dev`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libdbus-1-dev`, `pkg-config`, `build-essential` — xem chi tiết tại [tauri.app](https://tauri.app/start/prerequisites/)

### Cài đặt & chạy dev

```bash
pnpm install
pnpm tauri dev
```

### Build bản release

```bash
pnpm tauri build
```

File cài đặt nằm ở `src-tauri/target/release/bundle/` (`.deb`/`.AppImage`/`.rpm` trên Linux, `.msi`/`.exe` trên Windows, `.dmg`/`.app` trên macOS — chỉ build được cho OS đang chạy lệnh).

⚠️ **Trước khi build release thật để phát hành**: cần code signing (bắt buộc với Windows/macOS để tránh cảnh báo "Unknown Publisher"), xem thêm ở mục [Giới hạn hiện tại](#giới-hạn-hiện-tại--việc-cần-làm-trước-khi-phát-hành).

### Vị trí lưu dữ liệu mặc định

| OS | Đường dẫn |
|---|---|
| Linux | `~/.local/share/<app-identifier>/vault.db` |
| Windows | `%APPDATA%\<app-identifier>\vault.db` |
| macOS | `~/Library/Application Support/<app-identifier>/vault.db` |

Có thể đổi sang vị trí khác qua Settings → xem mục dưới.

## Vault location — di chuyển / liên kết / export / import

| Thao tác | Ở đâu | Hành vi |
|---|---|---|
| **Export** | Settings (đã unlock) | `VACUUM INTO` tạo 1 file SQLite độc lập, nhất quán tại nơi bạn chọn — mang đi đâu cũng dùng lại được |
| **Import** | Màn hình Setup (máy chưa có vault) | Copy file đã chọn vào vị trí mặc định của app — an toàn vì máy vốn chưa có gì để mất |
| **Move** | Settings (đã unlock) | Copy vault hiện tại sang vị trí mới (VD thư mục Dropbox/Google Drive), từ đó dùng vị trí mới |
| **Link** | Settings **hoặc** ngay màn hình Unlock | Trỏ thẳng tới 1 file vault đã có sẵn ở nơi khác, **không copy** |

Lựa chọn vị trí (Move/Link) được lưu vào `app_config.json` (cạnh `vault.db`, **không mã hóa** vì chỉ chứa đường dẫn, không phải dữ liệu nhạy cảm) — mở app lần sau tự nhớ đúng vị trí.

## Danh sách Tauri commands

<details>
<summary>Bấm để xem đầy đủ (18 commands)</summary>

**Auth / vault lifecycle** (`commands/auth.rs`)
| Command | Mô tả |
|---|---|
| `cmd_setup_vault(password)` | Tạo vault mới |
| `cmd_vault_exists()` | Kiểm tra đã setup chưa |
| `cmd_unlock_vault(password)` | Mở khóa |
| `cmd_lock_vault()` | Khóa lại |
| `cmd_is_unlocked()` | Trạng thái hiện tại |
| `cmd_change_password(old, new)` | Đổi master password |
| `cmd_set_auto_lock_timeout(seconds)` | Đổi thời gian auto-lock |
| `cmd_export_vault(dest_path)` | Export ra file |
| `cmd_import_vault(src_path)` | Import (chỉ khi chưa có vault) |
| `cmd_get_db_path()` | Lấy đường dẫn vault đang dùng |
| `cmd_set_db_path(new_path, mode)` | Move/Link sang vault khác (`mode`: `"move"` \| `"link"`) |

**Quản lý key** (`commands/keys.rs`)
| Command | Mô tả |
|---|---|
| `cmd_add_key(input)` | Thêm key mới (tùy chọn kèm mật khẩu riêng) |
| `cmd_list_keys()` | Danh sách (đã giải mã metadata) |
| `cmd_get_key_secret(id)` | Xem nội dung key thường |
| `cmd_unlock_key_with_password(id, key_password)` | Xem nội dung key có mật khẩu riêng |
| `cmd_add_key_password(id, new_key_password)` | Bật bảo vệ thêm cho key |
| `cmd_remove_key_password(id, current_key_password)` | Gỡ bảo vệ thêm |
| `cmd_change_key_password(id, current, new)` | Đổi mật khẩu riêng |
| `cmd_delete_key(id)` | Xóa key |

</details>

## Đa ngôn ngữ (i18n)

- Mặc định **tiếng Anh**, chuyển được sang **Tiếng Việt** (nút góc màn hình), lưu lựa chọn vào `localStorage`.
- **Backend không bao giờ trả câu prose** — chỉ trả mã lỗi ổn định (`ERR_INVALID_PASSWORD`, `ERR_VAULT_LOCKED`...), frontend tự dịch qua `translateError()` trong `src/i18n/translations.ts`.
- Thêm ngôn ngữ mới: thêm 1 object trong `translations.ts` implement đúng interface `Translations` — TypeScript tự báo lỗi nếu thiếu key nào.

## Giới hạn hiện tại & việc cần làm trước khi phát hành

Đây là project đang phát triển, **chưa qua audit bảo mật độc lập** — không nên dùng cho dữ liệu thật giá trị cao (ví crypto lớn, key production quan trọng) cho tới khi hoàn thành các mục dưới:

- [ ] **Audit bảo mật độc lập** (bên thứ ba) — bắt buộc trước khi phát hành thật, không nên bỏ qua với loại app này.
- [ ] **Mã hóa toàn bộ file DB ở tầng đĩa** (nâng cấp `rusqlite` sang feature `bundled-sqlcipher`) — hiện tại chỉ mã hóa từng trường dữ liệu (đã đủ để bảo vệ nội dung/metadata), nhưng cấu trúc file/schema vẫn đọc được ở tầng file nếu ai đó phân tích sâu file SQLite.
- [ ] **Code signing** cho Windows (chứng chỉ từ CA) và macOS (Apple Developer + notarize) — thiếu bước này, hệ điều hành sẽ cảnh báo "Unknown Publisher"/chặn mở app.
- [ ] **Auto-update có ký số** (`tauri-plugin-updater`) — để vá lỗi bảo mật nhanh cho người dùng đã cài.
- [ ] Cân nhắc **bug bounty program** khi ra mắt công khai.

**Tình trạng dependency**: chạy `cargo audit` (trong `src-tauri/`) để kiểm tra dependency có lỗ hổng đã biết không. Một số cảnh báo dạng `unmaintained`/`unsound` (không khai thác được) đã được đánh giá và ghi lại lý do rõ ràng trong [`src-tauri/.cargo/audit.toml`](./src-tauri/.cargo/audit.toml) — chủ yếu là binding GTK3 mà Tauri kéo vào cho Linux. Vui lòng kiểm tra file đó trước khi báo lại các mục này; phát hiện thật sự mới vẫn luôn được hoan nghênh.

## Câu hỏi thường gặp khi dev

**App tự tắt/bật liên tục khi đang `tauri dev`?**
Kiểm tra `vault.db` không nằm trong thư mục `src-tauri` (watcher của Tauri sẽ hiểu nhầm ghi DB là code vừa đổi → tự rebuild). Đã cấu hình đúng để lưu ở `app_data_dir`, xem `main.rs`.

**Lỗi `dialog.save/open not allowed`?**
Thiếu permission trong `src-tauri/capabilities/default.json` — cần có `"dialog:default"` trong mảng `"permissions"`, và `"windows"` phải khớp đúng label cửa sổ (thường là `"main"`). Sửa xong phải **restart hẳn** `pnpm tauri dev`, hot-reload không áp dụng thay đổi capability.

**Build Rust báo thiếu crate `tauri_plugin_dialog`?**
Chạy `cd src-tauri && cargo add tauri-plugin-dialog`. Song song cần `pnpm add @tauri-apps/plugin-dialog` ở phía frontend — 2 package độc lập, thiếu 1 trong 2 vẫn lỗi.

**Copy code Claude đưa mà vẫn còn lỗi cũ?**
Các tính năng lớn (mật khẩu riêng, mã hóa metadata, export/import) đụng vào **nhiều file cùng lúc** — kiểm tra đã copy đè **đủ hết** file được liệt kê, không chỉ file vừa nhắc tới gần nhất.
