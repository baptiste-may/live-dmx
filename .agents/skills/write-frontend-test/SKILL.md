---
name: write-frontend-test
description: "Use when writing, updating, or fixing Vitest + Vue Testing Library tests under src/: colocated naming, global testing Pinia, stubbed Tauri APIs, typed mock refs, and the project's mocking conventions."
---

# Write frontend tests

Stack: Vitest + Vue Testing Library. Files are named `Component.test.ts` / `Component.test.vue` and **live next to the component**. Read `src/test-setup.ts` before adding any mock.

## What is already global (do not re-mock)

`src/test-setup.ts` sets up, for every test:

- a testing Pinia (`createTestingPinia({ stubActions: false })`) — use the real store composables directly;
- stubs for `@tauri-apps/api/core` (`invoke` resolves `{}`), `@tauri-apps/api/window`, `@tauri-apps/plugin-global-shortcut`, `@tauri-apps/plugin-log`;
- DOM cleanup after each test.

## Conventions

- Typed mock refs at the top of the file: `const mockInvoke = vi.mocked(invoke);` (import `invoke` from `@tauri-apps/api/core`).
- `vi.clearAllMocks()` in `beforeEach`.
- Need custom store state? Build it through the real store composable (e.g. `store.setShow(mockData)`) — not by hand-crafting Pinia state.
- Mocks specific to a test (e.g. `@tauri-apps/api/app` for updater/version APIs) belong in the test file, not in `test-setup.ts`.
- Interactions with `@testing-library/user-event`; query by role/text (`screen.getByRole(...)`).
- Shared mock data lives in the `__mocks__` folders (e.g. `src/stores/show/__mocks__`).

## Commands

- Whole suite: `bun run test:front`.
- Targeted while iterating: `bunx vitest related --run <files>`.

## Verify

Finish with the `verify-changes` skill.
