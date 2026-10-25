# Contributing

Help improve Dokime through focused fixes, useful equipment integrations and
clearer examples. Discuss substantial behavior or architecture changes before
implementing them.

## Development Setup

Install [mise](https://mise.jdx.dev) and prepare the checkout:

```sh
mise trust
mise install --locked
mise run prepare
mise run dev
```

Windows and Linux are supported. Native Windows GUI tests require an interactive
desktop and matching WebView2/EdgeDriver; `prepare:desktop` installs the driver.
Linux desktop tests use WebKitWebDriver and a display such as Xvfb. Their
desktop bootstrap reuses installed Chrome or provisions the pinned Chrome For
Testing build for real report rendering and owned-browser cleanup.

Native media capture also needs platform libraries. On Debian/Ubuntu, install
`libasound2-dev`, `linux-libc-dev` and `libclang-dev`; Fedora uses
`alsa-lib-devel`, `kernel-headers` and `clang`.

## Development Workflow

Keep a change focused and respect SDK, runtime, server, GUI and generated-client
boundaries. Update affected examples when changing their API; flag stale examples
rather than silently deleting or simplifying them.

Use Conventional Commits, such as `fix(runtime): preserve stopped execution evidence`.
Keep generated outputs with their source. Refresh embedded reports with
`mise run prepare:report` and organization assets with `mise run prepare:organization`.
Organization defaults generate JSON/theme under
`crates/dokime-gui/src/generated/organization/`; keep authored pack inputs separate.
Build exports the matching native installer payload. `mise run package:rust`
uses the built authoring tool to assemble and inspect it without recompiling;
see [Native Installers](docs/product-guide.md#native-installers) for downstream
organization distributions and signing prerequisites.

For coupled Harmos development, keep registry dependencies declared and place the
local Cargo patch in an untracked `.cargo/config.toml`. Preparation and frontend
tasks use that same explicit patch for the Harmos authoring tool. CI refuses
local overrides; remove the patch and pin the published dependency before delivery.

## Tests And Checks

During implementation, run targeted checks for changed behavior. Once all agreed
source work and targeted proof are complete, run the full local gate before
pushing the authorized candidate or opening/updating its review:

```sh
mise run ci:simulate
```

The default run covers every declared required platform; selecting one platform
is only partial proof. Local simulation runs one job at a time without CPU, memory
or thread limits. Changes limited to skills, agent guidance, README or
CONTRIBUTING files select no execution jobs; the required status gate still
completes. Other documentation uses its declared checks, while mixed source or
policy changes retain full selection.

Product and Tests build independently. Tests use Cargo's unoptimized test
profile; test lanes consume their artifacts. Nightly additionally runs coverage,
benchmarks and the generated-starter test, with release-profile installers.
Missing local executors and failed required jobs block delivery. After the local
gate passes, push the authorized candidate and let required hosted CI confirm
the same revision. Keep auto-merge enabled and investigate every required failure.

## Pull Requests

Target `dev`. Describe the user-visible change and its verification, and keep
required native checks enabled. Do not include secrets or proprietary device
data in source, fixtures, logs or screenshots.

Contributions use the repository's [Apache-2.0 license](LICENSE). Release
publication is a separate authorized workflow; ordinary candidate validation
does not publish packages or create tags.

## Reporting Issues

Use the [issue tracker](https://github.com/stokker-technologies/dokime/issues).
Include the operating system, revision, reproduction steps and relevant logs,
with secrets and private test data removed.
