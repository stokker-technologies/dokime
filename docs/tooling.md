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
mise -C sdk/python run sdk:test
mise -C sdk/csharp run test
mise -C sdk/cpp run test
mise run build -- --artifact tests
```

Host fixture production builds the Rust reference, pinned library conformance host,
content fixture, product host and pure Python wheel;
C# and C++ producers use their SDK tasks. `build:host-fixtures --language
rust|python|csharp|cpp` selects one producer. C++ uses `target/cpp/` with short
generated and object paths on Windows. Its CMake 4.2 pin supports Visual Studio 18
and selects the installed Visual Studio generator. Cached dependency builds resolve code
generators through the same Protobuf/gRPC package targets as uncached builds.
Fixtures are exported under
`target/build/<profile>/<target>/fixtures/host/`. CLI fixtures include a generated
Rust starter built through its own mise task. Stories never compile plugins.

`HARMOS_SOURCE`, `DOKIME_HARMOS_CACHE` and `DOKIME_CPP_BUILD_DIR` may select
workstation source/dependency caches; do not put them in project configuration.
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
in milliseconds.
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
| Median and Plot, 1,000,000 | Five summary tiles, exact median and one plot payload | Warm / cold | 286.488 / 204.213 | 356.91–367.41 warm; no comparable old cold figure |
| Three-Day Document, 307,200 | Five tiles and 30 plot payloads/presentation | Warm / cold | 24.073 / 27.393 | No comparable old figure |
| Three-Day Document, 1,536,000 | Same document at original density | Warm / cold | 22.508 / 25.160 | No comparable old figure |
| Three-Day Document, 3,072,000 | Same document at original density | Warm / cold | 23.894 / 25.349 | 39.359–43.803 warm; no comparable old cold figure |
| Document Full Decode, 3,072,000 | All fields of all 30 streams, consuming decoded buffers; no previews/rendering | Warm | 576.543 | No comparable old figure |
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
rows, produced-count record, 400-point pulse plot and empty frequency plot; the
current Plot contract has no selectable FFT-window field. Transport and destination
sync/drain are included, so the old export figure is not comparable.

No store format, durability order, budgets or data density changed. The measured
warm three-day documents remain in tens of milliseconds, and full-row decode is
below one second per million samples. The host's per-response batch admission
avoids revalidating the entire pending population once per individual sample.

The following desktop observations precede the desktop plot-document repair.
The workload contains its original mixed plot kinds. These are
elapsed observations **to refusal**, not successful drawing latencies. Cold opens
a fresh host and desktop before the timer; warm first exercises the same view;
the other cases use the active host. There is no comparable old figure for these
refused cases. The retained visible failure identifies the frequency refusal
quoted below for the 30-plot observations.

| Case | Timed Boundary | State | 30 Plots, ms | 100 Plots, ms | Comparable Old Figure |
| --- | --- | --- | ---: | ---: | --- |
| Cold | Request through refusal; host/desktop reopen excluded | Cold process, OS cache uncontrolled | 11,871.144 | 86.450 | No comparable old figure |
| Warm | Request through refusal after warm-up | Warm | 10,164.197 | 18.081 | No comparable old figure |
| Live | Initial request; 100-plot case also answers producer prompt | Active | 11,540.565 | 76.444 | No comparable old figure |
| Late | Initial request; 100-plot case also answers late-source prompt | Active | 12,199.464 | 52.052 | No comparable old figure |
| Zoom | Initial request; zoom blocked by refusal | Active | 12,189.889 | 22.192 | No comparable old figure |
| Resize | Request and native resize through refusal | Active | 12,820.390 | 336.244 | No comparable old figure |
| Navigation | Initial request; navigation blocked by refusal | Active | 10,911.475 | 20.215 | No comparable old figure |
| Stalled | Initial request; 100-plot case also answers 13 producer prompts | Active | 11,421.673 | 1,856.855 | No comparable old figure |

The 30-plot batch refuses with `Process Arrow Evidence: narrow the frequency source
window to its configured input bound; Check the Stream Contract`. The 100-plot
batch refuses with `Plots.Watch exceeded the native bounded read contract: more
than 32 plots`. The first refusal prevents live/late/zoom/navigation/stall actions
in the 30-plot cases; neither workload measures successful plot delivery. These
cases retain their original history, kinds and bounds, rather than reducing the
dataset or raising a budget. The benchmark records a visible failure even while
the frontend marks it reconnecting, so refusal does not wait for a drawing hang
guard. Prompt answers use a fresh request identity per question. Cleanup preserves
copied history credential references until final disposal while always removing
each disposable shell's independent credentials.

The repair retains the original 2,073,600-row, three-day, 8 Hz workload and its
16 cases. Additional desktop history cases use 307,200, 1,536,000 and 3,072,000
rows across the same three-day span, for both 30 and 100 plots, cold, warm and
active. Their sampling intervals are stated by the fixture; they are additional
case identities, not replacements for the original density. Every desktop open
now reads an authored document through `Documents.Get`, joins bounded frame pages,
requests visible plot groups through the native SDK, decodes their previews and
observes actual chart drawing plus two presentation opportunities. The original
refused surface constructed its declarations locally, so the new boundary also
includes document reading. Neither figure is a claim about physical display
presentation. Cold host/desktop startup and fixture population stay outside the
timer; saved-state queries use a fresh process without evicting the OS cache.

Live, late and stalled cases first draw the initial view outside the timer, then
time the producer prompt handoff through visibility of a changed saved selection.
Unrelated runtime notifications cannot complete that measurement. The existing
flush policy is unchanged; time awaiting saved visibility is distinct from query
work. Zoom, resize and navigation time their named action through visible drawing.
After navigation, and the additional history cases' cold opens, the driver visits
every plot outside the timer and verifies actual drawing, including all mixed
kinds. This traversal cannot silently omit an off-screen refusal. Frequency input
declares 512 samples. Pie authors select an explicit mean over duplicate categories;
all source rows and plot kinds remain present. Grid previews select bounded recent
complete frames instead of attempting to allocate a frame per history timestamp.

The instrumented original 30-plot cold observation on this machine was
10,740.274 ms to refusal. Inclusive host spans total 10,679.317 ms; their nested
plot queries total 10,615.268 ms, exact-statistics work 10,100.403 ms and saved
preview reads 14.160 ms. Other host work is 564.754 ms by subtraction, including
repeated frozen descriptor construction; the outer residual is 60.957 ms.
Encoding, decoding, retained data updates and drawing have no completed data
sample in that refused observation. Span totals may overlap cancelled/reopened
queries and are not an additive CPU profile. The benchmark-only Vite transform
records client-open/bridge waits, protobuf decode, native adaptation, retained
updates, React rendering and canvas calls. Its profiling React build is confined
to the benchmark bundle; shipping components and transport policy are unchanged.

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
