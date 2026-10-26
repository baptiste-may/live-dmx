---
name: publish-release
description: "Use when preparing or publishing a live-dmx release: version bump via scripts/release.mjs, tag and push, GitHub Actions publish workflow, draft release, and updater constraints."
---

# Prepare and publish a release

Reference: `docs/contributing/releasing.md` (versioning rules, auto-updates, signing, CI details).

## Procedure

1. Bump the version — **never hand-edit versions**:
   `bun run release:version <x.y.z>` (updates `package.json` + `src-tauri/tauri.conf.json`, validates semver, fails on drift; `v` prefix accepted).
2. Validate the sync: `node scripts/release.mjs --check` (CI runs `bun run release:version -- --check`).
3. Commit, tag `v<x.y.z>`, push the tag.
4. GitHub Actions `publish.yml` (`tauri-apps/tauri-action@v1`) builds the matrix (macOS x86_64 + aarch64, Ubuntu 22.04, Windows), signs the updater artifacts with `TAURI_SIGNING_PRIVATE_KEY`, and creates a **draft** GitHub Release.
5. Publishing the draft makes the release live and `latest.json` reachable (the updater endpoint pins `releases/latest`).

## Guardrails

- Version source of truth: `package.json`. `Cargo.toml`/`Cargo.lock` are intentionally left at their own version.
- Prereleases: `releases/latest` resolves to the latest non-prerelease — a lone prerelease release 404s the updater endpoint until a stable release exists. The first stable release must be ≥ `0.1.0`.
- Never commit the signing key (`~/.tauri/live-dmx.key`); the password secret is an empty string by design.
- Updater constraints: Linux AppImage only (deb/rpm manual), Windows self-closes before install, macOS unsigned builds warn via Gatekeeper.
- CI must stay green: `bun run check` + `bun run build:app` run on PRs; new native deps require mirroring the `apt-get` line in both workflows.

## Manual check

- [ ] `bun run docs:build` passes (it also catches dead links).
- [ ] Every UI label quoted in `docs/manual/` still exists in `src/views` — grep the bold labels before tagging.
- [ ] Screenshots under `docs/public/screens/`, if any, still match the released UI.

## Verify

`bun run release:version -- --check` after the bump; confirm the draft release contains the signed artifacts before publishing.
