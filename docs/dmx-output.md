# DMX Output Engine & Protocols

Read this when working on the live output path: the `LiveEngine`,
`LiveEngineRunner` loop, universe routing, network protocols, physical USB
interfaces, or the live control commands (`live_set_*`, `set_dmx_*`).

## Module map (src-tauri/src/output/)

| File               | Responsibility                                                           |
| ------------------ | ------------------------------------------------------------------------ |
| `engine.rs`        | `LiveEngine` — DMX computation, effects, flashes, colors, channels       |
| `runner.rs`        | `LiveEngineRunner` — 44 Hz background loop that sends universes          |
| `commands.rs`      | Tauri command handlers for live control (`live_*`, `set_dmx_*`, `get_*`) |
| `channels.rs`      | Channel resolution per fixture type                                      |
| `colors.rs`        | Color-mixing helpers                                                     |
| `color_routing.rs` | Color preference routing (mix / wheel / CCT / presets)                   |
| `effects.rs`       | Effect definitions (bpm, width, bounce, alignment)                       |
| `types.rs`         | `GroupEffectState`, `ChannelType`, transition structs                    |
| `protocols/`       | Network DMX senders (Art-Net, sACN, OSC, KiNET, OPC)                     |
| `interfaces/`      | Physical USB DMX (discovery, Enttec framing, sender)                     |

## Show state ↔ output state

The engine runs against a clone of the show snapshotted in the runner loop:

```rust
// src-tauri/src/output/runner.rs
match show_state.lock() {
    Ok(show) => {
        let show_snapshot = show.clone();
        drop(show);
        eng.compute_dmx(&show_snapshot);
        for universe_cfg in &show_snapshot.output.universes {
            if universe_cfg.enabled { /* route to target */ }
        }
    }
}
```

`UniverseOutputConfig { universe: u32, output: String, enabled: bool }`
(src-tauri/src/show/types.rs). The `output` string selects the target:
`"none"`, `"artnet"`, `"sacn"`, `"osc"`, `"kinet"`, `"opc"`, or a **physical
device target string**. Universe config state is managed in
`src-tauri/src/show/output/commands.rs` — see `docs/backend-architecture.md` for
the undo/redo + persistence story.

## Runner loop

`LiveEngineRunner` (runner.rs) spawns one background thread at ~44 Hz
(`Duration::from_micros(22727)`, the standard DMX rate). On each tick:

1. Clone the `ShowData` snapshot.
2. `engine.compute_dmx(&snapshot)` writes every enabled universe's 512-byte
   buffer into `engine.universe_buffers`.
3. For each **enabled** universe, dispatch its buffer to the configured sender.

Senders are created once at loop start and reused. The loop stops when
`running` flips to false, or on a poisoned show-state lock (logs
`live-dmx::engine` and stops). `AppState::default()` starts the loop
automatically (see `src-tauri/src/show/state.rs`).

## Live control commands (src-tauri/src/output/commands.rs)

All of these take `State<AppState>` and lock the engine:

- Flashes/buttons: `live_set_flash(group_id, flash_type, active)`,
  `live_set_button(button_id, active)`, `live_reset_all_buttons`
- Per-group: `live_set_group_master(level: u8)`, `live_set_group_effect`,
  `live_set_group_effect_width`, `live_set_group_color`,
  `live_set_group_secondary_color`, `live_set_group_color_effect`,
  `live_set_group_color_effect_width`, `live_reset_group_effect`,
  `live_reset_group_color_effect`
- Global color: `live_set_all_groups_color`, `live_set_color_fade_speed`,
  `live_set_color_fade_order`
- Strobe: `live_set_strobe_rate(rate: u8)`, `live_set_random_strobe_rate`
- Flash fades: `live_set_simple_flash_fade`, `live_set_white_flash_fade`
- BPM: `live_set_effect_bpm(u16)`, `live_set_color_effect_bpm(u16)`
- Direct manual control: `set_dmx_channel(universe, channel, value)` and
  `set_dmx_data(universe, data[≤512])` write into `engine.manual_channels`,
  which the engine overlays on the computed buffer.
- Readback: `get_dmx_data`, `get_all_dmx_data` return the current buffers.

`+` These are registered in `lib.rs` `generate_handler!`. Any new engine command
needs registration there — see the checklist in `docs/backend-architecture.md`.

## Network protocols (src-tauri/src/output/protocols/)

Each sender implements an internal `send_universe(universe, &[u8; 512])`. Tests
live in `src-tauri/tests/output/` (one file per protocol).

| Protocol             | File        | Port       | Notes                            |
| -------------------- | ----------- | ---------- | -------------------------------- |
| Art-Net              | `artnet.rs` | 6454 (UDP) | Uses the `artnet_protocol` crate |
| Streaming ACN (sACN) | `sacn.rs`   | 5568 (UDP) | UUID + sequence tracking         |
| OSC                  | `osc.rs`    | 8000 (UDP) | Via the `rosc` crate             |
| KiNET                | `kinet.rs`  | 6038 (UDP) | Philips Color Kinetics           |
| OPC                  | `opc.rs`    | 7890 (TCP) | Open Pixel Control               |

## Physical USB interfaces (src-tauri/src/output/interfaces/)

- `discovery.rs` — cross-platform scanning: Linux `/dev/ttyUSB*`/`/dev/ttyACM*`,
  macOS `/dev/cu.*` callout devices, Windows `COM*`.
- `sender.rs` — `PhysicalDmxSender` frames each universe with **Enttec DMX USB
  Pro framing at 57,600 baud** and auto-reconnects on disconnect.
- `types.rs` — `PhysicalInterface` model surfaced to the frontend via
  `get_output_interfaces` / `scan_output_interfaces`.

## Engine internals (src-tauri/src/output/engine.rs)

`LiveEngine` is a plain mutable struct (no thread of its own) holding:

- `universe_buffers: HashMap<u16, Vec<u8>>` — computed 512-byte frames,
  published by the runner loop and readable via `get_dmx_data`.
- `manual_channels: HashMap<(u16, u16), u8>` — raw value overlays from
  `set_dmx_*` (applied after computation).
- Per-group live state: masters, effects (bpm/width/bounce/alignment), colors
  (with `FixtureColorTransition` fades), flash buttons and transition state.

Key entry points: `compute_dmx(&ShowData)`, `set_group_color`, `set_group_effect`,
`set_flash`, `reset_all`, `reset_group_*`. It must stay resettable when the patch
or groups change — `AppState::mutate_show` calls `live.reset_all()` when the patch
or the group list changed (src-tauri/src/show/state.rs).

## Serialization & history

Universe config is part of `ShowData.output`, so it is:

- Undo/redo-able: mutations go through `AppState::mutate_show` /
  `history::ShowHistory` (100-step stacks).
- Persisted in `.ldmx` files under `fixture_definitions`-adjacent `output`
  section (see `show/types.rs` `ShowFileContent`).

Do not mutate `ShowData.output` from the live commands above — universe config is
edited via `show_output_*` commands (src-tauri/src/show/output/commands.rs) so
history and the `show-state-changed` event stay coherent.
