---
name: verify-changes
description: "Use before committing or handing work back in this repo: run the right gates (lint, format, tests) depending on what changed, keep versions in sync, and apply the mandatory documentation update."
---

# Verify changes (definition of done)

Run the smallest correct set while iterating, then the full gate before handing off.

## By changed files

| Changed                                                | Commands                                                           |
| ------------------------------------------------------ | ------------------------------------------------------------------ |
| `src/**`                                               | `bun run lint:front`, `bun run format:front`, `bun run test:front` |
| `src-tauri/**`                                         | `bun run lint:back`, `bun run format:back`, `bun run test:back`    |
| Versions (`package.json`, `src-tauri/tauri.conf.json`) | `bun run release:version -- --check`                               |
| Both sides                                             | both sets above                                                    |

- Targeted while iterating: `bunx vitest related --run <files>` (same as lint-staged).
- Full gate before handoff: `bun run check` (typecheck + lint + format + tests + build).
- Format fixes: `bun run format:fix` (front + back).

## Mandatory documentation update (every change)

- Behaviour, structure, conventions, workflows → update `AGENTS.md` and/or the matching `docs/` file **in the same change**.
- **Procedures live in `.agents/skills/`, reference material in `docs/`** — never duplicate a procedure into docs, link to the skill instead.
- New or changed Vue component → the colocated test is mandatory (skill `add-vue-component`).
- New or changed skill → update the Skills table in `AGENTS.md`.

## Finally

- Do not move on while lint/format are red (or intentionally, with a comment).
- Version drift: `bun run release:version -- --check` fails on `package.json` / `tauri.conf.json` mismatch.
- The global workflow's CodeRabbit review (`coderabbit review --agent --include-untracked`) still applies before closing significant work.
