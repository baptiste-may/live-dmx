# Project Instructions for AI Agents

Welcome to the `live-dmx` project. This file is the **hub**: always-relevant
conventions and workflow. Deep domain knowledge lives in `docs/` (see the index
below) and should be read on demand.

> [!IMPORTANT]
> This file is a living document. **Documentation must be updated on every
> change — not just major features.** Whenever you implement a feature, change
> project structure, or identify a new pattern, update this file (or the
> relevant `docs/` file or skill) in the same change. Procedures live in
> `.agents/skills/`, reference material in `docs/`.
>
> **Everything under `docs/` is publishable.** The directory is the source of the
> public VitePress site (`docs/manual/` for users, `docs/contributing/` for
> developers), built with `bun run docs:build`. It must stay in English, free of
> secrets, absolute paths and internal-only notes.

## Documentation index — read these when relevant

| Doc                                 | Read when...                                                                                                                                   |
| ----------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `docs/contributing/fixtures.md`     | Fixture parsing, generic fixtures, categories/icon plumbing, GDTF-Share catalog & credentials vault, MVR import, `.ldmx` fixture serialization |
| `docs/contributing/dmx-output.md`   | `LiveEngine`/runner, universe routing, protocols (Art-Net/sACN/OSC/KiNET/OPC), USB interfaces, live commands                                   |
| `docs/contributing/rendering.md`    | Profile compilation, channel classification, colour mixing/routing, effects, alignments, pixel preferences, RNG                                |
| `docs/contributing/show-openers.md` | Deep-link/file-association/single-instance, `.ldmx` opening flow, logging config                                                               |
| `docs/contributing/releasing.md`    | Versioning, auto-updates, CI/publish workflows, signing, release procedure                                                                     |
| `docs/contributing/frontend.md`     | Any Vue/TypeScript work: stores, views, components, conventions, testing                                                                       |
| `docs/contributing/architecture.md` | Any Rust/Tauri work: modules, state/history, command patterns, capabilities, backend tests, adding a Tauri command                             |

## Skills — task procedures

Procedures live in `.agents/skills/<name>/SKILL.md` (cross-client Agent Skills
format; auto-discovered by OpenCode, Claude Code, Cursor, ...). Tools without
skill support can read the Markdown directly. **Never re-duplicate a procedure
into `docs/` — link the skill.**

| Skill                 | Use when...                                                     |
| --------------------- | --------------------------------------------------------------- |
| `add-tauri-command`   | adding/changing a Tauri backend command                         |
| `add-generic-fixture` | adding a built-in generic fixture                               |
| `add-dmx-protocol`    | adding a DMX output protocol end to end                         |
| `add-vue-component`   | creating a Vue component or view (conventions + mandatory test) |
| `add-pinia-store`     | adding a Pinia store                                            |
| `add-setting`         | adding a user setting                                           |
| `migrate-show-format` | changing the `.ldmx` persisted show format                      |
| `write-frontend-test` | writing or fixing frontend tests                                |
| `verify-changes`      | before committing/handing off (gates + doc rule)                |
| `debug-live-output`   | DMX not reaching fixtures or wrong output                       |
| `debug-gdtf-share`    | GDTF-Share catalog/login/vault problems                         |
| `app-debug-loop`      | driving the running dev app through the Tauri MCP bridge        |
| `publish-release`     | preparing or publishing a release                               |

## Project overview

`live-dmx` is a DMX lighting control application: **Tauri v2 (Rust)** backend +
**Vue 3 (TypeScript)** frontend, speaking Art-Net and other DMX-over-network
protocols to lighting fixtures.

**Stack:** Bun (packages/tasks) · Vue 3 `<script setup>` + Vite + TypeScript ·
Tailwind + DaisyUI (via `daisy-ui-kit`) · Lucide icons · Pinia · Tauri v2
(Rust) + Serde.

## Directory structure

```
src/                Vue frontend
  main.ts, App.vue, Main.vue
  components/       global components (inputs, modals, feedback, layout, icons)
  stores/           Pinia: app, show/* (domain), output/*, patch/*, live/*, + toast, ...
  views/            Patch, Configure, Input, Output, live/pages/{intensity,colors}
  libs/             composables & utilities (show-opener, logger, shortcuts, ...)
  types/            TS type definitions
src-tauri/          Rust backend
  src/fixtures/     GDTF parsing, generics, gdtf_share (catalog, client, auth vault)
  src/show/         state, history, patch/, output/, commands, open, mvr
  src/output/       engine, runner, protocols/, interfaces/, commands
  src/capabilities/ Tauri v2 permissions (default.json)
  tests/            backend integration tests
fixtures/           cache dir (<app_data_dir>/fixtures/), GDTF catalog + archives
docs/               VitePress site root — everything here is published
  .vitepress/       site config (navigation, sidebar, search, base path)
  manual/           English user manual (setup/, live/, reference/)
  contributing/     developer documentation (architecture, frontend, ...)
  public/           static assets (screenshots)
```

Details per domain (file-level maps, data flows, conventions) are in the docs.

## Coding standards (non-negotiable)

- **Modularity:** small, decoupled, single-responsibility units — split Vue
  components, Pinia stores, and Rust modules aggressively; keep Tauri command
  handlers thin and delegate logic to services/utilities.
- **Language:** TypeScript only (`.ts`); every SFC uses `<script setup lang="ts">`.
- **Strings:** double quotes in TS/scripts; template attributes double quotes,
  single quotes for literals inside attribute expressions
  (`:class="[open ? 'active' : '']"`).
- **Vue:** remove empty `<script>`/`<style>` blocks; simple handlers inline
  (`@click="x = false"`).
- **Styling:** Tailwind utilities; prefer `daisy-ui-kit` components over raw
  DaisyUI classes.
- **Tauri calls:** `@tauri-apps/api/core` (`invoke`).
- **State:** drive UI from Pinia stores (`useShow()` etc.); never mutate props.
- **Notifications:** `useToastStore()` — not `alert()`/Tauri `message()`.
- **Backend:** `Result<_, String>` for fallible Tauri commands, `Result` in
  internal logic; undo/redo via `AppState::apply_mutation` (never hand-rolled).

## Key flows at a glance

- **Frontend ↔ backend:** components call thin `#[tauri::command]` handlers that
  mutate `AppState` and emit `show-state-changed`; frontend stores apply the
  snapshot. Long-running handlers (`show_load`, MVR import, `show_save`) also
  emit partial progress snapshots (`emit_progress`) and flip `ShowStatus` to
  `Saving`/back to `Loaded`, which drives the load/save UI.
  Capabilities/permissions live in `src-tauri/capabilities/default.json`
  (plugin permissions only — custom commands are always allowed once registered
  in `lib.rs` `generate_handler!`).
- **Live output:** `LiveEngineRunner` (44 Hz) clones `ShowData`, computes DMX,
  and routes each enabled universe to its target (protocol or USB interface).
- **Fixtures:** built-in generics + on-demand GDTF-Share fetch, cached locally
  and embedded in `.ldmx` (portable/offline). MVR import available.

## Issue intake

GitHub Issues are the intake for bugs and feature requests, using the Issue
Forms in `.github/ISSUE_TEMPLATE/`:

- `bug_report.yml` requires reproduction steps and expected/actual behaviour,
  and applies `bug`.
- `feature_request.yml` requires the proposed behaviour, and applies
  `enhancement`.
- `config.yml` keeps blank issues available and links the documentation.

Release versions are tracked with GitHub milestones, not with a form field.
Area labels are applied manually during triage; the forms do not ask for an
area, version or operating system. Keep the forms product-facing; do not
mention internal planning tools or workflows.

## Quality assurance

Frontend:

- Lint `bun run lint:front` (ESLint) · Format `bun run format:front`
  (Prettier) · Tests `bun run test:front` (Vitest + Vue Testing Library,
  `Component.test.ts` colocated).
- Tests get Pinia automatically (`createTestingPinia({ stubActions: false })`
  via `src/test-setup.ts`); the Tauri APIs are globally stubbed — open
  `src/test-setup.ts` before mocking anything.
- **Mocking conventions:** use typed mock refs at file top
  (`const mockInvoke = vi.mocked(invoke);`), clear with
  `vi.clearAllMocks()` in `beforeEach`.

Backend:

- Lint `bun run lint:back` (clippy `-D warnings`) · Format `bun run format:back`
  (rustfmt) · Tests `bun run test:back` (cargo test, `src-tauri/tests/`).

Automation: Husky + lint-staged on commit · CI runs
`bun run check` (typecheck, lint, format, tests, build) · Review requests:
`.github/CODEOWNERS` assigns every path to `@baptiste-may`.

Documentation:

- Preview `bun run docs:dev` · Build `bun run docs:build` (also catches dead
  links and missing sidebar targets) · Serve the build with
  `bun run docs:preview`.
- Published in English only (`lang: "en-US"`), on GitHub Pages, by the `Docs`
  workflow. `DOCS_BASE` overrides the site base path (default `/live-dmx/`).

## How to contribute as an agent

1. **Research:** check `.agents/skills/` for a matching task procedure, then
   explore the codebase and read the relevant `docs/` file from the index above
   before editing.
2. **Implement:** follow the conventions above.
   - **Mandatory:** ALWAYS add a `Component.test.ts`/`Component.test.vue` when you
     create or modify a Vue component.
3. **Validate:** run `bun run lint` and `bun run format` after every edit
   (use `bun run format:fix` if needed), then `bun run test`. Do not move on
   until lint/format are clean (or intentional, with a comment). Ensure
   `src`/`src-tauri` don't drift from CI (`release:version -- --check` when you
   touch versions).
4. **Document — mandatory on every change:** always update this `AGENTS.md` or the
   matching `docs/` file (or skill, for procedures) whenever your change affects
   behavior, structure, conventions, or workflows (not only subsystems/major
   features). Do it in the same change as the code.
