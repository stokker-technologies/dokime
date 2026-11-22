# Dokime

[![Release](https://img.shields.io/github/v/release/stokker-technologies/dokime)](https://github.com/stokker-technologies/dokime/releases)
[![CI](https://img.shields.io/github/actions/workflow/status/stokker-technologies/dokime/ci.yml?branch=dev&label=CI)](https://github.com/stokker-technologies/dokime/actions/workflows/ci.yml)
[![CI / Nightly](https://img.shields.io/github/actions/workflow/status/stokker-technologies/dokime/nightly.yml?branch=dev&label=Nightly)](https://github.com/stokker-technologies/dokime/actions/workflows/nightly.yml)
[![crates.io](https://img.shields.io/crates/v/dokime)](https://crates.io/crates/dokime)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

<!-- OpenSSF Scorecard badge: pending verified URL from the supply-chain owner. -->
<!-- OpenSSF Best Practices badge: pending enrolled project ID and verified status from the supply-chain owner. -->

<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="assets/organization/logos/transparent-on-dark.svg">
    <img src="assets/organization/logos/transparent-on-light.svg" alt="Dokime logo" height="256">
  </picture>
</p>

[Website](https://stokkertechnologies.ch/en/open-source/dokime) · [Getting Started](#quick-start) · [Tutorials](docs/remote/overview.md) · [Examples](docs/index.md#examples) · [Documentation](docs/index.md)

**Run repeatable engineering tests. Keep the evidence.**

Dokime runs plugin-defined procedures, captures typed measurements and records,
and retains the evidence behind each result. The execution host owns sequencing,
operator questions, retries, cancellation, cleanup and report publication.

## Features

- Reusable suites with parameters, ordered steps, observers and criteria.
- Managed executable plugins and remote gRPC plugins.
- Scoped records, typed Arrow stream files, frozen schemas and sealed evidence.
- Operator prompts, interactive execution controls and explicit cleanup.
- Bounded plot/document reads and self-contained reports retained with sealed cycles.
- Authenticated remote access through the product Access services.

## Interfaces

### GUI

The native desktop is back in the Cargo workspace and uses the execution host.
Its product includes native installers and self-contained HTML reports.
The desktop and report build entries live in `crates/dokime-gui` and share Svelte components for document layout and evidence, including plots, tables, media, artifacts, and traceability. The desktop opens live plot streams and submits operator notes from Svelte.
Captured document tables show all rows. The connected desktop provides plot settings and live viewport measurements.
The Svelte report entry renders captured data locally and can open without a backend connection.

The desktop uses plain Svelte tables for cycle lists and saved backends. Each list keeps its sort controls and displays all loaded rows.
See [GUI Guidance](docs/guides/gui.md) for host connections and controls.

#### Examples

The existing [screenshots](assets/screenshots/) document desktop workflows.

### CLI

`dokime` creates SDK projects, packages executable plugins and organization installers, hosts Dokime and
controls a running host. Each language embeds one starter project containing
its plugin and client programs. See [Command-Line Usage](docs/guides/execution-host.md#command-line-tool).

#### Examples

```sh
mise run build -- --package dokime-cli
./target/debug/dokime new bench --lang python
./target/debug/dokime dev --plugin ./bench
./target/debug/dokime dev --no-desktop --plugin ./bench
./target/debug/dokime plugin bundle ./bench --out ./bench.dokime
./target/debug/dokime installer package --manifest ./installer.toml --output ./dist
./target/debug/dokime server --endpoint 127.0.0.1:7000 --storage ./bench-data
./target/debug/dokime --endpoint http://127.0.0.1:7000 plugin register http://127.0.0.1:7001
./target/debug/dokime --endpoint http://127.0.0.1:7000 plugin install ./bench.dokime
./target/debug/dokime --endpoint http://127.0.0.1:7000 access pairing export --label Operator --out operator.pair
./target/debug/dokime --remote operator.pair --connect-timeout 10 status --json
```

### Python

The pure Python `dokime-sdk` package (import `dokime_sdk`) composes remote plugins and async clients over Harmos gRPC.
Resources close explicitly. See the [Python SDK](sdk/python/README.md).

`await connect("operator.pair", connect_timeout=10.0)` (from `dokime_sdk`) overrides the
30-second connection budget for each endpoint attempt. The CLI's
`--connect-timeout <seconds>` overrides `connect_timeout_s` for one invocation.

#### Examples

The [Python starter](sdk/python/examples/starter/) contains one uv project with
a confirmation suite, an echo operation and a client flow. Its host-free
`uv run python -m unittest` checks the plugin description. The folder name sets
its project name, import package and display title: `my-project` becomes
`my_project` and `My Project`. The client reads `DOKIME_REMOTE` from the
process environment or the working directory's `.env` and saves sealed evidence
under `_output/`. Answer its operator question in the desktop or through
`dokime execution prompt next` and `dokime execution prompt answer`:

```sh
mise -C sdk/python run build
mise -C sdk/python run install
mise -C sdk/python run test
```

### Rust

The [`dokime` crate](sdk/rust/) contains plugin authoring and a backend client;
its product dependencies stop at protocol. See the [Rust SDK](docs/sdks/rust.md).

#### Examples

```sh
mise -C sdk/rust run sdk:build
mise -C sdk/rust run sdk:test
```

The [starter](sdk/rust/examples/starter/) uses the public `dokime` and
`dokime::plugin` barrels. `dokime::testing::Harness` runs a template under
`cargo test` with no host and no hardware; see
[Test Templates Without a Host](docs/sdks/rust.md#test-templates-without-a-host).

### C#

The [C# SDK](docs/sdks/csharp.md) provides managed/remote plugins and clients on .NET 10.

#### Examples

```sh
mise -C sdk/csharp run build
mise -C sdk/csharp run test
```

### C++

The [C++20 SDK](docs/sdks/cpp.md) uses CMake-generated protobuf/gRPC code and
composes ordinary values for plugins, documents and clients.

#### Examples

```sh
mise -C sdk/cpp run prepare
mise -C sdk/cpp run build
mise -C sdk/cpp run test
```

## Quick Start

### Install the Latest Nightly

The Windows desktop installer installs for the current user in
`%LOCALAPPDATA%\com.stokker.dokime`, without administrator rights. If an earlier all-users
copy is installed, have an administrator uninstall it from Windows Settings
before running the current-user installer. Linux DEB/RPM packages use their
normal system package installation.
Uninstall older development copies that used `%LOCALAPPDATA%\Dokime` before
switching to the distribution-identifier layout.

The desktop installer includes `dokime` beside the desktop executable and adds
its folder to the current user's PATH. Uninstall removes that PATH entry.

Release and organization installers include the kit's default plugins in `plugins/`.
Set `installer.exclude_plugins = ["obc"]` to exclude a default; use
`[[plugin]]` entries to add bundles. Packaging progress goes to stderr; `--json`
keeps stdout machine-readable and disables progress.

Add the command-line tool from the `nightly` release to a `mise.toml`:

```toml
[tools."github:stokker-technologies/dokime"]
version = "nightly"
platforms.windows-x64 = { asset_pattern = "dokime-cli-*-x86_64-pc-windows-msvc.zip" }
platforms.linux-x64 = { asset_pattern = "dokime-cli-*-x86_64-unknown-linux-gnu.tar.gz" }
```

```sh
mise install
dokime --version
```

### Create a Python Starter

```sh
dokime new my-project --lang python
cd my-project
mise trust
dokime dev --plugin .
```

`dokime new` writes one Python project with `dokime.toml` and the
`src/my_project/` package, including its client and plugin modules. Rust, C# and
C++ also use one project with plugin and client programs. The Python starter does not embed `uv.lock`. The first `uv sync` resolves the SDK
dependency named in `pyproject.toml` and creates a lock file to commit with your project.
`dokime new my-project --lang rust --from .` uses the current checkout's starter
and a local SDK dependency. All four languages accept `--from` and
`DOKIME_NEW_FROM`; the argument wins. A missing starter is an error.
`dokime dev` starts a disposable host
with the plugin attached and prints its endpoint. Bundle the plugin for installation
with `dokime plugin bundle . --out ../my-project.dokime`.

### From Source

Build `dokime-cli`, create a project from the [SDK examples](docs/index.md#examples), and
start the host with an explicit endpoint and fresh storage directory. Install a
managed `.dokime` bundle with `plugin install` or register a remote plugin URL with
`plugin register`, materialize a case and run it.

For development, `dokime dev --plugin <bundle-or-plugin-folder>` reads `dokime.toml` in
plugin folders, runs their configured build, and starts a disposable host with console
logging. It opens a sibling `dokime-gui` desktop when installed and removes its data root
on exit.

Pass `--no-desktop` to start only the host and plugin without looking for a
desktop; automated callers always pass this flag. Python projects require `uv` on
PATH and `pyproject.toml`; the configured `uv sync` build creates `uv.lock` before launch or bundling.
The [Execution Host Guide](docs/guides/execution-host.md) lists the exact commands and
configuration; the [Product Guide](docs/index.md) explains the model.

Keep existing data when a development schema is incompatible: back it up, then
explicitly select a fresh root. An occupied root is refused before SQLite opens.
Database and evidence formats remain at baseline v1.

## Contributing

See [Contributing](docs/CONTRIBUTING.md), [Repository Tooling](docs/project/tooling.md),
[Architecture](docs/project/architecture.md) and [Security](docs/SECURITY.md).
`mise run docs:serve` previews the guides as the documentation website; see
[Documentation Site](docs/project/tooling.md#documentation-site).

## About

Dokime is developed by [Stokker Technologies](https://stokkertechnologies.ch)
for engineering test workflows that need traceable measurements and evidence.
It is available under [Apache-2.0](LICENSE).
