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
Regenerate Python and desktop bindings with `mise run prepare:bindings -- <compiler-messages.jsonl>`
using the compiler messages from one successful server build.
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

Use `mise run test:typescript:unit -- --select <test-file>` for focused frontend unit proof against the current source. Selected tests require their own generated inputs when they import them. The default unit lane restores the matched Product contracts and runs the complete suite.

During implementation, run targeted checks for changed behavior. Once all agreed
source work and targeted proof are complete, push the authorized candidate, verify its remote SHA and immediately run
the full local gate alongside hosted CI:

```sh
mise run ci:simulate
```

The default run covers every declared required platform; selecting one platform
is only partial proof. Local simulation runs one job at a time without CPU, memory
or thread limits. Changes limited to skills, agent guidance, README or
CONTRIBUTING files select no execution jobs; the required status gate still
completes. Other documentation uses its declared checks, while mixed source or
policy changes retain full selection.

The Tests producer alone enables
`dokime-runtime/testing`: a Rust fixture surface for private receipt, spool,
window, log and report zoom owners, with no extra Harmos visibility. Integration
exercises small histories; system uses seven days, 100,101 original report points
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

Build the benchmark and its matching plugin fixture, then run the save harness:

```sh
mise run build:rust:bench
mise run measure:report-save --copies 32
```

Copies must be between 1 and 128. The harness expands the same plugin cases, freezes at the
inspection prompt, saves the current cycle, seals it, and saves the sealed cycle.
Use identical copies, plugin revision, build profile and native hardware for
baseline and candidate runs. Preserve a baseline checkout before source edits;
a later measurement cannot reconstruct a missing baseline.

Each save emits JSON with total/open/first-byte timing, delivery batch and
writer retention, bytes and delivery passes. Materialization retention, scan
counts, materialization passes and concurrent ingest latency remain null until
source-owned instrumentation supplies them. Null is unavailable proof. The
harness does not assert timing or claim a speedup, and its delivery memory
measurement excludes report construction.

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
