---
name: debug-live-output
description: "Use when DMX output is wrong or missing: lights do not respond, a universe is not sending, a fixture stays dark, or the DMX monitor shows unexpected values."
---

# Debug the live DMX output

Work top-down: show state → engine computation → transport. Reference: `docs/contributing/dmx-output.md`.

## 1. Is the show state sending?

- A show is loaded, universes added and **enabled** (`UniverseOutputConfig.enabled`), and the target set — `enabled=false` or `output="none"` sends nothing.
- Output view: universe cards show the target badge and live channel grid. The target string must match a runner arm (`artnet`, `sacn`, `osc`, `kinet`, `opc`, or a physical device target); unknown strings silently fall into the USB sender path.

## 2. What is the engine computing?

- `get_dmx_data` / `get_all_dmx_data` return the current universe buffers — read them via devtools or the Tauri MCP bridge (skill `app-debug-loop`).
- `manual_channels` (`set_dmx_channel` / `set_dmx_data`) are overlaid **after** computation: stale manual values override engine output in the returned buffer.
- Group limit: `max_power` (0–100) scales the master (`effective_master = master * max_power / 100`); a group at 0% stays dark regardless of the fader. Fixtures without groups fall back to the All group (id 0) and are limited once there.

## 3. Fixture-specific semantics (one fixture dark while others work)

- Shutter/strobe: whenever dimmer > 0 the shutter opens; at 0 it returns to default. A strobe channel driven continuously open instead of pulsing is expected during washes.
- Dual-emitter fixtures: `BeamDimmer`/`BeamWhite` sections stay at 0 during intensity/wash looks by design (the white strobe tube stays off). If a hybrid fixture misses a section, check this classification first.
- CMY fixtures: white flash pulls Cyan/Magenta/Yellow down to 0 (open aperture) — expected, not a bug.
- Moving heads default Pan/Tilt to center (128, fine 0), not 0 — fixtures with unassigned channels get heuristic assignment (see `docs/contributing/dmx-output.md` §Channel resolution).

## 4. Logs

- Target `live-dmx::engine`. A poisoned show-state lock **stops the loop** ("live engine loop stopping...") — output freezes until restart. Level is Debug in dev builds, Info in release.

## 5. Transport

- **Network**: check firewall/VLAN first, then capture packets — Art-Net UDP 6454 (broadcast + 127.0.0.1), sACN 5568, OSC 8000, KiNET 6038, OPC TCP 7890.
- **USB**: discovery paths `/dev/ttyUSB*` and `/dev/ttyACM*` on Linux (user must be in the `dialout` group), `/dev/cu.*` on macOS, `COM*` on Windows. Framing is Enttec DMX USB Pro at 57,600 baud with auto-reconnect; check the interface status in the Output view.

## Verify

If the diagnosis leads to a code change, finish with the `verify-changes` skill.
