# Repository Tooling

Use `mise tasks` to discover entry points, with arguments after `--`. Preparation
installs the root `.venv/` and frontend dependencies. `prepare --without-frontend`
prepares SDK-only jobs without requiring Node or pnpm. Build, test and checks use
project tasks; compiler caches belong to the workstation.

Preparation synchronizes the `sdk-tooling` dependency group as part of
`--all-groups`: authored SDK build hooks and generators are type-checked with
their Hatchling, gRPC generator and mypy dependencies and pinned gRPC stubs.
Frontend generation resolves locked Cargo metadata online because a native
build does not populate dependencies for every platform in that metadata.
Product Build generates preferences, both TypeScript message roots and icons
before type checking, then builds the live and embedded report assets.

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
mise run generate:typescript
mise run check:generated
mise run check:locks
mise -C sdk/csharp run restore
```

`generate:typescript` regenerates schemas, dispatch and declared preference types
without compiling runtime-owned default values. The ordinary `generate` task still
exports those defaults from their product owner before a full frontend build.

The generated checks compile permanent consumers under each SDK's tests/generation
(tests/Generation for C#) and the desktop tests/generation. Select one with
`check:generated --language rust|python|csharp|cpp|typescript`, or use its SDK's
`test:generation`. These compiler contracts require no product runtime or host fixtures.
C# uses its own `target/cs/generation` output; its normal SDK build also compiles
the authored file-transfer consumer. Generated Python obeys
ordinary lint and typing; only protoc's private `_pb2*` files use the existing exclusion,
and the public barrels plus compiler consumer are checked explicitly.

`prepare:locks` refreshes Cargo and uv locks after a dependency pin change. C# uses
`restore --refresh-lock` explicitly, then ordinary `restore` and the generation story
use locked mode. C++ records its source archive digest and vcpkg baseline as inputs.
The repository check includes buf, callers, public surfaces and workflow lint. Its
wire-name allow-list covers private generated directories, canonical
gRPC paths, architecture and execution-host definition maps, and Markdown link targets.

## Fixtures and Stories

The Rust host-fixture producer stages the reference host and its Python
conformance package from the same locked library checkout. Python system and
coverage lanes consume that stamped package, without a machine-specific source
path. The SDK entry point selects the matching prepared profile with
`mise -C sdk/python run conformance -- --profile release`.

```sh
mise run build:host-fixtures
mise run build:cli-fixtures
mise run test -- --package dokime-runtime --target system
mise run test -- --package dokime-cli --target system
mise -C sdk/rust run sdk:test
mise -C sdk/python run test
mise -C sdk/csharp run test
mise -C sdk/cpp run test
mise run build -- --artifact tests
```

Host fixture production builds the Rust reference, pinned library conformance host,
content fixture, product host and pure Python wheel;
C# and C++ producers use their SDK tasks. `build:host-fixtures --language
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
Python consumes the built pure wheel and the pinned library SDK. Python system
and coverage tasks select the conformance executable from the host fixtures.
Linux host stories own an isolated session bus/keyring for credential storage;
install `dbus`, `gnome-keyring` and `libglib2.0-bin`. Native unit tests run without
a host; connection journeys belong to integration and thermal execution to system.
Windows uses its operating system credential store and removes
every disposable root's protected slots through the product removal operation
before deleting that root, including when a story unwinds.

## Desktop Integration

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
host and SDK thermal fixture. Frontend unit and integration selection includes all
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

The producer selects the runtime's `storage` benchmark in Cargo's bench profile.
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
integer-exact statistics, retained file readers and unchanged archive stream bytes.
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
at 1,500 points; the median case adds one exact-median tile and has one plot.
Neither includes HTML rendering or report-file writes. The former extra standard
deviation tile is removed. Full-decode diagnostics consume all fields of every row;
the LTTB window case uses the product query that must decode all selected rows.

The following Windows x86-64 observations were measured on this workstation
(AMD Ryzen 9 5950X) on 2026-11-15 with default storage settings and one iteration
per case in the bench profile, in milliseconds. The 18 storage/document cases
completed; the runtime figures below identify the earlier carrier separately.
The current SDK observer retains the exact-count refusal rather than publishing
an invalid timing. Native boundaries and observations are retained under
`target/benchmarks/storage/`.
Old ranges are the recorded cards-and-plots measurements at `43875c422` in
`docs/product-guide.md`; only the matching warm summary/plot workloads share them.
They are historical multi-sample ranges, not statistical comparisons with one run.

| Case | Timed Boundary | State | This Machine, ms | Comparable Old Figure, ms |
| --- | --- | --- | ---: | --- |
| Store Admission and Flush, 1,024 | Prepared protobuf batch, direct admission and durable flush; first file creation when absent | Open store | 10.820 | No comparable old figure |
| Window Preview, 1,000,000 | Saved preview bins, summary and bounded points | Warm / cold | 0.646 / 2.196 | No comparable old figure |
| Window Full Scan LTTB, 1,000,000 | Preview summary, full selected-row decode and 1,500-point LTTB payload | Warm / cold | 172.606 / 179.631 | No comparable old figure |
| Stream Finalize, 65,536 | Close stream files and join sample workers; durable | Warm | 1.224 | No comparable old figure |
| Minimal Archive in Memory, 16,384 | Current archive into a `Vec`; no report or output sync | Warm | 5.564 | No comparable old figure |
| Store Ingest Admission, 1,000,000 | Generate/encode phase, ordinal and value; admit in 4,096-row storage batches; automatic flushes included, last tail not necessarily durable | Fresh empty store | 7,623.027 | 8,167.407 on 2026-11-14 |
| Store Ingest and Flush, 1,000,000 | Same generation/admission plus explicit durable flush; shutdown excluded | Fresh empty store | 7,732.326 | 9,182.036 on 2026-11-14 |
| Median and Plot, 1,000,000 | Five summary tiles, exact median and one plot payload | Warm / cold | 126.300 / 132.884 | 356.91–367.41 warm; no comparable old cold figure |
| Three-Day Document, 307,200 | Five tiles and 30 plot payloads/presentation | Warm / cold | 25.408 / 27.227 | No comparable old figure |
| Three-Day Document, 1,536,000 | Same document at original density | Warm / cold | 24.142 / 26.645 | No comparable old figure |
| Three-Day Document, 3,072,000 | Same document at original density | Warm / cold | 26.693 / 29.164 | 39.359–43.803 warm; no comparable old cold figure |
| Document Full Decode, 3,072,000 | All fields of all 30 streams, consuming decoded buffers; no previews/rendering | Warm | 526.706 | No comparable old figure |
| Runtime Observer Publication and Evidence Barrier, 1,024 | Release prepared step, SDK observation/transport/admission, produced-count record and captured prompt; end/shutdown excluded | Warm host, fresh execution | Not Measured: 1,025 Rows Retained for 1,024 Requested | Earlier single-row carrier: 347.964; retired batched carrier was invalid |
| Execution End and Host Shutdown, 65,536 | Release final prompt, cleanup, stream finalization, seal, final snapshot join and host shutdown | Warm | Not Measured With the SDK Observer | Earlier single-row carrier: 174.858 |
| Execution Archive on Disk, 16,384 | Served `Evidence.Archive`/`Files.Read`, write received ZIP, `sync_all`, rewind and drain in 2 MiB chunks | Warm | Not Measured With the SDK Observer | Earlier single-row carrier: 27.696 |

The runtime and desktop fixtures use the real SDK observer and its tick-driven
Samples protocol. Each tick receives exactly one sample with that tick's
timestamp and the next sequence. The former proxy carrier is removed.
Runtime fixtures declare a 1 ms interval. Desktop fixtures declare an exact
duration setting for their intended three-day density. The upstream controlled
sleep-registration race can stretch canonical source time and leave an intended
zoom window empty; the desktop observations below identify those cases.
The empty spectrum is scheduled daily, outside these short runtime captures;
an idle device carrier would prevent virtual-clock
settlement. Desktop rows retain three days of source values in their timestamp
field, including late-source offsets. Live and late commands publish eight real
ticks and samples. Each SDK process remains owned until cleanup. No
product test hook or separate storage-admission path exists.

Direct storage ingest measured 131,181 rows/s for admission and 129,327 rows/s
through durable flush, compared with 122,438 and 108,908 rows/s in the previous
guide. These storage API batches are independent of plugin protocol messages.

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
Native credential calls run on one dedicated process-lifetime owner thread.
Cleanup checks absence and retains ownership when a key remains. Concurrent-host
stories also check native absence from the calling threads after root removal.
The desktop panic story now checks its exact access key and deliberate panic
type; full-lane failures and post-lane native inventories remain separate proof.

### Desktop Plot Documents

These Windows x86-64 observations were measured on 2026-11-15 with the final
library at `a8cee45c`. The desktop fixture uses a local control connection for
producer handoff and completion. Its observer interval matches each requested
row density over three days, including fractional-second intervals. The SDK
observer publishes every row through the real carrier. The producer has no
operator prompt gate. Drawing deadlines and input counts are unchanged.

The complete matrix was attempted through focused `--once` runs: 33 cases drew
every card with data. `desktop_round_trip/100/zoom` refused with
`desktop.plot.0: Waiting for SamplesNo numeric samples are available in the selected window.`
The retained Arrow data confirms a controlled-clock jump from
1,767,319,077.625 s to 1,767,484,800 s between consecutive samples at 8 Hz.
That jump crosses the entire requested zoom window. This is the upstream
sleep-registration race; it can stretch the configured three-day source span.
The fixture retains the failure with the original window, row count and producer
handoff. The 30-plot zoom completed.

The largest saved history retains all 3,072,000 rows, but the upstream clock
race stretches its source-time span to 390,990.713 s instead of three days.
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
| Zoom | 591.000 | Upstream Refusal |
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

Raw observations, screenshots, cleanup results and retained upstream-refusal
evidence remain under `target/benchmarks/criterion/`; the task repair inventory
also retains the focused-run observations.

## Hosted Validation

Pull requests and Nightly retain native Linux and Windows host/SDK validation.
PR host fixtures select Rust/Python; Nightly runs the same core host lane without
depending on C#/C++ fixture builds, and retains their equivalence stories in a
separate lane on both platforms. All local host stories are available by target.
Dev pushes run quick checks/builds; PRs run stories. Full validation belongs to
hosted CI and Nightly after combined source
closure. Local work uses focused proof and never substitutes Windows for Linux.
Nightly publication depends on packages; its final report appends suite outcomes.
Release publication requires separately authorized exact-commit green evidence.

Native test reports remain under `target/test-reports/` and traces under
`target/traces/`. Preserve them before native `cargo clean`. Disk monitoring and
cleanup are workstation responsibilities, never project preflights.
