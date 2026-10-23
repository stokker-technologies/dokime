# Contributing To Dokime

Dokime is open to bug reports, documentation fixes, focused code changes, and
small examples that make the test workflow easier to understand or extend.

## Before You Start

Use the [issue tracker](https://github.com/stokker-technologies/dokime/issues)
for bugs and feature proposals. For a substantial change, open an issue before
writing the implementation so the workflow, compatibility cost, and ownership
boundaries are clear.

Please include the operating system, Dokime revision, reproduction steps, and
relevant logs in a bug report. Remove secrets and proprietary test data first.

## Development Setup

Dokime supports Windows and Linux. Install
[`mise`](https://mise.jdx.dev), then prepare the checkout:

```sh
mise trust
mise install --locked
mise run prepare
```

Development and test builds leave host media capture out, so its operations
refuse with "capture not available in this build". The release build
(`mise run build --profile release`) enables the `media-native` feature to capture
screenshots, camera photos and audio. On Linux it links the system ALSA library
and generates V4L2 camera bindings with Clang; screenshots go through the
desktop portal and need no further libraries. Install the build packages first:

```sh
# Debian and Ubuntu
sudo apt install libasound2-dev linux-libc-dev libclang-dev
# Fedora
sudo dnf install alsa-lib-devel kernel-headers clang
```

The installed server then needs only `libasound.so.2` at runtime, plus a
running `xdg-desktop-portal` for screenshots.

Native desktop system validation runs the same scenarios on Windows and Linux.
`mise run prepare:desktop` installs a Microsoft EdgeDriver matching the installed
WebView2 runtime on Windows; `test:typescript:system` runs this setup automatically.
Run Windows UI validation in an interactive desktop session. Linux uses
WebKitWebDriver and a display; CI bootstrap supplies its runtime libraries and Xvfb.

Start the desktop application with `mise run dev`. The repository layout and other
entry points are described in [`README.md`](README.md).

## Make A Change

- Keep a change focused. Do not mix a feature, dependency update, formatting
  sweep, and unrelated cleanup in one merge request.
- Follow the existing crate and module boundaries. The public SDK, runtime,
  desktop shell, protocol, and generated bindings have different owners.
- Update user documentation and examples when behavior changes.
- Add a focused test for a fixed bug or new behavior. Prefer a workflow test
  over many small tests of implementation details.
- Use Conventional Commit subjects, such as `fix(runtime): preserve stopped
  execution evidence` or `docs: clarify sidecar setup`.

Changes to the GUI report or root branding must refresh their tracked runtime
payload with `mise run prepare:report`. Commit the resulting `dist-report/` and
`dist-brand/` directories in `crates/dokime-runtime/`; Rust Check reproduces and
compares them without rewriting stale files. These crate-local resources let
published host-feature builds work outside the repository.

Run targeted checks while implementing. Once the complete source is ready,
run the local CI flow before opening a pull request:

```sh
mise run ci
```

`ci` orders Check, shipping Product Build, Tests Build, generated-client Check,
ordinary Test layers, Package and Release / Dry. A failed required command
blocks delivery. Product exports live under `target/build/<profile>/<target>/`;
Tests Build owns its archive and stamped fixtures there, including the offline
starter dependency closure. Compile-contract stories run inside Tests Build.
Test leaves such as `test:rust:integration` consume existing artifacts
and retain JUnit under `target/tests/<profile>/<target>/<layer>-<language>/`.
Generated-client types/lint/import checks are `check:<language>:generated`.
Package consumes matching Product bytes without compiling, retaining unsigned
development installers, companion executables, wheels and plugin bundles under
`target/package/<profile>/<target>/<language>/`. Development installers embed
their frontend and launch without a development server.

`.github/workflows/ci.yml` is rendered from the shared pipeline shape: the
Check, Build, Test, Package and Release stages, with required Rust, native build
and package jobs on Linux and Windows. Pull requests and development pushes
run the same non-publishing proof, including Package and Release / Dry.
Sparse `ci.jobs` declarations select Linux native/desktop prerequisites and
exact whole-job limits; jobs without bootstrap receive none. Heavy Rust jobs
select `runner = "stokker-local-linux-x64"` or
`runner = "stokker-local-windows-x64"` in their matching `ci.jobs` selector.
Nightly benchmarks use the local Linux runner. Lightweight checks, generated
checks, packaging and coverage retain hosted defaults. Provider variables do
not override these routes.
Tags select release-profile packaging and protected publication.
`nightly.yml` runs coverage and benches. `mise.toml` owns every recipe the
workflows call.

Windows signing runs only for stable-tag Package jobs under the protected
release environment. Updater signatures and `latest.json` are required signed
release assets and are excluded from unsigned candidate/rehearsal packages.
Release verification checks their presence, installer/version/manifest
consistency and downloaded checksums. The existing consistency check does not
independently verify minisign cryptographic authenticity.

Cut `release/<version>` from published `dev` and run `mise run release:prepare`
to prepare the stable patch version, changelog and screenshots. The changelog
is rendered by git-cliff from history; do not edit `CHANGELOG.md` by hand.
Promote the stabilization branch into `main` without squashing, then
incorporate its history into `dev`. Publication tasks live under
`release:publish:*`.

## Pull Requests

Describe the user-visible problem, the chosen behavior, and how you verified
it. Keep generated files in the same commit as the source that produced them.
Do not include secrets, private device data, or credentials in fixtures, logs,
screenshots, or commit history.

By submitting a contribution, you agree that it is licensed under the
repository's `Apache-2.0` terms.
