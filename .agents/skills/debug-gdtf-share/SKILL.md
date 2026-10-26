---
name: debug-gdtf-share
description: "Use when GDTF-Share misbehaves: login or credentials failures, fixtures missing from the browser, stale or empty catalog, download errors, or vault/keyring issues."
---

# Debug GDTF-Share (catalog, download, credentials)

Reference: `docs/contributing/fixtures.md`. Backend lives in `src-tauri/src/fixtures/gdtf_share/`; commands in `src-tauri/src/fixtures/commands.rs`.

## Fixture resolution order (`gdtf_fetch_mode`)

1. Local cache / built-in generic with a matching mode name.
2. Preview cache (`persist=false`, used by the Patch browser tooltip).
3. Download from GDTF-Share by `rid` (requires login), then persist — or keep preview-only.

If a fixture "disappears", check which step serves it and whether it was persisted.

## Cache layout (under `<app_data_dir>`)

- `gdtf_catalog.json` — online catalog index (`gdtf_sync_catalog` refreshes it).
- `fixtures/<manufacturer>/<fixture>.gdtf` — downloaded archives; `load_local_fixtures` re-parses them and injects generics.
- `gdtf_import_file` imports a third-party `.gdtf` from disk. Heavy work runs in `spawn_blocking`; the HTTP client timeouts are 15 s connect / 60 s read (not app freezes).

## Credentials vault (`auth.rs`)

- Vault: `<app_data_dir>/vault.hold`, encrypted with `tauri-plugin-stronghold` (Rust API only; the webview never sees credentials).
- Key: random 32 bytes in the OS keyring (`keyring` crate, service `fr.baptiste-may.live-dmx`, user `gdtf-vault-key`), with a permission-restricted fallback `vault.key` (0600) in `<app_local_data_dir>` when no keyring is available (e.g. headless Linux).
- Self-healing: an unopenable vault (lost key, corruption) is discarded and recreated — the user just logs in again. A genuine access error aborts instead of replacing a valid key: read the logs before "fixing" files by hand.
- Status / logout commands: `gdtf_get_credentials_status`, `gdtf_logout`.

## Common findings

- Missing categories on non-installed online items are inferred conservatively (`infer_categories_from_name`) — not a bug until the fixture is installed and parsed.
- `check_fixtures_installed` is always `true` because built-in generics exist — it does not mean the GDTF-Share cache is populated.
- Downloads require an authenticated session for some operations; check `gdtf_get_credentials_status` before blaming the query.

## Verify

If a fix is needed: `bun run lint:back && bun run test:back` (vault tests use temp dirs), then the `verify-changes` skill.
