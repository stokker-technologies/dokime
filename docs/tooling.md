# Repository Tooling

Use `mise tasks` for the public entry points and pass arguments after `--`.
Preparation is explicit; focused builds and tests consume its installed dependencies:

```sh
mise run prepare
mise run build -- --package dokime-domain --check
mise run test -- --package dokime-domain --target lib --exact state::records::set::tests::capture_snapshot_retains_only_owner_and_frontier
mise run test -- --package dokime-runtime --features testing --target integration --exact config::settings_backends
mise run format -- --check --language python
mise run check -- --repository
```

`build` and `test` accept `--features "feature-a,feature-b"` (or a space-separated list)
with `--package`, forwarding the list unchanged to Cargo with defaults enabled.

`build --artifact tests` produces the CI archive; focused tests compile one Cargo
target directly. `check` includes formatting, lint, repository configuration and
generated drift; `--language` limits language checks. `--repository`, `--bindings`,
`--supply` and `--wasm` select individual checks.

`mise run lint -- --language rust` owns Clippy with warnings denied.
`mise run check -- --language rust` includes it. Build, test, bench and focused
compilation do not run Clippy or scan compiler output. For custom compiler
arguments use `mise exec -- cargo <operation> <arguments>`.

`.stkr/project.toml` contains only company source policy. Project recipes read
`tools/project.toml` for release metadata, packaging, language roots and test policy.
`mise.toml` and `mise.lock` pin tools, including Minisign and protoc;
`rust-toolchain.toml` declares Rust components and the WebAssembly target.
Starters and binding generation remain owned by the project tooling.

Mise is the public entry point. Python modules in `tools/python/src/dokime_tools/`
own environment setup, synchronization, version metadata, binding generation,
frontend preparation, native packaging, CI and
test execution. The shared Python environment lives at repository-root `.venv/`.

Explicit-path tool configuration lives in `tools/config/`: Cargo About uses
`about.hbs`, git-cliff uses `cliff.toml`, Cargo Deny and the notices collector use
`deny.toml`, Taplo uses `taplo.toml`, and supply-chain checks read
`supply-chain-exceptions.toml`. Tasks pass these paths explicitly; direct tool or
editor invocations must select the same configuration. Native Cargo, mise, uv,
Clippy and rustfmt discovery files remain at the repository root.

Contributor and security guidance live in [Contributing](CONTRIBUTING.md) and
[Security](SECURITY.md). The environment example is
[`docs/examples/.env.example`](examples/.env.example); published captures live in
[`docs/screenshots/`](screenshots/README.md).

`mise run screenshots` runs the fixed native desktop journey on fresh storage and
captures all seven scenes in light and dark mode. `--scene cycle` limits output to
one scene and its prerequisites; `--check` validates the complete published set and its manifest without
starting the application. Build development product artifacts and run
`mise run prepare:desktop` first. Captures use real time until the host virtual
clock is adopted; review the images and commit them as a chore. See the
[screenshot guide](screenshots/README.md) for viewports and scene content.


After changing Python dependencies, `mise run prepare -- --refresh-lock` refreshes
the workspace `uv.lock` before the ordinary locked synchronization. The test group
includes both optional dataframe libraries; focused tabular tests also block their
imports to verify dependency-free iteration and missing-extra errors.

The desktop GUI owns its web tooling under `crates/dokime-gui/`: Biome, the
pnpm lock and the RPC generator in its `tools/`. Rust
performance workloads live under the owning crate's `benches/`.

Desktop development process trees are owned by the task-built `cargo-dokime`
executable with the application's validated packaging settings. `mise run dev`
selects that executable from its existing Cargo compiler record. The unchanged
`tauri:dev` and `build:report:watch` package commands select the development
artifact from `mise run build -- --language rust` by default; `--build-json <record>` selects
another successful task build. Missing artifacts fail explicitly; these
launchers never build or install a supervisor. `tauri:dev --print-config`
continues to inspect frontend launch configuration without an artifact.
Rust Product writes `product/compiler.pending.json` while Cargo runs and replaces
`product/compiler.json` only after success. A failed build retains its pending
record for diagnosis without publishing partial compiler output to frontend consumers.
`operational.packaging.process_stop_ms` bounds each cleanup stage separately:
Unix gets one graceful-wait budget and one forced-reap budget, up to twice the
configured duration; Windows uses only the forced-reap budget. Surviving owned
descendants are terminated even when their leader already exited.

Each Python package carries its own `[tool.ty]` configuration. `lint -- --language python`
type-checks the repository tooling with `ty check --project tools/python`;
`check -- --bindings` checks the hand-maintained SDK stubs against the built extension with mypy stubtest and type-checks the SDK.

`mise run build -- --artifact tests` packages the host and OBC fixture bundles from the
Tests compiler output under `target/build/<profile>/<target>/fixtures/plugins/`.
Rust stories resolve those fixtures from the Tests export, so the archive owns
the bundles its integration lane requires.

## CI Compiler Caches

Compiler caches are separated by OS, architecture, build profile and producer.
All use `SCCACHE_CACHE_SIZE=8G`; archive-only test consumers need no compiler cache.
`tools/python/src/dokime_tools/ci/native.py` derives the keys during setup:

```text
ci-v6-sccache-<os>-<arch>-<dev|release|test|bench>-<Rust version>-<Cargo lock digest>
```

Each cache restores its exact key, then the prefix before the Cargo lock digest.
Changing an unrelated tool preserves compiler-cache compatibility. GitHub provider
steps save successful dev and Nightly runs on an exact-key miss; PRs restore only.
Setup keeps job-level `COMPILER_CACHE_BACKEND` and `COMPILER_CACHE_MODE` unless
its corresponding nonempty input overrides them. Cache save steps pass the
resolved compiler cache `key` and `path`. Tasks own compiler processes and timing summaries.

## Benchmark Baselines

Nightly Rust benchmarks restore `target/benchmarks/` by OS, architecture and layer:
`ci-v6-baselines-<os>-<arch>-<layer>-`. Successful Nightlies on dev save refreshed
measurements with run ID and attempt suffixes. The restore identity stays stable
across source and tool-lock changes.

`mise run bench -- --language rust --layer <leaf> --refresh-baseline` seeds missing native baselines
and emits a `::notice::` naming each benchmark. Without refresh, a missing
baseline fails and names the refresh command. Existing baselines retain their
regression comparison and threshold; failed benchmark executions still fail.

## Native Python SDK Builds

`sdk/python` contains native Rust client, backend and callback wrappers, Python authoring declarations, contexts and readers. The `dokime-python` distribution and `dokime` import names are provisional; registry publication is refused until naming is settled.

```sh
mise run build -- --profile dev
mise run build -- --language python --profile dev
mise run test -- --language python --wheel --profile dev
mise run package -- --language python --profile dev
```

The explicit Python producer uses maturin and PyO3's Python 3.13+ stable ABI.
Ordinary Rust Product, Tests and Bench producers exclude `dokime-python`; Cargo's
`default-members` also omit it. The SDK depends on `dokime` and `dokime-server` by
the owner-locked exception to facade-only bindings, and its source-root exception
is declared in `.stkr/project.toml`.

CI builds Product and then Python on Linux and Windows, restoring the matching
Product compiler cache. Both use the native Cargo target directory and the same
profile; release wheels enable the server's native media feature. Shared Cargo
units reuse only when their resolved dependency features match. The wheel and
package exports use `target/{build,package}/<profile>/<native-target>/python/`.
Package copies the built wheel and never compiles it. Release assets collect both
native wheels; no source distribution or platform-independent wheel is produced.
`test -- --language python --wheel` installs into a disposable environment outside the checkout
and imports the native extension with isolated Python path handling.

## Protocol Generation

`proto/dokime/v1/` owns the checked-in execution, plugin, context-operation,
catalog and document contracts. The Harmos transport is copied read-only under
`proto/harmos/v1/` and must match the adopted Harmos revision.

```sh
mise run generate -- --protocol
mise run test -- --package dokime-protocol --target integration
mise run test -- --package dokime-cli --target integration --exact proto_export::proto_export
```

Generation and drift checks use the pinned vendored protobuf compiler; no
installed `protoc` is needed. Tests Build transfers the pinned native compiler and
its includes under `fixtures/inputs/vendor/`; archived tests select these inputs
with `TEST_PROFILE` and restore executable permission on Unix. Focused tests use
the compiler in the local Cargo registry. Builds consume the checked-in Rust under
`crates/dokime-protocol/src/`, exposed as `dokime_protocol::v1` and
`dokime_protocol::harmos::v1`. `mise run generate -- --protocol --check` runs the
same drift check without regenerating committed source. Existing GUI/client
contract generation continues separately through the ordinary `generate` task.

## Focused Python Checks

`mise run test -- --language python --layer unit --select <expression>` and `mise run test -- --language python --layer integration --select <expression>` run the moved client tests against the installed native wheel. Build it first with `mise run build -- --language python`. `check -- --bindings` runs stubtest from an isolated installed-wheel environment. Python contract generation and its checked-in wire tree have been removed; private interpreter message views come from Rust reflection.

For client acceptance without the desktop or Rust test archive, build the focused
native inputs and select their fixture root:

```sh
mise run build -- --language python --artifact fixtures
mise run test -- --language python --layer acceptance --artifacts target/build/dev/windows-x86_64/fixtures/python-client
```

Add `--select backend` to run only the installed-wheel backend startup, pairing,
storage refusal and installed-plugin collision stories during backend work.

Use the corresponding native target directory on Linux. The foreign-operation
socket fixtures come from Rust contract construction; refresh their JSON with
`mise exec -- cargo run --locked -p dokime-python --example client-contracts -- sdk/python/tests/integration/client/fixtures/operations.json`.

## Python System Story and Batch Measurements

`sdk/python/tests/{unit,integration,acceptance,system}/` separate pure value
invariants, in-process callback ownership, installed-wheel stories and the
CLI/SDK composition. `sdk/python/examples/starter/` supplies `dokime new --lang
python`; it is embedded by `crates/dokime-cli/src/commands/new.rs`, so changing the
starter requires rebuilding the CLI as well as validating the Python source.

```sh
mise run build -- --language python --artifact fixtures
mise run test -- --language python --layer system --artifacts target/build/dev/windows-x86_64/fixtures/python-client
```

The system story starts the generated Python plugin, exports pairing, runs a
prompt and measurement execution, seals its report, and reopens it without the
plugin. It consumes only the native CLI and wheel, never a desktop or test
archive. Nightly runs the same task on Linux and Windows; PR and dev jobs do not
select it. Use `linux-x86_64` for the corresponding local Linux fixture path.

After validation has stopped, measure the same 10,000-row, 1,000-row-batch workload:

```sh
mise exec -- cargo run --locked -p dokime-python --example authoring-batch
mise run bench -- --language python --layer system
```

The native example and Python benchmark both await admitted ingestion. Python's
native pytest-benchmark JSON under `target/benchmarks/` additionally records
synchronous bridge preparation (snapshot, encoding and value conversion) and the
awaited publication separately. These are development-profile workstation
measurements, not a throughput target. Timings are absent from correctness tests.

## Test Hang Guards

`[conventions.testing.deadlines]` in `tools/project.toml` owns the lane hang guards.
The test runner passes these values to nextest, pytest and Vitest. Script lanes,
including native desktop system tests, also have a whole-lane process-tree timeout
using the same configured deadline. These guards terminate hangs; completed story
durations remain measurements and never change a runner's pass/fail result.
Rust fixture operations and in-story waits
share `STORY_BOUND` and `FIXTURE_POLICY` in
`crates/dokime-runtime/tests/support/settings.rs`; the desktop stories size their
waits with the constants at the top of `crates/dokime-gui/e2e/native-desktop.mjs`.
Product operation policies keep their own deadlines.

## Workstation Builds

`prepare` synchronizes the root Python environment and frontend dependencies.
Every other task uses `uv run --no-sync`; run preparation after dependency changes.
Focused package builds and tests start one Python interpreter, validate the selection
and launch Cargo directly. They do not inspect or delete compiler output, acquire
an output lock, create an archive or manage a compiler-cache server.

For local builds, `mise.toml` selects `RUSTC_WRAPPER=sccache`,
`SCCACHE_DIR=<OS user cache>/sccache` and `SCCACHE_CACHE_SIZE=20G`.
sccache owns its persistent background server. Incremental compilation stays enabled;
only CI sets `CARGO_INCREMENTAL=0`. Each checkout keeps its own `target/`.
Contract generation keeps each invocation's compiler record in a private temporary
folder, so overlapping checks cannot overwrite the record another invocation reads.
The session running compilation monitors free disk space outside project tooling.
If allocation failures recur, disable it for one run by setting
`RUSTC_WRAPPER` to an empty value:

```sh
RUSTC_WRAPPER= mise run build -- --language rust
```

In PowerShell use `$env:RUSTC_WRAPPER = ''; mise run build -- --language rust`, then remove
the override with `Remove-Item Env:RUSTC_WRAPPER`.

## Validation

Run targeted tasks locally for changed behavior. Hosted CI and Nightly provide
full validation across the declared platforms.

Every pull request runs the complete check, build, test and package matrices,
including reviews into `dev`. There is no path-impact planner. A job checks out,
uses the single setup action to install tools and run `prepare`, restores caches,
downloads its declared inputs, runs one work task and uploads results. Platform
values select the existing Linux and Windows runner images in each matrix.
The setup action provisions Python 3.13 before native bootstrap; Ubuntu 22.04's
system Python lacks the standard-library TOML reader used to select cache keys.
Mise then installs the project's locked Python version for work tasks.

`build --artifact product|tests|bench`, language builds and `package` own archive
transport through `dokime_tools/ci/artifacts.py`. `CI_TRANSPORT=true` extracts only
downloaded archives under `target/ci`; `CI_OUTPUTS` declares producer paths and
transport names. Tasks expand `{profile}`, `{target}` and `{platform}`. These
variables are provider transport metadata, not workstation preparation options.
Compiler caches are scoped by OS, architecture, compiler profile and the selected
Rust toolchain, followed by the restorable Cargo lock digest.

Hidden `ci:nightly-freshness`, `ci:nightly-publish`, `ci:nightly-report`, `ci:review`,
`ci:release-mode`, `ci:required`, `ci:mirror`, `ci:bindings`, `ci:sign-product` and `ci:sign-windows`
own provider operations. They accept `--help` and stay out of `mise tasks`.
`prepare` only prepares dependencies. Release preparation includes its review;
`ci:release-mode` selects tagging when a successful `CI` completion belongs to
an already merged release review. Tagging uses that review's squash commit,
even after later dev merges. A manual dispatch on dev retries the prepared,
untagged release, or prepares the next release otherwise. Merging a release review
keeps its validation running. Review artifact lookup requires green review validation;
tagging and publication also require exact-commit green dev CI. Windows signing prepares
product bytes in a task, retains Azure's provider action, then packages verified
signed bytes before installer, updater and archive checksums. Both signing jobs
use explicit status conditions on tag pushes so the skipped dev-only mode job
does not suppress signing; failed or cancelled prerequisites still block them.
Provider scheduling stays in the workflow; `ci:sign-product` and
`ci:sign-windows` retain the signing and packaging logic.

Packaging waits for its builds. Tests and generated checks consume their build
artifacts; benchmarks wait only for their benchmark build. Release rehearsal and
dry publication consume packages. Nightly publication depends only on packages;
its final report joins the suite results and records them in the published notes.
Dev pushes keep the existing check/build-only scope. The final required gate
rejects failed, cancelled or unexpectedly skipped jobs; only event-excluded work
may be skipped.

Nightly runs every four hours or on manual dispatch, serializes runs without cancelling an active run, and skips the whole run after Plan / Freshness (every downstream job) for scheduled heads already published or heads with another in-progress or queued Nightly run.

Release staging reads `NOTICE` and `THIRD-PARTY-NOTICES` from each target's
`target/package/<profile>/<target>/rust/dokime-distribution-<version>-<triple>.zip`.
Package jobs already include these ZIPs in their Rust artifacts. Staging requires
byte-identical notices across targets and fails with the artifact path if a copy
is missing or differs. Both Nightly and stable publication reuse these bytes for
the shared notice downloads and CLI and sidecar archives; publication does not
regenerate notices or require GUI `node_modules` for them.

Release tooling shares the notice download helper for all HTTP requests, including
provider pages, the crates index and release identity tokens. It makes at most six
attempts on connection errors, timeouts, incomplete reads, SSL EOF and HTTP 5xx;
HTTP 4xx responses are never retried. Exponential delays start at two seconds,
cap at thirty seconds and use jitter between half and all of the delay. Small
requests use sixty-second timeouts; uploads add one second per decimal MB, capped
at thirty minutes. Before retrying a GitHub asset upload, the tooling lists assets
and deletes matching names, including partial uploads. Successful uploads must
match the local size and, when the API supplies it, SHA-256 digest; staged asset
hashes come from the same inventory as SHA256SUMS. Exhausted requests report the
operation, URL, attempt count and last cause.

## Legal Notices and Release Notes

Packaging generates complete product notices at
`target/notices/THIRD-PARTY-NOTICES` and copies the authored `NOTICE` beside them.
Python-only notices live under `target/notices/python/`. Installer staging copies
these generated inputs into the distribution and appends them to its installed
`LICENSE`; archives and wheel license directories retain their existing notice names.
No generated notices are tracked in the source root.

Git-cliff reads `tools/config/cliff.toml` for version selection and per-release
notes. Release rehearsal consumes staged build/package artifacts; inspect its
credential-free options with `mise run release:rehearse -- --help`.
