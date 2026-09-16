# Project Instructions for AI Agents

Welcome to the `live-dmx` project. This document provides essential information for AI agents working on this codebase to ensure consistency, safety, and efficiency.

> [!IMPORTANT]
> This file is a living document. Agents are encouraged to update this file as they implement new features, change project structure, or identify new common patterns.

## Project Overview

`live-dmx` is a DMX lighting control application built with Tauri (Rust) for the backend and Vue 3 (TypeScript) for the frontend. It uses the Art-Net protocol for communication with lighting fixtures.

## Tech Stack

- **General:**
  - [Bun](https://bun.sh/) for package management and task running.
- **Frontend:**
  - [Vue 3](https://vuejs.org/) (Composition API, `<script setup>`)
  - [Vite](https://vitejs.dev/)
  - [TypeScript](https://www.typescriptlang.org/)
  - [Tailwind CSS](https://tailwindcss.com/) & [DaisyUI](https://daisyui.com/) with [daisy-ui-kit](https://github.com/alexandre-fernandez/daisy-ui-kit) for UI components and styling.
  - [Lucide Vue Next](https://lucide.dev/icons) for iconography.
- **Backend:**
  - [Tauri v2](https://v2.tauri.app/) (Rust)
  - [Serde](https://serde.rs/) for JSON serialization/deserialization.

## Directory Structure

- `src/`: Vue frontend source code.
  - `main.ts`: Frontend entry point.
  - `Main.vue`: Primary application layout.
  - `App.vue`: Core application component.
  - `components/`: Reusable Vue components (Modals, Toasts, Inputs, etc.).
  - `stores/`: Pinia state management (Show, Toast, Recent Shows, Fixture Browser, Patch).
  - `views/`: Components for application views (Patch, Configure, Live, Input, Output).
  - `libs/`: Frontend utilities.
  - `types/`: TypeScript type definitions.
- `src-tauri/`: Rust backend source code.
  - `src/fixtures/`: Rust logic for handling fixture data and caching (`commands.rs`, `parser.rs`, `generic.rs`, `gdtf_share/`).
  - `src/show/`: Show management, patch batching, history tracking, and show serialization (`types.rs`, `state.rs`, `history.rs`, `commands.rs`, `identity.rs`, `recent.rs`, `patch/`).
  - `src/output/`: DMX output engine and command handlers (`engine.rs`, `effects.rs`, `channels.rs`, `colors.rs`, `runner.rs`, `commands.rs`, `types.rs`).
- `fixtures/`: Cache directory in OS AppData (`<app_data_dir>/fixtures/`) containing downloaded GDTF archives (`.gdtf`). The GDTF-Share catalog index (`gdtf_catalog.json`) is cached directly in `<app_data_dir>/`.

## Coding Standards & Conventions

- **Modularity & Separation of Concerns:** Strongly prioritize splitting code into small, decoupled, and single-responsibility units across the entire stack:
  - **Vue Components:** Break down large views and UI blocks into small, focused child components.
  - **State Management:** Divide Pinia stores and state into granular, domain-specific stores.
  - **Backend (Rust):** Keep Tauri command handlers thin and split backend business logic into dedicated modules, services, and helper functions.
  - **Composables & Utilities:** Extract reusable logic and helpers into dedicated composable functions and utility modules.
- **Strings:** Use double quotes (`"`) for strings in TypeScript and script blocks.
- **Vue Templates:** ALWAYS use double quotes for HTML/Vue attributes. For string literals _inside_ attribute expressions, use single quotes (`'`) to avoid quote nesting issues (e.g., `:class="[condition ? 'active' : '']"`).
- **Language:** ALWAYS use TypeScript (`.ts`) instead of JavaScript (`.js`). All Vue components MUST use `<script setup lang="ts">`.
- **Vue Components:** Remove any empty `<script>` or `<style>` blocks. If a component does not need logic or custom styling, these tags should not be present.

### Frontend

- **Composition API:** Use `<script setup>` in SFCs.
- **Inline Handlers:** When an event handler (such as `@click`) in a Vue component only executes a simple single-line statement or method call, it can be placed directly inline in the template (e.g., `@click="isOpen = false"`) instead of defining a dedicated single-line handler in `<script>`.
- **Styling & UI Components:** Prefer Tailwind CSS utility classes. For DaisyUI, if possible, prefer using `daisy-ui-kit` components (`Button`, `Card`, `CardTitle`, `Modal`, `Table`, `Badge`, `Divider`, etc.) instead of directly using DaisyUI class names.
- **Tauri Commands:** Invoke backend commands using `@tauri-apps/api/core`.
- **State Management:** The application state is primarily managed within the Pinia store (`src/stores/show/show.ts`). Components should use `useShow()` to access and update the current show state instead of direct prop mutations. Avoid mutating props directly to satisfy Vue linting rules.
- **Notifications:** Use `useToastStore()` from `src/stores/toast.ts` to display non-blocking alerts (toasts) in the bottom-right corner. Prefer this over native `alert()` or Tauri `message()` for a consistent UI experience.
- **Testing:** Wrap components with `createTestingPinia({ stubActions: false })` in tests when store interaction is needed. Initialize state using `store.setShow(mockData)` or similar actions.

### View-Specific Logic

- **Patch View (`src/views/Patch.vue`):**
- **Setup Views:**
  - **Input View (`src/views/Input.vue`):** Input triggers and controllers mapping.
  - **Output View (`src/views/Output.vue`):**
    - Manages universe output routing and lists physical interfaces.
    - Divided into `OutputUniverses` (universe configuration with None, Physical, and Virtual target selection) and `OutputInterfaces` (hardware physical device listing and discovery).
  - **Patch View (`src/views/Patch.vue`):**
    - Handles the inventory of fixtures and their DMX addressing.
    - Uses `PatchForm` for fixture search and batch adding, `PatchVisualizer` for universe grids, and `PatchGroups` for group organization.
    - Interactions focus on speed: quick searching of manufacturers and easy adding of fixtures.
  - **Configure View (`src/views/Configure.vue`):**
    - Manages fixture parameters like Dimmer, Pan/Tilt limits, and inversions.
    - The configure store (`src/stores/show/show-configuration.ts`) is a thin view-model over the backend `show_configure_get_overview`: all group capabilities (`has_pan`, `has_tilt`, `has_color`), bounds (`panBounds`, `tiltBounds`), color options and clamped default settings come from the backend. Frontend-side guarded fallbacks (`useShowConfigurationSync` watchers + `reconcileSettings`) only handle UI state (selected group, debounced save, settings cache sync) — no business computation is duplicated.
    - Rendered through `ConfigureSelection` (group tree) and `ConfigureSettings` (per-group limits/inversions) child views.
- **Live Views (`src/views/live/pages/`):**
  - Navigated directly via the sidebar's "Live" section (Intensity, Colors).
  - **Intensity (`LiveIntensity.vue`):** Live intensity control including effects, flash buttons, and master faders.
  - **Colors (`LiveColors.vue`):** Live color control including color tables, fades, and color effects.

### Backend (Rust)

- **Tauri Commands:** Keep command handlers concise; delegate business logic to modules.
- **Error Handling:** Use `Result` for functions that can fail, especially in Tauri commands.
- **DMX Logic:** Centralize DMX computation and engine runner in the `output` module.

## DMX Fixtures & GDTF Standard

The application uses the modern DIN SPEC 15800 standard (General Device Type Format / GDTF) for fixture definitions and connects directly to the GDTF-Share open database:

- **Built-in Generics:** Essential fixtures (Dimmer 1ch, RGB 3ch, RGBW 4ch & 5ch, RGBAW+UV 6ch & 10ch, Strobe 1ch & 2ch, Moving Head Spot 16ch, Moving Head Wash 12ch) are built directly into the application binary, enabling immediate offline usage without any download.
- **GDTF-Share Browser:** The fixture browser searches thousands of fixtures from GDTF-Share (`gdtf-share.com`) with instant multi-token filtering.
- **On-Demand GDTF Fetching:** When a user patches a fixture from GDTF-Share, the `.gdtf` archive is fetched on-demand, parsed, and cached locally in `<app_data_dir>/fixtures/`.
- **Embedded Show Definitions:** Show files (`.ldmx`) embed the parsed GDTF fixture definitions (`show.fixture_definitions`), making shows 100% portable and fully functional on offline systems without external dependencies.
- **Manual Import:** Users can import any third-party `.gdtf` file directly into their library via the browser's "Import GDTF" button.
- **DIN SPEC 15800 Parsing:** Backend parser in `src-tauri/src/fixtures/parser.rs` inspects `description.xml` inside `.gdtf` zip archives, resolving multi-byte channel offsets (Coarse, Fine, Ultra) and DIN SPEC 15800 attributes (`ColorAdd_R`, `Pan`, `Dimmer`, etc.). It also extracts the `Categories` attribute of `FixtureType` (comma-separated DIN SPEC 15800 LightCategories such as `Moving Head`, `Spot`, `Color Changer`, `Strobe`, ...).
- **Categories & Icons:** Fixture categories flow from the GDTF `FixtureType` `Categories` attribute through `ParsedGdtfFixture.categories` and `GdtfCatalogItem.categories` into the frontend `FixtureType.categories`. They are surfaced in the Patch browser/tooltip via `FixtureIcon` (`src/components/icons/FixtureIcon.vue`), which maps categories to SVG assets under `src/assets/icons/fixtures/`. Built-in generic fixtures define their own categories in `src-tauri/src/fixtures/generic.rs`. `clean_fixture_type` preserves the `categories` array in `.ldmx` show files.
  - **Category inference:** Many GDTF files omit the `Categories` attribute (and the GDTF-Share catalog API does not expose it at all). When it is missing, the parser (`infer_categories_from_attributes` in `parser.rs`) derives categories from the fixture's `ChannelFunction` `Attribute` refs — `Pan`+`Tilt` → `Moving Head`, `Color*` → `Color Changer`, `Shutter*`/`Strobe*` → `Strobe`, `Dim*` → `Dimmer`, `Laser` → `Laser`. For non-installed online catalog items with no categories, `infer_categories_from_name` in `gdtf_share/catalog.rs` applies a conservative name-keyword fallback (e.g. `Strobe`, `Laser`, `Wash`, `PAR`, `Beam`→`Moving Head`, plus curated `FAMOUS_MOVING_HEAD_NAMES` such as `Sharpy`). Truly unrecognized names stay uncategorized until installed/parsed.
- **Authentication & Credentials:** GDTF-Share credentials are persisted in an encrypted Stronghold vault (snapshot at `<app_data_dir>/vault.hold`) using `tauri-plugin-stronghold`'s Rust component (`stronghold::Stronghold`) directly — no JS bindings, Rust stays the sole owner of credentials. The vault key is a random 32-byte secret unique to each installation: it lives in the OS credential store (Keychain / Credential Manager / Secret Service, service `fr.baptiste-may.live-dmx` via the `keyring` crate), with a permission-restricted fallback file (`vault.key` in `<app_local_data_dir>`) when no keyring is available (e.g. headless Linux). Unreadable vault snapshots (lost key or corruption) are discarded and recreated on next use, so the user just re-enters their credentials. Sessions are automatically recovered on-demand or established in the background, and can be cleared via `gdtf_logout`.
- **Physical Pan/Tilt Bounds:** Mode definitions include `pan_bounds` and `tilt_bounds` (`[i32; 2]`) parsed from GDTF `PhysicalFrom` and `PhysicalTo` attributes (or defaults for generic moving heads like `[-270, 270]` and `[-135, 135]`). These bounds propagate through catalog modes, embedded definitions, show serialization (`clean_mode`), and into the backend `show_configure_get_overview` (`GroupConfigureInfo.pan_bounds`/`tilt_bounds`), dynamically scaling Configure view sliders and visual grids per group.
- **MVR Import (DIN SPEC 15801):** Shows can be created directly from My Virtual Rig archives (`.mvr`) via the "Import from MVR" button on the home page or `show_import_mvr` command. The backend parser (`src-tauri/src/show/mvr.rs`) inspects `GeneralSceneDescription.xml`, unpacks embedded `.gdtf` archives into show fixture definitions, maps layers to fixture groups, assigns DMX universe/addresses, and configures output universes.
- Refer to `src/types/fixtures.ts` for the frontend structure of GDTF fixture definitions and catalog items.

## Working with DMX Output

- The DMX output engine (`LiveEngine`) and continuous background loop (`LiveEngineRunner`) are initialized in `src-tauri/src/output/runner.rs`.
- Universe configuration state (`UniverseOutputConfig`: `universe`, `output`, `enabled`) is managed in `src-tauri/src/show/output/` with full undo/redo history and persistent `.ldmx` show serialization.
- Network DMX protocols (`src-tauri/src/output/protocols/`) support Art-Net (`artnet_protocol` on port 6454), Streaming ACN / sACN (`sacn.rs` on port 5568 with UUID and sequence tracking), Open Sound Control / OSC (`rosc` on port 8000), Philips Color Kinetics KiNET (`kinet.rs` on port 6038), and Open Pixel Control / OPC (`opc.rs` on port 7890).
- Physical USB DMX interfaces (`src-tauri/src/output/interfaces/`) are discovered cross-platform across Linux (`/dev/ttyUSB*`, `/dev/ttyACM*`), macOS (`/dev/cu.*` callout devices), and Windows (`COM*`). Frames are formatted with Enttec DMX USB Pro framing at 57,600 baud and streamed with automatic reconnection on disconnect.
- The frontend sends live control updates (`live_set_*`) or direct manual channel updates via `set_dmx_channel` / `set_dmx_data` commands.

## Opening Shows from the OS (Deep-Link, File Association, Single-Instance)

- The app registers the `live-dmx://` URL scheme (`plugins.deep-link.desktop.schemes` in `tauri.conf.json`) and the `.ldmx` file association (`bundle.fileAssociations`). This allows opening show files from a double-click, an argument passed to the executable, or a deep link.
- Four plugins are registered in `src-tauri/src/lib.rs`, always in this order: `tauri-plugin-single-instance` (first, with the `deep-link` feature so a second launch forwards its arguments to the running instance), `tauri-plugin-log`, `tauri-plugin-deep-link`, and `tauri-plugin-window-state` (persists/restores the window's position and size across launches).
- `tauri-plugin-stronghold` is used as a Rust-only library (not registered as a plugin, no JS bindings): its `kdf` and `stronghold` modules back the encrypted GDTF-Share credential vault in `src-tauri/src/fixtures/gdtf_share/auth.rs`.
- Backend entry logic lives in `src-tauri/src/show/open.rs`: `handle_open_request` normalizes candidates (plain paths, `file://` URLs, `live-dmx://open?path=...` deep links), rejects non-`.ldmx` values, logs them under the `live-dmx::open` target, stores the first valid path in `PendingOpenStore`, and emits the `open-show-request` event to the frontend. Path extraction is centralized in the pure function `extract_show_path` (unit-tested).
- The frontend consumes open requests in `src/libs/show-opener.ts` (`useShowOpener()`): on mount it asks the backend for any pending path via the `get_pending_open_path` command (covers requests that arrived before the webview was ready) and listens for `open-show-request` afterwards. Paths are deduplicated and loaded through `useApp().loadShow(path)`. `App.vue` wires it up.
- Logging output is configured in `src-tauri/src/lib.rs` (stdout + `LogDir` file `live-dmx` + webview), and the frontend attaches to it via `attachConsole()` in `src/libs/logger.ts` (`initLogger()`). Backend subsystems log under dedicated targets (e.g. `live-dmx::show`, `live-dmx::engine`, `live-dmx::open`, `live-dmx::main`). The plugin is built with the `colored` feature: the `Stdout` target uses a custom `log_formatter` (rendering logic in `render_record` in `src-tauri/src/lib.rs`) that styles the line for the terminal — timestamp dimmed gray, level bold + severity-colored (Error `91m`, Warn `93m`, Info `92m`, Debug `34m`, Trace `90m`), target dimmed, and Error/Warn messages tinted red/yellow — while the `LogDir`/`Webview` targets keep the same plain timestamped format via `clear_format()` + per-target formatters.
- File associations only take effect once the app is bundled/installed; during development, test with a CLI argument (`tauri dev -- <path>.ldmx`) or a deep link (`live-dmx://open?path=<urlencoded-absolute-path>`). The Tauri `RunEvent::Opened` variant (macOS/other) is additionally handled in the run callback.

## Quality Assurance & Tooling

The project uses a strict linting and testing suite to ensure code quality.

### Frontend

- **Linting:** ESLint with Vue 3 and TypeScript plugins.
- **Formatting:** Prettier (integrated with ESLint).
- **Testing:** [Vitest](https://vitest.dev/) for unit and component testing. Use [Vue Testing Library](https://testing-library.com/docs/vue-testing-library/intro/) for component tests to focus on user-centric testing. Test files should be named `Component.test.ts` or `Component.test.vue` and placed next to the component they test.
  - Pinia is configured globally for all tests via `src/test-setup.ts` using `createTestingPinia({ stubActions: false })`. To customize or inspect store state in tests, access store composables directly (e.g. `const store = useShow()`).
  - **Mocking Conventions:** When mocking functions or modules in tests, prefer defining typed mock references at the top of the file using `const mockFn = vi.mocked(fn)` (e.g., `const mockOpenDialog = vi.mocked(openDialog);` or `const mockInvoke = vi.mocked(invoke);`) instead of calling inline `vi.mocked(...)` repeatedly inside individual test cases. Always clear mocks in `beforeEach(() => { vi.clearAllMocks(); })`.

### Backend

- **Linting:** [Clippy](https://doc.rust-lang.org/clippy/) for Rust logic.
- **Formatting:** `rustfmt` via `cargo fmt`.
- **Testing:** Built-in `cargo test` for unit and integration testing (`src-tauri/tests`).

### Automation

- **Git Hooks:** [Husky](https://typicode.github.io/husky/) and [lint-staged](https://github.com/lint-staged/lint-staged) automatically format and lint changed files on every commit.

## How to Contribute as an Agent

1. **Research:** Before making changes, explore the relevant modules.
2. **Implement:** Follow established patterns for Vue and Rust.
   - **Mandatory:** ALWAYS create an associated test file (`Component.test.ts` or `Component.test.vue`) whenever you create or modify a Vue component. Place it in the same directory as the component.
3. **Validate:** You MUST run `bun run lint` and `bun run format` after EVERY code edit. Use `bun run format:fix` to automatically resolve formatting issues. Additionally, run `bun run test` to ensure the project builds correctly and no regressions are introduced. Do not proceed to the next task until all linting and formatting errors are resolved or explicitly acknowledged as intentional (e.g., using disable comments for known issues).
4. **Document:** If you add a new subsystem or major feature, update this `AGENTS.md` file to help future agents.
