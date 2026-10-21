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

Run the matching checks while iterating. `mise run check` covers the complete
Check stage. For committed changes with project-wide impact, simulate the pull
request graph:

```sh
mise run check
mise run ci --event pr
```

The simulation uses a clean clone of HEAD and cold dependency homes; a failing
lane fails the run. `build --profile dev` produces binaries, the OBC bundle,
the frontend and generated clients under `target/build/<profile>/<target>/`.
The canonical Python wheel uses target `any`. `build:tests --profile dev`
separately compiles the shared Nextest archive and fixtures, runs the macro
compile contracts and verifies the generated starter. Test lanes consume that
archive; acceptance and system lanes also require the matching Product
exports. Reports use `target/tests/<profile>/<target>/<layer>-<language>/`.
`test:generated` checks the Python and TypeScript clients' types, lint and import
smoke. `package --profile dev` assembles unsigned installers, companion CLIs,
the plugin bundle and Python distributions from Build bytes without compiling,
under `target/package/<profile>/<target>/`.

`.github/workflows/ci.yml` and `nightly.yml` are thin callers of the rendered
`managed-*.yml` workflows. Pull requests and `dev` pushes run Check, Build,
Validate, development Package and `Release / Dry`; configured native jobs run
on both Linux and Windows. Nightly adds `test:coverage` and benches.
`mise.toml` owns every recipe the workflows call.

Run `Release / Prepare` manually from `dev` with the requested version bump.
It opens a `release/v<version>` review into `dev`, which runs the required
native graph and unsigned `Release / Rehearsal`. Squash-merging that review
starts `Release / Tag`: it verifies the rehearsed tree, tags the merged `dev`
commit, publishes Rust crates and fast-forwards protected `main`. The tag
pipeline signs the declared Windows packages, publishes release assets and the
Python client, and verifies installation on each platform. Release notes are
rendered from Conventional Commits by git-cliff at tag time; no changelog file
is tracked. Publication tasks live under `release:publish:*`.

## Pull Requests

Describe the user-visible problem, the chosen behavior, and how you verified
it. Keep generated files in the same commit as the source that produced them.
Do not include secrets, private device data, or credentials in fixtures, logs,
screenshots, or commit history.

By submitting a contribution, you agree that it is licensed under the
repository's `Apache-2.0` terms.
