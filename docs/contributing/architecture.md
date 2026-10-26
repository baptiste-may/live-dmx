# Backend Architecture (Tauri v2 / Rust)

Read this when working on backend code: Tauri commands, show state/history,
patch, groups, configure, serialization, or backend tests.

## Module map (src-tauri/src/)

| Module      | Responsibility                                                                                   |
| ----------- | ------------------------------------------------------------------------------------------------ |
| `main.rs`   | Binary entry point; delegates to `live_dmx_lib::run()`                                           |
| `lib.rs`    | Plugin registration, `generate_handler!`, logging formatters, `run()` entry                      |
| `fixtures/` | GDTF parsing/catalog/generics + credentials vault (see `docs/contributing/fixtures.md`)          |
| `output/`   | DMX engine, runner, protocols, interfaces, live commands (see `docs/contributing/dmx-output.md`) |
| `show/`     | Show state, history/undo, patch, groups, configure, MVR, open routing, output config             |

Submodules:

| Module   | Files                                                                                                                                     |
| -------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| fixtures | `commands.rs`, `parser.rs`, `generic.rs`, `gdtf_share/{client,catalog,auth,types}.rs`                                                     |
| output   | `engine.rs`, `runner.rs`, `pipeline.rs`, `profile.rs`, `channels.rs`, `colors.rs`, `color_routing.rs`, `effects.rs`, `types.rs`           |
| show     | `state.rs`, `history.rs`, `types.rs`, `commands.rs`, `identity.rs`, `configure.rs`, `patch/`, `mvr.rs`, `open.rs`, `recent.rs`, `output/` |

## Crate layout (Cargo)

- Managed from the workspace root via `cd src-tauri && cargo` scripts in
  `package.json` (`lint:back`, `test:back`, `format:back`, `build:app`).
- Tests live in `src-tauri/tests/` (integration): `fixtures/`, `output/`,
  `show/` (mirrors each module; run via `bun run test:back`).
- Key crates: `tauri v2`, `serde`, `quick-xml` (GDTF/MVR `description.xml`),
  `zip`, `artnet_protocol`, `rosc`, `serialport`, `stronghold`/`keyring`,
  `tauri-plugin-*`.

## Execution model

- **Tauri main thread** — receives commands, applies mutations, emits events.
- **Runner thread** — `AppState::default()` starts `LiveEngineRunner` at
  ~44 Hz (`Duration::from_micros(22727)`, the standard DMX rate). See
  `docs/contributing/dmx-output.md`.
- **No thread per fixture or universe** — network and USB senders are created
  once when the loop starts and reused.
- **Locks** — `show` and `engine` are `Mutex`-protected. Any code holding both
  releases them before network or serial I/O.
- **Shutdown** — the loop stops when `running` flips to `false` or when the
  `ShowData` lock is poisoned (logs `live-dmx::engine`, then stops).

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
`PendingOpenStore` (same file) holds a path received before the frontend was
ready (deep link, file association, CLI argument).

**Mutation path — never mutate `state.show` directly.** Use:

- `state.apply_mutation(app, |show| { ... })` when the frontend needs the
  `show-state-changed` event emitted (`emit_show_changed`), or
- `state.mutate_show(|show| ...)` when callers emit themselves.

Both follow the same sequence:

1. snapshot the show;
2. run the closure — on `Err`, restore the snapshot and return the error, so a
   failed mutation leaves neither history entry nor partial state;
3. compare the previous and current snapshots;
4. call `live.reset_all()` when the **patch** or the **group list** changed;
5. push the previous snapshot onto the undo stack and clear the redo stack;
6. recompute `is_dirty`/`can_undo`/`can_redo`;
7. emit `show-state-changed` when the snapshot changed (`apply_mutation` only).

### What triggers `live.reset_all()`

- any difference on `patch` — fixtures, addresses, universes, names, group
  membership;
- any difference on `groups.groups` — create, remove, rename.

Changes to `groups.settings` do **not** reset the engine: it re-reads those
settings on the next tick. Universe output config changes do not reset it
either; the runner reads the new list on the next tick.

## Undo/redo (src-tauri/src/show/history.rs)

- `ShowHistory` holds `undo_stack`/`redo_stack` of `ShowSnapshot` plus
  `saved_snapshot` (used for dirty tracking). Max stack size: 100
  (`MAX_UNDO_STACK_SIZE`).
- A mutation pushes the _previous_ snapshot onto the undo stack and clears the
  redo stack.
- `show_undo` pops undo, pushes the current snapshot onto redo, restores, then
  recomputes dirty state and resets the engine if patch/groups changed.
  `show_redo` does the inverse.
- `show_create` and `show_load` call `reset_with_saved` (history starts empty,
  current state considered clean). `show_import_mvr` and `show_unload` call
  `reset_empty` (no saved snapshot). `show_save` calls `mark_saved` with the
  current snapshot.
- `is_dirty` compares the current snapshot against `saved_snapshot`; without a
  saved snapshot the show is considered dirty.

## Show model (src-tauri/src/show/types.rs)

`ShowData` aggregates all show state. Three shapes exist, all serialized in
camelCase:

| Type              | Role                                                                                                   |
| ----------------- | ------------------------------------------------------------------------------------------------------ |
| `ShowData`        | Full runtime state: `status`, `identity`, `file`, `patch`, `groups`, `output`, `fixture_definitions`   |
| `ShowSnapshot`    | Historized subset: `identity`, `patch`, `groups`, `output`, `fixture_definitions` (no `status`/`file`) |
| `ShowFileContent` | Persisted `.ldmx` shape: `identity`, `patch`, `groups`, `output`, `fixture_definitions`                |

- `ShowStatus` is `NOT_LOADED`, `LOADING`, `SAVING` or `LOADED`
  (`SCREAMING_SNAKE_CASE` on the wire).
- `identity.displayName` is the show name.
- `patch.fixtures` holds the `PatchedFixture` list.
- `groups` holds groups plus per-group `settings`.
- `output.universes` holds `UniverseOutputConfig { universe, output, enabled }`.
- `fixture_definitions` embeds GDTF definitions keyed by
  `manufacturer@name` in lowercase.
- `output` and `fixture_definitions` carry `#[serde(default)]` so older files
  still load.

`clean_fixture_type` / `clean_mode` / `clean_patched_fixture` strip
non-essential fields before serializing (they preserve `categories`, bounds,
channel names, defaults, shutter values, and identifying keys). Use them when
writing `.ldmx`; `docs/contributing/fixtures.md` lists exactly what survives.

## Serialization & events (src-tauri/src/show/commands.rs)

- `show_get_data`, `show_create`, `show_load`, `show_save`, `show_unload`,
  `show_undo`/`show_redo`, `show_identity_update`.
- The `show-state-changed` event carries either the full `ShowData`
  (`emit_show_changed`) or a partial `{status, progress, statusMessage}`
  snapshot for long-running work (`emit_progress`); frontend stores diff/apply
  both (see `docs/contributing/frontend.md`).
- `show_load` reports progress at 5 % (`Reading show file...`), 25 %
  (`Parsing show data...`), 40–70 % (`Cleaning fixtures...`), 80 %
  (`Enriching fixture data...`), 92 % (`Applying show data...`) and 100 %
  (`LOADED`).
- `enrich_fixture_modes_bounds` fills missing `panBounds`/`tiltBounds` from the
  embedded definitions, then from the built-in generics. It runs on load and on
  `show_get_data`.
- `show_save` flips `ShowStatus` to `Saving` before writing and back to
  `Loaded` afterwards (even on error). `show_import_mvr` reports 10–95 % and
  then replaces the whole show.
- `.ldmx` is written through `write_show_file`, which serializes pretty JSON to
  `<path>.tmp.<16 hex chars>` created exclusively (`create_new`) in the
  destination directory, then renames atomically; the temporary file is removed
  on write or rename failure.
- `.ldmx` is the application's save format (extension registered via
  `bundle.fileAssociations`, see `docs/contributing/show-openers.md`). It has no
  version field: prefer additive, `#[serde(default)]`-compatible changes (see
  the `migrate-show-format` skill).

## Tauri commands — patterns

Every command is declared once, in `lib.rs` `generate_handler!`; a `pub fn`
that is not listed there is not callable from the frontend. Handlers must be
**thin** and delegate to modules.

| Domain     | Examples                                                                          | Module                                |
| ---------- | --------------------------------------------------------------------------------- | ------------------------------------- |
| Show       | `show_get_data`, `show_load`, `show_save`, `show_create`, `show_unload`           | `show/commands.rs`                    |
| History    | `show_undo`, `show_redo`                                                          | `show/commands.rs`                    |
| Identity   | `show_identity_update`                                                            | `show/identity.rs`                    |
| Configure  | `show_configure_get_overview`, `show_configure_get_group`                         | `show/configure.rs`                   |
| Patch      | `show_patch_fixture`, `show_patch_batch`, `show_patch_update_*`                   | `show/patch/`                         |
| Groups     | `show_group_create`, `show_group_remove`, `show_group_rename`                     | `show/patch/`                         |
| Output     | `show_output_get_universes`, `show_output_set_target`, `show_output_add_universe` | `show/output/`                        |
| Live       | `live_set_*`, `live_reset_*`                                                      | `output/commands.rs`                  |
| Direct DMX | `set_dmx_channel`, `set_dmx_data`, `get_dmx_data`, `get_all_dmx_data`             | `output/commands.rs`                  |
| Fixtures   | `gdtf_search`, `gdtf_fetch_mode`, `gdtf_sync_catalog`, `reload_fixtures`          | `fixtures/commands.rs`                |
| Interfaces | `get_output_interfaces`, `scan_output_interfaces`                                 | `output/interfaces/`                  |
| Opening    | `get_pending_open_path`, `get_recent_shows`, `save_recent_show`                   | `show/open.rs`, `show/recent.rs`      |
| Import     | `show_import_mvr`, `gdtf_import_file`                                             | `show/mvr.rs`, `fixtures/commands.rs` |

Rules:

- Async commands that do heavy work (catalog, network) use
  `tauri::async_runtime::spawn_blocking` (see `fixtures/commands.rs`).
- Commands returning `Result<T, String>` and taking `State<AppState>` are the
  norm. Do not leak crate-specific error types to the frontend.
- Take `State<'_, AppState>` for show/history/engine access and `AppHandle` to
  emit events or resolve paths.
- Never hold the `show` lock during a download, a scan, or a disk write.
- Prefix commands by domain (`show_`, `live_`, `gdtf_`, `set_dmx_`, `get_`).
  Live output commands never write into the show.

### Adding a new Tauri backend command

Follow the `add-tauri-command` skill
(`.agents/skills/add-tauri-command/SKILL.md`) — full checklist: module,
`generate_handler!` registration, plugin permissions, frontend wiring, tests on
both sides, `apply_mutation`, docs.

## Capabilities/permissions (src-tauri/capabilities/default.json)

Tauri v2 gates every `invoke` behind a capability. Frontend calls fail with
"not allowed" (silently or with an error log) unless the permission exists:

```json
"permissions": [ "core:default", "opener:default", "dialog:default",
  "core:path:default", "core:window:allow-close", "core:window:allow-destroy",
  "core:event:default", "global-shortcut:default", "log:default",
  "mcp-bridge:default", "updater:default", "process:default" ]
```

Custom app commands (the ones in `lib.rs`) don't need an entry here — only
**plugin** commands do. If a new plugin appears (or you call a plugin API not
yet covered by `:default`), add the permission. `core:window:allow-close` and
`core:window:allow-destroy` are explicit because `core:default` does not cover
them. The capability targets the `main` window only; a second window needs its
own capability.

## Patch & batch (src-tauri/src/show/patch/)

`show_patch_fixture`:

1. derives `channel_count` from the mode's channel list (minimum 1);
2. validates `universe ∈ [1, 512]`, `address ≥ 1` and
   `address + channel_count - 1 ≤ 512`;
3. refuses any address collision with a fixture on the same universe (the error
   names the existing fixture and its range);
4. allocates `id = max(existing ids) + 1` (1 when the patch is empty);
5. cleans the fixture type and mode with `clean_fixture_type`/`clean_mode`;
6. appends the fixture with `groups: vec![group_id]`.

`show_patch_batch` creates `count` fixtures from a universe, start address,
channel count, name, type and mode. It addresses on an absolute channel line:

```text
abs = (universe - 1) * 512 + (start_address - 1)
```

For each fixture: if `abs % 512 + channel_count > 512`, jump to the next
universe; derive `universe = abs / 512 + 1` and `address = abs % 512 + 1`;
reject collisions with both the existing patch and the fixtures already planned
in this batch; then advance `abs` by `channel_count`. Exceeding universe 512
fails with `Patch exceeds universe bounds`.

The batch always creates a new group named after the requested name
(`extract_standard_name` strips a trailing number). A single fixture keeps the
requested name; several become `<base> 1`, `<base> 2`, ... All created fixtures
join that group, with ids from `max(existing ids) + 1`.

| Command                                     | Behaviour                                                                     |
| ------------------------------------------- | ----------------------------------------------------------------------------- |
| `show_patch_remove_fixtures(ids)`           | Removes the listed ids                                                        |
| `show_patch_update_name(ids, name)`         | One fixture: exact name; several: base + index                                |
| `show_patch_update_address(ids, address)`   | Re-addresses the selection in sequence, rolling over to following universes   |
| `show_patch_update_universe(ids, universe)` | Same, starting from the universe and address of the first fixture             |
| `show_patch_toggle_group(ids, group_id)`    | Adds the group to all if at least one lacks it, otherwise removes it from all |

Address and universe updates deliberately **do not** re-check collisions; they
move an already consistent selection. Ids are allocated by `max + 1`, never by
reusing gaps.

## Groups

`FixtureGroup { id, name }`. The **All** group always has `id = 0` and exists by
default.

| Command                                | Rules                                                                           |
| -------------------------------------- | ------------------------------------------------------------------------------- |
| `show_group_create(name)`              | `id = max + 1`                                                                  |
| `show_group_remove(id)`                | `id == 0` is a no-op; removes the group, its settings and its memberships       |
| `show_group_rename(id, new_name)`      | `id == 0` is a no-op                                                            |
| `show_group_update_settings(group, s)` | Checks the group, computes bounds/options for its fixtures, clamps, then stores |

For `group_id == 0` the concerned fixtures are **all** fixtures; otherwise those
whose `fixture.groups` contains the id. A fixture always belongs to at least one
group.

## Configure (src-tauri/src/show/configure.rs)

The backend is the source of truth for per-group configure info. The view calls:

- `show_configure_get_overview` → `ConfigureOverview { rows, groups }`;
- `show_configure_get_group(group_id)` → `Option<GroupConfigureInfo>`.

`GroupConfigureInfo` exposes `has_pan`, `has_tilt`, `has_color`, `has_pixels`,
`pan_bounds`, `tilt_bounds`, `color_options`, `pixel_options`, `max_pixels` and
the current `settings` (or computed defaults).

- `get_fixture_pan_bounds`/`get_fixture_tilt_bounds` search the patched mode,
  then `fixture_definitions`, then the built-in generics (case-insensitive
  names). `is_pan_channel`/`is_tilt_channel` exclude Fine/LSB/16-bit channels.
- `get_group_pan_bounds`/`get_group_tilt_bounds` aggregate the group's fixtures.
- `get_group_color_options` derives the available colour preferences (mix,
  wheel, CCT, presets); `get_group_pixel_options` derives pixel modes and the
  maximum pixel count.
- `clamp_group_settings` brings values back inside the real bounds and options,
  so the frontend must not expect out-of-range values to survive.

`GroupSettings` defaults: `Color Mix`, `Single` pixels, `pixel_group_count: 2`,
pan `-180..180`, tilt `-110..110`, `inverse_pan`/`inverse_tilt` false,
`max_power: 100`.

## Debug tooling — MCP bridge (dev only)

`tauri-plugin-mcp-bridge` (0.13) exposes a WebSocket server that lets the
companion `@hypothesi/tauri-mcp-server` MCP server drive a **running dev app**
(screenshots, DOM, JS/IPC execution, logs). It is tooling, not app code:

- Registered **only in debug builds** (`#[cfg(debug_assertions)]` in `lib.rs`),
  so it is never part of a release binary.
- Bound to `127.0.0.1` explicitly (the plugin defaults to `0.0.0.0`, which
  would expose command/JS execution on the network).
- Needs the `mcp-bridge:default` permission and, for eval-based features,
  `app.withGlobalTauri: true` — which lives in `src-tauri/tauri.dev.conf.json`,
  so launch dev with `bun run dev:app`, not plain `tauri dev`.
- Client config lives **outside the repo**, in
  `~/.config/opencode/opencode.jsonc` (`mcp.tauri` →
  `npx @hypothesi/tauri-mcp-server`).

For the full driver loop (what works, pitfalls), use the `app-debug-loop` skill
(`.agents/skills/app-debug-loop/SKILL.md`).

## Logging

Backend logging targets are documented in `docs/contributing/show-openers.md`.
Custom per-command/task logs should use dedicated targets (e.g.
`live-dmx::show`). The log plugin also exposes a `Webview` target, so backend
log lines can appear in the frontend console.

## Testing (backend)

- Unit tests: `#[cfg(test)]` modules next to code (e.g. `open.rs`,
  `auth.rs`, `lib.rs` log formatter).
- Integration: `src-tauri/tests/{fixtures,output,show}/` — one harness file per
  module. Cover parsers, engine behavior, protocols, history, serialization,
  MVR, and the credential vault with temp dirs.
- Notable harnesses: `tests/show/{state,history,serialization}.rs` for the
  mutation path and `.ldmx`, `tests/show/patch/{batch,groups}.rs` for patch
  rules, `tests/show/configure.rs` for capabilities and clamping,
  `tests/show/mvr.rs` for the import.
- Run with `bun run test:back`; lint with `bun run lint:back` (clippy `-D
warnings`); format with `bun run format:back`.

## Cross-cutting rules for backend changes

- Keep command handlers thin; business logic in modules.
- Use `Result` everywhere fallible; `Result<T, String>` at the Tauri boundary.
- Do not hand-roll undo/redo: go through `AppState::apply_mutation`.
- A failed mutation must leave the previous state intact (rollback).
- The `.ldmx` format is camelCase and has no version field; prefer additive,
  defaulted fields and keep the `clean_*` helpers intact.
- Respect the workspace's clippy/strict-format bar (CI runs
  `cargo clippy -- -D warnings` and `cargo fmt -- --check` on push).
