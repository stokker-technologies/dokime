# Remote Plugins and the Automation Contract Design

Status: proposed on 2026-11-10; not locked.

## Interfaces

| Interface | Who Connects | Contract | Used By |
| --- | --- | --- | --- |
| Plugins | Dokime connects to the plugin's URL | `harmos.plugin.v1` plus Dokime sections | Instrument drivers and test suites, in any language |
| Automation | Clients connect to Dokime with a pairing file | `dokime.automation.v1` | Scripts, the CLI, the Python client, the desktop GUI |

## Plugins by URL

**Add plugin** takes an address such as `http://bench-pc:7001`. Each entry
shows connecting, ready (id, version and catalog digest) or unavailable (with
the reason). Remove deletes the entry.

| Section | Schema | Purpose |
| --- | --- | --- |
| `dokime.suites` | 2 | Test templates, parameters, steps and criteria |
| `dokime.records`, `dokime.listeners` | 1, 2 | Record contracts the plugin publishes or listens to |
| `dokime.telemetry`, `dokime.analyses` | 1 | Telemetry stream contracts and their analyses |
| `dokime.settings` | 3 | Settings the plugin exposes |
| `dokime.run-start` | 1 | A capture every execution runs before its first step |

Operations-only plugins in other languages need only the protos and a gRPC
library. Test suites also need the `dokime.suites` JSON and the
`<template>.run` call sequence (`crates/dokime-runtime/src/execution/serving.rs`),
so suites stay in Rust until a customer needs them elsewhere.

## Automation

- Attach the exported protos to every release under `dokime.automation.v1`.
- Keep `.dokime-remote`: endpoints, pinned CA and access key in metadata.
- Show short Python (`grpcio`) and `grpcurl` examples next to
  `docs/remote/overview.md`.
- Keep `dokime-client` for the CLI and the desktop shell, with plugin
  operations reachable through the same server.

## What It Removes

| Area | Path |
| --- | --- |
| Install, store, boot sweep, extraction, reload and dependency-ordered start | `crates/dokime-runtime/src/runtime/plugins*`, `src/plugins/`, `src/runtime/sidecars*` |
| Upload, install and remove operations | `crates/dokime-server/src/grpc/plugins*`, `bundle.rs` |
| Bundle build and packaging commands | `crates/dokime-packaging/src/{build,bundle,process}/`, `crates/cargo-dokime/src/commands/sidecar/` |
| Release-feed plugin archives and installer `[[plugin]]` entries | `crates/dokime-domain/src/model/plugins/release.rs`, `crates/dokime-packaging/src/installer/`, `crates/dokime-gui/src-tauri/src/app/updates/` |
| In-process Python plugins and the Python backend | `crates/dokime-runtime/src/runtime/embedded.rs`, `sdk/python/src/{authoring,backend}/`, `sdk/python/examples/starter/` |
| Plugin drop zone and feed pages | `crates/dokime-gui/src/features/settings/plugins/` |

## Open Decisions

None is decided. Each needs the owner before any task starts.

1. **Transport.** gRPC is parked since 2026-11-07 for native SDKs over a C
   ABI. Either gRPC returns as the plugin transport (this change and Harmos
   `plugin-grpc-protocol` proceed, and the native SDK changes are rescoped),
   or this change is recast on the native ABI or dropped.
2. **In-process Python plugins.** `EmbeddedProvider` was kept (#141) so Python
   plugins run inside a Python-started backend. This change replaces that with
   Python plugins as gRPC servers and retires the starter that
   `dokime new --lang python` embeds.
3. **Settings and secrets.** Dokime keeps storing settings and sends them on
   connect, or each plugin owns its configuration and secrets and reports its
   effective settings in a section so that evidence keeps them. Today plugins
   read Secret Storage by identifier.
4. **Host telemetry isolation.** In-process loses the crash isolation a
   separate process gives native Wi-Fi, Bluetooth and radio calls. The
   alternative is a host server the installer autostarts on a localhost URL.
5. **Reversible effects.** The host journals and restores radio, adapter and
   power-plan changes. That stays if host telemetry moves in-process; a remote
   plugin that changes machine state owns its own restore.
6. **Development data.** Bundle-backed installations in existing roots stop
   loading; confirm the clean break with a fresh root.
7. **Plugin security roadmap.** Publisher signing and the organization
   allowlist in `docs/plugin-security.md` assume bundles; decide what replaces
   them for a URL (for example pinned server identity) or that they drop.
