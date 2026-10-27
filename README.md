# Dokime

[![Release](https://img.shields.io/badge/release-unreleased-lightgrey)](https://github.com/stokker-technologies/dokime/releases)
[![CI](https://img.shields.io/github/actions/workflow/status/stokker-technologies/dokime/ci.yml?branch=dev&label=CI)](https://github.com/stokker-technologies/dokime/actions/workflows/ci.yml)
[![CI / Nightly](https://img.shields.io/github/actions/workflow/status/stokker-technologies/dokime/nightly.yml?branch=dev&label=CI%20%2F%20Nightly)](https://github.com/stokker-technologies/dokime/actions/workflows/nightly.yml)
[![crates.io](https://img.shields.io/badge/crates.io-unpublished-lightgrey)](#quick-start)
[![PyPI](https://img.shields.io/badge/PyPI-unpublished-lightgrey)](#quick-start)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="organization/logos/transparent-on-dark.svg">
    <img src="organization/logos/transparent-on-light.svg" alt="Dokime logo" height="256">
  </picture>
</p>

[Website](https://stokkertechnologies.ch/en/open-source/dokime) · [Getting Started](#quick-start) · [Tutorials](packages/dokime-client/examples/remote/README.md) · [Examples](docs/product-guide.md#examples) · [Documentation](docs/product-guide.md)

**Run repeatable engineering tests. Keep the evidence.**

Dokime turns hardware and software test procedures into operator workflows,
live measurements and self-contained reports. Define a procedure, run it across
operating conditions, and review the evidence behind each result.

![A charger test running in Dokime with live measurements and execution progress](screenshots/cycle.png)

## Features

- **Reusable Procedures:** Define suites, typed parameters, ordered steps, acceptance criteria and explicit cleanup.
- **Parameter Sweeps:** Expand one template into operating points without copying the procedure.
- **Measurements And Reports:** Capture live data and retain plots, records and attachments in self-contained HTML reports.
- **Operator Workflows:** Combine automation with confirmations and manual checkpoints; keep responses with the execution.
- **Equipment Plugins** [`plugin`]: Extend Dokime with equipment operations, test suites and settings through its Rust authoring API.
- **Remote Control** [`client`]: Control an existing backend through the CLI or a programmatic client.
- **Native Media Capture** [`media-native`]: Attach screenshots, camera photos and audio where the platform supports them.

## Interfaces

### GUI

The desktop brings the test library, active cycles, measurements and evidence
together. Start with the built-in Local backend or select a saved remote in
Settings. The desktop remembers that choice; operators do not need a source checkout.

#### Examples

![Test templates and cases in the Dokime library](screenshots/library.png)
![Saved test history in Dokime](screenshots/history.png)

### CLI

Use `dokime` to run a server or control an existing backend. Use
`cargo dokime sidecar` to create, build and inspect plugins, and
`cargo dokime installer` to package prebuilt binaries with an organization pack. The [CLI reference](docs/product-guide.md#cli-reference)
contains the complete workflow.

#### Examples

```sh
dokime server --application-root ~/dokime-lab
dokime status --backend lab --json
dokime operation list --backend lab
```

### Python

The async Python client controls a running backend and reads live or saved
evidence. The distribution is `dokime-client`; imports use `dokime_client`.
Follow the [locked remote tutorial](packages/dokime-client/examples/remote/python/README.md) for a complete example.

#### Examples

Install the wheel from the same candidate as your backend:

```sh
uv add ./dokime_client-0.1.0-py3-none-any.whl
```

The [Python client guide](packages/dokime-client/README.md) covers authentication,
backend profiles and execution APIs; the [runnable examples](packages/dokime-client/examples/remote/python/) retain their complete setup.

### Rust

The Rust SDK defines plugins, test templates, equipment operations and typed
evidence. Dokime builds on [Harmos](https://github.com/stokker-technologies/harmos)
for contracts and managed plugin lifecycles.

#### Examples

The [OBC plugin](plugins/obc-plugin/) simulates a charger without physical
hardware. Follow its [procedure and measurement examples](docs/product-guide.md#run-the-procedure-and-capture-evidence)
to see the public authoring API in context.

## Quick Start

1. **Download An Installer.** Choose a matching successful [CI candidate](https://github.com/stokker-technologies/dokime/actions). Dokime has no published release yet; the [download walkthrough](packages/dokime-client/examples/remote/README.md#obtain-one-matching-candidate) identifies the Windows and Linux packages.
2. **Download The Example Plugin.** Take `obc-plugin.dokime` from that same candidate. Drop it into the application's plugin settings to install it, then restart to activate it.
3. **Try A Test.** Create a cycle, add an OBC charging template from the library, select its operating points, then run it and inspect the measurements and report. No physical charger is required.
4. **Write Your Plugin.** Start with the [plugin starter](crates/cargo-dokime/templates/) and [authoring commands](docs/product-guide.md#plugin-authoring).

Dokime is pre-1.0. Pin matching application, client and plugin revisions;
public APIs and development storage formats may change. See the
[product guide](docs/product-guide.md) for the detailed model and limitations.

### Configuration

Dokime stores application preferences in its SQLite storage root. Use Settings
in the desktop application to change appearance, client timeouts, report plot
defaults, telemetry cadence and remote listener preferences. Updates carry a
revision; conflicting edits require refreshing before saving again.

The application folder defaults to `%LOCALAPPDATA%/com.stokker.dokime` on Windows
or `${XDG_DATA_HOME:-$HOME/.local/share}/com.stokker.dokime` on Linux.
`dokime server --application-root <directory>` selects another application folder;
storage paths are bootstrap options, not editable preferences. Dokime ignores legacy `config.toml` and `settings.json` files and preserves them; settings live in the database.
Development roots containing the older JSON stream-row format require a fresh storage root. Dokime reports incompatible data and never resets an existing root.

The Backends table and CLI backend commands share `backends.json`. Import a `.dokime-pair` file with an Identifier and Label, connect to it, or
remove it from the registry without connecting first. Revoke individual clients
from the host’s Paired Clients table.
Credentials remain in protected credential stores. Plugin settings and secret
catalog metadata persist in SQLite; credential values are not stored there.

Implementation limits use domain and subsystem constants. See the
[application settings model](crates/dokime-domain/src/model/settings/application.rs)
and [storage defaults](crates/dokime-runtime/src/config/storage/defaults.rs).

Live plots display saved telemetry. The time-range slider filters storage queries:
selecting a wider range retrieves older saved samples, and selecting a narrower
range changes only what is displayed. It does not change storage retention.

Client sample following watches saved storage progress and resumes from delivered
stream cursors after reconnecting. Execution-scoped follows drain the saved seal
before ending; they do not poll a second live-sample history.

Plot Default uses each plot's authored window; All queries its full saved history.

Plot groups show numeric labels in ascending order, then text labels alphabetically.
Different labels with equal numeric values remain separate and sort alphabetically.

The live log, diagnostics bundle and remote sessions remain available; remote sessions refresh when opened or focused; remote sessions
also provide Refresh. Cycle history shows absolute update times. Busy document
reads continue retrying, while other document failures retry once.

## Contributing

Bug reports, focused fixes, clearer examples and equipment integrations are
welcome. Read [Contributing](CONTRIBUTING.md) for setup and the review workflow.
The [storage benchmarks](CONTRIBUTING.md#storage-benchmarks) mirror Harmos's storage
suite using Dokime's default settings and SQLite backend, including writes, reads,
queries, filesystem operations and concurrent workloads.

## About

Dokime is developed by [Stokker Technologies](https://stokkertechnologies.ch)
for engineering test workflows that need traceable measurements and evidence.
It is available under [Apache-2.0](LICENSE).
