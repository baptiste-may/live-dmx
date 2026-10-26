---
name: add-tauri-command
description: "Use when adding, renaming, or changing a Tauri v2 backend command in this repo (#[tauri::command]). Covers the thin handler, registration in lib.rs, plugin permissions, frontend invoke wiring through Pinia, undo/redo via apply_mutation, and tests on both sides."
---

# Add or modify a Tauri backend command

Reference: `docs/contributing/architecture.md` (module maps, state/history model, command patterns).

## Checklist

1. **Rust — implement** the `#[tauri::command]` in the right module (thin handler → dedicated logic in a service/utility). Return `Result<T, String>`; never leak crate-specific error types. Heavy work (network, catalog, IO) goes through `tauri::async_runtime::spawn_blocking` (see `src-tauri/src/fixtures/commands.rs`).
2. **Register** it in `tauri::generate_handler![...]` in `src-tauri/src/lib.rs`. Forgetting this means `Command X not found` at runtime.
3. **Permissions** — only for _plugin_ commands: if the command calls a plugin API, add the matching permission to `src-tauri/capabilities/default.json`. Custom app commands registered in step 2 need no entry.
4. **Frontend — call it**: `invoke("command_name", { camelCaseArgs })` from `@tauri-apps/api/core`. Wire state through a Pinia store (see `docs/contributing/frontend.md`) instead of calling `invoke` ad hoc from deep components.
5. **Tests**: Rust coverage under `src-tauri/tests/` (mirrored harness) + mock `invoke` in frontend tests (`vi.mocked(invoke)` — see skill `write-frontend-test`).
6. **Events**: if the command changes show state, go through `AppState::apply_mutation` so `show-state-changed` fires and undo/redo + `is_dirty` stay coherent. Use `state.mutate_show` only when the caller emits itself. Never mutate `state.show` directly.
7. **Docs**: update `docs/contributing/architecture.md` (and the matching docs file) when the command opens a new area; update the command inventory in `docs/contributing/dmx-output.md` for live commands.

## Gotchas

- Patch or group-list changes reset the live engine (`live.reset_all()`) — that is handled inside `apply_mutation`, do not duplicate it.
- Long-running handlers (`show_load`, `show_save`, MVR import) emit progress via `emit_progress` and flip `ShowStatus` to `Saving` / back to `Loaded`; follow the existing patterns in `src-tauri/src/show/commands.rs`.
- Universe output config is edited via the `show_output_*` commands (`src-tauri/src/show/output/commands.rs`) so history and events stay coherent — never mutate `ShowData.output` from live commands.

## Verify

`bun run lint:back && bun run test:back`, plus `bun run test:front` when the frontend was touched (or the `verify-changes` skill).
