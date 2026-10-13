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
mise run sync
```

Start the desktop application with `mise dev`. The repository layout and other
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

Run the matching checks while iterating, then the main gates before opening a
merge request:

```sh
mise run check
mise run build:release
mise run test
```

`check` runs read-only formatting, lint, and the remaining checks sequentially.
`build:release` collects uninstrumented native products under `dist/build/`.
Linux release wheel builds require a running Docker daemon. The Python stage
builds the prepared source archive in a digest-pinned manylinux 2.28 image,
using the repository-pinned Rust, Python, uv, and Maturin versions. Container
native libraries, compiler outputs, and Python caches remain separate from
the host build. The download archive includes the image/tool/package receipt
bound to both the wheel and source-archive hashes; native ABI3 and Linux
portability audits remain mandatory.
`test` requires those exact host, plugin tool, and Python wheel artifacts; it
compiles test harnesses and fixtures but does not rebuild missing products.
Set `STKR_COVERAGE=1` when running `test` to include coverage for the same suites.
Test and coverage evidence lives under `.stkr/tests/`.

CI bootstraps the latest stable mise release, verifies its published SHA-256,
and logs the actual tools. Other tool versions and dependency resolutions stay
locked. `.github/workflows/` owns native Linux and Windows jobs; `mise.toml`
owns recipes. Feature pushes run `check`; an open pull request suppresses the
duplicate push check. Pull requests into `dev` and `main` additionally require approval
through the `full-validation` environment, native release builds, and dependent
full tests with `STKR_COVERAGE=1`. `Ci Gate / Full Validation Gate` rejects missing, failed,
cancelled, or skipped required jobs and target advances that change the tested
integration tree. A green check-only run does not satisfy that gate.

Each native download archive in `dist/downloads/` carries `manifest.json` at
its root beside `bin/` (every executable), `installers/`, `python/` (wheel and
source archive), `plugins/`, `licenses/`, and `proto/` (the unpacked portable
contract: `.proto` files, descriptor set, `contracts.json`, `manifest.txt`,
`revision.txt`). The manifest lists every file with its SHA-256 and records the
published contract archive's hash under `contracts.archive_sha256`. Everything
validation needs but users do not lives under `validation/`: the report template
that `ci:artifacts:restore` rehydrates, `audits/`, Cargo timings, `fixtures/`,
and, after successful validation, `tests/` and `coverage/`. Only `python/` may
nest archives: the Nextest archives the Rust lanes replay stay in
`target/test-archives/`, travel as the build job's own
`test-archives-<os>-<job>` artifact, and are bound to the package through
`manifest.json`'s `test_archives` inventory; `ci:artifacts:archives` admits a
downloaded set only when every name, size and SHA-256 matches. After
validation, `examples/charger-report.html` is the sealed report the Python
acceptance lane captured from this same candidate against the Mini charger
sidecar (`DOKIME_EXAMPLE_REPORT` declares where that lane exports it; a declared
export without a completed capture fails the lane). The build job also exposes
the installer set (`installers-<os>-<job>`, roles deb/rpm/installer/signature/
updater) and, on Linux, the pure wheel and source archive (`python-<job>`) as
their own artifacts; after validation the unchanged installer set is re-exposed
as `installers-validated-<os>-<job>`, the set a release consumes. The zip keeps
every file; the manifest binds each hash. The
catalog records both platforms' original native build/test jobs, run attempt,
artifact IDs, and archive hashes. Dependency and tool caches live in `.cache/`;
instrumented Cargo output is cached separately from release packages. JUnit,
Cobertura, and Cargo timing reports are retained as native workflow artifacts.

Cut `release/<version>` from published `dev` and run `mise run release:prepare`
to prepare the stable patch version, changelog, and native lockfiles. Prerelease
version modes are not supported. Commit the
candidate before validation. Promote the stabilization branch into `main`
without squashing, then incorporate its history into `dev`. Only stabilization
branches and `fix/` branches with declared `[release.hotfix]` context may target
`main`; a hotfix starts from an actual published stable tag and increments its
patch version.

Every new commit SHA requires fresh build and validation, including identical
source trees after squash, merge, or rebase. `VALIDATED_RUN_ARTIFACT_ID` supports
explicit recovery of intact successful artifacts for the same SHA and resolved
inputs only; pull requests always run native build and test jobs. Caches
accelerate builds but do not supply validation evidence.

An approved stabilization or hotfix merge into protected `main` automatically
builds and tests both platforms on the final commit, then publishes through the
protected `release` environment without another manual approval. Configure that
environment for protected `main`, with no required reviewers, and provide the
Windows Tauri signing key there. Missing signing material blocks publication.
The tag created for that validated SHA does not trigger another workflow.
Publication consumes original packages and archives without rebuilding or
restamping. GitHub draft assets include Python distributions, native installers,
platform manifests, `release-manifest.json`, and `SHA256SUMS`. Finalization
downloads every remote asset, verifies its bytes and the Windows updater
signature, then publishes the draft. `mise run release:verify` repeats published
remote verification and records the machine receipt. Conflicting tags or assets
fail; corrections require a new patch. Retire a stabilization branch only after
verified publication and history incorporation into published `dev`.

## Pull Requests

Describe the user-visible problem, the chosen behavior, and how you verified
it. Keep generated files in the same commit as the source that produced them.
Do not include secrets, private device data, or credentials in fixtures, logs,
screenshots, or commit history.

By submitting a contribution, you agree that it is licensed under the
repository's `Apache-2.0` terms.
