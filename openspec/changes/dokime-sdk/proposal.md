# Dokime SDK

## Why

Rust plugin authors depend on all of `dokime` with `plugin-sidecar`, which always compiles
`dokime-runtime`. Harmos already separates `harmos-sdk`. Dokime's SDK should match it, so plugins
stop compiling the host.

## What Changes

- Add `dokime-sdk` at `sdk/rust`, holding the test authoring API over `harmos-sdk`.
- Extract authoring contexts and contracts from `dokime-runtime`, keeping the crate tree acyclic.
- `dokime` stays the host and application library.
- Migrate the starter template, sample plugins and `cargo dokime`.
- Move `dokime-python` to `sdk/python` as part of the shared layout.

## Locked Decisions

- The SDK has no dependency on `dokime-runtime`.
- Concept names are shared across every language's SDK.
- Clean replacement: no compatibility feature stays on `dokime`.
