# DMX Output Engine & Protocols

Read this when working on the live output path: the `LiveEngine`,
`LiveEngineRunner` loop, universe routing, network protocols, physical USB
interfaces, or the live control commands (`live_set_*`, `set_dmx_*`).

## Module map (src-tauri/src/output/)

| File               | Responsibility                                                                             |
| ------------------ | ------------------------------------------------------------------------------------------ |
| `engine.rs`        | `LiveEngine` — DMX evaluation, effects, flashes, colors, group masters                     |
| `profile.rs`       | `CompiledFixtureProfile` — strongly typed GDTF compilation into normalized channel layouts |
| `pipeline.rs`      | Pure rendering pipeline: `LogicalFixtureState` -> `encode_fixture_dmx`                     |
| `runner.rs`        | `LiveEngineRunner` — 44 Hz background loop with lock-free network/serial streaming         |
| `commands.rs`      | Tauri command handlers for live control (`live_*`, `set_dmx_*`, `get_*`)                   |
| `channels.rs`      | Channel resolution and raw capability extraction helpers                                   |
| `colors.rs`        | Color-mixing helpers                                                                       |
| `color_routing.rs` | Color preference routing (mix / wheel / CCT / presets)                                     |
| `effects.rs`       | Effect definitions (bpm, width, bounce, alignment)                                         |
| `types.rs`         | `GroupEffectState`, `ChannelType`, transition structs                                      |
| `protocols/`       | Network DMX senders (Art-Net, sACN, OSC, KiNET, OPC)                                       |
| `interfaces/`      | Physical USB DMX (discovery, Enttec framing, sender)                                       |

## Show state ↔ output state

The runner thread borrows `ShowData` without cloning it 44 times per second. Both the show lock and the engine lock are released before any network or serial I/O occurs:

```rust
// src-tauri/src/output/runner.rs
let transmissions = match show_state.lock() {
    Ok(show) => {
        let configs = show.output.universes.iter().map(...).collect();
        let mut to_send = Vec::new();
        if let Ok(mut eng) = engine.lock() {
            eng.compute_dmx(&show);
            // Snapshot only enabled universe buffers into reusable thread-local buffers
            // ...
        }
        drop(show); // Locks released immediately!
        to_send
    }
};

// Network and serial I/O happen completely outside any lock:
for (u_val, output) in transmissions {
    // send Art-Net, sACN, USB, etc.
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

1. Briefly borrow `ShowData` and `LiveEngine` under lock to compute DMX and copy enabled universe frames into reusable thread-local transmission buffers.
2. Release both locks so live UI controls and backend mutations never experience lock contention.
3. Transmit enabled universes to their respective network senders (Art-Net, sACN, OSC, KiNET, OPC) or USB serial interfaces.

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

### Channel resolution & default behaviors

Channel classification and defaults are fully generic (never tied to specific fixture models):

- **Pan & Tilt defaults**: Moving heads and battens default to center (`128` coarse, `0` fine) rather than `0` (which often sits against physical hard stops or leaves battens turned downwards), even if dummy GDTF definitions specify default 0 or leave position channels unassigned (`NoFeature`) preceding the master dimmer.
- **Dimmer & DimmerCurve classification**: Channel classification ensures curve, mode, speed, and timing parameter channels (e.g. `DimmerCurve`, `Dimmer Mode`, `Dimmer Speed`, `Dimmer Time`) are classified as `ChannelType::Other` rather than `Dimmer`, preventing single-source fixtures from falsely triggering dual-emitter segregation or driving curve parameters to 255.
- **PositionXYZ channels**: Hoists and positional DMX movers default to top/max (`255`) and are excluded from intensity master modulation. The `treat_first_as_dimmer` fallback is explicitly disabled when position channels exist.
- **Generic Shutter & Strobe handling**: Fixtures with a `Strobe`/`Shutter` channel extract the "Open" DMX value from GDTF `ChannelSet` / `ChannelFunction Default` (stored in `shutter_open_values`, supporting both 0-indexed and 1-indexed key mappings, hydrated from embedded `fixture_definitions`). If an open value is resolved $\le 15$ (which represents closed/LED OFF ranges in fixtures like Ayrton EagleStrike), the engine safely defaults to 255. Duration and curve channels (e.g. `StrobeDuration`) are excluded from shutter categorization so they don't lock flash durations to maximum. Whenever `Dimmer > 0`, the shutter is opened; when `Dimmer == 0`, it returns to default (`0` or GDTF default).
- **Batten & Moving Head Shutter heuristic**: When vendor GDTF definitions leave channels unassigned (e.g. `NoFeature` or placeholder names, such as in GLP Impression X4 Bar), unassigned initial coarse/fine pairs before Dimmer in fixtures without pan/tilt are recognized as Tilt and Tilt Fine (centered at 128). An unassigned channel immediately preceding master dimmer (e.g. Ch 1-2 Tilt, Ch 3 Shutter, Ch 4 Dimmer) is designated as `LogicalAttribute::Shutter` (driven open to 255 when Dimmer > 0). Subsequent unassigned channels remain `LogicalAttribute::Other` and default to 0.
- **Dual-emitter segregation (`BeamDimmer` & `BeamWhite`)**: Hybrid strobe/wash fixtures (such as GLP JDC1 / JDC2 IP, Chauvet Color STRIKE M) featuring independent light sources (a high-power central white strobe tube and outer RGB wash plates) are generically partitioned: in multi-dimmer fixtures with RGB mixing, dimmers whose sections contain no RGB color channels are classified as `BeamDimmer`, and isolated white channels in RGB fixtures are classified as `BeamWhite`. During standard intensity and wash looks, these remain at `0`, preventing the white strobe tube from firing continuously while the wash plates illuminate in full color. When strobe open values are defined (e.g. DMX 251 on JDC2 IP), the strobe channels output continuous/open rather than pulsing random flash.
- **Subtractive CMY vs Additive Color Mixing**: Subtractive fixtures (fixtures without RGB emitters possessing Cyan, Magenta, and/or Yellow channels) follow subtractive absorption laws: Cyan = `255 - r`, Magenta = `255 - g`, Yellow = `255 - b`. During white flash, subtractive flags are pulled down toward `0` (open white aperture) via `base * (1.0 - white_factor)`, while additive RGB channels drive towards `255`. Fixtures possessing additive RGB emitters treat any supplemental Cyan, Magenta, or Yellow channels as additive accent emitters, keeping them clean during two-color pulses or contrasting blends. Both 8-bit and 16-bit color channels (`Bits16 { coarse, fine }`) are supported, properly encoding coarse (`val >> 8`) and fine (`val & 0xFF`) bytes and zeroing both bytes when gated or when color mix routing is inactive.
- **Color neutrality with Lime (`ColorAdd_GY`)**: `ColorAdd_GY` attributes are classified as `ChannelType::Lime` (distinct from `Green`). During RGB mixing or white illumination, Lime remains at `0` to keep the white point neutral rather than shifting toward warm yellow.
- **Channel defaults fallback**: Control, mode, and auxiliary channels receive their parsed GDTF `Default` value (from `channel_defaults`).

## Serialization & history

Universe config is part of `ShowData.output`, so it is:

- Undo/redo-able: mutations go through `AppState::mutate_show` /
  `history::ShowHistory` (100-step stacks).
- Persisted in `.ldmx` files under `fixture_definitions`-adjacent `output`
  section (see `show/types.rs` `ShowFileContent`).

Do not mutate `ShowData.output` from the live commands above — universe config is
edited via `show_output_*` commands (src-tauri/src/show/output/commands.rs) so
history and the `show-state-changed` event stay coherent.
