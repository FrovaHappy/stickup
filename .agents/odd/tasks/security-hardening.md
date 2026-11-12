---
id: security-hardening
title: Security Hardening (SEC-01..SEC-10)
status: completed
created: 2026-11-12
---

# Security Hardening (SEC-01..SEC-10)

## Scope

Remediate all 10 security vulnerabilities identified across the Rust backend, Tauri configuration, SvelteKit frontend, and release CI pipeline while maintaining full feature compatibility (custom storage directories, auto-updater, AI background removal, page scanner, DB backups).

## Checklist

- [x] **SEC-01 (High):** Replace `cmd /c start` shell execution in `open_external_url` and `launch_installer` with Win32 `ShellExecuteW` and strict URL/file validation.
- [x] **SEC-02 (High):** Enable strict CSP in `tauri.conf.json` and restrict `assetProtocol.scope`, granting runtime scope to dynamic user-selected storage/DB directories.
- [x] **SEC-03 (High):** Add origin allowlist, SHA-256 checksum verification (`SHA256SUMS.txt` / asset digest), and verified installer registry for `download_update_asset` + `launch_installer`, plus CI `SHA256SUMS.txt` generation in `.github/workflows/build.yml`.
- [x] **SEC-04 (Medium):** Remove unsafe URL fallback in `process_scanned_urls`, block `data:image/svg` in `resolve_url`, enforce URL scheme checks in `domCollector.ts`, and guard `<a href>` links in `scanner/+page.svelte`.
- [x] **SEC-05 (Medium):** Enforce bounded streaming byte limits on HTTP downloads (`download_url_bytes`, `download_html`, `download_update_asset`, `model_manager.rs`).
- [x] **SEC-06 (Medium):** Enforce canonical path containment in `delete_db_backup`, `restore_db_backup`, `create_db_backup`, `delete_emote`, and GIF loop commands.
- [x] **SEC-07 (Medium):** Remove all external `curl` subprocess spawns and standardize HTTP requests on async `reqwest` (`rustls-tls`).
- [x] **SEC-08 (Low):** Validate HuggingFace HTTPS origin, `MAX_VALID_MODEL_SIZE`, and `X-Linked-Etag` / `ETag` SHA-256 verification in `model_manager.rs`.
- [x] **SEC-09 (Low):** Add integer-overflow and dimension/size caps on Win32 `CF_DIB` and MIME `GlobalSize` reads in `clipboard/windows.rs`.
- [x] **SEC-10 (Low):** Add SSRF protection (reject loopback, private RFC1918, link-local, CGNAT, ULA, and embedded credentials) on public URL fetches and redirects.
- [x] **Verify & Release Notes:** Update `changelogs/unreleased.md` under `### Security`, eliminate all warnings, and pass `pnpm run prebuild`.
