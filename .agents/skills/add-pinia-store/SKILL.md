---
name: add-pinia-store
description: "Use when adding a new Pinia store under src/stores/: event-sourced snapshot pattern, thin view-models, composable access, and testing conventions."
---

# Add a Pinia store

Stores are small, single-purpose and **event-sourced from backend snapshots**. Reference: `docs/frontend-architecture.md` (store map).

## Pattern

1. Put the store under the right folder: `src/stores/show/*` (show domain), `src/stores/output/*`, `src/stores/patch/*`, `src/stores/live/*`, or top-level for app-wide UI state (`app.ts`, `toast.ts`, ...).
2. Define it with `defineStore("name", ...)` — match the style of its neighbours (setup or options API).
3. Show-domain state is applied from `show-state-changed` snapshots: either the full `ShowData` (`emit_show_changed`) or a partial `{status, progress, statusMessage}` (`emit_progress`). The core store is `show.ts`; split per concern (`show-patch.ts`, `show-groups.ts`, ...) instead of growing one store.
4. Keep it a **thin view-model**: no domain computation the backend already owns. Canonical example: `show-configuration.ts` — all capabilities, bounds, color options, and defaults come from `show_configure_get_overview`; the store only handles UI state (selection, debounced save, cache sync).
5. Expose it as a composable `useX()` and consume it from components; never mutate props, and never call `invoke` from deep components when the store owns that state.

## Test

Add `store.test.ts` colocated. The testing Pinia is global (`stubActions: false`), so call the real composable and its actions; mock `invoke` per test to feed snapshots (see skill `write-frontend-test`).

## Verify

Targeted `bunx vitest related --run <files>`, then the `verify-changes` skill.
