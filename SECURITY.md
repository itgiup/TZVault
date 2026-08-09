# Security Policy

TZVault is under active development and **has not yet been through an independent professional security audit**. Please do not store high-value secrets (large crypto wallets, critical production credentials) in it until this notice is removed.

## Reporting a vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Instead, use one of these private channels:

- **Preferred**: [GitHub Security Advisories](../../security/advisories/new) — lets us collaborate on a fix privately before public disclosure.
- **Email**: `security@your-domain.example` *(replace with a real address before publishing)*

When reporting, please include:

- A description of the vulnerability and its potential impact
- Steps to reproduce (proof-of-concept code/commands if possible)
- The affected component (e.g. `crypto/cipher.rs`, `commands/keys.rs`, a specific React component)
- Your suggested severity, if you have one

## Response process

- We aim to acknowledge reports within **5 business days**.
- We'll keep you updated as we investigate and develop a fix.
- Once a fix is ready and released, we'll credit you in the release notes (unless you prefer to stay anonymous).
- We currently do **not** run a paid bug bounty program. We will, however, publicly credit researchers who report valid issues responsibly.

## Scope

**In scope:**
- The Rust backend (`src-tauri/src/`), especially the `crypto/`, `vault/`, and `commands/` modules
- The Tauri IPC boundary (command inputs/outputs, permission/capability configuration)
- The React frontend's handling of secrets (memory retention, clipboard behavior, error messages)
- The SQLite storage layer and its encryption-at-rest guarantees
- The build/release pipeline (dependency supply chain, code signing)

**Out of scope (for now):**
- Physical access attacks assuming an already-unlocked, unattended machine
- Attacks requiring a compromised OS kernel, a malicious browser extension inside the WebView, or root/admin-level access on the host
- Denial-of-service against a single local desktop instance
- Social engineering

If you're unsure whether something is in scope, report it anyway — we'd rather triage a borderline report than miss a real one.

## Safe harbor

We will not pursue legal action against researchers who:
- Make a good-faith effort to avoid privacy violations, data destruction, or service disruption
- Report vulnerabilities privately through the channels above before any public disclosure
- Give us a reasonable amount of time to investigate and fix the issue before disclosing it publicly (we suggest 90 days as a starting point, negotiable based on severity and fix complexity)

## Known limitations

See the [README](./README.md#known-limitations--pre-release-checklist) for the current list of known architectural limitations we're already tracking (e.g. database-file-level encryption is not yet enabled by default). Reporting these again is welcome if you have new angles or want to help fix them, but they're not "new" findings.
