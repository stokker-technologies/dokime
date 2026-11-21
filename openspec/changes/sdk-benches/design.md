# SDK Benches Design

See `proposal.md` for the motivation and the locked scope. This design records the
current bench surface, the reference plugins, the scenario and the jobs. Items
marked **proposed** are not part of the locked decision; the implementation may
change them without changing the specs.

## Context

### Bench Surface Today

| Surface | Target | Local Task | Hosted |
| --- | --- | --- | --- |
| `crates/dokime-runtime/benches/storage/{main,publication,reports,windows}.rs` | `storage` (Criterion groups `storage`, `telemetry`, `runtime`, `reports/save`, `windows`) | `mise run bench -- --binary storage` → `bench:rust:storage` → `tools/python/src/dokime_tools/test/storage_bench.py` | None |
| `crates/dokime-gui/benches/system/` | `system-desktop` (`crates/dokime-gui/src-tauri/Cargo.toml`) | `bench:rust:system` → `dokime_tools.test.run bench --category system --language rust`, built by `build:rust:bench` → `tools/python/src/dokime_tools/build/bench.py` | `Bench / Desktop / Round Trip` and `Bench / Desktop / History` on `unix` and `windows`, after `Build / Bench` |

- Hosted `Build / Bench` (`.github/workflows/validate-build-tests.yml`) runs
  `mise run build -- --artifact bench --binary system-desktop`; its archive holds
  no SDK fixture.
- `tools/python/src/dokime_tools/task/bench.py` selects benches for `rust` and
  for `python` at the system layer only. `tools/python/src/dokime_tools/test/benchmark.py`
  runs `rust` (Criterion) and `python` (pytest-benchmark) producers and refuses
  any other language. Its `--language` is the language the bench is written in,
  not the language under measurement.
- Each producer writes `target/benchmarks/benchmarks-<language>-<category>.json`;
  the desktop bench writes `benchmarks-rust-desktop.json` through
  `tools/python/src/dokime_tools/test/desktop_bench.py`. Rows carry `name`,
  `unit`, `value`, `baseline_value`, `threshold_percent` and, for a stage that
  could not be measured, `status: not_measured` with a `reason`.
- `tools/python/src/dokime_tools/release/reports.py` collects every
  `benchmarks-*.json` from the job results. `stage` writes the merged
  `benchmarks.json` into the Nightly release inventory, and `table` adds one
  `Benchmark / <suite> / <case>` row per case plus one row per regression above
  `threshold_percent` to the release notes.
- In `.github/workflows/nightly.yml`, `Release / Publish / GitHub` needs only
  `package` and `freshness`. `Release / Report / Nightly` needs every job
  including `build_bench` and `bench`, and runs under `always()`. A failing bench
  job therefore never blocks publication; it only shows in the report.
- Baselines: Criterion keeps a saved baseline named `dev` under
  `target/benchmarks/criterion` (`BASELINE` in `test/benchmark.py`), Python keeps
  `dev.json` under `target/benchmarks/python/<identity digest>/`, and the desktop
  bench compares with the previous report rows keyed by name and unit.
  `.github/actions/setup/action.yml` restores a `bench_baselines` cache keyed
  `ci-v6-baselines-<platform>-<layer>-` (`tools/python/src/dokime_tools/ci/native.py`)
  only when `CI_BENCH` is `true`. Nothing in `.github/`, `mise.toml` or
  `tools/python/src/dokime_tools` sets `CI_BENCH`, and no workflow saves that cache, so every hosted desktop bench seeds a fresh baseline with
  `--refresh-baseline` and no run is compared with the one before it.
- The library already saves its bench baselines on `dev` from its own Nightly.
  Its transport, runtime, query and storage-engine benches stay its own.

### Reference Plugin Seeds Today

Every language has a thermal plugin with a `thermal.cycle` template, a
`temperature` stream with a 1 s observer and a `counter` stream with a 1 kHz
window observer. The SDK equivalence stories
`crates/dokime-runtime/tests/system/{python,csharp,cpp}_plugin_matches_rust_thermal_evidence.rs`
compare each one with the Rust evidence. Nightly `Test / SDK / Equivalence` runs
the C# and C++ stories.

| Language | Thermal Source | Built Today By | Echo Operation |
| --- | --- | --- | --- |
| Rust | `sdk/rust/examples/thermal.rs`; host fixture `dokime-rust-thermal` (`tools/python/src/dokime_tools/build/host_fixtures.py`) | `mise run build:host-fixtures -- --language rust` | Missing |
| Python | `sdk/python/examples/thermal` (`thermal_example`); fixture `sdk/tests/fixtures/python/thermal/plugin.py` | Runs from source against the Python SDK; no compiled fixture | Missing |
| C# | Test fixture only: `sdk/tests/fixtures/csharp/SystemPlugin` (`ThermalPlugin`) | `Build / SDK / C# / Fixtures` (`mise -C sdk/csharp run publish:system`) | Missing |
| C++ | Test fixture only: `sdk/tests/fixtures/cpp/thermal` (`main.cpp`, `thermal.proto`) | `Build / SDK / C++ / Fixtures` (target `dokime-system-suite`) | Missing |

An echo operation exists only in the C++ and C# starters
(`sdk/cpp/examples/starter/src/plugin/operations/echo.cpp`,
`sdk/csharp/examples/Starter/src/Example/Plugin/Operations/Echo.cs`). Those
starters stay out of the benches.

## Goals / Non-Goals

**Goals:** the same scenario and the same measures in all four SDKs, a catalog
guard, baselines per language and per platform that persist between hosted runs,
and the product's own protocol and storage costs.

**Non-Goals:** cross-language pass/fail thresholds; benches of the library's
transport, runtime or storage engine; benching the starters; benching the store
that the `operation-boundary` cutover deletes; any wall-clock assertion in a test.

## Decisions

### Reference Plugin: Thermal Extended in Place (Proposed)

Each language's thermal plugin becomes its reference plugin. It gains one
plugin-level `echo` operation and a second template, `thermal.load`, that holds
the load streams. The `thermal.cycle` template and its evidence stay as they are,
so the equivalence stories keep their assertions. One plugin per language then
backs both the equivalence stories and the benches, and the existing fixture jobs
build it.

Alternative: a separate `reference` plugin per language. Rejected as proposed: it
duplicates four plugins, their build entries and their declarations, and the
equivalence and bench catalogs could drift apart.

C# and C++ thermal plugins stay under `sdk/tests/fixtures/`, which
`Build / SDK / <Language> / Fixtures` already archives. The C++ bench reuses that
archive and adds no dependency build time.

Declared catalog, identical in every language (identifiers proposed):

```text
operation  thermal.echo                 request: bytes payload  response: bytes payload
template   thermal.cycle                unchanged (equivalence evidence)
template   thermal.load
  stream   thermal.load.channel_000 .. thermal.load.channel_127
           row: Counter, window observer every 1 ms
  stream   thermal.load.burst
           row: Counter, window observer default 1 ms, interval from the
           plugin duration setting thermal.load.interval
  step     hold                         holds until the host stops the execution
```

Illustrative Rust declaration (final names follow the SDK at implementation time):

```rust
let plugin = Plugin::of::<ThermalPlugin>("1.0.0")
    .operation(Operation::new("thermal.echo", echo))
    .suite(Suite::of::<ThermalSuite>().template(cycle).template(load));

async fn echo(_: &Context, request: Echo) -> Result<Echo> {
    Ok(request)
}
```

Illustrative Python declaration:

```python
async def echo(ctx: Context, request: Echo) -> Echo:
    return request


class ThermalPlugin(Plugin):
    suites = [ThermalSuite]
    operations = [echo]
```

### One Driver With a Catalog Guard (Proposed Location)

The driver is one Rust bench target, `system-sdk`, at
`sdk/benches/system-sdk/` in the private `dokime-sdk-benches` package. It starts a host the way the equivalence
stories do, launches one language's reference plugin as a managed sidecar and
runs the scenario. The language under measurement is a variant selected with
`--select`; the bench itself is written in Rust, so `test/benchmark.py` and
`task/bench.py` need no new producer language.

```text
mise run bench -- --language rust --layer system --binary system-sdk --select "^sdk/csharp/"
```

Before measuring, the driver reads the admitted catalog of the plugin under
measurement and of the Rust reference plugin, which the SDK bench build adds
next to the driver (proposed). It compares operations, templates, streams, observers and their
intervals, and row schemas. On any difference it writes no measurement and fails
with the first differing identifier and both languages. Comparing every language
with Rust makes all four equal without a hand-maintained catalog file. Without
`--select` the driver measures every language whose plugin is present.

### Scenario and Measures

The scenario is fixed and identical for every language:

1. Start to ready: spawn the plugin process, stop the clock when the host admits it.
   Repeat with fresh processes.
2. Idle memory: read the plugin process's resident memory after admission, with no
   execution running.
3. Round trip: call `thermal.echo` with a fixed small payload, one call at a time,
   for the latency distribution; then with a fixed number of concurrent callers
   for calls per second.
4. Throughput at target: start `thermal.load` with all 128 channels at 1 kHz and
   count retained samples whose host `received_at` falls inside the steady-state
   measurement window, divided by elapsed time. Source timestamps and rows
   delivered after that window do not inflate the accepted rate.
5. Loaded memory: read the plugin process's peak resident memory during step 4.
6. Ceiling: run `thermal.load.burst` with decreasing values of the existing
   observer interval setting `thermal.load.interval` until the accepted rate stops
   rising; record the highest accepted rate. Every response contains exactly the
   host-granted row count; the observer contract is unchanged.

Rows follow the existing report shape. Case names (proposed):

| Case | Label | Unit |
| --- | --- | --- |
| `sdk/<language>/round_trip/echo` | `Round Trip p50 [ns]`, `Round Trip p95 [ns]`, `Round Trip p99 [ns]` | `ns` |
| `sdk/<language>/round_trip/echo_rate` | `Calls [calls/s]` | `calls/s` |
| `sdk/<language>/throughput/channels_128_at_1khz` | `Throughput [samples/s]` | `samples/s` |
| `sdk/<language>/throughput/ceiling` | `Throughput [samples/s]` | `samples/s` |
| `sdk/<language>/start_to_ready` | `Start to Ready [ms]` | `ms` |
| `sdk/<language>/memory/idle` | `Memory [bytes]` | `bytes` |
| `sdk/<language>/memory/loaded` | `Memory [bytes]` | `bytes` |

A language that cannot sustain 128 channels at 1 kHz still records its accepted
rate; it is a measurement, not a failure. A stage that cannot be measured on a
platform records `status: not_measured` with a reason, as the desktop bench does.

The cross-language comparison is one extra report section built from the same
rows: one table per measure, one column per language, one block per platform. It
has no thresholds.

### Jobs

Job names follow Stage / Component / Variant in `.github/workflows/nightly.yml`:

```yaml
bench_sdk:
  name: Bench / SDK / ${{ matrix.language.name }} (${{ matrix.platform }})
  needs: [build_bench, build_host_fixtures, build_python]
  strategy:
    fail-fast: false
    matrix:
      platform: [unix, windows]
      language:
      - { name: Rust, select: ^sdk/rust/ }
      - { name: Python, select: ^sdk/python/ }
      - { name: C#, select: ^sdk/csharp/ }
      - { name: C++, select: ^sdk/cpp/ }
bench_protocol:
  name: Bench / Protocol (${{ matrix.platform }})
bench_storage:          # only after the operation-boundary storage cutover
  name: Bench / Storage (${{ matrix.platform }})
```

`Release / Report / Nightly` adds these jobs to its `needs`. `Release / Publish / GitHub`
does not. Each job restores its baseline cache and saves it on `dev`, keyed by
platform and job variant, so a run is compared with the previous `dev` run.

### Protocol and Storage Benches

`Bench / Protocol` is a Criterion target over this product's messages in
`proto/dokime/model/v1/evidence.proto` and `proto/dokime/host/v1/{evidence,plots,reports}`:
encode and decode time per message and the encoded size in bytes for evidence,
sample batches, plot frames and report responses at fixed sizes. Proposed
location: `crates/dokime-protocol/benches/protocol/`.

`Bench / Storage` measures this product's objects through the library's storage
operations: write an execution with its evidence, seal it, and query one million
samples. It is added after the `operation-boundary` storage cutover.

Today's `crates/dokime-runtime/benches/storage/` target drives the product's own
`dokime_storage::Store` directly (ingestion, sealing, windows, report saves and
telemetry queries), not the library's storage operations. It gets no hosted job.
Whether the cutover retires it, migrates it into `Bench / Storage`, or keeps it
as a local measurement is an owner decision; `test-stories` also names the
storage bench as a measurement owner.

### Materialized Load Selection

`thermal.load` has a Boolean `burst` parameter, default false. Its pure
materializer retains the 128 channel streams when false and only the burst
stream when true. Both placements use the same hold operation and granted-window
counter sampler. The duration setting changes only the burst observer's interval.

The private `sdk/benches/` workspace package owns both native bench targets, so
the driver, protocol bench, desktop bench and thermal example share the product
dependency graph without adding host dependencies to the author SDK.
The bench producer stages the thermal example as `dokime-rust-thermal`; the
standalone SDK consumer continues to own external-consumer proof.

### SDK Capability Prerequisites

The current C# author API (`sdk/csharp/src/Dokime/Templates.cs` and `Plugin.cs`)
has no pure materialization hook for selecting the declared load streams. The
current C++ `Plugin` (`sdk/cpp/include/dokime/plugin.hpp`) has no settings
declaration entry, although its observers accept interval setting names. Those
SDK owners must expose the existing concepts before their reference plugins can
run this same scenario; the bench does not invent a second sampling mechanism.

## Risks / Trade-Offs

- [Shared hosted runners are noisy] → Baselines are kept per platform and per
  variant, the report shows the change against the previous `dev` run, and no
  threshold gates anything.
- [Extending thermal changes a plugin the equivalence stories use] → The new
  operation and template sit beside `thermal.cycle`; the equivalence stories keep
  materializing `thermal.cycle` only.
- [Start-up changes with the launch handshake] → Implementation waits for that
  change, so no baseline is recorded and then discarded.
- [Python at 128 × 1 kHz may not keep up] → The accepted rate is recorded as it is;
  the gap is the finding.

## Migration Plan

No data migration. Baselines start fresh on the first hosted `dev` run of each new
job. Removing a job removes its rows from the report and nothing else.

### Owner Decisions (2026-11-21)

- Single-stream ceiling varies the existing observer interval setting, never rows
  per window. Fixed-interval observers enforce the host-granted row count.
- Baseline compatibility hashing has one owner,
  `tools/python/src/dokime_tools/test/benchmark.py::baseline_identity`; both native
  and Python producers use it. The second implementation is removed.

## Open Questions

- The plugin launch handshake change and the plugin contract freeze have no
  change in `openspec/changes/` yet; this change names them in words until they do.
- Thermal extended in place versus a separate reference plugin per language;
  proposed: extend in place.
- The fate of `crates/dokime-runtime/benches/storage/` after the cutover;
  proposed: migrate its cases into `Bench / Storage` through the library's
  storage operations and delete the direct-store target.
