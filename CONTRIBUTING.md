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

Run the matching checks while iterating, then the local CI flow before opening a
pull request:

```sh
mise run format --check
mise run lint
mise run check
mise run build
mise run test
```

`mise run ci` runs that sequence on the current tree; a failing lane fails the
run, and that is the whole verdict. `build` compiles the workspace once into
binaries, the OBC bundle, the frontend, the generated GUI client and the
Nextest test archive under `dist/build/<platform>/`, and the Python wheel under
`target/wheels/`; the test layers (`test:unit`, `test:integration`,
`test:generated`, `test:acceptance`, `test:system`) run from those outputs and
never rebuild. Each lane takes `--profile dev|release` and keeps its JUnit
report under `target/tests/`. `test:generated` checks the generated Python and
TypeScript clients (types, lint and an import smoke). `mise run package
--profile dev` bundles an unsigned development installer through the Tauri
bundler in debug mode, with the CLI and server as companions, beside the
client wheel and the plugin bundle under `target/package/<platform>/`.

`.github/workflows/ci.yml` is rendered from the shared pipeline shape: the
Check, Build, Validate, Package and Publish stages, with Rust, Build, test lane
and package jobs on both the Linux and Windows legs. A pull request runs Check,
`Build / Dev` and Validate; pushes to `dev` add `Package / Dev` and
`Publish / Dry`; tags build, package and publish the release profile.
`nightly.yml` runs coverage and benches. `mise.toml` owns every recipe the
workflows call.

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
