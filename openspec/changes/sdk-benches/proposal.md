# SDK Benches

Status: scenario, measures and job names locked with the owner on 2026-11-20;
implementation has not started. It starts after the plugin launch handshake
change lands and before the plugin contract freeze.

## Why

The four language SDKs (Rust, Python, C#, C++) promise one authoring model, and
the product targets 128 or more streams at up to 1 kHz each, yet nothing measures
what a plugin costs in each language. The only Nightly benches today are
`Bench / Desktop / Round Trip` and `Bench / Desktop / History`. Without one shared
scenario, a slow SDK, a start-up regression or a memory leak in one language only
shows up in a lab run.

## What Changes

- **`Bench / SDK / <Language>` jobs** for Rust, Python, C# and C++ measure the same
  scenario in every SDK.
- **One reference plugin per language** is the subject. All four declare identical
  operations, observers and streams. The `thermal` plugin, which already backs the
  SDK equivalence stories, is the seed. The starter projects are not benched: a
  starter stays the smallest thing a new author reads, and its baselines must not
  move when onboarding content changes.
- **One driver**, written once, runs the identical scenario against each language's
  reference plugin and records the same measures. It refuses to run when one
  plugin's declared catalog differs from the others (harmonization guard).
- **Measures:** operation round trip (latency distribution and calls per second for
  an echo operation); stream throughput (samples per second from an observer,
  including 128 or more streams at up to 1 kHz each, and the ceiling of a single
  stream); start to ready (process start until the host admits the plugin); idle
  and loaded memory of the plugin process. Labels put units in square brackets,
  for example `Throughput [samples/s]`.
- **Measurements, not tests.** Results carry regression baselines per language and
  per platform. They never pass or fail a test, never gate Nightly publication, and
  the cross-language comparison is a report. No test asserts wall-clock time. C++
  and C# benches reuse the fixtures their Nightly fixture jobs already build, so
  they add no dependency build time.
- **Related product benches.** `Bench / Protocol` measures the encode and decode
  cost and the encoded size of this product's own messages: evidence, samples,
  plots and report responses. `Bench / Storage` measures this product's objects
  through the library's storage operations: write an execution, seal it, query one
  million samples. `Bench / Storage` is added only after the storage cutover
  recorded in `operation-boundary`. Benches of the transport and the storage
  engine belong to the library the product is built on, which already has them.
- Job names follow the existing Stage / Component / Variant taxonomy in
  `.github/workflows/nightly.yml`. This change edits no workflow; the
  implementation does.

## Sequencing

1. Locked now: the scenario, the measures, the harmonization guard and the job names.
2. Implementation starts after the plugin launch handshake change lands. That
   change alters plugin start-up and the SDK run entries, so earlier start-to-ready
   and round-trip baselines would be discarded.
3. The SDK benches land before the plugin contract freeze, so the freeze is
   decided with measured costs per language.
4. `Bench / Protocol` can land with the SDK benches. `Bench / Storage` waits for
   the `operation-boundary` storage cutover.

## Capabilities

### New Capabilities

- `sdk-benchmarks`: one harmonized reference plugin per language, a single driver
  with a catalog guard, the four measures, per-language and per-platform baselines
  that never gate, and the product's protocol and storage benches.

### Modified Capabilities

None. `native-desktop-performance` and `plot-refresh` keep the desktop benches.

## Impact

- New reference plugin sources in each SDK, seeded from
  `sdk/rust/examples/thermal.rs`, `sdk/python/examples/thermal`,
  `sdk/tests/fixtures/python/thermal`, `sdk/tests/fixtures/cpp/thermal` and
  `sdk/tests/fixtures/csharp/SystemPlugin`.
- A new bench target for the driver, protocol and storage bench targets, bench
  tasks in `mise.toml`, and the bench and report adapters in
  `tools/python/src/dokime_tools/{task/bench.py,test/benchmark.py,release/reports.py}`.
- New Nightly jobs in `.github/workflows/nightly.yml` and the hosted baseline
  cache (`.github/actions/setup/action.yml`, `tools/python/src/dokime_tools/ci/native.py`).
- Documentation impact for the owner: `docs/tooling.md` (bench tasks, jobs, the
  report) and the SDK guides that name the reference plugin.
