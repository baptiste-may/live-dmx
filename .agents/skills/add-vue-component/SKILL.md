---
name: add-vue-component
description: "Use when creating a new Vue 3 component or view under src/ in this project: project conventions (script setup lang=ts, quoting, daisy-ui-kit, store-driven state) and the mandatory colocated Component.test.ts."
---

# Add a Vue component or view

Reference: `docs/contributing/frontend.md`. Start from the templates in `assets/`.

## Steps

1. Pick the location:
   - Global-purpose components → `src/components/<category>/` (inputs, modals, feedback, layout, icons).
   - View-specific children → next to the view, e.g. `src/views/<view>/...`.
2. Create the SFC with `<script setup lang="ts">` (mandatory) — start from `assets/Component.vue.template`.
3. Create the colocated test `ComponentName.test.ts` — **mandatory for every component** (start from `assets/Component.test.ts.template`; details in the `write-frontend-test` skill).
4. Wire state through Pinia stores (`useShow()`, domain stores) — never mutate props, never duplicate backend logic in components.
5. If the component is a settings section, register it in `src/components/modals/SettingsModal.vue` (see skill `add-setting`).

## Conventions

- TypeScript only — never `.js`; no empty `<script>`/`<style>` blocks.
- Strings: double quotes in TS; in templates, double quotes for attributes and **single** quotes for string literals inside attribute expressions (`:class="[open ? 'active' : '']"`).
- Simple handlers inline (`@click="isOpen = false"`); anything longer becomes a named function.
- Styling: Tailwind utilities; prefer `daisy-ui-kit` components (`Button`, `Card`, `Modal`, `Table`, `Badge`, ...) over raw DaisyUI classes.
- Icons: `lucide-vue-next`.
- Tauri calls: `invoke` from `@tauri-apps/api/core` — prefer doing it inside a store action.
- Notifications: `useToastStore()` — never `alert()` or Tauri message dialogs.
- Imports use the `@/` alias (e.g. `@/stores/settings.ts`).

## Verify

Targeted: `bunx vitest related --run <files>`. Before handing off, run the `verify-changes` skill.
