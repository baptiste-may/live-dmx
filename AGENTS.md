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
  - [Tailwind CSS](https://tailwindcss.com/) & [DaisyUI](https://daisyui.com/) for styling.
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
  - `src/lib.rs`: Tauri application setup and command handlers.
  - `src/fixtures/`: Rust logic for handling fixture data and caching (`commands.rs`, `loader.rs`, `types.rs`).
  - `src/show/`: Show management, patch batching, history tracking, and show serialization (`types.rs`, `state.rs`, `history.rs`, `commands.rs`, `identity.rs`, `recent.rs`, `patch/`).
  - `src/output/`: DMX output engine and command handlers (`engine.rs`, `compute.rs`, `effects.rs`, `channels.rs`, `colors.rs`, `runner.rs`, `commands.rs`, `types.rs`).
- `fixtures/`: JSON data files for lighting fixtures (cached in OS AppData directory, or downloaded for development via `download-fixtures.ts`).
- `download-fixtures.ts`: Script to download fixtures from the Open Fixture Library.

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
- **Styling:** Prefer Tailwind CSS utility classes. Use DaisyUI components where appropriate.
- **Tauri Commands:** Invoke backend commands using `@tauri-apps/api/core`.
- **State Management:** The application state is primarily managed within the Pinia store (`src/stores/show/show.ts`). Components should use `useShow()` to access and update the current show state instead of direct prop mutations. Avoid mutating props directly to satisfy Vue linting rules.
- **Notifications:** Use `useToastStore()` from `src/stores/toast.ts` to display non-blocking alerts (toasts) in the bottom-right corner. Prefer this over native `alert()` or Tauri `message()` for a consistent UI experience.
- **Testing:** Wrap components with `createTestingPinia({ stubActions: false })` in tests when store interaction is needed. Initialize state using `store.setShow(mockData)` or similar actions.

### View-Specific Logic

- **Patch View (`src/views/Patch.vue`):**
  - Handles the inventory of fixtures and their DMX addressing.
  - Uses `PatchForm` for fixture search and batch adding, `PatchVisualizer` for universe grids, and `PatchGroups` for group organization.
  - Interactions focus on speed: quick searching of manufacturers and easy adding of fixtures.
- **Configure View (`src/views/Configure.vue`):**
  - Manages fixture parameters like Dimmer, Pan/Tilt limits, and inversions.
- **Live View (`src/views/Live.vue`):**
  - **Status:** Currently a placeholder.
  - **Goal:** Should implement high-performance, low-latency UI for live control (buttons, faders, playbacks).

### Backend (Rust)

- **Tauri Commands:** Keep command handlers concise; delegate business logic to modules.
- **Error Handling:** Use `Result` for functions that can fail, especially in Tauri commands.
- **DMX Logic:** Centralize DMX computation and engine runner in the `output` module.

## DMX Fixtures

The application relies on fixture definitions from the Open Fixture Library (OFL).

- Fixtures are stored and cached in the native OS AppData directory (`<app_data_dir>/fixtures/`).
- When the fixture browser is opened and fixtures are not yet installed in AppData, the user is prompted to download the Open Fixture Library on-demand (`download_fixtures` command).
- Once downloaded or when already present, the backend loads and parses them on-demand (`get_fixtures` command) with progress reporting.
- New fixtures should be contributed directly to the [Open Fixture Library](https://github.com/OpenLightingProject/open-fixture-library).
- Refer to `src/types/fixtures.ts` for the expected structure of fixture definitions.

## Working with DMX Output

- The DMX output engine (`LiveEngine`) and continuous background loop (`LiveEngineRunner`) are initialized in `src-tauri/src/output/`.
- The frontend sends live control updates (`live_set_*`) or direct manual channel updates via `set_dmx_channel` / `set_dmx_data` commands.

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
