# Frontend Architecture (Vue 3 + TypeScript)

Read this when working on any frontend code: components, Pinia stores, views,
composables, or frontend tests.

## Stack

- Vue 3 Composition API, `<script setup lang="ts">` (mandatory)
- Vite + TypeScript
- Tailwind CSS + DaisyUI, via **daisy-ui-kit** components by preference
- Pinia (state management), VueUse, lucide-vue-next icons
- Vitest + Vue Testing Library (tests colocated by default)

## Directory map (src/)

| Path                     | Purpose                                                                             |
| ------------------------ | ----------------------------------------------------------------------------------- |
| `main.ts`                | Entry point (mounts `App.vue`)                                                      |
| `App.vue`                | Core app component; wires `useShowOpener()`                                         |
| `Main.vue`               | Primary application layout (sidebar, view routing)                                  |
| `components/`            | Global components (see below)                                                       |
| `stores/`                | Pinia stores (see below)                                                            |
| `views/`                 | Feature views (Patch, Configure, Input, Output, Live)                               |
| `libs/`                  | Utilities + composables                                                             |
| `types/`                 | TS type definitions (`fixtures.ts`, `live.ts`, `output.ts`, `patch.ts`, `shows.ts`) |
| `assets/icons/fixtures/` | SVG fixture icons consumed by `FixtureIcon.vue`                                     |
| `test-setup.ts`          | Vitest global setup: Tauri API mocks + testing Pinia                                |

## Components (src/components/)

- **entry/screens:** `Welcome.vue`, `RecentShowsList.vue`
- **layout:** `AppLayout.vue`, `AppSidebar.vue`
- **inputs:** `Fader.vue`, `FlashButton.vue`, `Knob.vue`, `NumberStepper.vue`,
  `TextForm.vue`, `ToggleButton.vue`
- **modals:** `ConfirmationModal.vue`, `UnsavedChangesModal.vue`,
  `SettingsModal.vue`, plus `modals/settings/*` (e.g. `SettingsAbout.vue`
  surfaces the updater)
- **feedback:** `ToastContainer.vue`, `ProgressScreen.vue`, `SavingBar.vue`
  (thin top-of-window bar tracking the backend `Saving` state), `WipAlert.vue`
- **icons:** `FixtureIcon.vue`, `NumberIcon.vue`

## Views (src/views/)

- **`Patch.vue`** — fixture inventory + DMX addressing. Children:
  `patch/form/` (`PatchForm.vue` = browser + adder), `patch/visualizer/`
  (`PatchVisualizer.vue`, universe grids), `patch/groups/` (`PatchGroups.vue`,
  group organization with `PatchGroupsEditModal`, list/table subviews).
  Design focus: speed — quick manufacturer search, batch adding.
- **`Configure.vue`** — fixture parameters (Dimmer, Pan/Tilt limits, inversions,
  Max Power). Children: `configure/selection/` (group tree), `configure/settings/`
  (`ConfigureSettings.vue` = per-group limits). See **configuration store**
  below.
- **`Input.vue`** — input triggers and controller mapping (triggers/controllers).
- **`Output.vue`** — universe routing + physical interfaces. Children:
  `output/universes/` (`OutputUniverses.vue`: None/Physical/Virtual target
  selection), `output/interfaces/` (`OutputInterfaces.vue`: hardware listing and
  discovery), `output/dmx/` (`OutputDmxMonitor` = live universe monitor,
  `OutputUniverseCard`).
- **`live/`** — live control, navigated via the sidebar "Live" section:
  - `pages/intensity/` — `LiveIntensity.vue` (master faders, flash buttons,
    effects) plus `effects/`, `LiveIntensityMasters/Flashs`
  - `pages/colors/` — `LiveColors.vue` (color table, fades, color effects) plus
    `table/`, `effects/`
  - `live/components/` — `LiveEffectsRow.vue`

## Pinia stores (src/stores/)

- `app.ts` — app-level navigation/state, `loadShow(path)` (the open-entry funnel),
  `isShowLoaded`, `handleOpenRequest`.
- `show/` — domain stores. Each file is small and event-sourced from backend
  snapshot events:
  - `show.ts` — core `useShow()` store (current show state, snapshots, plus
    load/save `progress`/`statusMessage`; `isSaving`/`isLoaded` derive from
    `ShowStatus`, with `Saving` treated as active so the layout stays mounted)
  - `show-changes.ts` — dirty/undo/redo/can-undo/can-redo + unsaved-changes modal
  - `show-identity.ts` — show name/display identity
  - `show-patch.ts` — patched fixtures
  - `show-groups.ts` — groups
  - `show-configuration.ts` — **thin view-model** over the backend
    `show_configure_get_overview` (see below)
- `live/live-buttons.ts` — live flash/button state
- `output/output-universes.ts`, `output/output-interfaces.ts`,
  `output/output-dmx.ts` — universe config, interface discovery, DMX monitor
- `patch/patch-form.ts`, `patch/patch-visualizer.ts`, `patch/patch-groups.ts` —
  Patch view state
- `fixture-browser.ts` — GDTF-Share browser state and search
- `recent-shows.ts`, `settings.ts`, `toast.ts`, `confirmation.ts`,
  `updates.ts`, `navigation.ts` — supporting UI state

### Configuration store exception (`show-configuration.ts`)

The Configure view does **not** duplicate business logic on the frontend:

- All group capabilities (`has_pan`, `has_tilt`, `has_color`), bounds
  (`panBounds`/`tiltBounds`), color options, and clamped defaults come from the
  backend `show_configure_get_overview`.
- Frontend `useShowConfigurationSync` watchers + `reconcileSettings` only handle
  UI state: selected group, debounced save, settings cache sync.

Do not move domain computation to this store — keep it a thin view-model.

## Composables & libs (src/libs/)

- `show-opener.ts` — `useShowOpener()` (see `docs/contributing/show-openers.md`)
- `show-configuration-sync.ts` — Configure store sync logic
- `logger.ts` — `initLogger()`: attaches webview console to backend logs
- `shortcuts.ts`, `date.ts`, `fixture.ts` — small utilities
- `utils.ts` — `sliceString` preserves short strings and prefixes truncated
  suffixes with `...` (the ellipsis is additional to the length limit).
  A zero limit currently prefixes the entire nonempty string, because
  JavaScript treats `slice(-0)` as `slice(0)`.

## Conventions (non-negotiable)

- TypeScript everywhere — never `.js`; `<script setup lang="ts">` only.
- **Strings:** double quotes in TS; in templates, double quotes for attributes
  and _single_ quotes for string literals inside attribute expressions
  (`:class="[open ? 'active' : '']"`).
- Remove empty `<script>` or `<style>` blocks from SFCs.
- **Inline handlers:** simple one-liners go inline in the template
  (`@click="isOpen = false"`); otherwise extract a handler.
- **Styling:** prefer Tailwind utilities; for DaisyUI components prefer
  `daisy-ui-kit` (`Button`, `Card`, `Modal`, `Table`, `Badge`, `Divider`, ...)
  over raw DaisyUI classes.
- **Tauri calls:** `@tauri-apps/api/core` `invoke`.
- **State:** use `useShow()`/domain stores instead of mutating props (Vitest/Vue
  linting requires no direct prop mutation). Compute store-derived values with
  getters/composables.
- **Notifications:** `useToastStore()` (`src/stores/toast.ts`) for non-blocking
  alerts; avoid `alert()` / Tauri message dialogs.

## Testing (frontend)

- Vitest + Vue Testing Library; test files named `Component.test.ts` /
  `Component.test.vue` **colocated next to the component**.
- `src/test-setup.ts` installs a testing Pinia globally and stubs the Tauri
  APIs — open it before mocking anything.
- Mocking conventions, shared mocks, and commands live in the
  `write-frontend-test` skill (`.agents/skills/write-frontend-test/SKILL.md`).

## Adding a new store/component

Follow the `add-vue-component` and `add-pinia-store` skills
(`.agents/skills/…`) — scaffolding, conventions, mandatory colocated tests, and
the store patterns. Update this doc and `AGENTS.md`'s directory map if you add a
significant area.
