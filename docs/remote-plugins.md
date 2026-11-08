# Remote Plugins and Automation (Proposal)

Status: proposal for discussion. Nothing here is implemented yet. The plugin
protocol is Harmos's `harmos.plugin.v1`, described in the Harmos
[Remote Plugins guide](https://github.com/stokker-technologies/harmos/blob/dev/docs/remote-plugins.md).

Dokime would have two external interfaces, both plain gRPC with versioned protos.
They differ only in which side opens the connection:

| Interface | Who connects | Contract | Used by |
| --- | --- | --- | --- |
| Plugins | Dokime connects to the plugin's URL | `harmos.plugin.v1` plus Dokime sections | Instrument drivers and test suites, in any language |
| Automation | Clients connect to Dokime with a pairing file | `dokime.automation.v1` | Scripts, the CLI, the Python client, the desktop GUI |

## Plugins by URL

Settings › Plugins becomes a list of URLs. **Add plugin** takes an address such as
`http://bench-pc:7001`, and each entry shows connecting, ready (with id, version
and catalog digest) or unavailable (with the reason). Remove deletes the entry.
The user runs, updates and restarts the plugin process; Dokime reconnects on its
own when the process comes back.

Dokime reads these sections from the plugin's catalog. They keep the JSON schemas
the native plugins already produce:

| Section | Schema | Purpose |
| --- | --- | --- |
| `dokime.suites` | 2 | Test templates, parameters, steps and criteria |
| `dokime.records`, `dokime.listeners` | 1, 2 | Record contracts the plugin publishes or listens to |
| `dokime.telemetry`, `dokime.analyses` | 1 | Telemetry stream contracts and their analyses |
| `dokime.settings` | 3 | Settings the plugin exposes |
| `dokime.run-start` | 1 | A capture every execution runs before its first step |

Each template runs as two operations, `<template>.run` and `<template>.signal`,
as it does today (`crates/dokime-runtime/src/execution/serving.rs`). Step code
calls Dokime's Host-visibility operations, such as record publication, evidence
delivery and prompts, as child calls on the session. The plugin lists the ones it
uses in `Catalog.requires`.

### First-Party Plugins

- **Host telemetry** (`plugins/sidecars/host/`) moves into the backend as built-in
  operations and streams, so a default installation needs no plugin process.
- **OBC** (`plugins/sidecars/obc/`) becomes the reference example: a Rust server
  that tests start themselves and connect to by URL.

### Other Languages

Operations-only plugins need only the protos and a gRPC library, as the Harmos
guide's Python example shows. Test suites in other languages also need the
`dokime.suites` JSON and the `<template>.run` call sequence, so suites stay in
Rust until a customer needs them elsewhere. A small helper library can follow then.

## Automation

`dokime.automation.v1` is the set of Public product operations that
`cargo dokime bindings proto` already exports. The change is to treat it as a
contract:

- Attach the exported protos to every release, under the `dokime.automation.v1`
  package. Breaking changes move to `v2`.
- Keep the `.dokime-remote` pairing file as the one way to connect: endpoints,
  pinned CA and access key in metadata.
- Show short Python (`grpcio`) and `grpcurl` examples next to the
  [remote walkthrough](remote/overview.md).
- Keep `dokime-client` for the CLI and the desktop shell, and keep plugin
  operations reachable through the same server.
- Keep the Python client as a thin client. Retire `dokime.Backend`, because
  `dokime server` already runs a headless backend.

## What It Removes

| Area | Path |
| --- | --- |
| Install, store, boot sweep, extraction, reload and dependency-ordered start | `crates/dokime-runtime/src/runtime/plugins*`, `src/plugins/`, `src/runtime/sidecars*` |
| Upload, install and remove operations | `crates/dokime-server/src/grpc/plugins*`, `bundle.rs` |
| Bundle build and packaging commands | `crates/dokime-packaging/src/{build,bundle,process}/`, `crates/cargo-dokime/src/commands/sidecar/` |
| Release-feed plugin archives and installer `[[plugin]]` entries | `crates/dokime-domain/src/model/plugins/release.rs`, `crates/dokime-packaging/src/installer/`, `crates/dokime-gui/src-tauri/src/app/updates/` |
| In-process Python plugins and the Python backend | `crates/dokime-runtime/src/runtime/embedded.rs`, `sdk/python/src/{authoring,backend}/`, `sdk/python/examples/starter/` |
| Plugin drop zone and feed pages | `crates/dokime-gui/src/features/settings/plugins/` |

## Decisions Needed

- **In-process Python plugins.** `EmbeddedProvider` was kept (dokime #141) so
  Python plugins could run inside a Python-started backend. This proposal
  replaces that with Python plugins as gRPC servers.
- **Settings and secrets.** Either Dokime keeps storing settings and sends them on
  connect, or each plugin owns its configuration and secrets and reports its
  effective settings in a section, so evidence keeps them.
- **Host telemetry isolation.** In-process loses the crash isolation a separate
  process gives native Wi-Fi, Bluetooth and radio calls. The alternative is a
  host server that the installer autostarts on a localhost URL.
- **Reversible effects.** The host journals and restores radio, adapter and
  power-plan changes. That stays if host telemetry moves in-process; a remote
  plugin that changes machine state owns its own restore.
- **Development data.** Bundle-backed installations in existing roots stop
  loading. The storage policy allows this clean break with a fresh root.
