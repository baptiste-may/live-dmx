---
name: add-setting
description: "Use when adding a user-facing setting in live-dmx: defaults and validation in src/stores/settings.ts, a section under components/modals/settings/, wiring in SettingsModal, and tests."
---

# Add a user setting

Settings live in `src/stores/settings.ts` (localStorage key `live-dmx-settings`); the UI is a grid of self-contained sections in `src/components/modals/SettingsModal.vue`.

## Steps

1. Add the field and its default to the `LiveSettings` / `SettingsState` interfaces in `src/stores/settings.ts`.
2. Handle it in `getStoredSettings()`: validate the stored value and fall back to the default on anything unexpected (see how `theme`, `scale`, and `live.*` are validated) — never trust localStorage.
3. Persist on every mutating action via `persistSettings(...)` (same pattern as `setTheme` / `setScale`).
4. Add a section component `src/components/modals/settings/Settings<Name>.vue` (start from an existing one) and register it in `SettingsModal.vue` (2-column grid, next to Global/Live/Gdtf/About).
5. Add or adjust tests: `src/stores/settings.test.ts` and the section's colocated test (skills `write-frontend-test`, `add-vue-component`).

## Conventions

- A setting is UI-only state: no backend round-trip. If it drives show behaviour, it belongs in the show model instead (see skill `migrate-show-format`).
- Use `daisy-ui-kit` controls and the existing `ToggleButton` / `TextForm` / `NumberStepper` inputs where they fit.

## Verify

`bun run test:front`, then the `verify-changes` skill.
