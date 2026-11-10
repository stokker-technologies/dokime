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

## One Command-Line Tool

`cargo dokime` is removed and its commands move to `dokime`, so every language uses one tool and
no language needs Cargo to reach it:

```sh
dokime new my-plugin --lang cpp
dokime plugin build | package | dev | inspect
dokime installer package
dokime bindings
```

`cargo harmos` stays in Harmos, a library for developers. Dokime does not mirror it.

## Pure Starters

Each starter is an ordinary project in its language, plus one shared packaging step wrapped by a
mise task. `build.rs` cannot bundle the plugin, because it runs before the crate compiles, and Cargo
has no stable post-build step. Starters drop release tooling and pin only what the language needs.

## Alignment With gRPC Plugins

Recorded on 2026-11-10. This change follows `grpc-plugins`:

- The SDK has two halves: a plugin half over the Harmos SDK for the language
  and a client half over `dokime.v1`.
- A plugin is a gRPC server. A step, criterion, initialize and cleanup are
  operations the host calls; there is no run loop in the SDK.
- `dokime proto export` provides the `.proto` files; there is no
  `dokime bindings` command.
