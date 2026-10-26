# Releases, Versioning & Auto-Updates

Read this before touching versions, release scripts, CI workflows, the updater,
or build/signing configuration.

## Version source of truth

- **`package.json`** is the single source of truth.
- Bump with `bun run release:version <x.y.z>`, a wrapper around
  `node scripts/release.mjs <version>`. It updates both `package.json` and
  `src-tauri/tauri.conf.json` and fails on drift. It also accepts `v` prefixes
  and validates semver.
- `node scripts/release.mjs --check` only validates the sync (used by CI).
- `Cargo.toml`/`Cargo.lock` keep their own `0.1.0` and are **intentionally left
  untouched**.
- Prereleases (`0.1.0-beta`, `0.1.0-alpha.1`, …) are valid versions and follow
  regular semver ordering. The first **stable** release must be `≥ 0.1.0`: a
  prerelease (`-beta` < `0.1.0`) is never offered as an update to a stable
  install. Note that `releases/latest` resolves to the latest **non-prerelease**
  release, so a lone prerelease GitHub release 404s the updater endpoint until a
  stable release exists — intended during the pre-release phase.

## Auto-updates

- Served by `tauri-plugin-updater` from GitHub Releases:
  `plugins.updater.endpoint` → `https://github.com/baptiste-may/live-dmx/releases/
latest/download/latest.json`. The public key is embedded in `tauri.conf.json`;
  `bundle.createUpdaterArtifacts: true` produces the updater artifacts.
- **Frontend logic** lives in `src/stores/updates.ts` (actions `init`,
  `checkForUpdates`, `downloadAndInstall` + `relaunch()`, `requestUpdate`),
  surfaced on the Welcome page (`src/components/Welcome.vue`) and in the Settings
  modal (`src/components/modals/settings/SettingsAbout.vue`).
- `tauri-plugin-process` provides `relaunch()`/`exit()`. Permissions
  `updater:default` and `process:default` are granted in
  `src-tauri/capabilities/default.json`. Tests must mock `@tauri-apps/api/app`
  (`getVersion`/`getTauriVersion`) because the global test-setup stubs `invoke`.

## Release procedure

Follow the `publish-release` skill (`.agents/skills/publish-release/SKILL.md`):
`bun run release:version <x.y.z>` → validate with `release:version -- --check` →
commit + tag `v<x.y.z>` + push → CI `publish.yml` builds and signs a **draft**
GitHub Release → publishing the draft makes `latest.json` reachable (the
endpoint pins `releases/latest`). `workflow_dispatch` is also available for
manual builds.

## Signing secrets (GitHub Actions)

- `TAURI_SIGNING_PRIVATE_KEY` — contents of the private key generated at
  `~/.tauri/live-dmx.key` (never commit it).
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` — empty string (key generated without a
  password), stored anyway for workflow compatibility.
- macOS builds are ad-hoc signed (`APPLE_SIGNING_IDENTITY: "-"`; no certificate
  configured yet).

## Updater constraints

- **Linux:** only the **AppImage** is auto-updatable (deb/rpm need manual install).
- **Windows:** the app closes itself before installing the new version.
- **macOS:** unsigned builds trigger Gatekeeper warnings for end users.

## CI (`.github/workflows/ci.yml`)

Runs on PRs and `main` on `ubuntu-22.04`:

1. Linux deps installed (`libwebkit2gtk-4.1-dev`, `libappindicator3-dev`,
   `librsvg2-dev`, `patchelf`, `xdg-utils`, `build-essential`, `libssl-dev`
   `libudev-dev` — serialport needs `libudev-dev`/`libssl-dev`).
2. `oven-sh/setup-bun`, `dtolnay/rust-toolchain@stable` with \*\*explicit `clippy`
   - `rustfmt` components\*\* (not included by default), `Swatinem/rust-cache` for
     `./src-tauri -> target`.
3. `bun install --frozen-lockfile --ignore-scripts` — scripts are skipped because
   native deps pulled transitively (`better-sqlite3` via `daisy-ui-kit` →
   `@nuxt/content`, `canvas`) are unused at build/test time; skipping keeps CI
   deterministic and avoids native builds.
4. Version sync guard: `bun run release:version -- --check`.
5. `bun run check` (typecheck + lint + format + unit/cargo tests + build).
6. `bun run build:app` — real `tauri build --no-bundle` to validate the release
   build without needing the signing key.

## Documentation site (`.github/workflows/docs.yml`)

The user manual and the contributor docs are published as a VitePress site
built from `docs/`:

- `bun run docs:build` checks every nav/sidebar target
  (`scripts/check-docs-links.mjs`) and builds `docs/.vitepress/dist`; `docs:dev`
  serves it with hot reload, `docs:preview` serves the built output.
- `docs.yml` runs on pushes to `main`, pull requests and manual dispatch,
  filtered to `docs/**`, `scripts/check-docs-links.mjs`, `package.json`,
  `bun.lock` and the workflow itself. The `build` job installs with
  `--frozen-lockfile --ignore-scripts` and runs `bun run docs:build`; the
  `deploy` job runs on `main` only and uses `actions/deploy-pages`.
- `DOCS_BASE` sets the site base path. It defaults to `/live-dmx/` locally, and
  the workflow sets it to `/<repository-name>/` so forks work. Set it to `/`
  when a custom domain is configured.
- **Pages must be enabled once**: repository Settings → Pages → Source =
  **GitHub Actions**. Without it the `build` job still proves the site is valid
  but the `deploy` job fails.
- GitHub Pages is only available for **private** repositories on a paid plan.
  While this repository is private on the Free plan, the deploy job stays red;
  the site becomes reachable once the repository is public or the plan changes.

## Release checklist for agents

The operational checklist — version sync, no hand-edits, signing secrets, CI
native deps — lives in the `publish-release` skill
(`.agents/skills/publish-release/SKILL.md`).
