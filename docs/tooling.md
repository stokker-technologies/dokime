# Repository Tooling

Use `mise tasks` to discover entry points, with arguments after `--`. Preparation
installs the root `.venv/`, builds the Python SDK wheel and installs it there, and
installs frontend dependencies. `prepare --without-frontend`
prepares SDK-only jobs without requiring Node or pnpm. Build, test and checks use
project tasks; compiler caches belong to the workstation.

Preparation synchronizes the `sdk-tooling` dependency group as part of
`--all-groups`: authored SDK build hooks and generators are type-checked with
their Hatchling, gRPC generator and mypy dependencies and pinned gRPC stubs.
Frontend generation resolves locked Cargo metadata online because a native
build does not populate dependencies for every platform in that metadata.
Product Build generates preferences, both TypeScript message roots and icons
before type checking, then builds the live and embedded report assets.

Python notices verify each locked wheel's hash and include its license files.
For a wheel that omits its license text, the notices owner must declare an
exact package version, immutable upstream license URL and SHA-256 digest;
missing declarations and mismatched source bytes fail packaging.

The Rust host-fixture task builds and stages separate capture and publication
boundary plugins. Each catalog declares only its own sampled streams, so a
capture execution never samples the boundary counter's execution resources.

## Python Tooling

Regenerate the C++ formatter lock with
`mise -C sdk/cpp lock pipx:clang-format@21.1.8` and commit `sdk/cpp/mise.lock`.
All `.mise/` folders contain ignored local state and must remain untracked.

The C++ SDK formatter and check use the shared generator's LLVM style with a
140-column limit, empty-only short functions and expanded lambdas. This keeps
generated headers and authored SDK sources under the same formatting policy.

The root uv workspace installs `tools/python` into the root `.venv/`. The
`dokime_tools` package ships `py.typed`; Ruff and ty cover all its authored
modules, including documentation generation, desktop development, release and
test orchestration. Formatting and linting cover the complete package without
subpackage exclusions or handpicked file lists.

`ci/tasks.py` parses the public arguments. Each module under `task/` selects
one operation's existing mise leaves, including the hidden `ci:*` tasks used
by workflows. `mise run build -- --plan` prints the selected commands without
receiving artifacts or compiling. Focused package plans retain their native
Cargo commands.

Release preparation, review, staging, rehearsal, tagging, publication,
finalization and verification have separate modules under `release/`.
`release/run.py` owns argument parsing and explicit dispatch. Registry
transport, crate publishing, Python publishing and review evidence have their
own modules under `release/registry/`. Package definitions, assembly, executable
verification and installer checks have separate modules under `package/`. Test lane execution, coverage,
benchmarks and child-process execution have separate owners under `test/`.
The tools package depends on `harmos-tools` from immutable revision
`b72dd3d52d29599502f7edf92644f451ffdc1fc2`. `tools/python/pyproject.toml` declares its
`tools/python` subdirectory source, and the root `uv.lock` records the same
tooling revision. No sibling library checkout is needed for preparation.
When changing the carrier pin, update this source revision and refresh the root
lock together. Carrier and SDK dependencies remain at
`c2cfb153ab136159f07741d1061fcb3a1191e86c`, independently of generator repairs.
Local editable installs can exercise unpublished tooling; no local path override
is committed.

`build/protocol.py` passes the product namespace, formatter inputs, restored
Grpc.Tools path and request selector policy to
`harmos_tools.build.resources.sdk.generate`. Shared tooling owns descriptor
analysis, the four language renderers, output inventory/drift and C# wire
projection. SDK C++ and C# generation checks use the same product configuration and complete
shared SDK inventory; both restore the C# protocol compiler without compiling it.
`build/protocol_callers.py` retains product public-surface checking; caller option
decoding uses `harmos_tools.source.protocol`. The shared checker accepts product
namespace and path configuration, but scans private `_generated` C++ headers that
the product checker excludes, so it is not a drop-in replacement. Both currently
reject repeated namespace exports introduced by the new barrels; that policy
needs an owner decision before the surface check can pass.

The pin at `c2cfb153ab136159f07741d1061fcb3a1191e86c` removes generated barrel
renames. Callers use original names and qualify collisions by their unversioned
namespace. Python exposes `dokime.client.ClientRoot`, `dokime.plugin.Window` and
`dokime.model.Window`; the last is the evidence selection, independent of the
plugin sampling grant. `sdk.generate` consumes descriptors and product
configuration without a composition file. `protocol/composition.toml` still has
runtime and desktop host-binding readers.

The Python wire compiler remains product-owned because its `surface.pb`,
`sources.json`, package markers and check-only comparison are required by wheel
and installed-consumer checks; shared `python_protocol.generate` does not emit
that complete inventory. The operations reference renderer remains product-owned
because the shared renderer labels host operations Product-Defined instead of
preserving their descriptor-declared caller audiences. These implementations are not drop-in
replacements at this pin. Shared subprocess console policy remains a separate
owner decision.

All four generated-consumer compile checks pass with the repaired tooling.
Python imports the original exception and payloads from their defining library
modules, and Rust imports library payloads from `harmos-protocol` rather than
assuming a matching SDK barrel layout. Rust payload/resource collisions keep
their original names under qualified package paths. C# optional assignments
preserve protobuf property names such as `StepId`, `MaximumSeconds`, `CycleId`
and `ExecutionId`. CLI, OBC and native SDK test callers use the qualified names
and current optional-argument shapes. Regenerate through
`mise run generate -- --protocol`; generated output is never patched by hand.
The Python starter supplies record scope as the `scope` keyword; the authored
materialization story raises `FailureError` for its transient failure.

Disposable directory owners expose their path and explicit `close()` operation.
Scratch roots, private credential stores, headless displays and build phases
have explicit `close()` operations. Native streaming handles use short
standard-library resource scopes where no path-level operation completes the
work. Locks, temporary credential environment changes and staged manifest edits
keep their rollback scopes. The pre-mise native bootstrap loads
shared process policy by file because the tools package is not installed yet;
installed workflow operations use regular imports.

Run the tooling stories through the public selection:

```sh
mise run test -- --language python --layer integration --project tools/python
mise run format:python -- --project tools/python
mise run check:python:tools
```

## Focused Development

```sh
mise run prepare
mise run build -- --package dokime-runtime --check
mise run build -- --cargo check --locked --offline --package dokime-operations
mise run test -- --package dokime-runtime --target system --story storage_root_admission
mise run test -- --package dokime-protocol --target integration
mise run test -- --changed crates/dokime-runtime/tests/system/manager_lifecycles.rs --plan
mise run format
mise run check -- --language rust --package dokime-cli
mise run check -- --language python
mise run check -- --language typescript
mise run check -- --repository
```

Focused tests use nextest and compile only the selected package/target. `--exact`
selects a test, `--story` selects a module, and `--filterset` takes a nextest expression.
Use `--no-capture` for live output while diagnosing a focused source story; its
configured hang guard is unchanged.
`--artifact tests` consumes the matching prepared archive without compilation.
An empty selection fails. Clippy runs through lint/check, never build/test.
Focused Cargo commands and hosted producers both use `--locked`; the task adds
it when a native `--cargo` command omits it. Lock refreshes remain explicit.
After source closure, `mise run build -- --cargo check --workspace --tests --locked`
runs the explicit workspace compiler proof through the same task owner.

## Command-Line Operations

The CLI builds as package `dokime-cli`, executable `dokime`. Discovery uses the
composed `Operations.List/Get` catalog, including effective callers. Dynamic calls
use the discovered descriptors and preserve protobuf JSON integer strings and
exact rational fields. They add no mutation retries. Plugin bundles upload through
`Files.Uploads` before library `Plugins.Create` records the plugin's own name.
`plugin remove NAME` reads the current entry revision before deletion.

```sh
dokime --endpoint http://127.0.0.1:7000 operation list --json
dokime --endpoint http://127.0.0.1:7000 plugin secrets list NAME --json
dokime --endpoint http://127.0.0.1:7000 plugin secrets set NAME --bindings bindings.json --json
dokime --endpoint http://127.0.0.1:7000 plugin secrets delete NAME declared-secret --json
```

The binding file is a JSON object mapping declared identifiers to stored secret
names; it never contains secret values. Set replaces the complete binding map at
the revision returned by List. Delete removes only the named bindings. Revision
conflicts retain the action and cause and require an explicit retry.

## Generated Surface Proof

```sh
mise run generate -- --protocol
mise run generate -- --protocol --check
mise run generate:typescript
mise run check:generated
mise run check:locks
mise -C sdk/csharp run restore
```

`generate:typescript` regenerates schemas, dispatch and declared preference types
without compiling runtime-owned default values. The ordinary `generate` task still
exports those defaults from their product owner before a full frontend build.

The protocol check regenerates SDK roots in temporary storage and compares the
complete output inventory, including obsolete outputs. It reports drift without
rewriting sources; unrelated working-tree edits do not cause false drift.

The generated checks compile permanent consumers under each SDK's tests/generation
(tests/Generation for C#) and the desktop tests/generation. Select one with
`mise run check:generated -- --language rust|python|csharp|cpp|typescript`. These compiler contracts require no product runtime or host fixtures.
C# uses its own `target/cs/generation` output; its normal SDK build also compiles
the authored file-transfer consumer. Generated Python obeys
ordinary lint and typing; only protoc's private `_pb2*` files use the existing exclusion,
and the public barrels plus compiler consumer are checked explicitly.

`prepare:locks` refreshes Cargo and uv locks after a dependency pin change. C# uses
`restore --refresh-lock` explicitly, then ordinary `restore` and the generation story
use locked mode. C++ records its source archive digest and vcpkg baseline as inputs.
The repository check includes buf, callers, SDK public surfaces and workflow lint.
The documentation job runs `ci:docs` for guide public surfaces, then builds the site
and checks its local links and fragments without network requests. Its
wire-name allow-list covers private generated directories, canonical
gRPC paths, architecture and execution-host definition maps, and Markdown link targets.

## Fixtures and Stories

Runtime client stories create execution evidence through real plugins and host
operations. They use the generated SDK's optional selectors in call options;
native Store fixtures belong to backend fault and storage contract tests.
The runtime system target starts at `crates/dokime-runtime/tests/main.rs`, which
loads the story and shared support modules without source-path overrides.

The Rust host-fixture producer stages the reference host and its Python
conformance package from the same locked library checkout. Python system and
coverage lanes consume that stamped package, without a machine-specific source
path. The hidden SDK conformance selection selects the matching prepared profile with
`mise -C sdk/python run conformance -- --profile release`.

```sh
mise run build:host-fixtures
mise run build:cli-fixtures
mise run test -- --package dokime-runtime --target system
mise run test -- --package dokime-cli --target system
mise -C sdk/rust run test
mise -C sdk/python run test
mise -C sdk/csharp run test
mise -C sdk/cpp run test
mise run build -- --artifact tests
```

Host fixture production builds the Rust reference, pinned library conformance host,
content fixture, product host and pure Python wheel;
The Tests archive selects Rust and Python explicitly, independent of installed
tool detection. C# and C++ producers run independently through their SDK tasks. `build:host-fixtures --language
rust|python|csharp|cpp` selects one producer. C++ uses `target/cp/` with short
generated and object paths on Windows. Its CMake 4.2 pin supports Visual Studio 18
and uses Ninja with the native MSVC environment. Cached dependency builds resolve code
generators through the same Protobuf/gRPC package targets as uncached builds.
Fixtures are exported under
`target/build/<profile>/<target>/fixtures/host/`. CLI fixtures include a generated
Rust plugin and test examples built through their own mise tasks. `dokime new` embeds both
`examples/plugin` and `examples/test` directly (`examples/Plugin` and
`examples/Test` for C#); SDK example journeys prove their behavior. CLI scaffold
stories verify the embedded files and preserve existing destinations. Stories
never compile plugins.

`HARMOS_SOURCE` may select an existing checkout when preparing the C# dependency
feed. C++ uses its pinned vcpkg checkout in mise's machine cache and FetchContent
under `target/cp/`; no product-specific dependency-cache override is required.
Python consumes the built pure wheel and the pinned library SDK. A focused Python
lane (`mise run test -- --language python --layer system`) builds the wheel from the
current source and reinstalls it into the root `.venv/` before pytest; the prepared
schedule installs the staged wheel once before its concurrent lanes start. Python system
and coverage tasks select the conformance executable from the host fixtures.
Linux host stories own an isolated session bus/keyring for credential storage;
install `dbus`, `gnome-keyring` and `libglib2.0-bin`. Native unit tests run without
a host; connection journeys belong to integration and thermal execution to system.
Windows uses its operating system credential store and removes
every disposable root's protected slots through the product removal operation
before deleting that root, including when a story unwinds.

## Desktop Integration

`mise run dev -- --help` describes the entry point without generating or starting processes.
`mise run dev` is the single entry point for interactive desktop work
(`tools/python/src/dokime_tools/dev/desktop.py`). It runs `generate:typescript`,
builds `plugin-obc`, `dokime` and `dokime-host`, and bundles the plugin like the
product build does. It then installs that bundle through a short-lived
`dokime server` on a fresh root under the system temporary directory, using a
unique `dokime-dev-<id>` credential namespace. Finally it starts
`tauri dev --features console` with that root and namespace. Windows owns the
started tree through a kill-on-close job; Linux uses one session per started
process. On exit the tool waits for every owned process to stop. It then runs
`dokime-host --remove-credentials` for each root and deletes the folder. The
desktop `package.json` keeps only the scripts that tasks call: `build:live`,
`build:report` and `typecheck`, plus Vite's `dev`, which is the Tauri
`beforeDevCommand`.

The desktop native package is restored to workspace and default membership.
Build it with `mise run build -- --package dokime-gui`; add
`--features tauri/custom-protocol` to embed the prepared live frontend.
Product builds prepare generated clients, icons and the production frontend before
compiling and exporting the native shell with `custom-protocol`. The icon generator
orders ICNS representations canonically so repeat builds retain identical bytes
and integrity hashes. Tests Build also
prepares the SDK desktop bundle. Run focused native desktop stories with
`--features testing`; their disposable-root owner removes credentials after the
runtime stops, including on failure. WebView lanes are available through
`mise run test -- --language typescript --layer system --select native-desktop`
and the same command with `--select two-gui-remote`. Both drivers use the current
host and SDK thermal fixture; the native-desktop lane also runs the
[OBC](../plugins/sidecars/obc/) Exact Values template for its exact summary tiles. Frontend unit and integration selection includes all
retained suites. PR and Nightly select native acceptance plus both WebView lanes
on Linux and Windows; Nightly also produces and runs the desktop benchmark archive.

`mise run generate` builds the desktop's `export_preferences` example and reads
canonical defaults from the product and listener declarations, then generates
protobuf messages, preference view models and native command declarations.
Exporting defaults opens no host, storage root or credential store. Ordinary host
startup still requires the platform credential store.
`mise -C crates/dokime-gui run generate:host-frontend` regenerates canonical host
messages and native dispatch. Product staging carries both `generated/typescript`
and `generated/host`; consumers restore both before importing the frontend.
`mise -C crates/dokime-gui run build:frontend` typechecks and builds the live shell.
`mise -C crates/dokime-gui run generate:report` builds the standalone renderer into
GUI-owned `dist-report/`, without recreating a retired runtime package.

`mise run generate -- --protocol` regenerates checked-in Rust protocol output;
add `--check` to prove its drift. Never edit generated files.
The protocol task also generates four-language SDK roots through the pinned
library's descriptor table/generator; its check verifies those outputs for drift.
Never edit generated files.

## Documentation Site

The website renders the guides in `docs/` in place through the Starlight project in
`site/`, with static search and a release selector; the operations reference is
generated from the protocol definitions and their comments. Edit guides only in
`docs/`; each guide's first heading becomes its page title.

```sh
mise run docs:serve
mise run docs:build
mise run docs:build -- --release --archive
mise run docs:image -- --archive target/docs/dokime-docs.zip
```

`docs:serve` previews the development entry with live reload. `docs:build` writes
the development entry to `target/docs/dev/` and fails on a broken page link or
heading anchor; `--release` builds the release line of the current version under
`/<MAJOR.MINOR>/` instead, and `--archive` packs the entry as
`target/docs/dokime-docs.zip`. Release staging publishes that archive as
`dokime-docs-<version>.zip` (required for a stable release) or
`dokime-docs-nightly.zip`. The staging owner in
`tools/python/src/dokime_tools/release/staging.py` checks the archive entry and
version before adding it to the shared download inventory. `docs:image` downloads
the newest archive of each release line and the Nightly archive (it needs `GH_TOKEN`), or assembles the given archives,
and packs `target/docs/dokime-docs-site.zip` with `versions.json` and the `latest`
redirects. Pull requests run `Build / Documentation`; Nightly builds the archive
beside publication and its report states whether the archive was attached.

## Build and Delivery Artifacts

`mise run build` produces the current host/CLI workspace. Tests Build produces
one archive plus host/CLI fixtures and the vendored protobuf compiler input.
The native desktop package compiles with workspace selections; Product now exports
its shell and frontend inputs for the restored hosted consumers. Shipping sidecar
plugins and removed macro/compiler fixtures are not selected. The current Cargo
workspace also contains protocol, storage, operations, runtime, CLI,
SDK and bundle inspection packages.

Product outputs live under `target/build/<profile>/<target>/`; packages under
`target/package/<profile>/<target>/`. CLI archives retain the `dokime` binary.
The distribution ZIP contains that CLI, the desktop, authored organization resources
and legal notices. Native Tauri packaging consumes those verified Product bytes
without compiling or patching them and produces Windows NSIS or Linux DEB/RPM
installers. Stable publication signs the validated Windows Product, refreshes its
distribution inventory and packages it before generating updater signatures;
Nightly stays unsigned. Shipping sidecar feeds await their re-authoring tasks.
The pure Python wheel/source pair uses the shared `any/python/` target.

Local `mise run package -- --language rust --profile dev` consumes native Product
outputs, including `product/target.txt` and the desktop distribution. Package
the Python pair with `mise run package -- --language python --profile dev`.
Then `mise run release:rehearse -- --profile dev` verifies the native packages
and registry publication dry runs. `--stage-only` selects local staging and
download verification without the registry dry runs; hosted rehearsal remains
complete and uses the release profile. Protocol archives require both owners'
model, host and plugin definitions.

`mise run release:rehearse -- --channel nightly --profile release` runs the
credential-free Nightly rehearsal against native packages. It uses the same
Nightly staging mode as publication, including Git dependencies and report
assets, without registry publication dry runs. Stable remains the default and
rejects Git dependencies. Local output names the covered target and explicitly
lists untested targets; `--target all` requires their packages to be available.

`tools/project.toml` owns package/release/test policy; `mise.toml` owns commands
and tool pins. `.stkr/project.toml` contains only Hub source policy. Configuration
under `tools/config/` is selected explicitly. Do not redirect another project's
Python environment or place TEMP/TMP inside a checkout.

## Storage Benchmarks

```sh
mise run build -- --artifact bench --binary storage
mise run bench -- --binary storage
mise run bench -- --binary storage --once
mise run build -- --artifact bench --binary system-desktop
mise run bench -- --language rust --layer system --binary system-desktop --once
```

The producer selects the runtime's `storage` benchmark in Cargo's bench profile; its
in-process hosts run on the product host's multi-thread runtime.
The consumer verifies the exported executable and fixture stamp. Storage observations
and its console report stay under `target/benchmarks/storage/`; desktop observations
stay under `target/benchmarks/criterion/`. Run producers and consumers sequentially:
the producer replaces its exported benchmark directory, which Windows locks while
an executable is running. Storage selection needs no desktop frontend.
The storage backend's focused proof uses package selection:

```sh
mise run format -- --language rust --package dokime-storage
mise run check -- --language rust --package dokime-storage
mise run test -- --package dokime-storage --target lib
mise run test -- --package dokime-storage --target integration
```

These stories cover compressed IPC rollback/recovery, root-scoped native reads,
original append receipts, resumable uploads, recovery snapshots and entries,
typed integer values/sums, library floating-point ranks, retained file readers
and unchanged archive stream bytes.
The runtime benchmark additionally depends on the runtime and Rust SDK consumers;
a backend package check alone does not establish that benchmark's build or run.
On Linux its consumer owns the same isolated Secret Service as native host
fixtures, so the runtime archive case needs no unlocked personal keyring.

Each case prints its timed boundary. `--once` runs one measured iteration per case
in the bench profile and prints an observation without a statistical estimate or
baseline promotion. Omit it for the unchanged Criterion sampling configuration.
Case identities distinguish preview reads, full scans, store admission and runtime
publication; their baselines must never be interchanged. Benches record durations
and refusals; stories assert deterministic behavior, never timing or sampled load.
Nightly identities include the lane, native platform, owning harness, case and state.
The report inventory retains every metric and its timed boundary; release notes
list each case and boundary, including cases without baselines. Unmeasured desktop
cases retain their failure reason with no numeric value. Native Criterion output
and case boundaries remain in each harness's `run.log`.

Warm queries use an already queried store. Cold queries include `Store::open` and
the first query through that new store, with independent codec/query state. They
do not evict the operating system's file cache or claim cold physical I/O. Fixture
population is outside query/finalize/export timers. Document cases retain the
three-day deterministic three-phase waveform, 30 streams/plots and original row
densities. They capture expansion, count/min/max/mean/RMS tiles and plot payloads
at 1,500 points; the median case adds one median tile and has one plot.
Neither includes HTML rendering or report-file writes. The former extra standard
deviation tile is removed. Full-decode diagnostics consume all fields of every row;
the LTTB window case uses the product query that must decode all selected rows.

The following Windows x86-64 observations were measured on this workstation
(AMD Ryzen 9 5950X) on 2026-11-16 at `170e08f63` with library `82e394da`, default
storage settings and ZSTD level 1 stream batches, in milliseconds. Each figure is the
median of five `--once` passes in the bench profile, one iteration per case and pass;
every case completed and each runtime case retained exactly its requested rows.
Native boundaries and observations are retained under `target/benchmarks/storage/`.
The previous column is the single 2026-11-15 iteration this table replaces: storage
then encoded at ZSTD level 3 and flushed on the admitting caller, and the runtime
cases ran on the earlier per-sample tick carrier, so those rows are not comparable.

| Case | Timed Boundary | State | Median, ms | Previous, ms |
| --- | --- | --- | ---: | ---: |
| Store Admission and Flush, 1,024 | Prepared protobuf batch, direct admission and durable flush; first file creation when absent | Open store | 3.413 | 10.172 |
| Window Preview, 1,000,000 | Saved preview bins, summary and bounded points | Warm / cold | 0.671 / 2.022 | 0.653 / 3.411 |
| Window Full Scan LTTB, 1,000,000 | Preview summary, full selected-row decode and 1,500-point LTTB payload | Warm / cold | 160.498 / 170.362 | 171.856 / 190.483 |
| Stream Finalize, 65,536 | Close stream files and join sample workers; durable | Warm | 0.850 | 1.134 |
| Minimal Archive in Memory, 16,384 | Current archive into a `Vec`; no report or output sync | Warm | 5.496 | 5.929 |
| Store Ingest Admission, 1,000,000 | Generate/encode phase, ordinal and value; admit in 4,096-row storage batches; background flushes included, last tail not necessarily durable | Fresh empty store | 590.849 | 7,832.789 |
| Store Ingest and Flush, 1,000,000 | Same generation/admission plus explicit durable flush; shutdown excluded | Fresh empty store | 579.026 | 8,323.425 |
| Median and Plot, 1,000,000 | Five summary tiles, median and one plot payload | Warm / cold | 124.753 / 125.748 | 149.882 / 129.962 |
| Three-Day Document, 307,200 | Five tiles and 30 plot payloads/presentation | Warm / cold | 24.562 / 26.285 | 24.588 / 27.853 |
| Three-Day Document, 1,536,000 | Same document at original density | Warm / cold | 24.677 / 26.983 | 35.895 / 31.670 |
| Three-Day Document, 3,072,000 | Same document at original density | Warm / cold | 26.978 / 28.756 | 27.459 / 29.248 |
| Document Full Decode, 3,072,000 | All fields of all 30 streams, consuming decoded buffers; no previews/rendering | Warm | 492.886 | 539.972 |
| Runtime Observer Publication and Evidence Barrier, 1,024 | Release prepared step, SDK observation/transport/admission, produced-count record and captured prompt; end/shutdown excluded | Warm host, fresh execution | 2,225.324 | 5,289.888 (tick carrier) |
| Execution End and Host Shutdown, 65,536 | Release final prompt, cleanup, stream finalization, seal, final snapshot join and host shutdown | Warm | 79.798 | 99.126 (tick carrier) |
| Execution Archive on Disk, 16,384 | Served `Evidence.Archive`/`Files.Read`, write received ZIP, `sync_all`, rewind and drain in 2 MiB chunks | Warm | 20.072 | 170.798 (tick carrier) |

These recorded measurements predate the operation-boundary statistics-kernel
cutover and do not establish performance for the current candidate.

The runtime and desktop fixtures use the real SDK observers and the windowed
Samples protocol: the host grants each interval stream a window of samples and
the plugin answers it with columns. The runtime rows above use that window carrier.
Runtime fixtures declare a 1 ms interval. Desktop fixtures declare an exact
duration setting for their intended three-day density. Each producer sleeps once
for its requested rows; the sleep settles after every window due by its end, so no
final callback wait remains. The empty spectrum is scheduled daily, outside these
short runtime captures; an idle device carrier would prevent virtual-clock
settlement. Desktop rows retain three days of source values in their timestamp
field, including late-source offsets. Live and late commands publish eight real
samples. Each SDK process remains owned until cleanup. No product test hook or
separate storage-admission path exists.

Direct storage ingest measured 1,692,479 rows/s for admission and 1,727,038 rows/s
through durable flush, compared with 127,668 and 120,143 rows/s on 2026-11-15.
These storage API batches are independent of plugin protocol messages.

The current host has no public flush-only operation for an active execution;
runtime publication therefore crosses the ordinary produced-count record barrier,
which flushes accepted samples before committing evidence. It cannot inherit the
old publication-plus-flush figure because it includes that record and its prompt
handoff. Final snapshot joins reread evidence, and combined end/shutdown cannot
inherit shutdown-only timing.
Current archives contain sealed database evidence in `evidence.pb`, published
report evidence and unchanged Arrow files, rather than the retired raw SQLite
database snapshot. The export fixture retains the original 16,384 one-value pulse
rows, produced-count record, 400-point pulse plot and empty frequency plot. The
frequency fixture has no samples; desktop frequency declarations independently
select their bounded input with `input_samples`. Transport and destination
sync/drain are included, so the old export figure is not comparable.

No store format, durability order, budgets or data density changed. The measured
warm three-day documents remain in tens of milliseconds, and full-row decode is
below one second per million samples. Storage validates each newly admitted call
without revalidating the entire pending population. Native benchmark scratch is
parent-owned: after child exit, including termination, the runner removes each
root's credentials through `dokime-host --remove-credentials` before discarding
the root. Cleanup failures retain ownership metadata with the diagnostics.
Existing workstation credentials outside those fresh roots are untouched.
Windows native credential calls share a per-user cross-process file lock, including
story reads: even a concurrent read can race a persistent write to an unrelated
target and resurrect a deleted credential. Callers use
`dokime_runtime::credentials::Entry` instead of accessing keyring directly.
Cleanup checks absence and retains ownership when a key remains. Concurrent-host
stories use separate processes and check native absence from their calling threads
after root removal. External callers that do not take the product lock remain outside
this coordination boundary.
The desktop panic story now checks its exact access key and deliberate panic
type; full-lane failures and post-lane native inventories remain separate proof.

### Interval Window Ingestion

`windows/interval_windows_stored/<clock>/<streams>x<period>ns/<samples>` re-executes
the storage benchmark binary as a remote Rust SDK plugin with 1, 16 or 128 interval
streams of one `double`, at 1 kHz (70,050 samples per stream) and 1 MHz (200,000
samples per stream), each answered by a window observer. Every stream passes the
default 65,536-row flush count, so admission wakes background flush rounds unless
pending bytes reach their 64 MiB bound first, as they do at 128 streams. 1 kHz streams
use the host's default `interval_window` (100 ms), so the 1 kHz capture ends inside a
window and the step's sleep cuts its last window early. 1 MHz streams take a 10 ms
window from the plugin's `sample_window` Duration setting through `window_setting`,
the library bench's 1 MHz condition. Each case runs on the controlled clock and on the
real clock, with in-process hosts on the product host's multi-thread runtime and the
default `interval_lag_windows` (10). The timed boundary runs from releasing the ready
step to its captured prompt: host window grants, the observer's columns, transport,
Harmos batch admission, Dokime's columnar Arrow admission and the produced-count
record, whose evidence barrier makes every sample durable. Execution end and shutdown
are excluded.

The case reports stored samples per second through Criterion throughput and, with
`--once`, a `Measured` line per iteration over five iterations and a `Median` line
naming the clock and window. Figures are never assertions. The controlled clock must
retain exactly the requested samples. The real clock samples from the start of the
execution, including while the ready prompt waits, so it stores at least the
requested span; its figure divides every stored sample by the timed span. Real-time
overload is a measured outcome, not a refusal: every real-clock run, and any
controlled run that loses a sample, prints a `Retained` line with per-stream counts,
the samples the observers answered by the sleep's end, the execution outcome and its
failure evidence.

Measured on 2026-11-16 at `befdc27d8` with library `a14c724f` (Windows x86-64, AMD
Ryzen 9 5950X, bench profile, `--once`, ZSTD level 1 stream batches), as median stored
samples per second over five iterations, summed over all streams:

| Streams | Controlled, 1 kHz, 100 ms Windows | Controlled, 1 MHz, 10 ms Windows | Real, 1 kHz, 100 ms Windows | Real, 1 MHz, 10 ms Windows |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 425,553 | 3,120,169 | 1,000 | 1,064,373 |
| 16 | 433,150 | 9,047,059 | 15,996 | Interrupted 5 of 5 (lag) |
| 128 | 1,214,286 | 9,897,189 | 128,020 | Interrupted 5 of 5 (lag) |

Stream intake has its own lock and never waits for a file sync, a catalog commit or an
evidence barrier. Every controlled case retained exactly its samples, and every
real-clock 1 kHz run and the single 1 MHz stream completed with at least the requested
span. 16 and 128 streams at 1 MHz cannot keep up with real time: each execution failed
fast and ended Interrupted with a stream failure naming the stream and the action, a
stream that fell more than ten window lengths behind (`Lagging`). Their stored rates
(about 6.9 and 13.3 million samples per second) cover only the span before the failure,
so they are not sustained figures.
These real-clock failures are measurements, not tests. The deterministic stories run on
the controlled clock: `window_lag_names_its_stream` proves the `Lagging` refusal with an
observer that waits on host time inside each window, and
`window_overflow_names_its_stream` refuses a window larger than the shared publication
pool. Storage alone retains 128 streams of 20,000 samples
(`interval_batches_of_many_streams_survive_pressure_flushes`).

### Desktop Plot Documents

These Windows x86-64 observations were measured on 2026-11-15 with the
library at `a8cee45c`. The desktop fixture uses a local control connection for
producer handoff and completion. Its observer interval matches each requested
row density over three days, including fractional-second intervals. The SDK
observer publishes every row through the real carrier. The fixture host grants
ten-minute interval windows, so the three-day history takes hundreds of window
round trips instead of one per sample (about 11 s rather than 13 minutes for
2,073,600 rows on that machine), and a controlled sleep still cuts the live
window at its end. The producer has no
operator prompt gate. Drawing deadlines and input counts are unchanged.

The complete matrix was attempted through focused `--once` runs at `a8cee45c`:
33 cases drew every card with data and `desktop_round_trip/100/zoom` refused with
`desktop.plot.0: Waiting for SamplesNo numeric samples are available in the selected window.`
At `8fb9f3f3c`, with library `7181b5ed` and the single-sleep producer, that case
draws every card. The retained 100-plot live/late replay used the earlier gapped
history; its elapsed values are observed redraws, not proof that the new samples
entered the intended time-series window.

At `a8cee45c`, the largest saved history retained all 3,072,000 rows, but the
source-time span was stretched to 390,990.713 s instead of three days.
The requested time-series window selects 1,517,145 of those rows. Both cold
and warm cases for each plot count reopen that same saved history.
Their elapsed values below describe the observed drawing and are not a valid
uniform three-day workload comparison. The 100-plot active history also has
a source-clock gap: its new samples fall outside the intended time window.
Its elapsed value is an observed redraw, not proof of the named active scenario.
The 30-plot active history has verified uniform cadence through all 3,072,008
initial and newly published rows. Every verified card still draws real data.
The retained source-range diagnostics identify this qualification separately
from the drawing refusal.

The fixture source is `d5477640`; remaining-case selection uses `1ffd1347`.
Each case has one observation; these are elapsed measurements, not regression
estimates or timing assertions. Cold and warm measure **document request to
visible drawing** (`timedBoundaryMs`). Active measures **producer handoff to
changed drawing**, without a publication durability barrier. Active delivery
includes normal save visibility; the fixture retains the five-second flush
interval in `crates/dokime-storage/src/telemetry/config.rs`. Monotonic time
origins align the handoff marker with the WebView observation. Startup, history
population, preparation of the initial active view, shutdown and full traversal
are outside these boundaries. Cold reopens the host and desktop before the timer;
OS cache state is uncontrolled. Visible drawing includes two presentation
opportunities and does not claim physical display presentation.

| History Rows | Plots | Cold [ms] | Warm [ms] | Active [ms] |
| ---: | ---: | ---: | ---: | ---: |
| 2,073,600 | 30 | 612.600 | 684.600 | 5,494.990 |
| 2,073,600 | 100 | 741.000 | 834.000 | 5,630.847 |
| 307,200 | 30 | 366.200 | 421.900 | 5,178.669 |
| 307,200 | 100 | 564.300 | 494.000 | 6,288.176 |
| 1,536,000 | 30 | 584.000 | 575.300 | 5,425.121 |
| 1,536,000 | 100 | 651.200 | 597.800 | 6,656.435 |
| 3,072,000 | 30 | 1,206.100 | 1,380.400 | 5,676.352 |
| 3,072,000 | 100 | 1,166.300 | 1,385.600 | 6,043.812 |

The supplied earlier independent observations at 3,072,000 rows are shown
beside the current results below. Visible-group priority is enabled: lookahead
waits for every visible group to deliver data. These runs do not isolate its
effect from the library and fixture changes; there is no matched priority-off
measurement, so the differences cannot be attributed to priority alone.

| Plots, 3,072,000 Rows | Earlier Cold/Warm/Active [ms] | Current Cold/Warm/Active [ms] |
| ---: | --- | --- |
| 30 | about 840 / 332 / 445 | 1,206.100 / 1,380.400 / 5,676.352 |
| 100 | about 780 / 350 / 447 | 1,166.300 / 1,385.600 / 6,043.812 |

The original-history actions measure the named action to changed visible
drawing; Late and Stalled use producer handoff. Stalled includes the intentional
consumer pause and publication burst. Full traversal remains separate.

| Action, 2,073,600 Rows | 30 Plots [ms] | 100 Plots [ms] |
| --- | ---: | ---: |
| Late | 5,610.857 | 5,499.144 |
| Zoom | 591.000 | 651.800 at `8fb9f3f3c` |
| Resize | 181.100 | 270.700 |
| Navigation | 732.100 | 542.600 |
| Stalled | 5,602.884 | 5,458.847 |

Every drawn case checks mounted renderers and actual canvas work, then scrolls
every authored card outside the first-view timer. Empty states and errors fail
with the plot identifier and displayed message. Initial visible drawing does
not imply that offscreen cards have already drawn. The following observations
use the cold case at each density.

| History Rows | Verified Plots | WebView Traversal [ms] | Traversal With Driver [ms] |
| ---: | ---: | ---: | ---: |
| 2,073,600 | 30 | 2,164.300 | 2,170.365 |
| 2,073,600 | 100 | 5,534.300 | 5,545.994 |
| 307,200 | 30 | 1,017.900 | 1,023.457 |
| 307,200 | 100 | 3,963.500 | 3,970.619 |
| 1,536,000 | 30 | 1,824.700 | 1,830.121 |
| 1,536,000 | 100 | 4,573.200 | 4,579.481 |
| 3,072,000 | 30 | 2,454.700 | 2,459.657 |
| 3,072,000 | 100 | 4,284.800 | 4,291.039 |

### Desktop Path Profile

The largest-history cold traversal records inclusive host and frontend spans.
Spans overlap and must not be added together as a CPU profile. Bridge wait
includes host work. Canvas entries measure drawing API calls. The values below
combine visible opening and the subsequent full traversal.

| Stage, 3,072,000 Rows | 30 Plots [ms] | 100 Plots [ms] |
| --- | ---: | ---: |
| Plot Queries, Summed | 9,875.242 | 11,276.268 |
| Schema Construction, Summed | 15.412 | 14.387 |
| Native Adaptation, Summed | 1.800 | 3.600 |
| Protobuf Decode, Summed | 10.200 | 19.100 |
| Retained Updates, Summed | 0.300 | 0.100 |
| React Render, Summed | 213.000 | 664.300 |
| Canvas Calls, Summed | 4.700 | 13.400 |
| Transport and Bridge Wait, Summed | 9,008.900 | 10,173.700 |

`src/plots/live-data.ts` keeps stable, consecutive groups by plot kind, capped
at 32 requests. Visible groups load independently in parallel; lookahead starts
after all visible groups deliver data. New generations cancel old groups and
fence queued frames. Existing plot payloads remain visible during view changes.

Each open reads the authored document through `Documents.Get`, with at most 32
complete frames per page and the existing byte bound. Native plots retain
`GUI_PLOT_REQUESTS`, the 4 MiB frame limit and 32-frame stream credit. Frequency
declarations retain 512 input samples. Category projection keeps native scalar
keys; pie plots select a mean over duplicate categories. Publication durability,
input budgets and deadlines are unchanged.

Raw observations, screenshots, cleanup results and retained refusal
evidence remain under `target/benchmarks/criterion/`; the task repair inventory
also retains the focused-run observations.

## Hosted Validation

`ci:impact` owns the pure path classifier and event planner in
`tools/python/src/dokime_tools/ci/impact.py`. The check and test matrices live in
`ci/matrices.py`; workflow routing uses only the planner's job outputs and matrices.
Markdown files (including nested READMEs), `docs/`, `openspec/` and `site/` select
only `Build / Documentation`, its guide surface check and deterministic local-link
check. Mixed changes union selections. Language changes select their checks and
stories plus downstream consumers and every required artifact producer. C# and
C++ SDK checks and stories retain native Linux and Windows jobs; the C++ host
equivalence story remains in Nightly.

PR comparisons use merge-base to head; pushes use before to after. Complete
NUL-delimited Git diffs include deletions and both rename sides. Unknown inputs,
missing comparison commits, malformed or incomplete diffs select all jobs eligible
for the event. Shared tooling, root locks/configuration, protocols, descriptors and
workflow/action changes fan out across their dependent graph.

Dev pushes select only quick checks and product builds, including Python and
documentation products; they never produce test archives or host fixtures.
Draft and closed reviews plan no validation jobs. Release reviews and manual full
validation run the complete graph. Nightly requests the complete matrix without
a diff and retains its freshness rule. `CI / Required` requires a successful planner,
the complete declared job inventory, success for selected jobs and success or an
explicitly planned skip for other jobs. Failed, cancelled, missing and unexpected
skipped jobs fail Required.

The public task list contains standard operations plus documentation and development
entry points. Necessary selection leaves are hidden; redundant generation aliases
and unused C++ wrappers are removed. SDK tests use `test -- --layer unit` for focused
units, and starter `dev` tasks serve their examples. The expected lane inventory is
`conventions.testing.expected_lanes` in `tools/project.toml`, independent of task names.
`ci:bootstrap` runs declared native prerequisite tasks after mise installation and
before project preparation. The initial pre-mise native bootstrap remains a direct
file load because neither mise nor the installed tools package exists yet.
`ci:send`, `ci:receive`, `ci:sign-select`, `ci:sign-package`, `ci:distribution` and
`ci:summary` own recurring project operations used by provider steps. Tooling-only
planning and Required preparation build no SDK or fixtures; documentation preparation
adds only site dependencies.

Pull requests and Nightly retain native Linux and Windows host/SDK validation.
PR host fixtures select Rust/Python, and a separate PR lane builds the C# fixture
and runs its equivalence story on both platforms. Nightly runs the same core host
lane without depending on C#/C++ fixture builds, and retains their equivalence
stories in a separate lane on both platforms. All local host stories are available by target.
Dev pushes run quick checks/builds; PRs run stories. Full validation belongs to
hosted CI and Nightly after combined source
closure. Local work uses focused proof and never substitutes Windows for Linux.
Nightly publication depends on packages; its final report appends suite outcomes.
Release publication requires separately authorized exact-commit green evidence.

Native test reports remain under `target/test-reports/` and traces under
`target/traces/`. Preserve them before native `cargo clean`. Disk monitoring and
cleanup are workstation responsibilities, never project preflights.
