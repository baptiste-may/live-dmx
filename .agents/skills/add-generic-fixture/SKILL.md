---
name: add-generic-fixture
description: 'Use when adding or editing a built-in generic fixture (manufacturer "Generic") in src-tauri/src/fixtures/generic.rs, including its DIN categories, pan/tilt bounds, icon plumbing for new categories, and catalog/parser tests.'
---

# Add a built-in generic fixture

Reference: `docs/gdtf-fixtures.md`. Generics are available offline immediately (no `.gdtf` archive, no download) and are injected into the browser by `GdtfCatalogManager.load_local_fixtures()`.

## Steps

1. Add a `ParsedGdtfFixture` entry to the `vec![...]` in `build_generic_fixtures()` (`src-tauri/src/fixtures/generic.rs`).
   - Pick a stable kebab-case `fixture_type_id` (convention: `generic-<name>`, e.g. `generic-dimmer`).
   - List `categories` using the exact DIN SPEC 15800 LightCategories strings so the icon mapping works (`Moving Head`, `Spot`, `Wash`, `PAR`, `Color Changer`, `Dimmer`, `Strobe`, ...).
   - Give moving heads `pan_bounds`/`tilt_bounds` (degrees); static fixtures leave them `None`.
   - Channel names must be human-readable (`"Pan"`, `"Pan Fine"`, `"Red"`, `"Dimmer"`, ...). For fine channels, mirror the `"Pan"`/`"Pan Fine"` suffix convention — the engine resolves Fine channels by suffix.
2. **Icons**: no work needed if the `categories` match an existing SVG key. For a **new** category, add an SVG in `src/assets/icons/fixtures/<key>.svg`, plus a `categoryKeyMap` entry and a `priorityKeys` entry in `src/components/icons/FixtureIcon.vue`.
3. **Tests**: add catalog-level coverage in `src-tauri/tests/fixtures/catalog.rs` if you change how generics are exposed; parser/channel coverage lives in `src-tauri/tests/fixtures/parser.rs`.

## Gotchas

- `check_fixtures_installed` always returns `true` because of the generics — do not touch that logic.
- Mode names are user-visible in the Patch form; keep them short (`"3 Channel (RGB)"`) and unique per fixture.
- `.ldmx` embeds fixture definitions; the `clean_fixture_type`/`clean_mode` helpers must keep any new engine-relevant fields you add (see skill `migrate-show-format`).

## Verify

`bun run lint:back && bun run test:back`, plus `bun run test:front` if `FixtureIcon.vue` changed.
