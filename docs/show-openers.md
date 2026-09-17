# Opening Shows from the OS (Deep-Link, File Association, Single-Instance)

Read this when touching how `.ldmx` files get opened from outside the app: the
`live-dmx://` scheme, the `.ldmx` file association, CLI-argument openings,
single-instance forwarding, or the logging configuration.

## How a show reaches the app

There are three entry points, all funneling into `handle_open_request`:

1. **CLI argument / file double-click** — when the OS opens the app with a file
   argument. A second launch instance forwards its arguments to the running
   instance (single-instance plugin) instead of starting a new process.
2. **Deep link `live-dmx://open?path=...`** — via the deep-link plugin
   (`plugins.deep-link.desktop.schemes` in `tauri.conf.json`).
3. **macOS `RunEvent::Opened`** — handled in the run callback for macOS (and
   other platforms using that event variant).

Registration order and wiring live in `src-tauri/src/lib.rs` `run()`:

```rust
builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, cwd| { ... })) // first
builder = builder.plugin(tauri_plugin_process::init());
// ... then log, deep-link, window-state
// setup(): tauri_plugin_updater::Builder::new().build()
```

The single-instance callback resolves relative args against `cwd` before
forwarding, focuses the existing main window, and calls `handle_open_request`.

## Backend flow (src-tauri/src/show/open.rs)

- `extract_show_path(candidate: &str) -> Option<String>` — pure function,
  normalizes one raw candidate into a show path:
  - plain paths (`/abs/path.ldmx`, relative, Windows drive `C:\...`)
  - `file://` URLs (`url.to_file_path()`)
  - `live-dmx://open?path=...` (URL-decoded query param)
  - rejects anything that does not end in `.ldmx` (case-insensitive), rejects
    flag-like arguments starting with `-`, empty strings, and unrelated schemes.
  - Unit-tested exhaustively in the same file.
- `handle_open_request(app, &[String])` — picks the first valid path, logs under
  the `live-dmx::open` target, stores it in `PendingOpenStore` (managed as
  Tauri state), and emits the `open-show-request` Tauri event to the frontend.
- `get_pending_open_path` — Tauri command the frontend calls on mount to retrieve
  a pending path _before_ the webview could listen. Reading it **takes** the
  value (so a request is only delivered once).

## Frontend flow (src/libs/show-opener.ts)

`useShowOpener()` (wired in `src/App.vue`'s setup):

1. On mount, subscribes to the `open-show-request` event (`OPEN_SHOW_EVENT`).
2. Also calls `invoke("get_pending_open_path")` to catch requests that arrived
   before the listener was active (e.g. a deep link right after launch).
3. Each path is deduplicated (skip if already loaded or already opening) and
   loaded through `useApp().loadShow(path)`.

## Testing tips

- File associations only take effect once the app is **bundled/installed**. During
  development, test with:
  - `tauri dev -- <path>.ldmx` (CLI argument), or
  - a deep link: `live-dmx://open?path=<urlencoded-absolute-path>`.
- Rust unit tests for path extraction are in `src-tauri/src/show/open.rs`;
  frontend tests in `src/libs/show-opener.test.ts`.

## Logging configuration

Backend logging uses `tauri-plugin-log` with three targets (src-tauri/src/lib.rs):

- **Stdout** — custom colored formatter (`render_record`/`log_formatter`,
  `colored` feature): timestamp dimmed, level bold + severity-colored (Error
  bright red, Warn bright yellow, Info bright green, Debug blue, Trace bright
  black), target dimmed, Error/Warn messages tinted.
- **LogDir** — plain timestamped file `live-dmx` in the platform log dir
  (`clear_format()` + non-colored formatter).
- **Webview** — same plain format, surfaced via `attachConsole()` in
  `src/libs/logger.ts` (`initLogger()`).

Subsystems log under dedicated targets: `live-dmx::show`, `live-dmx::engine`,
`live-dmx::open`, `live-dmx::main`, `live-dmx::test`. Level is `Debug` in debug
builds, `Info` in release. Rotation keeps all files, capped at 1 MB each.
