# Remote Plugins and the Automation Contract

Status: proposed on 2026-11-10 from draft PR #380 (`docs/remote-plugins.md`);
not locked. Nothing here is implemented, and implementation waits for the
owner's decisions listed in `design.md`.

## Why

Plugins install as bundles that Dokime stores, extracts, starts in dependency
order and reloads. That machinery is large, only Rust can speak the native
plugin protocol, and release feeds and installers carry plugin archives. A
plugin that is a server the user runs, reached by URL, removes the install
path and opens plugins to other languages. The same step turns the exported
Public product operations into a versioned automation contract.

## What Changes

- Dokime gets two external interfaces, both gRPC with versioned protos, that
  differ in which side connects: Dokime connects to a plugin's URL
  (`harmos.plugin.v1` plus Dokime catalog sections), and clients connect to
  Dokime with a pairing file (`dokime.automation.v1`).
- Settings › Plugins becomes a list of URLs with a connection state per entry.
  The user runs, updates and restarts the plugin process; Dokime reconnects.
- Dokime reads the existing JSON sections from the plugin's catalog
  (`dokime.suites`, `dokime.records`, `dokime.listeners`, `dokime.telemetry`,
  `dokime.analyses`, `dokime.settings`, `dokime.run-start`). Templates keep
  running as `<template>.run` and `<template>.signal`, and step code calls
  Host-visibility operations as child calls listed in `Catalog.requires`.
- Host telemetry (`plugins/sidecars/host/`) moves into the backend as built-in
  operations and streams. OBC (`plugins/sidecars/obc/`) becomes the reference
  remote plugin that tests start and connect to by URL.
- The protos that `cargo dokime bindings proto` exports are attached to every
  release under the `dokime.automation.v1` package; breaking changes move to
  `v2`. The `.dokime-remote` pairing file stays the one way to connect.
- The Python client stays a thin client and `dokime.Backend` retires.
- Removed: plugin install, store, boot sweep, extraction, reload and
  dependency-ordered start; upload, install and remove operations; bundle
  build and packaging commands; release-feed plugin archives and installer
  `[[plugin]]` entries; in-process Python plugins and the Python backend; the
  plugin drop zone and feed pages. `design.md` lists the paths.

## Impact

- Breaking for every installed plugin and for existing data roots with
  bundle-backed installations; the development storage policy allows the
  clean break with a fresh root.
- Depends on the Harmos change `plugin-grpc-protocol`. gRPC was parked on
  2026-11-07 in favour of native SDKs over a C ABI (`dokime-sdk`,
  `dokime-sdk-cpp`, `dokime-sdk-go`), so the first decision is whether gRPC
  returns as the plugin transport.
- Overlaps `template-authoring-simplification` (`Catalog.requires`) and the
  unapproved roadmap in `docs/plugin-security.md` (publisher signing,
  organization allowlist, WebAssembly guests), which assumes bundles.
- Private plugins migrate in their own repositories.
