# GDTF Fixtures & Standards (GDTF / DIN SPEC 15800)

This document covers everything about fixture definitions: the built-in generics,
the DIN SPEC 15800 (GDTF) parser, the GDTF-Share catalog + credentials vault, and
the MVR import. Read it when touching fixture parsing, category/icon plumbing,
the fixture browser, or show serialization of fixture data.

## Standards overview

- **GDTF (DIN SPEC 15800)** — General Device Type Format. A `.gdtf` file is a ZIP
  archive whose `description.xml` describes fixture types, modes, channels, and
  physical ranges.
- **MVR (DIN SPEC 15801)** — My Virtual Rig. A `.mvr` file is a ZIP archive whose
  `GeneralSceneDescription.xml` describes the rig: layers, fixture instances, and
  DMX addressing. MVR import is documented further down in this file.

## Data model

Core parsed structures live in `src-tauri/src/fixtures/parser.rs`:

```rust
pub struct ParsedGdtfFixture {
    name, short_name: Option<String>,
    manufacturer: String,
    description: Option<String>,
    fixture_type_id: Option<String>,
    categories: Option<Vec<String>>,   // DIN LightCategories: "Moving Head", "Spot", ...
    modes: Vec<ParsedGdtfMode>,
}

pub struct ParsedGdtfMode {
    name, short_name: Option<String>,
    channel_count: usize,
    channels: Vec<String>,             // human-readable channel names
    channel_defaults: Option<Vec<u8>>, // parsed default DMX values per channel
    shutter_open_values: Option<HashMap<usize, u8>>, // channel index -> open DMX value
    pan_bounds: Option<[i32; 2]>,      // degrees, e.g. [-270, 270]
    tilt_bounds: Option<[i32; 2]>,     // degrees, e.g. [-135, 135]
}
```

On the show side these are embedded as `EmbeddedGdtfDefinition` / `EmbeddedGdtfMode`
(`src-tauri/src/show/types.rs`), stored in `ShowData.fixture_definitions`
(`HashMap<String, EmbeddedGdtfDefinition>`). `.ldmx` files embed them so shows are
portable and fully functional offline.

## Built-in generic fixtures

Defined in `src-tauri/src/fixtures/generic.rs` inside a `OnceLock` and available
immediately, offline, without any download. `check_fixtures_installed` always
returns `true` because of them.

Available generics (manufacturer `"Generic"`):

| Fixture          | `fixture_type_id`  | Categories             | Modes                                       |
| ---------------- | ------------------ | ---------------------- | ------------------------------------------- |
| Dimmer           | `generic-dimmer`   | `Dimmer`               | 1ch                                         |
| RGB PAR          | `generic-rgb`      | `Color Changer`, `PAR` | 3ch                                         |
| RGBW PAR         | `generic-rgbw`     | `Color Changer`, `PAR` | 4ch, 5ch (Dimmer+RGBW)                      |
| RGBAW+UV PAR     | `generic-rgbaw-uv` | `Color Changer`, `PAR` | 6ch, 10ch (Dim+Strobe+RGBAW+UV+Macro+Speed) |
| Strobe           | `generic-strobe`   | `Strobe`               | 1ch, 2ch (Dimmer+Strobe)                    |
| Moving Head Spot | `generic-mh-spot`  | `Moving Head`, `Spot`  | 16ch, pan `[-270,270]`, tilt `[-135,135]`   |
| Moving Head Wash | `generic-mh-wash`  | `Moving Head`, `Wash`  | 12ch, pan `[-270,270]`, tilt `[-135,135]`   |

### How to add a built-in generic fixture

1. Add a new `ParsedGdtfFixture` entry to the `vec![...]` in
   `build_generic_fixtures()` (src-tauri/src/fixtures/generic.rs).
   - Pick a stable kebab-case `fixture_type_id` (convention: `generic-<name>`,
     e.g. `generic-dimmer`).
   - List `categories` using the exact DIN SPEC 15800 LightCategories strings so
     the icon mapping works (see below).
   - Give moving heads `pan_bounds`/`tilt_bounds`; pure static fixtures leave
     them `None`.
   - Channel names should be human-readable (`"Pan"`, `"Pan Fine"`, `"Red"`,
     `"Dimmer"`, ...). For moving heads with fine channels, mirror the existing
     `"Pan"`/`"Pan Fine"` convention — the engine resolves Fine channels by
     suffix.
2. No icon work is needed if the `categories` match an existing SVG key (the
   mapping in `FixtureIcon.vue` covers the current set). If you add a **new**
   category, create an SVG in `src/assets/icons/fixtures/<key>.svg` and add a
   `categoryKeyMap` entry + a `priorityKeys` entry in `src/components/icons/FixtureIcon.vue`.
3. Generics flow into the browser through `GdtfCatalogManager.load_local_fixtures()`
   (they are injected as always-installed items). No `.gdtf` archive is involved.
4. Tests: add catalog-level coverage in `src-tauri/tests/fixtures/catalog.rs` if
   you change how generics are exposed; channel/parser coverage lives in
   `src-tauri/tests/fixtures/parser.rs`.

## GDTF parser (src-tauri/src/fixtures/parser.rs)

`parse_gdtf_archive(&[u8]) -> Result<ParsedGdtfFixture, String>` reads
`description.xml` from the in-memory ZIP archive. Key behaviors:

- **Channel mapping** — `map_gdtf_attribute_to_name(attribute, is_fine)` converts
  DIN SPEC 15800 `ChannelFunction` `Attribute` refs into human-readable names
  (`ColorAdd_R`/`ColorAdd_RY` → `Red`, `ColorAdd_G` → `Green`, `ColorAdd_GY` → `Lime`,
  `ColorAdd_B`/`ColorAdd_BC` → `Blue`, `ColorAdd_W`/`ColorAdd_WW`/`ColorAdd_CW` →
  `White`, `ColorAdd_A` → `Amber`, `ColorAdd_UV` → `UV`, `ColorSub_C/M/Y` →
  Cyan/Magenta/Yellow, `Shutter*`/`StrobeRate` → `Strobe`, otherwise passthrough).
  A `Fine` suffix (or the `is_fine` flag from multi-byte offsets) appends
  `" Fine"`.
- **Multi-cell geometry expansion** — per DIN SPEC 15800, multi-cell/bar fixtures
  (such as pixel strips, multi-cell washlights, and bar fixtures) declare cell
  channels on a sub-geometry (e.g. `RGB Cluster` or `Pixel`) with relative offsets,
  instantiated through `<GeometryReference Geometry="...">` and one or more
  `<Break DMXBreak="N" DMXOffset="...">` elements. Geometry references are keyed
  by `(root_geometry, target_geometry, dmx_break)`, enabling mode-specific root trees
  and break offsets to resolve accurately. The parser expands each channel definition
  across all matching instance offsets (`inst_offset = bo + rel_off - 1`), ensuring
  full cell footprint coverage without fixture-specific logic.
- **Channel defaults & shutter open values** — parses GDTF `ChannelFunction` `Default`
  attributes (supporting single-byte and 16-bit fraction syntax like `128/1`, `32768/2`
  scaled down to channel byte resolution) into `channel_defaults`. Additionally,
  extracts open shutter values (`"Open"`, `"No function"`, `"On"`, etc., rejecting effect/strobe
  keywords) from `ChannelSet` into `shutter_open_values` (strictly 0-indexed channel offsets
  matching engine and live output conventions). Both `channelDefaults` and `shutterOpenValues`
  are preserved across show serialization and mutations (`clean_mode`).
- **Multi-byte offsets** — Coarse/Fine/Ultra channels are combined into a single
  channel line with the `Fine` qualifier.
- **Physical bounds** — `parse_physical_bounds(from, to)` derives `pan_bounds`/
  `tilt_bounds` from GDTF `PhysicalFrom`/`PhysicalTo`. Symmetric positive spans
  become `[-span, span]`, otherwise `[min, max]`. Equal/missing/malformed values
  stay `None` (defaults are applied by `GroupSettings` on the show side).
- **Categories** — the `FixtureType` `Categories` attribute (comma-separated DIN
  LightCategories) is extracted verbatim. When absent,
  `infer_categories_from_attributes` derives them from the fixture's channel
  `Attribute` refs: `Pan`+`Tilt` → `Moving Head`, `Color*` → `Color Changer`,
  `Shutter*`/`Strobe*` → `Strobe`, `Dim*` → `Dimmer`, `Laser` → `Laser`.

## Categories & icon plumbing (frontend)

The categories flow:

```
GDTF FixtureType Categorie → parser.rs categories
  → ParsedGdtfFixture.categories
  → GdtfCatalogItem.categories (catalog.rs) / EmbeddedGdtfDefinition
  → frontend FixtureType.categories (src/types/fixtures.ts)
  → Patch browser tooltip + FixtureIcon.vue
```

`src/components/icons/FixtureIcon.vue` maps a normalized category string to an SVG
asset under `src/assets/icons/fixtures/` (e.g. `"moving head"` → `moving-head.svg`,
`"par"` → `par.svg`, `"strobe"` → `strobe.svg`). Category selection uses a
`priorityKeys` list; unknown/uncategorized categories fall back to `generic.svg`.
The SVGs are inlined eagerly at module load (`import.meta.glob`), have stroke
colors transparently recolored via `stroke`/`fill` replacements, and are served
through `v-html`.

**Important:** the `.ldmx` serialization must keep categories. `clean_fixture_type`
in `src-tauri/src/show/types.rs` explicitly preserves `categories` (plus
`name`, `shortName`, `fixtureKey`, `manufacturerKey`) and strips the rest; use it
via `clean_patched_fixture` whenever a `PatchedFixture` is serialized.

## GDTF-Share catalog (src-tauri/src/fixtures/gdtf_share/)

- `client.rs` — `GdtfShareClient`: HTTP client against gdtf-share.com (search,
  catalog list, `.gdtf` download). A static `OnceLock` instance is created per
  process (`get_client` in commands.rs).
- `catalog.rs` — `GdtfCatalogManager`: local cache + multi-token search + category
  fallbacks.
  - Cache layout: `<app_data_dir>/gdtf_catalog.json` (online catalog index) and
    `<app_data_dir>/fixtures/<manufacturer>/<fixture>.gdtf` (downloaded archives).
  - `load_local_fixtures` re-parses every cached `.gdtf` and injects generics.
  - For **non-installed online items** with no categories,
    `infer_categories_from_name` applies a conservative keyword fallback (e.g.
    `Strobe`, `Laser`, `Wash`, `PAR`, `Beam` → `Moving Head`, plus curated
    `FAMOUS_MOVING_HEAD_NAMES` such as `Sharpy`). Unrecognized names stay
    uncategorized until installed/parsed.
- `types.rs` — `GdtfCatalogItem`, `GdtfCatalogMode`, `GdtfShareCredentials`.

### On-demand fetch flow (fixtures/commands.rs)

`gdtf_fetch_mode` resolves a mode in this order:

1. Local cache / generic fixture with a matching mode name.
2. Preview cache (`persist=false`, used by the Patch browser tooltip).
3. Download from GDTF-Share by `rid` (requires login, see below), then persist
   (or keep preview-only).

`gdtf_import_file` imports a third-party `.gdtf` from disk into the local
library. `gdtf_sync_catalog` refreshes the online index. All commands run heavy
work inside `tauri::async_runtime::spawn_blocking`.

### Credentials vault (src-tauri/src/fixtures/gdtf_share/auth.rs)

GDTF-Share credentials are persisted in an encrypted Stronghold vault,
encrypted with `tauri-plugin-stronghold`'s Rust API directly (`stronghold::
Stronghold` — no JS bindings; Rust stays the sole owner of credentials).

- **Vault snapshot:** `<app_data_dir>/vault.hold`
- **Encryption key:** a random 32-byte secret unique per installation, stored in
  the OS credential store first (`keyring` crate, service
  `fr.baptiste-may.live-dmx`, user `gdtf-vault-key`), with a
  permission-restricted fallback file `vault.key` (mode `0o600`) in
  `<app_local_data_dir>` when no keyring is available (e.g. headless Linux).
  The key is only generated when both sources are empty; genuine access errors
  abort instead of silently replacing a valid key.
- **Client/record keys:** Stronghold client `gdtf`, store key `gdtf_credentials`
  (JSON `{ user, password }`).
- **Self-healing:** an unopenable snapshot (lost key, corruption) is discarded and
  recreated on next use — the user just re-enters credentials.
- **Session:** login is recovered on demand (`gdtf_get_credentials_status`) and
  cleared via `gdtf_logout`.

## MVR import (DIN SPEC 15801) — src-tauri/src/show/mvr.rs

`show_import_mvr` builds a full show from a `.mvr` archive:

1. Inspects `GeneralSceneDescription.xml` (layers → fixture instances → DMX
   addresses).
2. Unpacks embedded `.gdtf` archives into show `fixture_definitions`.
3. Maps layers to fixture groups, assigns universe/address
   (`parse_dmx_address` handles `"U.A"`, `"U/A"`, `"U,A"`, and absolute
   integers).
4. Configures `ShowOutputState.universes`.

The import drives the load progress bar through an `MvrProgressCallback`
(`import_mvr_from_bytes_with_progress`); `show_import_mvr` passes its
`AppHandle` to `import_mvr_file`, which forwards each step to `emit_progress`.

Tests: `src-tauri/tests/show/mvr.rs`.

## Frontend fixture types

`src/types/fixtures.ts` defines the TS structure mirroring catalog items and
fixture definitions (`FixtureType`, modes, `categories`, ...). Keep it in sync
when backend structs gain fields.
