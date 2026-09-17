# Backend Architecture (Tauri v2 / Rust)

Read this when working on backend code: Tauri commands, show state/history,
patch, configure, serialization, or backend tests.

## Module map (src-tauri/src/)

| Module      | Responsibility                                                                       |
| ----------- | ------------------------------------------------------------------------------------ |
| `lib.rs`    | Plugin registration, `generate_handler!`, logging formatters, `run()` entry          |
| `fixtures/` | GDTF parsing/catalog/generics + credentials vault (see `docs/gdtf-fixtures.md`)      |
| `output/`   | DMX engine, runner, protocols, interfaces, live commands (see `docs/dmx-output.md`)  |
| `show/`     | Show state, history/undo, patch, groups, configure, MVR, open routing, output config |

## Crate layout (Cargo)

- Managed from the workspace root via `cd src-tauri && cargo` scripts in
  `package.json` (`lint:back`, `test:back`, `format:back`, `build:app`).
- Tests live in `src-tauri/tests/` (integration): `fixtures/`, `output/`,
  `show/` (mirrors each module; run via `bun run test:back`).
- Key crates: `tauri v2`, `serde`, `quick-xml` (GDTF/MVR `description.xml`),
  `zip`, `artnet_protocol`, `rosc`, `serialport`, `stronghold`/`keyring`,
  `tauri-plugin-*`.

## App state & cross-cutting flow

`AppState` (src-tauri/src/show/state.rs) is Tauri-managed state created once:

```rust
pub struct AppState {
    pub show: Arc<Mutex<ShowData>>,     // current show (cloned by the engine loop)
    pub history: Mutex<ShowHistory>,    // undo/redo/saved-snapshot
    pub output: Arc<LiveEngineRunner>,  // LiveEngine + 44Hz loop
}
```

`AppState::default()` immediately starts the output loop with the (empty) show.

**Mutation path — never mutate `state.show` directly.** Use:

- `state.apply_mutation(app, |show| { ... })` when the frontend needs the
  `show-state-changed` event emitted (`emit_show_changed`), or
- `state.mutate_show(|show| ...)` when callers emit themselves.

Both commit the change to history, recompute `is_dirty`/`can_undo`/`can_redo`,
and call `live.reset_all()` when the **patch** or the **group list** changed
(so live output drops stale references).

## Undo/redo (src-tauri/src/show/history.rs)

- `ShowHistory` holds `undo_stack`/`redo_stack` of `ShowSnapshot` plus
  `saved_snapshot` (used for dirty tracking). Max stack size: 100
  (`MAX_UNDO_STACK_SIZE`).
- Snapshot & restore are defined on `ShowData` (`snapshot()`/`restore_snapshot`),
  and mutations push the _previous_ snapshot onto the undo stack.
- `is_dirty` compares the current snapshot against `saved_snapshot`.

## Show model (src-tauri/src/show/types.rs)

`ShowData` aggregates all show state. `ShowFileContent` is the `.ldmx`
serialization shape (both are camelCase-serialized):

```
identity (ShowIdentityState)  status (ShowStatus)
file (ShowChangesState: path/dirty/unsaved modal/can_undo/can_redo)
patch (ShowPatchState: Vec<PatchedFixture>)
groups (PatchGroupsState: groups + settings)
output (ShowOutputState: Vec<UniverseOutputConfig>)
fixture_definitions (HashMap<String, EmbeddedGdtfDefinition>)
```

- `clean_fixture_type` / `clean_mode` / `clean_patched_fixture` strip
  non-essential fields before serializing (they preserve `categories`, bounds,
  channel names, and identifying keys). Use them when writing `.ldmx`.
- Casing is `camelCase` on the wire (prettier/dev vs field names above).

## Tauri commands — patterns

Command handlers must be **thin** and delegate to modules. Register every
command, in one place, in `lib.rs`:

```rust
.invoke_handler(tauri::generate_handler![
    show_...,        // show/commands.rs
    live_...,        // output/commands.rs
    ...
])
```

- Async commands that do heavy work (catalog, network) use
  `tauri::async_runtime::spawn_blocking` (see `fixtures/commands.rs`).
- Commands returning `Result<T, String>` and taking `State<AppState>` are the
  norm. Do not leak crate-specific error types to the frontend.

## Capabilities/permissions (src-tauri/capabilities/default.json)

Tauri v2 gates every `invoke` behind a capability. Frontend calls fail with
"not allowed" (silently or with an error log) unless the permission exists:

```json
"permissions": [ "core:default", "opener:default", "dialog:default",
  "core:path:default", "core:window:allow-close", "core:window:allow-destroy",
  "core:event:default", "global-shortcut:default", "log:default",
  "updater:default", "process:default" ]
```

Custom app commands (the ones in `lib.rs`) don't need an entry here — only
**plugin** commands do. If a new plugin appears (or you call a plugin API not
yet covered by `:default`), add the permission.

### Checklist — adding a new Tauri backend command

1. **Rust:** implement the `#[tauri::command]` in the right module (thin
   handler → dedicated logic). Return `Result<T, String>`.
2. **Register:** add it to `tauri::generate_handler![...]` in
   `src-tauri/src/lib.rs`. (Forget this → `Command X not found`.)
3. **Permissions:** only for _plugin_ commands — if your command calls a plugin
   API, add the matching permission to `src-tauri/capabilities/default.json`.
4. **Frontend:** call it with `invoke("command_name", { camelCaseArgs })` via
   `@tauri-apps/api/core` (see `docs/frontend-architecture.md`). Wire state
   through a Pinia store (`docs/frontend-architecture.md`).
5. **Tests:** add Rust coverage (cargo test, e.g. under `src-tauri/tests/`) and
   mock the `invoke` in frontend tests (`vi.mocked(invoke)`, doc conventions in
   `docs/frontend-architecture.md`).
6. **Events:** if the command changes show state, use `apply_mutation` so the
   snapshot event fires.
7. **Docs:** update `AGENTS.md` list of commands/the relevant `docs/` module
   section if your command is a new area.

## Debug tooling — MCP bridge (dev only)

`tauri-plugin-mcp-bridge` (0.13) exposes a WebSocket server that lets the
companion `@hypothesi/tauri-mcp-server` MCP server drive a **running dev app**
(screenshots, DOM, JS/IPC execution, logs). It is tooling, not app code:

- Registered **only in debug builds** (`#[cfg(debug_assertions)]` in `lib.rs`),
  so it is never part of a release binary.
- Bound to `127.0.0.1` via
  `Builder::new().bind_address("127.0.0.1").build()` — the plugin defaults to
  `0.0.0.0`, which would expose command/JS execution on the network.
- Permission `mcp-bridge:default` in `capabilities/default.json` (plugin
  commands need an entry; custom app commands don't).
- `app.withGlobalTauri: true` is **required**: the injected `bridge.js` calls
  `window.__TAURI__.*`, so without the global every eval-based feature
  (execute_js, screenshots, DOM/interaction, execute_command) silently times
  out. Pure Rust commands (e.g. `get_backend_state`) work either way since they
  don't go through the webview.
- That flag lives in `src-tauri/tauri.dev.conf.json` (not the base config), so
  it stays out of release builds. Launch dev with `bun run dev:app` (which
  passes `--config`). Plain `tauri dev` skips that file: the bridge and its
  Rust commands still work, but `app.withGlobalTauri` is off, so the
  eval-based features above break.
- Client config lives **outside the repo**, in
  `~/.config/opencode/opencode.jsonc` (`mcp.tauri` →
  `npx @hypothesi/tauri-mcp-server`). Start the app with `bun run dev:app`,
  then connect from the tooling side.

## Patch & batch (src-tauri/src/show/patch/)

- `batch.rs` — fixture batching (patch many fixtures at once).
- `groups.rs` — group create/remove/rename/settings.
- Commands: `show_patch_fixture`, `show_patch_batch`, `show_patch_remove_fixtures`,
  `show_patch_update_name/address/universe`, `show_patch_toggle_group`,
  `show_group_*`.

## Configure (src-tauri/src/show/configure.rs)

Backend is the source of truth for per-group configure info:
`show_configure_get_overview` / `show_configure_get_group` compute capabilities,
bounds, color options, and clamped defaults. The frontend only renders and saves
(`show_group_update_settings`).

## Serialization & events (src-tauri/src/show/commands.rs)

- `show_get_data`, `show_create`, `show_load`, `show_save`, `show_unload`,
  `show_undo`/`show_redo`, `show_identity_update`.
- The `show-state-changed` event carries either the full `ShowData`
  (`emit_show_changed`) or a partial `{status, progress, statusMessage}`
  snapshot for long-running work (`emit_progress`); frontend stores diff/apply
  both (see `docs/frontend-architecture.md`).
- `show_load` and MVR import report progress via `emit_progress`; `show_save`
  flips `ShowStatus` to `Saving` before writing and back to `Loaded` afterwards
  (even on error). `.ldmx` is written through `write_show_file`, which
  serializes to a randomly-named temp file created exclusively in the
  destination directory before an atomic rename.
- `.ldmx` is the application's save format (extension registered via
  `bundle.fileAssociations`, see `docs/show-openers.md`).

## Logging

Backend logging targets/targets are documented in `docs/show-openers.md`. Custom
per-command/task logs should use dedicated targets (e.g. `live-dmx::show`).

## Testing (backend)

- Unit tests: `#[cfg(test)]` modules next to code (e.g. `open.rs`,
  `auth.rs`, `lib.rs` log formatter).
- Integration: `src-tauri/tests/{fixtures,output,show}/` — one harness file per
  module. Cover parsers, engine behavior, protocols, history, serialization,
  MVR, and the credential vault with temp dirs.
- Run with `bun run test:back`; lint with `bun run lint:back` (clippy `-D
warnings`); format with `bun run format:back`.

## Cross-cutting rules for backend changes

- Keep command handlers thin; business logic in modules.
- Use `Result` everywhere fallible.
- Do not hand-roll undo/redo: go through `AppState::apply_mutation`.
- Respect the workspace's clippy/strict-format bar (CI runs
  `cargo clippy -- -D warnings` and `cargo fmt -- --check` on push).
