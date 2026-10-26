---
name: app-debug-loop
description: "Use when driving or inspecting the running dev app through the Tauri MCP bridge: screenshots, DOM inspection, JS/IPC execution, backend logs and state, interactive UI debugging of live-dmx."
---

# Debug the running app (Tauri MCP bridge)

The dev app can be driven from the outside by the `tauri` MCP server (`@hypothesi/tauri-mcp-server`, configured in `~/.config/opencode/opencode.jsonc`). Reference: `docs/contributing/architecture.md`.

## Start the app

```bash
bun run dev:app
```

This passes `--config src-tauri/tauri.dev.conf.json`, which sets `app.withGlobalTauri: true` — **required** for the eval-based bridge features. Plain `tauri dev` starts the bridge but breaks screenshots, DOM/interaction, and `execute_js` (they time out silently).

## What the bridge offers

- Screenshots and DOM/interaction against the webview.
- JS/IPC execution (eval-based; needs `withGlobalTauri`).
- Backend logs and state; pure-Rust commands like `get_backend_state` work even without the flag since they don't go through the webview.

## Pitfalls

- The plugin binds to `127.0.0.1` explicitly (never `0.0.0.0` — it exposes command/JS execution on the network).
- Registered only in debug builds (`#[cfg(debug_assertions)]` in `lib.rs`) — never part of a release binary.
- Plugin commands need the `mcp-bridge:default` permission (`src-tauri/capabilities/default.json`).
- If eval-based features time out, first check you launched with `bun run dev:app`, not `tauri dev`.

## Verify

No code change: nothing to verify. If debugging leads to a fix, finish with the `verify-changes` skill.
