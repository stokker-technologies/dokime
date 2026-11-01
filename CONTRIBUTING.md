# Contributing

Help improve Dokime through focused fixes, useful equipment integrations and
clearer examples. Discuss substantial behavior or architecture changes before
implementing them.

## Development Setup

Install [mise](https://mise.jdx.dev) and prepare the checkout:

```sh
mise trust
mise install --locked
mise run prepare
mise run dev
```

Windows and Linux are supported. Native Windows GUI tests require an interactive
desktop and matching WebView2/EdgeDriver; `prepare:desktop` installs the driver.
Linux desktop tests use WebKitWebDriver and a display such as Xvfb. Their
desktop bootstrap reuses installed Chrome or provisions the pinned Chrome For
Testing build for real report rendering and owned-browser cleanup.

Native media capture also needs platform libraries. On Debian/Ubuntu, install
`libasound2-dev`, `linux-libc-dev` and `libclang-dev`; Fedora uses
`alsa-lib-devel`, `kernel-headers` and `clang`.

## Development Workflow

Keep a change focused and respect SDK, runtime, server, GUI and generated-client
boundaries. Update affected examples when changing their API; flag stale examples
rather than silently deleting or simplifying them.

Use Conventional Commits, such as `fix(runtime): preserve stopped execution evidence`.
Keep generated outputs with their source. Refresh embedded reports with
`mise run prepare:report` and organization assets with `mise run prepare:organization`.
The report and TypeScript checks/tests depend on `prepare:generated`, which builds the
contract generator and server export with the dev compiler profile before generating their
modules. Product builds compile against the committed report and generate the frontend's
modules from their own `cargo-dokime`, so `mise run build:typescript` follows
`mise run build:rust`. Use `mise run prepare:generated -- --write` to refresh tracked Python
and TypeScript clients; ordinary preparation checks them. CI lanes restore the matching
Product modules and skip this producer instead of compiling again.
Organization defaults generate JSON/theme under
`crates/dokime-gui/src/generated/organization/`; keep authored pack inputs separate.
Build exports the matching native installer payload. `mise run package:rust`
uses the built authoring tool to assemble and inspect it without recompiling;
see [Native Installers](docs/product-guide.md#native-installers) for downstream
organization distributions and signing prerequisites.

For coupled Harmos development, keep registry dependencies declared and place the
local Cargo patch in an untracked `.cargo/config.toml`. Cargo applies it to every
build, including the binding generator in `cargo-dokime`; remove the patch and pin
the published dependency before delivery.

Development and test builds use line tables for workspace crates and disable
debug information for dependencies to reduce compilation and linking costs.
Workspace backtraces retain source locations; type and variable inspection
require rebuilding the affected crate with full debug information.
Debug assertions and overflow checks remain enabled.

## Tests And Checks

Build matching artifacts, then run every test lane concurrently (the five-minute
execution target is advisory and excludes compilation):

```sh
mise run build
mise run build:rust:tests
mise run test
```

Remove `UV_PROJECT_ENVIRONMENT` from the calling environment first (`env -u
UV_PROJECT_ENVIRONMENT mise run test` on Unix, `Remove-Item
Env:UV_PROJECT_ENVIRONMENT -ErrorAction SilentlyContinue` in PowerShell).
The entry prepares desktop prerequisites, installs the built Python wheel,
restores generated contracts and extracts the Rust archive once before concurrent
readers start. Standalone Rust lanes also reuse one extraction for listing and
execution. The entry runs all ten
lanes even if one fails and returns failure if any lane fails. Full lane logs,
exit codes, wall times and an aggregate `time-summary.md`/`.json` live under
`target/test-reports/run-<profile>-<native>/`; every lane also keeps its own time
summary under `target/test-reports/<category>-<language>/`. Individual lane tasks
remain independently usable; avoid running the same lane twice in one checkout.
Only one native desktop lane may use an interactive desktop session at a time,
including runs from other worktrees. Reserve an idle test host for wall-clock
comparisons; simultaneous external test runs can trip existing liveness guards.

The scheduler reserves CPU headroom for the desktop and divides the remaining
logical CPU slots across the four Rust nextest lanes, Python integration and both
Vitest lanes. All seven pools use that worker count. Standalone
Rust lanes keep nextest's CPU-sized pool; standalone Python integration uses
xdist's automatic pool with work stealing. Vitest uses isolated worker threads
and parallel files; tests within a file retain their authored order. Rust tests
already run in isolated processes, with fixture-local storage and ephemeral
listeners. Python workers own temporary roots and ephemeral gRPC listeners.
Pytest caches and Vite/Vitest caches belong to their lane, so lanes do not replace
one another's results.

Shared input preparation completes before test consumers start:
preparation mutates the wheel environment and generated modules and extracts shared
read-only Rust binaries. Within its exclusively owned interactive desktop session,
the native system lane runs independent workflows concurrently. Each workflow owns
its temporary data roots, WebDriver/backend ports, browser profiles and native pickers.
Process-local test fixture mutexes protect callbacks or runtime state inside one
story; they do not lock other nextest processes. No cross-test nextest group is
needed. Hosted CI and nightly test jobs already depend on their artifact producers
and impact selection independently, without test-to-test dependencies or a shared
concurrency group; each keeps its own public bootstrap and required platform legs.

Use `mise run test:rust:acceptance -- --select 'test(=facade::typescript_facade_bridge)'` for one native acceptance story from the Tests archive.
Use `mise run test:typescript:unit -- --select <test-file>` for focused frontend unit proof against the current source. Preparation generates imports even for selected tests. The default unit lane restores the matched Product contracts and runs the complete suite.
Use `mise run test:typescript:system -- --select two-gui-remote` to run the native two-desktop remote-control story alone. It consumes dev Product and Tests artifacts, including the OBC fixture; prepare them with `mise run build:rust`, `mise run build:typescript`, `mise run build:rust:desktop` and `mise run build:rust:tests`. The default native system lane runs execution/recovery (`native-desktop`), configuration/remote access (`desktop-configuration`), and two-desktop control on Windows and Linux.
Use `mise run test:python:unit -- --select <pytest-expression>` for focused Python unit proof against the staged client wheel.

During implementation, run targeted checks for changed behavior. Once all agreed
source work and targeted proof are complete, push the authorized candidate, verify its remote SHA and immediately run
the full local gate alongside hosted CI:

```sh
mise run ci:simulate
```

The default run covers every declared required platform; selecting one platform
is only partial proof. Local simulation runs independent jobs concurrently in dependency order without CPU, memory
or thread limits. Changes limited to skills, agent guidance, README or
CONTRIBUTING files select no execution jobs; the required status gate still
completes. Other documentation uses its declared checks, while mixed source or
policy changes retain full selection.

The Tests producer alone enables
`dokime-runtime/testing`: a Rust fixture surface for private receipt, spool,
window, log and report zoom owners, with no extra Harmos visibility. Integration
exercises small histories; system uses seven days of complete raw evidence, bounded report envelopes,
and real segment-sized log volume.
The GUI Tests feature `dokime-gui/testing` exposes the native Save workflow with
a controlled picker result; its integration story uses real verified streams and
atomic filesystem publication, including refusal and generation-change cleanup.
Product enables neither testing feature. Focus one small guard with
`mise run build:rust:focused -- test --locked -p dokime-runtime --features testing --test integration -- retained_evidence::retained_evidence --exact`.

Product and Tests build independently. Tests use Cargo's unoptimized test
profile; test lanes consume their artifacts. Nightly additionally runs coverage,
benchmarks and the generated-starter test, with release-profile installers.
Report missing local executors and local failures; repair every failed required
job. Hosted required checks and reviews gate integration. Create or update the
authorized ready PR and enable auto-merge without waiting for local completion.

### System Story Reports

Both Rust and native desktop system lanes enforce a 300-second budget per story.
Keep workloads proportional to the behavior under test; retain boundary sizes
when the boundary is the point. Terminal interruption and transport disconnection
are independently selectable plugin recovery stories.

Report-producing system stories automatically save and reopen their final sealed
execution reports during shared fixture teardown or desktop scene finalization.
Desktop scenes declare the executions they materialize and the reports they own.
Shutdown and recovery scenes own reports sealed by their boundary, including
unstarted members. The native finalizer saves only those declared reports once;
read-only scenes do not re-export earlier history.
The finalizer verifies execution identity, the durable seal/checkpoint and artifact
inventory/content. No separate report-saving story or command is needed. Stories
which do not produce execution reports have explicit reasons in
`crates/dokime-runtime/tests/support/system_reports.json`.
Native shutdown stories exercise their stop boundary first; finalization runs on
their recovered session's ordinary teardown. Filtered desktop runs name their
selected journeys in JUnit. The default lane runs both walkthrough workflows
with every maintained scene; each has its own fixture, JUnit result and enforced
story budget.
The native harness logs scene totals and retains per-step timings in each story's
`timings.json` under `target/tests/<profile>/<native>/system-typescript/`.
Categories distinguish launches, subprocesses, execution waits, polling, native
pickers, report exports and browser reopens; nested categories overlap. Timings
are diagnostic output, never correctness assertions.

### Reading Test Traces

After `mise run build:rust:tests`, select a small integration story with a nextest
filterset:

```sh
mise run test:rust:integration -- --select 'test(=retained_evidence::retained_evidence)'
```

Integration, acceptance and system lanes set `DOKIME_TEST_TRACE_DIR` to
`target/test-reports/<category>-<language>/traces/`. Native tests, desktop stories
and inherited application/sidecar processes write `<process>-<pid>.pftrace` files.
Open these Chrome JSON traces in [Perfetto](https://ui.perfetto.dev) to inspect
Rust test, desktop scene/step and Harmos/Dokime operation span lifetimes.
Place `#[dokime_tracing::trace_test]` before the existing `#[test]` or
`#[tokio::test]` on each Rust lane story, with `dokime-tracing` in the package's
dev-dependencies. Stories which install application logging use
`#[dokime_tracing::trace_test(logging)]` so that logging installs the subscriber
once; isolated parent processes call `dokime_tracing::process()` before spawning
the child. Desktop journeys and wait steps share
`crates/dokime-gui/e2e/test-trace.mjs`.
Tracing is disabled unless this variable is set; it may also be set explicitly
when diagnosing a test application. Trace fields and event payloads are excluded.

Each lane writes `time-summary.md` and `time-summary.json` beside `traces/`, with
the top 30 spans by total duration and self time, JUnit totals for every test,
and the slowest tests. CI retains this directory with the lane artifacts, even
on failure. Each run replaces that lane's traces; copy them before rerunning
when comparing results. Different lanes have independent directories.

Durations include async suspension. Self time subtracts the union of direct
child lifetimes; overlapping spans/processes cannot be summed as wall time or
CPU time. Native traces use a separate Perfetto track per span to preserve
concurrent lifetimes, with process-local parent IDs in metadata. Normal completion
flushes export; killed processes may omit open spans or leave unreadable files,
which the summary lists. Test totals come from JUnit and include harness overhead.

### Storage Benchmarks

The runtime's `integration-storage` Criterion target uses the actual SQLite
backend and unmodified `dokime_runtime::config::storage_defaults()`. It mirrors
Harmos's storage suite and reduced workloads: a 30 MiB file, 10,000 records
with 256-byte payloads, and 1 million timestamped f64 samples. File chunks are 1 MiB;
record and sample batches contain at most 4,096 items and 1 MiB of bounded input.
The sample workload represents one second at 1 MHz without paced sleeps.

The shared fixture and suite cover accepted and durable ingestion, symmetric
reads, serial and concurrent files, queries, transforms, contention and filesystem
operations. Workload generation, timing boundaries, correctness checks and resource
sampling match Harmos; fixture storage configuration and persistence use Dokime.
The query suite also preserves Dokime's end-to-end manager RMS, histogram and
extrema-downsample cases on the shared waveform fixture, alongside separately
identified Arrow transform kernels. They use the same production settings, reduced
waveforms and first-batch/completion timing as the other manager queries.
Storage benchmarks have one owner in `integration-storage`; `integration-runtime`
contains runtime capture and durable plot-visibility cases.
Distinct-file reads have both serial and concurrent cases. Concurrent read timing
counts each parallel open/page round once; offset, length and digest checks run
outside that interval. The small journal/captured-record/transaction comparison
cases retain Harmos's explicit single-entry replay/capture policy to isolate those
paths; they do not represent Dokime's default managed-state throughput.

Effective settings, backend identity and source provenance accompany the native
Criterion results. Dokime's machine-sized budgets differ from Harmos's fixed
reference budgets, so the comparison measures configured systems rather than
backend cost alone. Pure transform cases are identified separately.

```sh
mise run build:rust:bench
mise run bench:rust:integration -- storage/
```

The standard runner retains native Criterion results under `.stkr/benchmarks/`.
Keep workload, settings, compiler profile and hardware consistent when comparing
before and after results; these timings are not correctness-test deadlines.

### Large-Cycle Report Saves

Report-save timing belongs to the ordinary Criterion `system-runtime` target.
Build the matching benchmark, host and OBC fixtures, then select its large-cycle saves.
The standard producer builds OBC separately so the host's runtime feature cannot
link another sidecar declaration into its executable:

```sh
mise run build:rust:bench
mise run bench:rust:system -- report_saves_large_cycle
```

The fixed workload expands each plugin case 32 times. Each state uses a fresh
matching workload at its inspection prompt. The sealed case seals during fixture
setup; current iterations share one saved evidence frontier. The Criterion cases
measure materialization, verified streaming, file writes and durable sync. Use the
same plugin revision, build profile and native hardware for comparisons. Delivery
timing does not prove materialization memory, scan work or ingest latency; those
require source-owned counters. Preserve a baseline before source edits; a later
measurement cannot recreate it.

### Long Execution Finalization

The opt-in `long_execution` integration benchmark uses the real runtime and
SQLite with deterministic typed host rows at their default cadences, a 10 Hz
device-like stream, minute records, traceability, five document plots and logs.
One-hour and four-hour volumes share the same batch bound. Setup persists the
historical evidence before measuring Stop to drained closure, Cancel of an unstarted
sibling, report readiness from Stop admission, and verified, synced HTML save
separately. Reports read durable time envelopes and bounded first/last rows.

Set `DOKIME_FINALIZE_MEASURE=profile` in the shell for one pair of workloads with
JSON timings and runtime/report tracing on stderr, then run:

```sh
mise run build:rust:focused -- bench --locked -p dokime-runtime --features testing,fault-injection --bench integration-runtime -- long_execution
```

The focused task retains the standard compiler preflight and Clippy. Use
`DOKIME_FINALIZE_MEASURE=criterion` with the same selection for repeated native
Criterion measurements. Fresh fixture construction is outside
the measured intervals. The long workloads do not run without this explicit opt-in.
Run measurements without concurrent validation and preserve the original executable
before changing production source. A running execution uses Stop; Cancel skips an
execution that has not started and retains its own report/evidence boundary.

## Pull Requests

Target `dev`. Describe the user-visible change and its verification, and keep
required native checks enabled. Do not include secrets or proprietary device
data in source, fixtures, logs or screenshots.

Contributions use the repository's [Apache-2.0 license](LICENSE). Release
publication is a separate authorized workflow; ordinary candidate validation
does not publish packages or create tags.

## Reporting Issues

Use the [issue tracker](https://github.com/stokker-technologies/dokime/issues).
Include the operating system, revision, reproduction steps and relevant logs,
with secrets and private test data removed.
