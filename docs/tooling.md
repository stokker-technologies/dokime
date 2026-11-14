# Repository Tooling

Use `mise tasks` to discover entry points, with arguments after `--`. Preparation
installs the root `.venv/` and frontend dependencies. `prepare --without-frontend`
prepares SDK-only jobs without requiring Node or pnpm. Build, test and checks use
project tasks; compiler caches belong to the workstation.

## Focused Development

```sh
mise run prepare
mise run build -- --package dokime-runtime-next --check
mise run build -- --cargo check --locked --offline --package dokime-operations-next
mise run test -- --package dokime-runtime-next --target system --story storage_root_admission
mise run test -- --package dokime-protocol --target integration
mise run test -- --changed crates/dokime-runtime-next/tests/system/manager_lifecycles.rs --plan
mise run format
mise run check -- --language rust
mise run check -- --language python
mise run check -- --language typescript
mise run check -- --repository
```

Focused tests use nextest and compile only the selected package/target. `--exact`
selects a test, `--story` selects a module, and `--filterset` takes a nextest expression.
`--artifact tests` consumes the matching prepared archive without compilation.
An empty selection fails. Clippy runs through lint/check, never build/test.
After source closure, `mise run build -- --cargo check --workspace --tests --locked`
runs the explicit workspace compiler proof through the same task owner.

## Fixtures and Stories

```sh
mise run build:host-fixtures
mise run build:cli-fixtures
mise run test -- --package dokime-runtime-next --target system
mise run test -- --package dokime-cli-next --target system
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
Rust starter built through its own mise task. Stories never compile plugins.

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
workspace also contains protocol, storage, operations-next, runtime-next, CLI-next,
SDK and bundle inspection packages.

Product outputs live under `target/build/<profile>/<target>/`; packages under
`target/package/<profile>/<target>/`. CLI archives retain the `dokime-next` binary.
The distribution ZIP contains that CLI, the desktop, authored organization resources
and legal notices. Native Tauri packaging consumes those verified Product bytes
without compiling or patching them and produces Windows NSIS or Linux DEB/RPM
installers. Stable publication signs the validated Windows Product, refreshes its
distribution inventory and packages it before generating updater signatures;
Nightly stays unsigned. Shipping sidecar feeds await their re-authoring tasks.
The pure Python wheel/source pair uses the shared `any/python/` target.

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
On Linux its consumer owns the same isolated Secret Service as native host
fixtures, so the runtime archive case needs no unlocked personal keyring.

Each case prints its timed boundary. `--once` runs one measured iteration per case
in the bench profile and prints an observation without a statistical estimate or
baseline promotion. Omit it for the unchanged Criterion sampling configuration.
Case identities distinguish preview reads, full scans, store admission and runtime
publication; their baselines must never be interchanged. Benches record durations
and refusals; stories assert deterministic behavior, never timing or sampled load.

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
(AMD Ryzen 9 5950X) on 2026-11-14 with default storage settings and one iteration,
in milliseconds. The nine document/median/full-decode observations were refreshed
at `ae0cfada8`; other rows retain their earlier measurements at the storage-boundary
checkpoint because their separately named workloads were not rerun here.
Old ranges are the recorded cards-and-plots measurements at `43875c422` in
`docs/product-guide.md`; only the matching warm summary/plot workloads share them.
They are historical multi-sample ranges, not statistical comparisons with one run.

| Case | Timed Boundary | State | This Machine, ms | Comparable Old Figure, ms |
| --- | --- | --- | ---: | --- |
| Store Admission and Flush, 1,024 | Prepared protobuf batch, direct admission and durable flush; first file creation when absent | Open store | 11.556 | No comparable old figure |
| Window Preview, 1,000,000 | Saved preview bins, summary and bounded points | Warm / cold | 0.407 / 1.800 | No comparable old figure |
| Window Full Scan LTTB, 1,000,000 | Preview summary, full selected-row decode and 1,500-point LTTB payload | Warm / cold | 167.598 / 182.450 | No comparable old figure |
| Stream Finalize, 65,536 | Close stream files and join sample workers; durable | Warm | 1.285 | No comparable old figure |
| Minimal Archive in Memory, 16,384 | Current archive into a `Vec`; no report or output sync | Warm | 6.125 | No comparable old figure |
| Store Ingest Admission, 1,000,000 | Generate/encode phase, ordinal and value; admit in 4,096-row batches; automatic flushes included, last tail not necessarily durable | Fresh empty store | 8,167.407 | No comparable old figure |
| Store Ingest and Flush, 1,000,000 | Same generation/admission plus explicit durable flush; shutdown excluded | Fresh empty store | 9,182.036 | No comparable old figure |
| Median and Plot, 1,000,000 | Five summary tiles, exact median and one plot payload | Warm / cold | 142.496 / 140.810 | 356.91–367.41 warm; no comparable old cold figure |
| Three-Day Document, 307,200 | Five tiles and 30 plot payloads/presentation | Warm / cold | 21.405 / 22.632 | No comparable old figure |
| Three-Day Document, 1,536,000 | Same document at original density | Warm / cold | 22.719 / 24.451 | No comparable old figure |
| Three-Day Document, 3,072,000 | Same document at original density | Warm / cold | 25.590 / 27.169 | 39.359–43.803 warm; no comparable old cold figure |
| Document Full Decode, 3,072,000 | All fields of all 30 streams, consuming decoded buffers; no previews/rendering | Warm | 727.202 | No comparable old figure |
| Runtime Observer Publication and Evidence Barrier, 1,024 | Release prepared step, sample transport/admission, produced-count record barrier and captured prompt; end/shutdown excluded | Warm host, fresh execution | 63.335 | No comparable old figure |
| Execution End and Host Shutdown, 65,536 | Release final prompt, cleanup, stream finalization, seal, final snapshot join and host shutdown | Warm | 80.225 | No comparable old figure |
| Execution Archive on Disk, 16,384 | Served `Evidence.Archive`/`Files.Read`, write received ZIP, `sync_all`, rewind and drain in 2 MiB chunks | Warm | 16.747 | No comparable old figure |

The runtime fixture uses a benchmark-owned producer speaking the real tick-driven
Samples protocol, forwarding ordinary invocations to the SDK plugin. Responses
carry 1,024 samples for runtime workloads and up to 4,096 for desktop history.
The host uses its controlled clock, validates sequences/source timestamps and
admits each response through its ordinary batch writer. Desktop history retains
2,073,600 four-field rows over three days at 8 Hz. Live and late commands publish
eight real samples; late source times are carried on Samples, not simulated by a
display-only timestamp. Both producer processes remain owned until cleanup. No
product test hook or separate storage-admission path exists.

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
below one second per million samples. The host's per-response batch admission
avoids revalidating the entire pending population once per individual sample.

### Desktop Plot Documents

These Windows x86-64 observations come from one fresh `--once` run on
2026-11-14, run from `c61d90897` with the exact-moment implementation at
`9b6f8191d`. Each case has one observation; these are elapsed measurements,
not regression estimates or timing assertions. The history and plot-kind
inventories, admission bounds and drawing deadlines are unchanged.

Cold and warm measure **document request to visible drawing** (`timedBoundaryMs`).
Active measures **producer handoff to changed drawing**; its marker is taken
immediately before writing the producer command. Monotonic time origins align
that marker with the WebView drawing observation, excluding the UI-reset driver
round trip. Startup, population, initial active-view preparation, shutdown and
full traversal are outside these boundaries. `durationMs` retains the broader
driver round trip for diagnostics and is not the value in this table.
Visible drawing includes two presentation opportunities, without claiming
physical display presentation. Warm reads use an already queried store.
The original 2,073,600-row, three-day history runs at 8 Hz; additional histories
retain their real row counts across the same span. Live is labelled Active here.

| History Rows | Plots | Cold [ms] | Warm [ms] | Active [ms] |
| ---: | ---: | ---: | ---: | ---: |
| 2,073,600 | 30 | 948.300 | 780.200 | 549.375 |
| 2,073,600 | 100 | 869.700 | 726.200 | 642.516 |
| 307,200 | 30 | 586.800 | 485.300 | 420.246 |
| 307,200 | 100 | 589.700 | 554.300 | 393.479 |
| 1,536,000 | 30 | 756.900 | 780.300 | 589.915 |
| 1,536,000 | 100 | 906.100 | 766.300 | 664.649 |
| 3,072,000 | 30 | 1,134.300 | 790.000 | 970.472 |
| 3,072,000 | 100 | 1,607.200 | 1,171.100 | 838.485 |

The remaining original-history actions use their named action to changed visible
drawing; Late and Stalled use producer handoff. Stalled includes the existing
intentional consumer pause and publication burst. Full traversal remains separate.

| Action, 2,073,600 Rows | 30 Plots [ms] | 100 Plots [ms] |
| --- | ---: | ---: |
| Late | 658.006 | 837.778 |
| Zoom | 554.700 | 574.300 |
| Resize | 149.000 | 163.700 |
| Navigation | 706.500 | 790.000 |
| Stalled | 1,850.560 | 2,109.001 |

Every case is marked drawn only after traversal verifies every authored card.
The bench checks the mounted renderer and actual canvas work; an error or empty
state fails with the plot identifier and displayed message. All 34 cases passed
this verification, including `desktop.plot.27` and `desktop.plot.97`. Initial
visible drawing is not a claim that all offscreen cards have already drawn.
Traversal scrolls every card inside the WebView, awaits drawing and records one
driver round trip separately. The table uses the cold case for each density.

| History Rows | Verified Plots | WebView Traversal [ms] | Traversal With Driver [ms] | Completed Group Requests |
| ---: | ---: | ---: | ---: | ---: |
| 2,073,600 | 30 | 2,795.200 | 2,802.368 | 13 |
| 2,073,600 | 100 | 5,327.000 | 5,334.955 | 10 |
| 307,200 | 30 | 1,054.800 | 1,064.806 | 9 |
| 307,200 | 100 | 4,528.300 | 4,538.013 | 11 |
| 1,536,000 | 30 | 2,362.400 | 2,371.362 | 13 |
| 1,536,000 | 100 | 5,131.400 | 5,140.389 | 11 |
| 3,072,000 | 30 | 3,607.700 | 3,613.852 | 13 |
| 3,072,000 | 100 | 6,352.100 | 6,366.564 | 11 |

The supplied independent verification reported 9,649.706/18,054.688 ms for
30/100 plots over **3,072,000 rows**, with 13/55 completed group requests during
traversal. Its card verification missed pie errors. The corresponding verified
final driver-inclusive traversals are 3,613.852/6,366.564 ms.
The stronger card check and different driver orchestration must be considered
when comparing these individual observations.

### Desktop Path Profile

The largest-history cold traversal records group latency, host work, adaptation,
React work and drawing. Spans are inclusive elapsed time and overlap; their sums
are not a CPU profile and must not be added together. Bridge wait includes host
work. Canvas entries count drawing API calls, rather than complete card redraws.

| Stage, 3,072,000 Rows | 30 Plots [ms] | 100 Plots [ms] |
| --- | ---: | ---: |
| Median Group Latency | 1,195.820 | 1,276.307 |
| Largest Group Latency | 2,284.451 | 2,471.978 |
| Plot Queries, Summed | 16,318.811 | 14,814.844 |
| Schema Construction, Summed | 289.193 | 297.195 |
| Native Adaptation, Summed | 1.300 | 2.600 |
| Protobuf Decode, Summed | 2.400 | 8.500 |
| Retained Updates, Summed | 0.100 | 0.100 |
| React Render, Summed | 152.700 | 890.700 |
| Canvas Calls, Summed | 2.600 | 15.000 |
| Transport and Bridge Wait, Summed | 13,099.000 | 15,063.800 |

`src/plots/live-data.ts` keeps stable, consecutive groups by plot kind, capped
at 32 requests. Visible groups load independently in parallel; lookahead starts
after visible data arrives. Arriving groups retain unchanged plot-state identities,
and already mounted cards retain their renderer. New generations cancel old groups
and fence queued frames. The 100-plot traversal removes repeated group reopening;
its completed request count falls from 55 to 11.
`crates/dokime-storage/src/telemetry/aggregates.rs::moments` owns the full-coverage
proof and reuses durable exact moments for statistics, previews and RMS. Clipped
windows retain their selected rows. This avoids rescanning history merely to
replace floating preview statistics with typed values. Rank selection remains exact.

Each open reads the authored document through `Documents.Get`, with at most 32
complete frames per page and the existing byte bound. Native plot requests retain
`GUI_PLOT_REQUESTS`, the 4 MiB frame limit and 32-frame stream credit. A frequency
declaration selects its latest finite input samples by source time and durable
position; this benchmark still declares 512. Analysis receives its captured raw
samples and aligned time axis. Category projection retains native scalar keys;
pie authors still select a mean over duplicate categories. No data density,
publication durability, input budget or deadline was changed.

Raw observations and the cleanup result remain in the desktop benchmark output
under `target/benchmarks/criterion/`. `completion.json` records 34 cases, no
failure, no cleanup errors and no retained fixture directories.

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
