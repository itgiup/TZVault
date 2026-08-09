# Threat Model

This document defines what TZVault is designed to protect against, who the assumed adversaries are, and — just as important — what it is **not** designed to protect against. Security researchers should use this as the starting point for testing: findings within scope are genuine bugs; findings outside scope are known, accepted risks (still worth discussing, but not "the vault is broken").

## Assets we're protecting

1. **Private key contents** (SSH keys, crypto wallet keys, PGP keys, API keys) — the actual secret bytes.
2. **Metadata** (key names, tags, notes) — not secret in the traditional sense, but leaking it still reveals *what infrastructure/accounts a person has* to anyone who steals the vault file.
3. **The master password** and any per-key passwords — must never be recoverable from anything written to disk or sent over IPC.
4. **The Vault Key** (the actual AES key that encrypts everything) — must never touch disk in plaintext, and must be wiped from RAM promptly when no longer needed.

## Assumed adversary capabilities

We design against an adversary who can:

- **Steal the `vault.db` file** (via a stolen laptop, a compromised backup, a misconfigured cloud sync folder, a lost USB drive after Export, etc.) but does **not** know the master password.
- **Observe network traffic** to/from the machine (irrelevant today since TZVault has no network calls, but worth stating explicitly).
- **Run arbitrary code as the same OS user**, but only *after* the vault has been locked again (e.g. malware that runs later and reads files, but wasn't present at the exact moment of unlock).

We do **not** currently design against an adversary who:

- Has **root/administrator/kernel-level access** on the machine *while the vault is unlocked* — they can read process memory directly, defeating any in-RAM protection.
- Controls the **OS clipboard manager** in a way that captures data faster than our 20-second auto-clear window.
- Compromises the **Tauri WebView** itself (e.g. via a supply-chain attack on an npm dependency) to inject JavaScript that calls our own `invoke()` commands legitimately — at that point they have the same access the real user's UI has.
- Has **physical access to an already-unlocked, unattended session** (shoulder-surfing, walking up to an open laptop). Use your OS's screen lock; TZVault's own auto-lock is a backstop, not a replacement.
- Performs **supply-chain attacks on our dependencies** before they reach a released build (see [Known limitations](./README.md#known-limitations--pre-release-checklist) — signed releases and reproducible builds are on the roadmap, not yet implemented).

## Attack scenarios (in scope)

| # | Scenario | Expected defense |
|---|---|---|
| 1 | Attacker has `vault.db`, no password. Tries to read key names/tags/notes without unlocking. | All metadata is AES-256-GCM encrypted; file contains zero plaintext. |
| 2 | Attacker has `vault.db`, brute-forces the master password offline. | Argon2id KDF makes each guess expensive; parameters chosen per OWASP 2023 guidance (`crypto/kdf.rs`). |
| 3 | Attacker has `vault.db` + correct master password, tries to read an extra-protected key without its per-key password. | Second independent AES-256-GCM layer; Vault Key alone is insufficient. |
| 4 | Attacker takes ciphertext from Key A and tries to substitute it for Key B (or metadata ciphertext for secret ciphertext) to see if it decrypts. | AAD (Associated Data) binds each ciphertext to its specific `key_id` and purpose (`"{id}"` vs `"{id}:meta"`) — swapped ciphertext fails AES-GCM authentication. |
| 5 | Attacker repeatedly guesses the master password through the running app (online brute-force). | Increasing delay per failed attempt (frontend) + Argon2id cost per attempt (backend) — see `UnlockScreen.tsx`. |
| 6 | Malicious/buggy frontend code tries to read the Vault Key or a key's secret without the user clicking "Show"/"Copy". | Decryption only happens inside Rust commands explicitly invoked by real UI actions; the Vault Key never crosses the IPC boundary to JS. |
| 7 | Attacker examines error messages returned to the UI for information disclosure (stack traces, file paths, DB internals). | All internal errors are logged server-side only (`stderr`) and collapsed to a single generic `ERR_INTERNAL` code before reaching the frontend — see `error.rs`. |
| 8 | Attacker with disk access after the process exits looks for the master password or Vault Key left behind in a swap file / crash dump. | Sensitive buffers are wiped via `zeroize` when no longer needed (though see limitations below — this is best-effort, not a guarantee against OS-level swapping). |

## Explicitly out of scope (for now)

- **Database-file-level encryption**: the SQLite file structure itself (table names, row count, schema) is inspectable even though field values are encrypted. Full-disk encryption via SQLCipher is planned but not yet implemented — see the README checklist.
- **Code signing / build integrity**: unsigned dev builds offer no guarantee the binary you're running matches this source code. Don't treat an unsigned build as trustworthy for real secrets.
- **Memory safety guarantees beyond what Rust's type system provides**: we use `zeroize` on a best-effort basis, but we make no formal guarantee against sophisticated memory-forensics attacks (cold boot attacks, DMA attacks, etc.).
- **Mobile platforms**: this threat model covers the desktop build only.

## Reporting

Found something within scope above (or a scenario we didn't think to list)? See [SECURITY.md](./SECURITY.md) for how to report it privately.
