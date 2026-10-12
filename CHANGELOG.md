# Changelog

## [Unreleased]

- Dokime now depends on the `harmos` crate alone: the tiered series store moved
  into `dokime-storage::series`, keeping the manifest and segment format
  existing telemetry directories already carry.

- Removed Specta type generation and the `dokime-desktop-contract` crate: the
  desktop IPC commands are declared in the GUI crate with their wire types
  beside them, and the TypeScript side is hand-written in
  `crates/dokime-gui/src/lib/desktop/`. `bindings:generate`/`bindings:check`
  now generate only the protobuf RPC bindings.

## [0.1.0]

- Initial public history: native sidecar plugins, gRPC desktop and CLI
  clients, bundled installer, and the Harmos-based runtime.
