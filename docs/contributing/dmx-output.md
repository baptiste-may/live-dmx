# DMX Output Engine & Protocols

Read this when working on the live output path: the `LiveEngine`,
`LiveEngineRunner` loop, universe routing, network protocols, physical USB
interfaces, or the live control commands (`live_set_*`, `set_dmx_*`). Rendering
internals (profiles, channels, colours, effects, pixels) are documented in
`docs/contributing/rendering.md`.

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
`src-tauri/src/show/output/commands.rs` — see `docs/contributing/architecture.md` for
the undo/redo + persistence story.

## Runner loop

`LiveEngineRunner` is the only owner of the render thread. It spawns one
background thread at ~44 Hz (`Duration::from_micros(22727)`, the standard DMX
rate). It never persists anything, never mutates the show, and never decides
targets — the configuration comes from `ShowData.output.universes`.

`AppState::default()` creates the runner (`running = false`) and calls
`start_loop`, which spawns a detached thread and sets `running = true`.
`start_loop` is a no-op if the loop already runs (`swap(true)`).

On each tick:

1. Briefly borrow `ShowData` and `LiveEngine` under lock to compute DMX.
2. Copy each enabled, non-`"none"` universe frame into a reusable
   `send_buffers` entry and remember its target.
3. Release both locks before I/O, so network and serial transmission never hold
   them. A live control can still wait while `compute_dmx` runs.
4. Transmit each frame to its network sender or USB serial interface.
5. Sleep the remainder of the 22 727 µs period.

There is **no catch-up**: a tick that overruns the period is followed
immediately by the next one. The sleep compensates for elapsed time only; no
absolute clock is used. `send_buffers` is allocated once before the loop and
reused, so no frame allocation happens per tick. If a target changes, only the
stored target string is replaced.

Senders are created once at loop start and reused. The loop stops when
`running` flips to false, or on a poisoned show-state lock (logs
`live-dmx::engine` and stops). `stop_loop` is cooperative: it does not join the
thread, so it can return before the thread actually exits. The runner is not
restarted when a show loads.

### Routing

| `output`        | Target                                                        |
| --------------- | ------------------------------------------------------------- |
| `none`          | Nothing is transmitted (the buffer is still computed).        |
| `artnet`        | `ArtNetSender`                                                |
| `sacn`          | `SacnSender`, when initialization succeeded                   |
| `osc`           | `OscSender`                                                   |
| `kinet`         | `KinetSender`                                                 |
| `opc`           | `OpcSender`                                                   |
| any other value | `PhysicalDmxSender::send_to_target(target, buf)` — USB target |

`enabled == false` suppresses transmission even when the target is valid. A
patched but unconfigured universe is computed but never sent, and a manual
overlay on such a universe is not sent either. An unknown target string is
treated as a serial port path, so a typo fails at port open, silently on the
loop.

The `universe` is a `u32` in the configuration but cast to `u16` for routing:
numbers above 65 535 wrap.

### Errors

- Poisoned `show` lock → log `live-dmx::engine`, set `running = false`, exit.
- sACN initialization failure → `None`, the universe is silently skipped.
- sACN/OPC `send_universe` and USB `send_to_target` errors are ignored by the
  loop; the senders log their own failures (or stay silent, see below).
- Art-Net, OSC and KiNET do not return `Result`; sending is best-effort.

### Universe configuration commands

All go through `apply_mutation`, so they are undoable and emit
`show-state-changed`:

| Command                                      | Rule                                                                              |
| -------------------------------------------- | --------------------------------------------------------------------------------- |
| `show_output_get_universes`                  | Returns a copy of the list                                                        |
| `show_output_set_target(universe, output)`   | `Universe N not found` when absent                                                |
| `show_output_set_enabled(universe, enabled)` | Same validation                                                                   |
| `show_output_add_universe`                   | Appends `max + 1` with `output = "none"` and `enabled = true`; returns the number |
| `show_output_remove_universe(universe)`      | Refuses to remove the **last** universe; errors when absent                       |

The default state is a single universe `{ universe: 1, output: "none", enabled: true }`.

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
needs registration there — see the checklist in `docs/contributing/architecture.md`.

## Engine internals (src-tauri/src/output/engine.rs)

`LiveEngine` is a plain mutable struct (no thread of its own). It holds:

| Field                                                     | Role                                                                |
| --------------------------------------------------------- | ------------------------------------------------------------------- |
| `universe_buffers: HashMap<u16, Vec<u8>>`                 | Computed 512-byte frames, read by `get_dmx_data`/`get_all_dmx_data` |
| `manual_channels: HashMap<(u16, u16), u8>`                | Raw `set_dmx_*` overlays, applied after computation                 |
| `flash_states`, `button_states`                           | Per-group flash buttons and generic UI buttons                      |
| `flash_fade_transitions`                                  | Release fades for simple/white/master flashes                       |
| `group_effects`, `group_color_effects`                    | Intensity and colour effect per group                               |
| `group_rng_permutations`, `group_color_rng_permutations`  | Memoized RNG permutations per group                                 |
| `group_masters`                                           | 0–100 master per group (default 100)                                |
| `group_colors`, `group_secondary_colors`                  | Primary and secondary colour per group                              |
| `fixture_color_transitions`, `fixture_last_colors`        | Per-fixture colour fades and last emitted colour                    |
| `strobe_rate`, `random_strobe_rate`                       | Strobe speeds (default 50)                                          |
| `simple_flash_fade`, `white_flash_fade`                   | Release fade durations (default 0)                                  |
| `effect_bpm`, `color_effect_bpm`                          | Effect BPM (default 60)                                             |
| `color_fade_speed`, `color_fade_order`                    | Colour transition (default 1.0 s, all simultaneous)                 |
| `cached_fixtures`, `cached_keys`, `cached_group_settings` | Compiled profiles and group positions                               |

At construction, universes 1–4 are pre-allocated and the **All** group (0) is
set to `white`. `button_states` is UI state: it is stored and reset but has no
effect on DMX.

### Tick order (`compute_dmx`)

```text
compute_dmx(show):
  1. create a 512-byte buffer for every show.output universe and every patched fixture
  2. fill every buffer with 0
  3. ensure_fixtures_cache(show)   // recompile only when patch/mode/settings change
  4. for each cached fixture:
       compute_fixture_dmx(...)     // effect, colour, master, pixels, encoding
  5. apply manual_channels         // overwrites the computed buffer, channel by channel
  6. purge finished transitions    // colour and flash fades
```

Consequences: the manual overlay always wins, and buffers are zeroed every tick
— nothing persists between ticks except `manual_channels`.

### Fixture cache

`ensure_fixtures_cache` rebuilds the whole cache whenever any of these changes:

- the fixture count;
- a fixture's `id`, `groups`, `universe`, `address` or `mode`;
- the whole `show.groups.settings` set.

Each cache entry holds the compiled `profile`, `primary_group_id =
fixture.groups.first()` (0 when ungrouped), `group_positions` for every group
including group 0, and `group_pixel_steps` derived from the pixel preference.
`reset_all` empties the cache. A settings change rebuilds it on the next tick
without a reset.

### Intensity, master and max power

```text
group_master   = master(primary) / 100 * max_power(primary) / 100
global_master  = master(0)        / 100 * max_power(0)       / 100
master_mult    = group_master * global_master
intensity      = clamp(effect(0..255) * master_mult, 0, 255)
max_power_fact = max_power(primary)/100 * max_power(0)/100
```

`master(g)` and `max_power(g)` default to 100. An ungrouped fixture has
`primary_group_id == 0`, and `max_power(primary)` is forced to 100 so the global
limit is not applied twice. `max_power_factor` carries the absolute ceiling to
the pipeline **without** the masters, so flash and strobe boosts are clamped to
it. The final value is `clamp(dimmer, 0, ceiling)`: a group limited to 50 % can
never exceed 128, even during a flash or a strobe (dedicated tests).

### Flashes

`set_flash` accepts `simple`, `white`, `master`, `strobe`, `randomStrobe` and
the alias `random_strobe`.

- A flash is active when its group **or group 0** enabled it.
- `simple` modulates intensity; `white` and `master` share the white factor.
- `flash_dimmer_factor = max(simple_factor, white_factor)`, aggregated over all
  the fixture's groups and group 0.
- On release, a fade is created when `simple_flash_fade`/`white_flash_fade` > 0;
  the factor decays linearly. Group 0 is the fallback for groups without their
  own fade.

The pipeline applies `intensity + (1 - intensity) * flash_dimmer_factor`, then
clamps to `max_power_factor`.

### Strobe

- Normal strobe: `freq = 1 + rate/100 * 19` Hz, on when
  `sin(elapsed * freq * 2π) > 0`; `rate` is 0–100 (default 50).
- Random strobe: `compute_random_strobe_active` decides per fixture and per tick
  from the fixture id, the time and `random_strobe_rate`.
- An active strobe **replaces** the computed intensity: the pipeline emits the
  ceiling (`max_power_factor`) on the on-phase and 0 otherwise.
- Priority in `effective_dimmer`: flash > strobe > intensity.

### Colours and pixels

The engine resolves the current colour (including transitions), the secondary
colour, the colour effect and per-pixel states, then builds
`LogicalFixtureState`. The algorithms live in
`docs/contributing/rendering.md`; the encoding lives there too.

- `white_accent` becomes 1.0 when the three normalized components are ≥ 0.784.
- Multi-pixel fixtures produce one state per pixel with the same
  `max_power_factor` and flash factor; their steps come from `group_pixel_steps`.
- A fixture whose `start_addr >= 512` is skipped. Encoding is bounded to
  `min(start_addr + footprint, 512)`.

### Manual overlays

| Command                                     | Effect                                                            |
| ------------------------------------------- | ----------------------------------------------------------------- |
| `set_dmx_channel(universe, channel, value)` | Writes one raw value                                              |
| `set_dmx_data(universe, data)`              | Writes up to 512 values from channel 1; extra bytes are ignored   |
| `get_dmx_data(universe)`                    | Copy of the buffer, or 512 zeros when the universe does not exist |
| `get_all_dmx_data()`                        | All buffers                                                       |

Overlays are not historized, not persisted, and not restricted to configured
universes. They survive ticks until overwritten or `reset_all` is called.

### Reset (`reset_all`)

`reset_all` returns the engine to its construction state: flashes, buttons,
effects, RNG permutations, masters, secondary colours, group 0 back to `white`,
strobe rates 50, fades 0, BPM 60, colour fade 1.0 s with no order, colour
transitions, last colours, manual overlays, fixture/settings caches, and zeroed
universe buffers.

Triggers (see `src-tauri/src/show/state.rs`): loading a show,
`AppState::default()`, and any mutation that changes the **patch** or the
**group list**. A `groups.settings` change does **not** reset.

## Network protocols (src-tauri/src/output/protocols/)

Each sender is self-contained, owns its socket, and exposes
`send_universe(universe: u16, data: &[u8])`; the runner always passes a
512-byte frame. Senders are created once before the loop, receive an already
computed frame, never read `ShowData`, and have no thread.
No protocol has a configurable destination IP: broadcast/multicast and loopback
are hardcoded.

| Protocol             | File        | Transport     | Port | Addressing                                   |
| -------------------- | ----------- | ------------- | ---- | -------------------------------------------- |
| Art-Net              | `artnet.rs` | UDP           | 6454 | 1-based universe → 0-based Port-Address      |
| Streaming ACN (sACN) | `sacn.rs`   | UDP multicast | 5568 | universes 1–63999, multicast `239.255.hi.lo` |
| OSC                  | `osc.rs`    | UDP           | 8000 | address `/dmx/universe/{n}`                  |
| KiNET                | `kinet.rs`  | UDP           | 6038 | universe as a 32-bit little-endian value     |
| OPC                  | `opc.rs`    | TCP           | 7890 | channel 0–255 (universes 1–256)              |

### Art-Net

`build_artnet_packet(universe, data, sequence)` produces a **530-byte** frame via
the `artnet_protocol` crate: `Art-Net\0`, OpCode `0x5000` (little-endian),
ProtVer 14 (big-endian), sequence, 16-bit Port-Address, length 512, then 512
channels. The 1-based DMX universe becomes a 0-based Port-Address
(`universe - 1`, 0 when universe is 0); an invalid Port-Address falls back to 0.
Data is zero-padded or truncated to 512. The sequence starts at **1**,
increments per send, and wraps from 255 back to 1 (0 is never used).

`ArtNetSender` opens an ephemeral UDP socket on first `new()`, enables broadcast,
and sends to `255.255.255.255:6454` then `127.0.0.1:6454`. A bind failure leaves
the socket `None` and sends are silently dropped; `send_universe` returns no
error.

### sACN / E1.31

`build_sacn_packet(universe, data, sequence, cid)` produces a **638-byte** frame
with Root, Framing and DMP layers: ACN Packet Identifier `ASC-E1.17`, 16-byte
CID, Source Name `Live-DMX Controller`, priority 100, synchronization address 0,
options 0, start code 0, then 512 channels. The universe must be in **1..=63999**
or construction fails with `Universe out of sACN valid range`. Short data is
implicitly zero-padded by the fixed frame; long data is truncated to 512.

`SacnSender::new()` derives a UUID-v4-shaped CID from nanoseconds, the PID and a
constant, then forces the RFC 4122 version/variant bits. Sequences are per
universe in a `HashMap<u16, u8>`, starting at 0 and using `wrapping_add`.
Initialization can fail (bind); the runner then stores `None` and skips sACN.
Destination is multicast `239.255.{high}.{low}:5568` plus a copy to
`127.0.0.1:5568`. `send_universe` returns `std::io::Result`; the runner ignores
the error.

### OSC

A single OSC message addressed `/dmx/universe/{universe}` with **one blob
argument** holding the DMX bytes, encoded with `rosc` and padded to 4 bytes. No
universe validation, no sequence. `OscSender` sends to `127.0.0.1:8000` then
broadcast `255.255.255.255:8000`. `send_universe` returns no error; a bind
failure makes sends silent. The address/blob layout is a local convention, not a
standardized DMX-over-OSC profile.

### KiNET

A **532-byte** frame: 20-byte header + 512-byte payload. Header: magic
`0x0401DC4A`, version 1, type `0x0101` (DMX out), sequence 0, port 0, flags 0,
timer 0, universe as a 32-bit little-endian value. The universe is cast to
`u32`; no range is validated. Data is truncated then zero-padded to 512.
`KinetSender` sends to `127.0.0.1:6038` then broadcast
`255.255.255.255:6038`. No sequence, no error return.

### OPC

A **TCP client** to `127.0.0.1:7890` (no broadcast, no configurable IP).
Universes are accepted in **1..=256**; the OPC channel is `universe - 1`
(0–255). Frame: `[channel, 0x00, len_hi, len_lo, data...]`, where command `0x00`
is "Set 8-bit pixel colors" and the length is the data length, capped at 65 535.
Connection is lazy: first send, 20 ms connect timeout, `TCP_NODELAY`, 20 ms write
timeout. After a connection failure, the next attempt is delayed by **1 second**
(`last_reconnect_attempt`). A write error drops the stream (`None`) and returns
`Err`; reconnection happens on a later tick. Without an OPC server,
`send_universe` returns `Err` — expected by the tests. The runner ignores it.

### Lifecycle and errors

| Aspect       | Behaviour                                                                       |
| ------------ | ------------------------------------------------------------------------------- |
| Creation     | Once per loop, before the `while`                                               |
| Socket       | Bound to `0.0.0.0:0` for UDP; local TCP for OPC                                 |
| Buffer       | Each protocol builds its own frame; the DMX buffer is never modified            |
| Bind failure | UDP: silent inactive sender; sACN: `None`; OPC: `Err` on send                   |
| Send failure | Art-Net/OSC/KiNET: ignored; sACN/OPC: `Err` returned then ignored by the runner |
| Reconnection | UDP: none (stateless socket); OPC: per tick, at most one attempt per second     |
| Sequence     | Art-Net global to the sender; sACN per universe; none elsewhere                 |

No sender logs its failures. To diagnose, use the DMX monitor (engine buffers)
then a network capture; see the `debug-live-output` skill. To add a protocol end
to end, follow the `add-dmx-protocol` skill
(`.agents/skills/add-dmx-protocol/SKILL.md`).

## Physical USB interfaces (src-tauri/src/output/interfaces/)

- `discovery.rs` — cross-platform scanning: Linux `/dev/ttyUSB*`/`/dev/ttyACM*`,
  macOS `/dev/cu.*` callout devices, Windows `COM*`.
- `sender.rs` — `PhysicalDmxSender` frames each universe with **Enttec DMX USB
  Pro framing at 57,600 baud** and auto-reconnects on disconnect.
- `types.rs` — `PhysicalInterface` model surfaced to the frontend via
  `get_output_interfaces` / `scan_output_interfaces`.

### Model

```rust
PhysicalInterface {
    id: String,      // "physical-1", "physical-2", ...
    name: String,    // "Physical 1", ...
    device: String,  // USB product, manufacturer, or a generic label
    port: String,    // "/dev/ttyUSB0", "COM3", "/dev/cu.usbmodem101", ...
    status: String,  // always "connected" at discovery time
    fps: u32,        // display metadata, 44
}
```

Serialized in camelCase. `fps` and `status` are display information: they are
neither measured nor updated live.

### Discovery

`discover_physical_interfaces()` calls `serialport::available_ports()` then
filters:

1. **Bluetooth excluded** — `BluetoothPort` or a name containing `bluetooth`.
2. **macOS** — `/dev/tty.*` entries are ignored; only `/dev/cu.*` callout
   devices are kept (opening a `tty.*` blocks waiting for DCD).
3. **Port type** — `UsbPort` uses product, then manufacturer, then
   `USB Serial (<port>)`; `PciPort` is kept only when its name matches the
   patterns below; `Unknown` uses `Serial Port (<port>)`.
4. **Accepted name patterns** — `USB`, `acm`, `cu.`, a `com` prefix, or
   `\\.\com`. This covers Linux `ttyUSB*`/`ttyACM*`, macOS `cu.usbserial*`/
   `cu.usbmodem*`, and Windows `COM1`…`COMn`.

Each detected interface gets `id = physical-{n}`, `name = Physical {n}` (1-based
position), `status = "connected"` and `fps = 44`. An `available_ports` error is
returned as a `String`. An empty result is valid.

`default_physical_interfaces()` returns three fictional interfaces (Enttec DMX
USB Pro, FTDI Open DMX, DMXking ultraDMX MAX) with OS-appropriate ports. It is
used by tests only; production always discovers real ports.

### Enttec DMX USB Pro framing

`build_enttec_pro_packet(dmx_data)` produces a **518-byte** frame:

| Offset | Byte     | Meaning                                     |
| ------ | -------- | ------------------------------------------- |
| 0      | `0x7E`   | Start of message                            |
| 1      | `0x06`   | Label "Output Only Send DMX Packet Request" |
| 2–3    | LSB, MSB | Length = `min(len, 512) + 1`                |
| 4      | `0x00`   | DMX start code                              |
| 5..517 | data     | 512 channels, zero-padded                   |
| 517    | `0xE7`   | End of message                              |

The length includes the start code: 513 for a full frame (`0x0201`). Short data
is padded to 512 before the end byte; long data is truncated.

### Sending and target resolution

`PhysicalDmxSender` holds `open_ports: HashMap<String, Box<dyn SerialPort>>`,
`cached_interfaces: Vec<PhysicalInterface>` and `last_interface_scan:
Option<Instant>`.

`resolve_port(target)`:

- `physical-N` → rescan when the cache is older than **5 s** or the id is
  missing; return its `port`, or `None` when absent;
- any other string → returned **as-is** as a port path, with no validation.

`send_to_target(target, data)` resolves the target, errors with
`Interface target '<target>' not found` when there is no interface, then calls
`send_universe(port, data)`. `send_universe` reuses an open port or opens
`serialport::new(port_name, 57_600)` with a **50 ms** timeout, writes the Enttec
frame with `write_all`, and drops the port from the cache on a write error. There
is no acknowledgement read: the interface is considered driven once the serial
write succeeds.

Reconnection is lazy: a dropped port is reopened on the next send; a
`physical-N` target triggers a scan when the cache is stale or the id is absent,
so a permanently missing id can scan on every tick. There is no backoff and no
watchdog thread. If the device is unplugged, writes fail, the port is forgotten,
and later ticks retry until it is plugged back in.

Both `get_output_interfaces` and `scan_output_interfaces` return
`Result<Vec<PhysicalInterface>, String>`; the second is an explicit alias. They
never modify `ShowData`.

## Serialization & history

Universe config is part of `ShowData.output`, so it is:

- Undo/redo-able: the `show_output_*` commands go through
  `AppState::apply_mutation`, which records history
  (`history::ShowHistory`, 100-step stacks) and emits `show-state-changed`.
- Persisted in `.ldmx` files under `fixture_definitions`-adjacent `output`
  section (see `show/types.rs` `ShowFileContent`).

Do not mutate `ShowData.output` from the live commands above — universe config is
edited via `show_output_*` commands (src-tauri/src/show/output/commands.rs) so
history and the `show-state-changed` event stay coherent.
