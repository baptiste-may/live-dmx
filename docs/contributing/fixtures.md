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
    channels: Vec<String>,             // human-readable channel names, index 0 = offset 1
    channel_defaults: Option<Vec<u8>>, // parsed default DMX values per channel
    shutter_open_values: Option<HashMap<usize, u8>>, // 0-indexed channel -> open DMX value
    pan_bounds: Option<[i32; 2]>,      // degrees, e.g. [-270, 270]
    tilt_bounds: Option<[i32; 2]>,     // degrees, e.g. [-135, 135]
}
```

- `channel_count` is the highest offset found in the mode; holes are filled with
  `"Channel N"`.
- Offsets are 1-based in the XML and 0-based in the vectors/maps; shutter open
  values are 0-based like the engine and pipeline expect.
- `channel_defaults` is filled for **every** channel of the mode: the GDTF value
  when present, otherwise 128 for pan/tilt coarse, 255 for XYZ, 0 elsewhere. The
  field is therefore `Some` even without explicit `Default` attributes.

On the show side these are embedded as `EmbeddedGdtfDefinition` /
`EmbeddedGdtfMode` (`src-tauri/src/show/types.rs`), stored in
`ShowData.fixture_definitions` (`HashMap<String, EmbeddedGdtfDefinition>`). `.ldmx`
files embed them so shows are portable and fully functional offline. An embedded
definition has no `categories` field: a patched fixture's categories live in
`PatchedFixture.fixture_type.categories` (preserved by `clean_fixture_type`).

## Built-in generic fixtures

Defined in `src-tauri/src/fixtures/generic.rs` inside a `OnceLock` and available
immediately, offline, without any download. `check_fixtures_installed` always
returns `true` because of them.

Available generics (manufacturer `"Generic"`):

| Fixture          | `fixture_type_id`  | Categories             | Modes                                                             |
| ---------------- | ------------------ | ---------------------- | ----------------------------------------------------------------- |
| Dimmer           | `generic-dimmer`   | `Dimmer`               | `1 Channel`                                                       |
| RGB PAR          | `generic-rgb`      | `Color Changer`, `PAR` | `3 Channel (RGB)`                                                 |
| RGBW PAR         | `generic-rgbw`     | `Color Changer`, `PAR` | `4 Channel (RGBW)`, `5 Channel (Dimmer + RGBW)`                   |
| RGBAW+UV PAR     | `generic-rgbaw-uv` | `Color Changer`, `PAR` | `6 Channel (RGBAW+UV)`, `10 Channel (Dimmer + Strobe + RGBAW+UV)` |
| Strobe           | `generic-strobe`   | `Strobe`               | `1 Channel (Rate)`, `2 Channel (Dimmer + Strobe)`                 |
| Moving Head Spot | `generic-mh-spot`  | `Moving Head`, `Spot`  | `Standard (16ch)`, pan `[-270,270]`, tilt `[-135,135]`            |
| Moving Head Wash | `generic-mh-wash`  | `Moving Head`, `Wash`  | `Standard (12ch)`, pan `[-270,270]`, tilt `[-135,135]`            |

Other modes have neither bounds nor explicit defaults. Generics are injected at
the head of catalog search results.

### Adding a built-in generic fixture

Follow the `add-generic-fixture` skill
(`.agents/skills/add-generic-fixture/SKILL.md`) — entry shape, `generic-*` id
convention, DIN categories, pan/tilt bounds, icon plumbing for new categories,
and tests.

## GDTF parser (src-tauri/src/fixtures/parser.rs)

`parse_gdtf_archive(&[u8]) -> Result<ParsedGdtfFixture, String>` reads
`description.xml` from the in-memory ZIP archive, at the root or in a
subdirectory (`ends_with("/description.xml")`); the first match wins. A missing
or invalid archive returns an explicit error.

### Modes and channels

At the close of a `DMXMode`, `max_offset` (the largest key of the channel
`BTreeMap`) materializes offsets `1..=max_offset` in order. An absent offset
becomes `"Channel {i}"`, and missing defaults follow the pan/tilt/XYZ rule. A
`DMXMode` without a `Name` is ignored. A `FixtureType` without a `Name` fails the
parse.

### Attribute mapping

`map_gdtf_attribute_to_name(attribute, is_fine)` converts DIN SPEC 15800
`ChannelFunction` `Attribute` refs into human-readable names:

| GDTF attribute                                        | Name                    |
| ----------------------------------------------------- | ----------------------- |
| `ColorAdd_R`, `ColorAdd_RY`                           | Red                     |
| `ColorAdd_G`                                          | Green                   |
| `ColorAdd_GY`                                         | **Lime**                |
| `ColorAdd_B`, `ColorAdd_BC`                           | Blue                    |
| `ColorAdd_W`, `ColorAdd_WW`, `ColorAdd_CW`            | White                   |
| `ColorAdd_A`                                          | Amber                   |
| `ColorAdd_UV`                                         | UV                      |
| `ColorSub_C` / `_M` / `_Y`                            | Cyan / Magenta / Yellow |
| `Shutter1`, `Shutter1Strobe`, `StrobeRate`, `Shutter` | Strobe                  |
| anything else                                         | the original string     |

A `Fine` suffix on the attribute **or** an index > 0 in a multi-byte offsets list
appends `" Fine"` (`Pan Fine`, `Dimmer Fine`, ...).

### Multi-cell geometry expansion

Per DIN SPEC 15800, multi-cell/bar fixtures (pixel strips, multi-cell washlights,
bars) declare cell channels on a sub-geometry (e.g. `RGB Cluster` or `Pixel`) with
relative offsets, instantiated through `<GeometryReference Geometry="...">` and
one or more `<Break DMXBreak="N" DMXOffset="...">` elements.

The parser:

1. collects `GeometryReference`s keyed by
   `(root_geometry, target_geometry, dmx_break)` with their `DMXOffset`;
2. looks up the direct key `(mode_geometry, channel_geometry, DMXBreak)`;
3. otherwise aggregates every offset of the same geometry and break, whatever
   the root;
4. computes the absolute offset `inst_offset = break_offset + relative_offset - 1`;
5. duplicates the channel on each instance.

Example: three RGB channels on a geometry referenced at offsets 1 and 4 become
six channels `Red, Green, Blue, Red, Green, Blue`. No fixture name is hardcoded.

### Channel defaults and shutter open values

- `<ChannelFunction Default="128/1">` goes through `parse_gdtf_dmx_value`: the
  `num/bytes` syntax is scaled to the channel resolution (1–4 bytes) and the byte
  index selects the coarse/fine byte. The first value encountered for an offset
  wins (`or_insert`).
- A channel is a shutter when its attribute or `ChannelFunction` contains
  `shutter` or `strobe`.
- "Open" function names are `open`, `no function`, `on`, `led on`;
  `pulse`, `ramp`, `strobe`, `random`, `effect` and `close` are excluded.
- The open value comes from `Default`, else `DMXFrom`, else 255; it is stored at
  `first_offset - 1` (0-based). A `ChannelSet` named "open" overrides it with its
  `DMXFrom`; a `Default` > 0 is a fallback via `entry().or_insert()`.
- The pipeline later replaces any open value ≤ 15 with 255 (closed/LED-off
  ranges).

### Geometry helpers

- A `NoFeature` attribute on a geometry whose name contains `RGBW`, `RGBA` or
  `RGB` synthesizes the missing colours in order R, G, B, (W or A), cycling over
  the geometry. This repairs GDTFs that declare cells without attributes.
- A geometry containing `beam` (without `plate`) renames `Dimmer` to
  `Dimmer Beam` and `White` to `White (Beam)`; `plate` (without `beam`) renames
  `Dimmer` to `Dimmer Plate`. Explicit `ChannelFunction` names win, including
  their `Fine` variants. These names feed the BeamDimmer/BeamWhite heuristics in
  `docs/contributing/rendering.md`.

### Physical bounds

`parse_physical_bounds(from, to)`:

- non-finite or equal values → `None`;
- when `min >= 0` and `max > 0` → `[-span, +span]` with
  `span = round((max - min) / 2)`;
- otherwise → `[round(min), round(max)]` in order.

Bounds are searched on the `LogicalChannel` first, then the `ChannelFunction`;
the first value found for Pan and Tilt wins. Missing values stay `None` and the
`GroupSettings` defaults apply.

### Categories

- The `FixtureType` `Categories` attribute is a comma-separated list, split and
  trimmed, preserved **verbatim** (no case normalization).
- When absent or empty, `infer_categories_from_attributes` derives them in this
  order: `Pan`+`Tilt` → `Moving Head`; any `Color*` → `Color Changer`;
  `Shutter*`/`Strobe*` → `Strobe`; `Dim*` → `Dimmer`; `Laser` → `Laser`.
- No recognized attribute → `None`.
- An explicit category is **never** completed by inference: it replaces the
  inferred result.

## Categories & icon plumbing (frontend)

The categories flow:

```
GDTF FixtureType Categorie → parser.rs categories
  → ParsedGdtfFixture.categories
  → GdtfCatalogItem.categories (catalog.rs) / PatchedFixture.fixture_type
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

## Preservation in `.ldmx`

- `clean_fixture_type` keeps `name`, `shortName`, `fixtureKey`, `manufacturerKey`
  and `categories`; everything else is dropped.
- `clean_mode` keeps `name`, `shortName`, `channels`, `panBounds`, `tiltBounds`,
  `channelDefaults` and `shutterOpenValues`.
- Object channels (`{ name, capabilities }`) are **reduced to their name** during
  cleaning: capabilities are not persisted in the patched mode. Wheel/CCT routing
  then falls back to the fallback palette or to capabilities still in memory.
- `clean_patched_fixture` applies both and keeps `id`, `groups`, `name`,
  `universe`, `address`.
- **Do not break these helpers**: categories, bounds and shutter values are needed
  by the engine and the UI after reload.

## GDTF-Share catalog (src-tauri/src/fixtures/gdtf_share/)

- `client.rs` — `GdtfShareClient`: HTTP client against gdtf-share.com (search,
  catalog list, `.gdtf` download). A static `OnceLock` instance is created per
  process (`get_client` in commands.rs).
- `catalog.rs` — `GdtfCatalogManager`: local cache + multi-token search + category
  fallbacks.
- `types.rs` — `GdtfCatalogItem`, `GdtfCatalogMode`, `GdtfShareCredentials`.

### HTTP client

`GdtfShareClient` wraps a shared `ureq::Agent` (`Arc<Mutex<Agent>>`) with a **15 s**
connect timeout, a **60 s** read timeout, and cookie support enabled in
`Cargo.toml` so the session survives between requests.

| Operation    | Method and URL                                                      |
| ------------ | ------------------------------------------------------------------- |
| Login        | `POST https://gdtf-share.com/apis/public/login.php`                 |
| Catalog list | `GET https://gdtf-share.com/apis/public/getList.php`                |
| Download     | `GET https://gdtf-share.com/apis/public/downloadFile.php?rid={rid}` |

Login sends `{ "user", "password" }` and expects `{ result, error, notice }`.
`is_logged_in()` only reflects an in-memory flag; `logout()` clears it and
**recreates the agent**, dropping session cookies. `fetch_catalog_list()` ignores
entries without `fixture` or `manufacturer`, tolerates missing fields, converts
`rating` (numeric or textual), and creates modes with `channels: None`,
`panBounds: None`, `tiltBounds: None`. `download_gdtf_file` returns raw bytes;
parsing and validation happen afterwards.

### Cache layout

`GdtfCatalogManager` is created once per process in a `OnceLock<RwLock<...>>`
(`fixtures/commands.rs`). At construction it loads the local cache and the online
index, ignoring read errors.

| Data                | Path                                                                                       |
| ------------------- | ------------------------------------------------------------------------------------------ |
| Online index        | `<app_data_dir>/gdtf_catalog.json`                                                         |
| Downloaded archives | `<app_data_dir>/fixtures/<manufacturer>/<fixture>[_<first 8 chars of fixtureTypeId>].gdtf` |
| Preview cache       | memory only, `MAX_PREVIEW_ENTRIES = 30`                                                    |

`load_local_fixtures()` scans the `fixtures/` root and **one** level of
subdirectories. Every archive is parsed; the key is `"<manufacturer>@<fixture>"`
in lowercase. Generics are not stored in this map: they are injected into search
and into `get_local_fixture`.

`save_downloaded_gdtf()` sanitizes manufacturer and name (any non-alphanumeric
character other than `-` and `_` becomes `_`), creates the directory and writes
the file. Two fixtures producing the same path overwrite each other: the filename
is not a uniqueness key.

### Preview cache

`save_preview_gdtf()` keeps the archive in memory under **two keys** (the
requested pair and the pair actually parsed) and shares the bytes through
`Arc<Vec<u8>>`. At 30 entries it removes an arbitrary key — this is **not** an
LRU policy. `persist_preview_fixture()` removes the entry and writes the archive
to disk, promoting a preview to an installed fixture.

### Search

`search(query)` splits the query on spaces. **All** tokens must appear
(case-insensitive substring) in the fixture name, the manufacturer or a mode
name. Results are assembled in this order:

1. **generics** (`is_generic: true`, `is_installed: true`);
2. **local fixtures** (`.gdtf` from the cache, `uploader: "Installed"`);
3. **online catalog**, only when the query has at least one token, capped at
   **100** added results.

An empty query therefore returns **only** generics and local fixtures: the online
index is never listed unfiltered.

Deduplication uses the key `"<manufacturer>@<fixture>"` in lowercase.
`is_better_item` applies this order:

1. a generic beats an online item;
2. an installed fixture beats a non-installed one;
3. an uploader whose name starts with `manuf` (case-insensitive) beats a
   community upload;
4. better `rating` (difference > 0.05);
5. larger `rid` (most recent revision).

Missing fields are merged: channels, pan/tilt bounds, categories and modes are
completed from the existing entry or the local cache. `is_installed` stays true
once any source has it.

### Category fallback by name

For an **online, non-installed** item without categories,
`infer_categories_from_name` applies conservative keywords:

| Pattern in the name                             | Category added |
| ----------------------------------------------- | -------------- |
| `strobe`                                        | Strobe         |
| `blinder`                                       | Blinder        |
| `laser`                                         | Laser          |
| `smoke`, `hazer`, `fog`                         | Smoke          |
| `matrix`                                        | Matrix         |
| `dimmer`                                        | Dimmer         |
| `confetti`                                      | Confetti       |
| `geyser`                                        | Geyser         |
| `pyro`                                          | Pyro           |
| `water`                                         | Water          |
| `wind`                                          | Wind           |
| word `scanner`                                  | Scanner        |
| word `pixel`                                    | Pixel Bar      |
| word `par`                                      | PAR            |
| word `spot`                                     | Spot           |
| word `wash`                                     | Wash           |
| `moving`, word `head`, word `beam`, or `sharpy` | Moving Head    |

Several categories can be added. An unknown name stays `None` until installed and
parsed. An installed fixture always uses parsed categories, never name inference.

### On-demand fetch flow (fixtures/commands.rs)

`gdtf_fetch_mode` resolves a mode in this order:

1. **local cache or generic**: `get_local_fixture`, then case-insensitive
   `trim`med mode-name match;
2. **preview**:
   - `persist = true` (default): `persist_preview_fixture` tries to promote an
     existing preview to disk;
   - `persist = false`: `get_preview_fixture` reads memory only;
3. **download**: when a `rid` is provided and the user is authenticated,
   `download_gdtf_file` then `save_downloaded_gdtf` or `save_preview_gdtf`
   depending on `persist`.

Without a `rid` and without a local copy, the command fails with an explicit
message. If the requested mode is absent from the downloaded archive, it fails
too. All heavy work runs inside `tauri::async_runtime::spawn_blocking`.

Other entry points:

| Command                       | Behaviour                                                                                                                       |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------- |
| `gdtf_search`                 | Catalog search (above)                                                                                                          |
| `gdtf_sync_catalog`           | Requires a session, fetches the list, reloads local, writes `gdtf_catalog.json`, returns the item count                         |
| `gdtf_import_file`            | Parses a third-party `.gdtf`, writes it to the cache, returns a `GdtfCatalogItem` (`uploader: "Imported"`, rating 5, installed) |
| `reload_fixtures`             | Rescans the `fixtures/` directory                                                                                               |
| `check_fixtures_installed`    | Always `true`: generics are available                                                                                           |
| `gdtf_get_credentials_status` | Attempts a login with the vault credentials                                                                                     |
| `gdtf_set_credentials`        | Logs in, then saves to the vault                                                                                                |
| `gdtf_logout`                 | Deletes the credentials, clears memory and resets the client                                                                    |

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
- **Session:** `GDTF_CREDS` holds the credentials in memory for the process.
  `ensure_logged_in` reloads the vault when needed and retries a login on demand;
  `gdtf_get_credentials_status` does the same explicitly. A restart keeps only the
  encrypted credentials, never an HTTP session.

### GDTF-Share invariants

- The online index **never** appears in an empty search.
- Generics always win deduplication and are never replaced by an online version.
- An installed fixture beats a non-installed one, even with a lower rating.
- Name inference applies only to online, non-installed, category-less items.
- Parsed categories of an installed fixture are never replaced by inference.
- Online search adds at most 100 results per query.
- The preview cache is capped at 30 entries with arbitrary eviction, not LRU.
- Filenames are sanitized: two different names can target the same path and
  overwrite each other.
- `gdtf_sync_catalog` requires a login, even though the catalog is public.
- `persist` defaults to `true`; a tooltip preview must pass `false`.
- A download or parse failure returns a `String`, never a panic.

## MVR import (DIN SPEC 15801) — src-tauri/src/show/mvr.rs

`show_import_mvr` builds a full show from a `.mvr` archive. It **replaces** the
current show; it does not merge with the existing patch.

### Entry point and progress

`show_import_mvr(path, app)` is a **synchronous** command. It calls
`import_mvr_file`, which reads the file, caches embedded `.gdtf` archives in the
local catalog (`GdtfCatalogManager::save_downloaded_gdtf`) when an `AppHandle` is
provided, then runs `import_mvr_from_bytes_with_progress` with a callback that
emits `emit_progress(ShowStatus::Loading, pct, msg)`:

| Step    | Message                                              |
| ------- | ---------------------------------------------------- |
| 10 %    | Reading MVR archive...                               |
| 20–50 % | Parsing fixtures... (per ZIP entry)                  |
| 55 %    | Reading scene layout...                              |
| 55–90 % | Patching fixtures... (35 % spread over the fixtures) |
| 95 %    | Finalizing show...                                   |

Progress stops at 95 %; the command and the UI handle the transition to loaded.
Afterwards `show_import_mvr` replaces `state.show`, resets history empty, resets
the live engine, then emits `show-state-changed`. The imported show is `Loaded`,
`is_dirty = true`, with no file path — it must be saved explicitly.

### Reading the archive

`import_mvr_from_bytes_with_progress` opens the ZIP and walks every entry:

- `GeneralSceneDescription.xml`, at the root or in a subdirectory;
- otherwise the **first** entry whose name ends with `description.xml`;
- every `.gdtf` entry is read and parsed; an invalid GDTF archive is ignored
  without failing the import.

Without any `GeneralSceneDescription.xml` the import fails with
`GeneralSceneDescription.xml not found in MVR archive`. An invalid ZIP also fails.

Each parsed GDTF is indexed under four keys: the full lowercase entry name, the
base name, the stem (without extension), and `"<manufacturer>@<fixture>"` in
lowercase plus the fixture name alone. These keys resolve `GDTFSpec`, which may
contain a path, a filename or an identifier.

### XML parser

`parse_mvr_xml` reads `Layer` and `Fixture` elements, including fixtures outside
a layer (attached to the first layer, or to a `Default` layer created if needed).
Per fixture it collects:

| MVR field                        | Use                                           |
| -------------------------------- | --------------------------------------------- |
| `name` attribute                 | Display name, disambiguated later             |
| `uuid`                           | Kept in the parsed model, unused by the patch |
| `GDTFSpec`                       | Definition resolution                         |
| `GDTFMode`                       | Mode selection                                |
| `FixtureID` / `FixtureIDNumeric` | Plot id when valid and unique                 |
| `UnitNumber`                     | Collected, unused by the current patch        |
| `Address` (with `break`)         | One or more addresses with their break        |

Repeated `Address` elements are all kept. The final choice prefers break `0` or
`1`, then the first available address.

### DMX addressing

`parse_dmx_address` accepts:

| Form                  | Example | Result     |
| --------------------- | ------- | ---------- |
| `Universe.Address`    | `2.100` | `(2, 100)` |
| `Universe/Address`    | `3/25`  | `(3, 25)`  |
| `Universe,Address`    | `4,50`  | `(4, 50)`  |
| Absolute integer      | `513`   | `(2, 1)`   |
| `0`, empty or invalid | `0`     | `(1, 1)`   |

Absolute integers are **1-based** over 512-channel universes: `1 → (1,1)`,
`512 → (1,512)`, `513 → (2,1)`, `1025 → (3,1)`. Universe and address are clamped
to `1..=512`.

For a fixture without an address, `find_first_free_span` searches the first free
space on universes 1–512, accounting for already reserved ranges. Explicit ranges
are pre-reserved with the resolved mode footprint (clamped to `1..=512`) so an
unaddressed fixture never overlaps an explicitly addressed one.

### Definition and mode resolution

`resolve_fixture_gdtf_and_mode` applies:

1. embedded GDTF matching `GDTFSpec` (full name, base name or stem);
2. generic inferred from `GDTFSpec`;
3. generic inferred from the fixture name;
4. fallback "1-channel dimmer" definition named after the spec stem.

Generic matching is deliberately simple:

| Pattern                     | Generic          |
| --------------------------- | ---------------- |
| `dimmer` or `intensity`     | Dimmer           |
| `rgbw`                      | RGBW PAR         |
| `rgbaw` or exact `uv` token | RGBAW+UV PAR     |
| `rgb`                       | RGB PAR          |
| `spot`                      | Moving Head Spot |
| `wash`                      | Moving Head Wash |
| `strobe`                    | Strobe           |

The `uv` token is matched as a whole word: `Chauvet` does **not** trigger the UV
generic (dedicated test). For the mode: case-insensitive match on `name` or
`shortName`, otherwise the **first** mode of the definition, otherwise a
`1 Channel` fallback with a `Dimmer` channel. An embedded definition is registered
once per `"<manufacturer>@<fixture>"` lowercase key, with `rid: None` and all its
modes converted to `EmbeddedGdtfMode`.

### Groups

The imported show always contains the `All` group (`id 0`). Then:

- a non-empty, non-default layer becomes a group with an incremental id from 1;
- an empty layer is ignored;
- fixtures on a "default" layer join an existing group whose name matches the
  fixture model, otherwise a new group named after the model.

Layer names go through `sanitize_group_name`: decorative characters
`> < [ ] * ~ - _`, leading/trailing spaces and newlines are removed; an empty
name becomes `Group`.

Default layers (after `trim`, lowercase) are: `calque par défaut`,
`calque par defaut`, `calque de dessin-1`, `calque-1`, `design layer-1`,
`design layer 1`, `default`, `default layer`, `none`, `unassigned`, and the empty
string. Model → existing-group matching is exact first (case-insensitive), then
fuzzy: prefix/suffix, or a common word of at least 3 characters. The `All` group
is never used for this match.

### Identifiers and names

`allocate_fixture_ids`:

1. keeps `FixtureIDNumeric`, or a parseable `FixtureID`, only when the integer is
   > 0;
2. keeps a candidate id when it is **unique** in the file;
3. assigns the first free integer from 1, avoiding ids already used.

`disambiguate_fixture_name`: empty name → `"Fixture <id>"`; unique name →
unchanged; duplicate name → suffix `" <FixtureID>"` (or the allocated id), unless
the name already ends with that suffix. Names are counted before allocation; only
non-empty names participate.

### Show construction

Each patch entry receives the allocated id, disambiguated name, resolved universe
and address, the layer group (or none for a fixture outside a layer), a
`fixtureType` JSON (`name`, `shortName`, `fixtureKey` = lowercase name with spaces
→ `_`, `manufacturerKey` = lowercase manufacturer with spaces → `_`, **no
categories**) and a `mode` JSON (`name`, optional `shortName`, `channels`, plus
`channelDefaults`, `shutterOpenValues`, `panBounds`, `tiltBounds` when present).

Fixtures are sorted by `(universe, address, id)`. Used universes (always at least
universe 1) become sorted `UniverseOutputConfig`s with `output: "none"` and
`enabled: true`: no network output is configured automatically.

The resulting `ShowData` has `status: Loaded`, `identity.displayName` = filename
without extension (`"MVR Show"` when empty), `file.path: None`, `is_dirty: true`,
no undo/redo, empty `groups.settings`, and `fixtureDefinitions` filled with the
resolved definitions.

### MVR invariants

- The import replaces the show; it never merges with the current patch.
- History is emptied: no undo after an import.
- The show is dirty with no path: saving is mandatory.
- Empty layers are ignored; "default" layers do not create a group.
- Invalid GDTFs are skipped; the import continues with generics or the fallback.
- An embedded definition has no categories; do not look for them in
  `fixtureDefinitions`.
- The first mode is the fallback when the requested mode is absent.
- Explicit addresses win; unaddressed fixtures must not overlap reserved ranges.
- Plot ids are kept only when positive and unique.
- Progress stops at 95 % in the callback.
- `show_import_mvr` is synchronous and also resets the live engine.
- No output universe is routed to a protocol: `output` is `"none"`.

### Tests

`tests/show/mvr.rs` covers: all `parse_dmx_address` forms and bounds, `parse_mvr_xml`
layers/fixtures/spec/mode/FID/addresses, a full import with an embedded GDTF
(groups, patch, universes, definitions), Dimmer and RGBW PAR generic fallbacks,
an archive without XML (explicit error), the UV false positive (`Chauvet Spot` →
Moving Head Spot), `sanitize_group_name`/`is_default_layer`, `allocate_fixture_ids`
(uniques, collisions, mixed valid/missing), `disambiguate_fixture_name` (unique,
duplicate, existing suffix, empty), complete transformations (decorative layers,
empty layer, merged default layer, patch sorting), and a real `.mvr` when
`LIVE_DMX_MVR_FIXTURE` points to an existing file.

## Frontend fixture types

`src/types/fixtures.ts` defines the TS structure mirroring catalog items and
fixture definitions (`FixtureType`, modes, `categories`, ...). Keep it in sync
when backend structs gain fields.
