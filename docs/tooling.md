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

The following instrumented Windows observations compare the original mixed-kind
workload before the repair with code `ae0cfada8`. Each case has one observation;
these are not regression estimates. Before measures elapsed time **to refusal**.
After measures completed visible drawing, with each named action timed as described
below. The after boundary also includes authored document reading on initial opens;
the refused surface constructed its definitions locally. The 100-plot refusal
numbers therefore measure rejection, not a faster successful document.

| Case | Before, 30 Plots [ms] | After, 30 Plots [ms] | Before, 100 Plots [ms] | After, 100 Plots [ms] |
| --- | ---: | ---: | ---: | ---: |
| Cold | 10,740.274 | 652.066 | 162.000 | 1,074.220 |
| Warm | 11,726.138 | 626.387 | 39.945 | 561.679 |
| Live | 10,895.873 | 548.805 | 122.715 | 597.247 |
| Late | 10,540.617 | 439.287 | 132.360 | 410.295 |
| Zoom | 9,981.665 | 406.071 | 83.168 | 390.558 |
| Resize | 9,052.881 | 333.580 | 390.631 | 373.929 |
| Navigation | 14,823.041 | 577.947 | 27.932 | 601.189 |
| Stalled | 13,662.201 | 2,304.425 | 2,117.609 | 2,118.553 |

Before the repair, the 30-plot batch refused with `Process Arrow Evidence: narrow
the frequency source window to its configured input bound; Check the Stream Contract`.
The 100-plot batch refused with `Plots.Watch exceeded the native bounded read
contract: more than 32 plots`. Neither drew its document. All 34 final cases draw;
all eight traversals verify every authored plot, including all mixed kinds.
The history, plot-kind inventory, message/read bounds and drawing guards remain
unchanged. Prompt answers use a fresh request identity per question. Disposal
removes each owned root's credentials without deleting unrelated credentials.

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
kinds. Separate `traversalMs` and `traversalBackendStages` observations retain that
whole-document work without adding it to initial visible drawing. This traversal
cannot silently omit an off-screen refusal. Frequency input
declares 512 samples. Pie authors select an explicit mean over duplicate categories;
all source rows and plot kinds remain present. Grid previews select bounded recent
complete frames instead of attempting to allocate a frame per history timestamp.

The additional histories have no measured pre-repair desktop counterpart. Their
cold and warm boundaries run from document request to visible drawing. Active
measures producer handoff through changed saved selection and visible drawing;
initial view preparation is outside that timer. All values are milliseconds.

| History Rows | Plots | Cold [ms] | Warm [ms] | Active [ms] |
| ---: | ---: | ---: | ---: | ---: |
| 307,200 | 30 | 559.584 | 454.993 | 531.996 |
| 307,200 | 100 | 626.552 | 571.102 | 639.327 |
| 1,536,000 | 30 | 791.797 | 474.126 | 542.689 |
| 1,536,000 | 100 | 815.431 | 528.701 | 594.319 |
| 3,072,000 | 30 | 1,852.583 | 523.471 | 778.682 |
| 3,072,000 | 100 | 1,360.599 | 700.697 | 716.950 |

Whole-document traversal is outside those opening/action timers. It scrolls every
card into view and awaits actual drawing, including each mixed analysis. The
largest individual host `plot.query` span is also reported, rather than treating
one bounded batch or the complete traversal as one plot query. These are elapsed
observations, not timing assertions or a physical-display presentation claim.

| History Rows | Verified Plots | Full Traversal [ms] | Largest Plot Query [ms] | Query per Million Source Rows [ms] |
| ---: | ---: | ---: | ---: | ---: |
| 2,073,600 | 30 | 10,135.906 | 1,507.964 | 727.220 |
| 2,073,600 | 100 | 17,866.350 | 1,128.255 | 544.104 |
| 307,200 | 30 | 2,469.905 | 195.398 | 636.062 |
| 307,200 | 100 | 8,893.971 | 226.675 | 737.875 |
| 1,536,000 | 30 | 7,191.830 | 927.032 | 603.536 |
| 1,536,000 | 100 | 18,090.956 | 1,012.284 | 659.039 |
| 3,072,000 | 30 | 16,030.751 | 2,233.193 | 726.951 |
| 3,072,000 | 100 | 31,640.014 | 2,322.839 | 756.133 |

Desktop mixed analyses share a stream containing the stated number of rows.
The storage document fixture distributes its total row count across 30 streams
and uses summary tiles plus ordinary plot previews. Its tens-of-milliseconds
figure does not include the desktop's mixed full-scan analyses or drawing.

### Desktop Path Profile

The original 30-plot, three-day cold observation is compared with the final cold
observation over the same 2,073,600 source rows. Spans below are inclusive elapsed
time, may overlap cancelled/reopened work and **must not be added** as a CPU
profile. Bridge wait includes host work; React state/render and presentation
opportunities overlap. A dash means no completed data sample before refusal,
not zero cost. The before presentation opportunities draw the error UI only.

| Stage | Before Refusal [ms] | After Drawing [ms] |
| --- | ---: | ---: |
| Desktop Request to Outcome | 10,740.274 | 652.066 |
| Client Call Open | — | 77.000 |
| Documents.Get Host Operation | — | 17.869 |
| Plots Host Operation | 10,679.317 | 301.028 |
| Plot Query, Nested | 10,615.268 | 269.727 |
| Store Exact Statistics, Nested | 10,100.403 | 267.892 |
| Store Saved Preview, Nested | 14.160 | 1.804 |
| Frozen Schema Construction, Nested | — | 32.582 |
| Plot Reply Encoding | — | 0.321 |
| Native Transport and Bridge Wait | — | 273.900 |
| Protobuf Decode | — | 6.600 |
| Native Frame Adaptation | — | 2.000 |
| Retained Update | — | 0.200 |
| Frontend State and React Render | — | 117.500 |
| Canvas Draw Calls | — | 2.100 |
| Presentation Opportunities | 126.600 | 236.000 |

The three root costs were repeated exact-statistics queries for identical cards,
full-history moment scans when the saved preview already covered the complete
window, and per-plot frozen descriptor construction:

- `crates/dokime-operations-next/src/client/plots_snapshots_list.rs::query_batch`
  now shares an identical selection within one bounded batch, with one isolated
  reader and catalog/pool per execution. Before there were 22 statistics calls in
  the refused opening; after there are two. This sharing ends with the batch.
- `crates/dokime-storage/src/plot_windows.rs::numeric_statistics` now uses durable
  moments when preview bounds prove full coverage; requested ranks remain exact.
  These first two causes share the measured statistics span; their individual
  times were not isolated. Statistics fall from 10,100.403 to 267.892 ms in this
  observation.
- `crates/dokime-operations-next/src/client/plots_snapshots_list.rs::snapshot_at`
  previously rebuilt the descriptor pool for each plot. Batch preparation replaces
  those rebuilds. The before non-statistics/non-preview query residual is 500.705 ms,
  including descriptor work; it is an upper bound, not an isolated pool timing.
  Final schema spans total 32.582 ms. Unrelated accepted/runtime notices no longer
  repeat saved plot queries in `plots_snapshots_watch.rs::handle`.

`crates/dokime-storage/src/plot_kinds.rs::grid_window` also retains typed frame
keys through reduction and formats only returned instants, instead of allocating
a timestamp string per source row. The largest measured individual queries in
all traversals remain below one second per million source rows in this run.
The benchmark-only Vite transform records client-open/bridge waits, protobuf
decode, native adaptation, retained updates, React rendering and canvas calls.
Its profiling React build is confined to the benchmark bundle.

The 32-plot declaration remains `GUI_PLOT_REQUESTS` in
`crates/dokime-gui/src/lib/api/native-plots.ts`; it bounds one desktop read/watch's
admitted work and retained payload family. It is separate from the unchanged
4 MiB native frame limit and 32-frame stream credit. `useLivePlotPayloads` selects
visible cards and opens as many independent groups of at most 32 as needed.
Document reads request at most 32 complete frames per page as well as the byte
bound, preserving counters, continuation and fixed-cut validation. Packed aligned
time columns avoid repeated recursive coordinate values; trajectory/category
projection retains source instants and category scalars for its renderer.

A frequency declaration's `input_samples` selects the latest N finite samples by
source time and durable position within its filters/window; absent N uses the
host's configured transform-input bound. The benchmark author declares 512.
Input taper and display resolution stay independent. Invalid explicit oversize
input still refuses; long history alone never makes a frequency plot unrenderable.

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
