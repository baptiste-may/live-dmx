---
name: add-dmx-protocol
description: "Use when adding a new DMX-over-network output protocol (Art-Net, sACN, OSC, KiNET, OPC style) end to end: sender in output/protocols/, runner wiring, frontend universe target option, backend + frontend tests, and docs."
---

# Add a DMX output protocol

The output path: `LiveEngineRunner` (44 Hz) computes universe buffers and transmits each enabled universe to its target. The target is the `output` string of `UniverseOutputConfig` (`src-tauri/src/show/types.rs`). Reference: `docs/dmx-output.md`.

## Checklist

1. **Sender** — create `src-tauri/src/output/protocols/<name>.rs`:
   - A struct created once at loop start and reused (`new()`), holding its socket/handle.
   - `send_universe(&mut self, universe: u16, data: &[u8])` following the signature and error style of the neighbouring senders (`artnet.rs`, `sacn.rs`, `osc.rs`, `kinet.rs`, `opc.rs`). Send failures are non-fatal for the loop.
2. **Export** — add `pub mod <name>;` and `pub use <name>::*;` to `src-tauri/src/output/protocols/mod.rs`.
3. **Wire the runner** — in `src-tauri/src/output/runner.rs`: instantiate the sender next to the others (loop setup) and add a match arm on the target string in the transmit loop. The UI id and the arm string must match exactly; unknown ids fall through to the physical USB sender.
4. **Backend tests** — add `src-tauri/tests/output/<name>.rs` and register `mod <name>;` inside the `mod output { ... }` block of `src-tauri/tests/output.rs`. Follow existing protocol tests (bind a local socket, receive, assert the frame bytes).
5. **Frontend** — add the option to `VIRTUAL_PROTOCOLS` in `src/types/output.ts`, and a label in `targetLabel` in `src/views/output/dmx/OutputUniverseCard.vue`. Update affected tests (`OutputTargetSelect.test.ts`, `OutputUniverseCard.test.ts`, `OutputUniverseRow.test.ts`).
6. **Docs** — add a row to the protocol table in `docs/dmx-output.md` (file, port, notes).
7. If the protocol needs new commands or plugin permissions, follow the `add-tauri-command` skill.

## Gotchas

- **Universe numbering is protocol-specific**: Art-Net maps `universe - 1` into its PortAddress; don't copy that blindly, check the target protocol spec.
- An option that "does nothing" while enabled is often a UI/server id mismatch — the runner sends unknown target strings to the USB sender (`physical_sender.send_to_target`).
- Do not mutate `ShowData.output` from live commands; universe config changes go through the `show_output_*` commands (`src-tauri/src/show/output/commands.rs`) for undo/redo and events.

## Verify

`bun run lint:back && bun run test:back && bun run test:front`.
