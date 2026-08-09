# TZVault

A desktop app for managing private keys (SSH keys, crypto wallets, PGP keys, API keys...) — encrypts everything at the application layer before it ever touches disk. Built with **Tauri 2 + Rust + React + TypeScript**.

[Tiếng Việt](./README.vi.md)

![platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-blue)
![status](https://img.shields.io/badge/status-development-orange)

---

## Table of contents

- [Features](#features)
- [Security architecture](#security-architecture)
- [Tech stack](#tech-stack)
- [Project structure](#project-structure)
- [Getting started](#getting-started)
- [Vault location — move / link / export / import](#vault-location--move--link--export--import)
- [Tauri commands reference](#tauri-commands-reference)
- [Internationalization (i18n)](#internationalization-i18n)
- [Known limitations & pre-release checklist](#known-limitations--pre-release-checklist)
- [Dev FAQ](#dev-faq)

---

## Features

- **Everything is encrypted, including metadata** — not just key contents, but names, tags, and notes are all AES-256-GCM encrypted before hitting the database. The `vault.db` file contains zero plaintext.
- **Master password** unlocks the whole vault (Argon2id → AES-256-GCM), with auto-lock after N minutes of inactivity (default 5, configurable in Settings).
- **Optional per-key password** — a fully independent second encryption layer for especially sensitive keys (e.g. a high-value crypto wallet).
- **Flexible vault location** — Export/Import, Move to a synced folder, or Link directly to another vault file without copying.
- **Light/dark theme**, multi-language (English default, Vietnamese available), responsive down to narrow window widths.
- **Auto-lock, clipboard auto-clear after 20s, two-step delete confirmation** — the security habits you'd expect from any serious password manager.

## Security architecture

```
Master Password (entered by the user, NEVER stored)
      │
      ▼ Argon2id (random salt, stored alongside the vault)
Key Encryption Key (KEK)
      │
      ▼ AES-256-GCM decrypt
Vault Key (randomly generated at setup, kept in RAM while unlocked)
      │
      ├─▶ encrypts each key's CONTENT      (AES-256-GCM, per-key nonce)
      └─▶ encrypts each key's METADATA     (name/type/tags/notes, separate AAD)
              │
              ▼ (optional, only for keys with an "extra password")
        A SECOND encryption layer using that key's own password
        (fully independent of the Vault Key — even if the Vault Key
         leaks, an extra-protected key's content stays unreadable)
```

**Design principles:**

- **Zero-knowledge within the app's own boundary**: all decryption happens inside the Rust process; data only reaches the frontend when the user explicitly clicks "Show"/"Copy".
- **Encryption layers are cryptographically separated** via distinct Associated Data (AAD) for metadata vs. key content vs. the per-key password layer — this prevents ciphertext-swap attacks (you can't take one field/key's ciphertext and pass it off as another's).
- **Errors never leak internal details to the UI**: every internal error (corrupted DB, mutex lock failure...) is logged in full to `stderr` via `eprintln!`, while the UI only ever receives a single generic `ERR_INTERNAL` code — this avoids information disclosure while keeping the i18n architecture clean (the frontend translates error codes; the backend never returns prose).
- **Auto-lock, unlock rate-limiting** (increasing delay per failed attempt), and an internal **audit log** (records actions + timestamps only, never key contents).

## Tech stack

| Layer | Technology |
|---|---|
| Desktop shell | [Tauri 2](https://tauri.app) |
| Backend | Rust — `argon2`, `aes-gcm`, `rand`, `zeroize`, `rusqlite` (SQLite), `uuid`, `serde`/`serde_json`, `tauri-plugin-dialog` |
| Frontend | React + TypeScript + Vite |
| Styling | Plain CSS, custom design tokens (a "steel vault, brass lock" theme) |

## Project structure

```
src-tauri/
├── src/
│   ├── main.rs                 # entrypoint — registers state, commands, plugins
│   ├── app_config.rs           # small, non-sensitive config: which vault path is active
│   ├── error.rs                # normalizes internal errors -> ERR_INTERNAL + server-side log
│   ├── models.rs                # shared data structs (KeySummary, StoredKeyRow...)
│   ├── crypto/
│   │   ├── kdf.rs               # Argon2id: derive a key from a password
│   │   ├── cipher.rs            # AES-256-GCM: low-level encrypt/decrypt
│   │   └── mod.rs               # high-level functions: vault setup/unlock, metadata
│   │                             #   encryption, double-layer encryption for
│   │                             #   per-key-password-protected keys
│   ├── vault/
│   │   ├── storage.rs           # the ONLY layer that touches SQLite — has no idea
│   │   │                         #   what the data it stores actually means
│   │   └── state.rs             # keeps the Vault Key in RAM, handles auto-lock
│   └── commands/
│       ├── auth.rs              # setup/unlock/lock, password change, export/import,
│       │                         #   move/link vault location
│       └── keys.rs              # key CRUD, per-key password management
└── capabilities/
    └── default.json             # Tauri permissions (core, opener, dialog)

src/
├── App.tsx                      # orchestrates Setup -> Unlock -> Vault flow
├── types.ts                     # shared types, mirror the Rust structs 1:1
├── api/vault.ts                 # the ONLY layer that calls invoke() — every command
│                                 #   goes through here
├── i18n/
│   ├── translations.ts          # EN/VI + backend error-code translator
│   └── LanguageContext.tsx
├── hooks/useTheme.ts
├── styles/vault.css             # design tokens (dark/light), animations
└── components/
    ├── SetupScreen.tsx          # create a new vault / import an existing one
    ├── UnlockScreen.tsx         # unlock, or switch to a different vault (link)
    ├── VaultScreen.tsx          # key list + detail (two-pane layout)
    ├── KeyDetail.tsx            # view/copy/delete a key, manage its extra password
    ├── AddKeyModal.tsx
    ├── SettingsModal.tsx        # auto-lock, change password, export, move/link vault
    ├── KeyPasswordModal.tsx     # shared modal for unlock/add/remove/change per-key password
    ├── Modal.tsx                 # shared open/close animation wrapper for every modal
    ├── ThemeToggle.tsx / LanguageToggle.tsx / Dial.tsx
```

## Getting started

### Prerequisites

- [Rust](https://rustup.rs) (latest stable)
- [Node.js](https://nodejs.org) + [pnpm](https://pnpm.io)
- Linux system dependencies for Tauri: `libwebkit2gtk-4.1-dev`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libdbus-1-dev`, `pkg-config`, `build-essential` — see [tauri.app](https://tauri.app/start/prerequisites/) for the full, up-to-date list.

### Install & run in dev mode

```bash
pnpm install
pnpm tauri dev
```

### Build a release

```bash
pnpm tauri build
```

Installers land in `src-tauri/target/release/bundle/` (`.deb`/`.AppImage`/`.rpm` on Linux, `.msi`/`.exe` on Windows, `.dmg`/`.app` on macOS — you can only build for the OS you're currently running the command on).

⚠️ **Before shipping a real release**: you'll need code signing (required on Windows/macOS to avoid "Unknown Publisher" warnings) — see [Known limitations](#known-limitations--pre-release-checklist) below.

### Default data location

| OS | Path |
|---|---|
| Linux | `~/.local/share/<app-identifier>/vault.db` |
| Windows | `%APPDATA%\<app-identifier>\vault.db` |
| macOS | `~/Library/Application Support/<app-identifier>/vault.db` |

This can be changed from Settings — see the section below.

## Vault location — move / link / export / import

| Action | Where | Behavior |
|---|---|---|
| **Export** | Settings (unlocked) | Uses `VACUUM INTO` to create a standalone, consistent SQLite file wherever you choose — take it anywhere and it just works |
| **Import** | Setup screen (no vault set up yet) | Copies the chosen file into the app's default location — safe because there's nothing to lose on a fresh machine |
| **Move** | Settings (unlocked) | Copies the current vault to a new location (e.g. a Dropbox/Google Drive folder) and switches to using it |
| **Link** | Settings **or** right from the Unlock screen | Points directly at an existing vault file elsewhere — **no copying** |

Move/Link choices are persisted in `app_config.json` (next to `vault.db`, **unencrypted** since it only holds a file path, not sensitive data) — the app remembers the right location on next launch.

## Tauri commands reference

<details>
<summary>Click to expand (18 commands)</summary>

**Auth / vault lifecycle** (`commands/auth.rs`)
| Command | Description |
|---|---|
| `cmd_setup_vault(password)` | Create a new vault |
| `cmd_vault_exists()` | Check whether a vault has been set up |
| `cmd_unlock_vault(password)` | Unlock |
| `cmd_lock_vault()` | Lock |
| `cmd_is_unlocked()` | Current lock state |
| `cmd_change_password(old, new)` | Change the master password |
| `cmd_set_auto_lock_timeout(seconds)` | Change the auto-lock duration |
| `cmd_export_vault(dest_path)` | Export to a file |
| `cmd_import_vault(src_path)` | Import (only when no vault exists yet) |
| `cmd_get_db_path()` | Get the currently active vault path |
| `cmd_set_db_path(new_path, mode)` | Move/Link to a different vault (`mode`: `"move"` \| `"link"`) |

**Key management** (`commands/keys.rs`)
| Command | Description |
|---|---|
| `cmd_add_key(input)` | Add a new key (optionally with a per-key password) |
| `cmd_list_keys()` | List keys (metadata decrypted) |
| `cmd_get_key_secret(id)` | View a regular key's content |
| `cmd_unlock_key_with_password(id, key_password)` | View an extra-protected key's content |
| `cmd_add_key_password(id, new_key_password)` | Turn on extra protection for a key |
| `cmd_remove_key_password(id, current_key_password)` | Remove extra protection |
| `cmd_change_key_password(id, current, new)` | Change a key's extra password |
| `cmd_delete_key(id)` | Delete a key |

</details>

## Internationalization (i18n)

- Defaults to **English**, switchable to **Vietnamese** (toggle button in the corner), preference saved in `localStorage`.
- **The backend never returns prose** — only stable error codes (`ERR_INVALID_PASSWORD`, `ERR_VAULT_LOCKED`...), translated on the frontend via `translateError()` in `src/i18n/translations.ts`.
- Adding a new language: add one object in `translations.ts` implementing the `Translations` interface — TypeScript will flag any missing keys automatically.

## Known limitations & pre-release checklist

This is an actively developed project and **has not gone through an independent security audit**. Don't use it for high-value real data (large crypto wallets, critical production keys) until the following are done:

- [ ] **Independent security audit** (third-party) — mandatory before any real release, not optional for this kind of app.
- [ ] **Full-disk database encryption** (upgrade `rusqlite` to the `bundled-sqlcipher` feature) — currently each field is individually encrypted (which already protects content/metadata), but the raw file/schema structure is still inspectable at the SQLite level.
- [ ] **Code signing** for Windows (CA-issued certificate) and macOS (Apple Developer + notarization) — without this, the OS will warn about or block an "Unknown Publisher" app.
- [ ] **Signed auto-updates** (`tauri-plugin-updater`) — to ship security fixes quickly to existing installs.
- [ ] Consider a **bug bounty program** at public launch.

**Dependency posture**: run `cargo audit` (from `src-tauri/`) to check for known-vulnerable dependencies. A small number of `unmaintained`/`unsound` (not exploitable) advisories are already reviewed and tracked with justification in [`src-tauri/.cargo/audit.toml`](./src-tauri/.cargo/audit.toml) — mostly transitive GTK3 bindings pulled in by Tauri's Linux backend. Please check that file before re-reporting one of these; genuinely new findings are still very welcome.

## Dev FAQ

**The app keeps closing and reopening while running `tauri dev`?**
Make sure `vault.db` isn't sitting inside the `src-tauri` folder — Tauri's dev watcher will mistake every DB write for a source code change and trigger a rebuild. It's already configured to live in `app_data_dir` — see `main.rs`.

**`dialog.save/open not allowed` error?**
Missing permission in `src-tauri/capabilities/default.json` — you need `"dialog:default"` in the `"permissions"` array, and `"windows"` must match your actual window label (usually `"main"`). After fixing it, **fully restart** `pnpm tauri dev` — capability changes aren't picked up by hot-reload.

**Rust build fails with a missing `tauri_plugin_dialog` crate?**
Run `cd src-tauri && cargo add tauri-plugin-dialog`. You also need `pnpm add @tauri-apps/plugin-dialog` on the frontend side — two independent packages, missing either one still breaks the build.

**Still seeing old bugs after pasting in the code?**
Larger features (per-key passwords, metadata encryption, export/import) touch **multiple files at once** — double check you've overwritten *every* file listed, not just the one most recently mentioned.
