# Contributing

Live DMX is a Tauri v2 (Rust) backend with a Vue 3 (TypeScript) frontend. These
pages describe how the codebase is organised, why it is built that way, and what
to check before opening a pull request.

If you only want to run a show, read the [user manual](/manual/) instead.

## Start here

| Page                                       | Read it when                                                         |
| ------------------------------------------ | -------------------------------------------------------------------- |
| [Architecture](/contributing/architecture) | You need the backend layout, state/history model or command pattern. |
| [Frontend](/contributing/frontend)         | You are touching Vue views, Pinia stores or component tests.         |
| [DMX output](/contributing/dmx-output)     | You are changing the engine, a protocol or universe routing.         |
| [Rendering](/contributing/rendering)       | You are touching profiles, channels, colours, effects or pixels.     |
| [Fixtures](/contributing/fixtures)         | You are working on GDTF parsing, generic fixtures or the catalog.    |
| [Show openers](/contributing/show-openers) | You are touching file association or single-instance handling.       |
| [Releasing](/contributing/releasing)       | You are preparing a version bump or a published release.             |

## Before you open a pull request

Run the gates that match what you changed:

```sh
bun run lint          # ESLint + clippy -D warnings
bun run format        # Prettier + rustfmt --check
bun run test          # Vitest + cargo test
bun run docs:build    # VitePress build, catches dead links and missing sidebar targets
```

Every change that affects behaviour, structure or workflow also updates the page
that describes it — in the same pull request.
