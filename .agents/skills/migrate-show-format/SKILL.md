---
name: migrate-show-format
description: "Use when changing the .ldmx persisted show format: adding or renaming fields on ShowData/ShowFileContent, embedded fixture definitions, or the clean_* serialization helpers."
---

# Change the .ldmx show format

Files: `src-tauri/src/show/types.rs` (`ShowData`, `ShowFileContent`, `clean_*` helpers), `src-tauri/src/show/commands.rs` (`show_save` / `show_load`, `write_show_file`). Reference: `docs/backend-architecture.md` §Show model.

## Rules

1. **Wire shape is camelCase** — every serialized struct uses `#[serde(rename_all = "camelCase")]`.
2. **Backward compatibility**: new fields on persisted structs need `#[serde(default)]` (follow `output` / `fixture_definitions` in `ShowFileContent`) so older `.ldmx` files still load.
3. **Use the clean helpers when writing**: `clean_fixture_type` / `clean_mode` / `clean_patched_fixture` strip non-essential fields but MUST keep `categories`, name/shortName/fixtureKey/manufacturerKey, bounds, channel names, `channelDefaults`, and `shutterOpenValues` — extend them when you add identifying or engine-relevant fields.
4. **Atomic writes**: saving goes through `write_show_file` (random temp file in the destination dir + atomic rename). Do not bypass it.
5. **History interplay**: `is_dirty` compares the current snapshot against `saved_snapshot`. `ShowSnapshot` and `ShowFileContent` are separate shapes — a field that belongs in both must be added to every relevant struct and to `snapshot()` / `restore_snapshot()`.
6. There is **no format version field** today: prefer additive changes. If a breaking change is unavoidable, accept both shapes on load rather than rejecting old files.

## Tests

- `src-tauri/tests/show/serialization.rs` — round-trip and defaulted-field coverage.
- `src-tauri/tests/show/history.rs` — if snapshot/dirty semantics change.
- Frontend types in `src/types/` if the wire shape changes.

## Verify

`bun run lint:back && bun run test:back`; `bun run test:front` when types changed. Then the `verify-changes` skill.
