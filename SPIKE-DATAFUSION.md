# DataFusion Query Engine Spike

This is a measurement prototype on `spike/datafusion` in both repositories. It is not a merge candidate. The public `dokime.query.aggregate_samples` and bucketed `dokime.query.list_samples` requests and responses are unchanged.

## Versions And Measurement Conditions

- DataFusion: released `=55.2.0`, with default features disabled. No DataFusion git dependency is used. Its Arrow requirement is `^59.2.0`; every Arrow crate resolves to `59.3.2` in both worktrees.
- Arrow changed from `=60.1.0` to `=59.3.2` **before** the baseline was built or measured. Both sides use the same Arrow version. The change required three metadata-map key/value conversions in `storage/ipc/schema.rs`; it changes no stored schema or query contract.
- Baseline source: Dokime `766330b7342bb63790fbc6328399f883ed7f037b` and Harmos `ad18e90a3f4121da85743c73ead2d48818263548`, plus the Arrow compatibility edits and identical benchmark/parity fixtures. Dokime used an ignored local Harmos override during implementation and measurement.
- Windows, Ryzen 9 5950X, 16 cores/32 threads, approximately 80 GiB RAM, Rust 1.99.0. Compiler jobs were limited to one. Original baseline tasks waited for the requested idle compiler slot; subsequent tasks and measurements used the project-owned machine-wide native build lock after the workstation guidance changed.
- Latency uses Cargo's `bench` profile, Criterion 0.8.2, ten flat samples, one-second warmup, and a three-second measurement target. Criterion extended collection when one query exceeded that target. Fixture construction is excluded from query timing.

The fixture stores one million samples in the existing SQLite catalog and Arrow IPC files. Counter values are `0..999999`, timestamps are counter times 1,000 ns, values repeat `0..999`, and sources alternate `a`/`b`. Aggregate and bucket requests select 900,000 samples using both `counter >= 100000` and the half-open window `[100 ms, 1 s)`. Buckets are 100 ms wide, yielding 18 source/bucket groups. The percentile request selects the first 50,000 samples from the same store and requests median and p95.

## Query Latency And Memory

Median latency and bootstrap 95% confidence intervals, in milliseconds:

| Query | Baseline | DataFusion | Speedup |
| --- | ---: | ---: | ---: |
| Aggregate: count, sum, mean, std, RMS, min, max | 2303.64 [2289.05, 2554.51] | 483.30 [462.71, 512.76] | 4.77x |
| Bucketed mean | 2726.64 [2697.39, 3049.33] | 615.20 [592.74, 626.39] | 4.43x |
| Exact median and p95, 50,000 selected samples | 1261.97 [1146.92, 1338.19] | 37.63 [36.63, 39.12] | 33.54x |

Sampled peak process RSS during queries, in MiB:

| Query | Baseline | DataFusion | Change |
| --- | ---: | ---: | ---: |
| Million-sample store, aggregate | 123.78 | 136.86 | +10.57% |
| Million-sample store, buckets | 127.80 | 137.36 | +7.48% |
| Percentiles | 127.94 | 137.46 | +7.44% |

RSS was sampled every 20 ms with Windows `GetProcessMemoryInfo`, between query markers. It includes the runtime, resident fixture/catalog state, code pages, and allocator retention; it is not a query-allocation delta. The DataFusion pool is bounded independently of process RSS.

These numbers measure the complete replacement path. They do not isolate DataFusion's contribution: native projected batch decoding replaces per-sample reconstruction, and percentile selection rescans a compact numeric spool instead of decoding the IPC evidence nine times. A batch-reader-only control was not measured.

## Implementation And Parity

The Harmos `IpcTable` implements DataFusion `TableProvider` over a single ordered stream. Dokime supplies catalog-selected IPC batches, preserving capture ranges, contract versions, snapshot frontiers, checksums, and scan budgets. Existing prepared predicates become DataFusion filter expressions using lossless tagged values; time windows use nanosecond decimal comparisons. Buckets use epoch floor alignment, group aggregation, and ordered output.

The existing storage-derived query working budget supplies a greedy DataFusion memory pool of half `working_bytes` (4 MiB with the fixture defaults). IPC decoding retains its existing half-budget check, with a bounded producer channel and 1,024-row slices. This bounds the engine pool, not every allocation in the process. Projection buffers, decoded output, percentile scratch, and runtime state also require memory.

One partition and disabled aggregate repartitioning preserve input order. Custom UDAFs reuse Dokime's sequential `Moments` and exponent-bin bucket sums. They reject intermediate-state merging rather than silently change float results. Exact percentiles preserve the existing rational rank and sign-safe linear interpolation, including signed zero and extreme values. Interior percentile requests retain the existing logical nine-scan work-budget charge.

All ten focused Dokime integration tests pass before and after: nine existing tests unchanged, plus the new floating-point cancellation/large-integer numeric parity case. Existing cases cover statistics, bucket boundaries/order/pages/limits, snapshots, running-query work limits, finite Arrow pages, and adjacent plot-query behavior. The three large-fixture result fingerprints match exactly, including returned floats, quality counts, versions, and bucket order. No tolerance relaxation or expected-result update was made.

Harmos's two existing query tests also pass unchanged. Its new exact-percentile spool test passes in memory and with forced disk spilling, covering signed zero, subnormals, duplicates, and opposite finite extremes. The primary workloads did not deliberately exhaust the engine pool or force engine spilling; the percentile benchmark fits its numeric spool in memory. Generic aggregate spilling/merging is not supported by these deterministic UDAFs.

The unchanged Dokime cases are `query_statistics::exact_query_statistics`, `query_buckets::epoch_query_buckets`, `query_running::query_running_work`, both `query_samples` cases, and the four `domains::telemetry::plots` cases selected by `query_` (`distribution_query_budget`, `numeric_query_reductions_match_spilled_results`, `record_query_absent`, `snapshot_query_samples`). The unchanged Harmos cases are `query::capture_statistics` and `query::cancellation_and_finiteness`. The added cases are Dokime's `numeric_query_parity` and Harmos's `exact_quantile_spill`.

## Source Accounting

Physical lines from `git diff --numstat`, including comments and retired inline kernel tests:

| Source Group | Added | Removed |
| --- | ---: | ---: |
| Harmos existing query kernels replaced | 0 | 0 |
| Harmos provider and exact numeric spool | 318 | 0 |
| Dokime statistics/bucket modules | 38 | 414 |
| Dokime DataFusion plans, reducers, normalization, catalog batch adapter | 1103 | 0 |
| Other query registrations and private re-exports, both repositories | 44 | 2 |
| **Query Source Total** | **1503** | **416** |

The measured slice therefore adds **1,087 net query-source lines**. Of the new implementation, 495 lines are numeric compatibility reducers/spooling, rather than stock engine operations. These two Dokime sample paths previously folded rows in Dokime and did not call Harmos's generic query kernels, so this spike removes no existing Harmos kernels. Benchmark/parity tests, dependency locks/manifests, Arrow compatibility, and workstation locking are accounted separately from query source.

Whole-branch `git diff --stat` summaries against the baseline source revisions, excluding this report itself:

| Repository | Changed Files | Added | Removed |
| --- | ---: | ---: | ---: |
| Harmos | 10 | 1959 | 118 |
| Dokime | 22 | 2672 | 549 |

The remaining changes include dependency locks/manifests, the Arrow compatibility edit, benchmark/parity fixtures (Dokime +304/-1; Harmos +65/-0), and the focused-task machine-lock workflow (+143/-1 in each repository). The lock addition follows the updated workstation guidance and is not credited as query-engine glue or kernel removal.

### Projection For The Remaining Query And Plot Code

Gross removable physical-line estimates from the original modules, not promises of a completed migration:

| Module Inventory | Current Lines | Potential Removal | Reason |
| --- | ---: | ---: | --- |
| Harmos accumulation, execution, reductions, order statistics, ordering, spill, and relational transform kernels | 2941 | 1800–2400 | Engine filter/project/sort/group/histogram execution can replace common kernels; exact numeric and domain downsampling semantics still need custom code. |
| Harmos readers, source columns, typed tables, and object schemas | 1761 | 250–450 | Share scans/projections; keep catalog ownership, file integrity, typed APIs, and schema contracts. |
| Dokime statistics and buckets | 825 | 350–450 | This spike removes 414 old lines, but adds adapters and retains custom numeric semantics elsewhere. |
| Dokime selection/schema/predicate modules | 873 | 80–150 | Replace row/window matching; retain validation, schema evolution, and lossless integer comparisons. |
| Dokime sample/record query plumbing and reduction entry points | 1261 | 200–400 | Replace relational scans and ordering; retain request/response schemas, continuations, limits, and attribution. |
| Plot payload grouping/projection/retention and spill plumbing | Approximately 6000 across the payload tree | 1000–1600 | Share grouping, sorting, projection, and spill execution; keep plot contracts, FFT/spectrum, WiFi analysis, correlation semantics, and specialized downsampling. |

Total projected gross removal is approximately **3,700–5,450 lines**. Allowing for shared provider/planning glue and the custom numeric code demonstrated here, a conservative net reduction is approximately **2,000–4,000 lines**. The remaining runtime query modules largely own domain histories, snapshots, pagination, and public contracts; their entire file sizes are not removable engine code. Only a broader implementation could validate this estimate.

## Cold Build And Arrow Transition

Fresh compiler-output directories, warm dependency downloads/filesystem caches, `CARGO_BUILD_JOBS=1`, incremental compilation disabled, and sccache bypassed for both measurements. This is a cold compiler-output measurement, not an empty-registry or clean-machine measurement. Waiting for a workstation slot is excluded.

| Dev-Profile Runtime Build | Baseline | DataFusion | Change |
| --- | ---: | ---: | ---: |
| Complete mise task, mandatory Clippy plus build | 720.24 s | 824.47 s | +14.47% |
| Cargo build phase, rounded task-log timings | 5m58s | 7m32s | Approximately +26% |

Command: `mise run build:rust:focused -- build --locked -p dokime-runtime --lib`. The baseline target was `target/spike-baseline-arrow59-final`; the DataFusion cold target was `target/spike-datafusion-cold`.

The `dokime` executable contains the CLI and server; `dokime-server` is a library and has no separate executable. Both size measurements use the default CLI/server configuration, dev profile, local-package line-table debug information, and no optional native media feature. PDB files are excluded. These are not release-profile shipping-size measurements.

| CLI/Server Executable | Baseline | DataFusion | Change |
| --- | ---: | ---: | ---: |
| Exact bytes | 155961344 | 233488896 | +49.71% |
| MiB | 148.74 | 222.67 | +73.94 MiB |
| Mise CLI build after the corresponding runtime build | 1224.79 s | 1358.15 s | +10.89% |

Command: `mise run build:rust:focused -- build --locked -p dokime-cli --bin dokime`. The CLI timing reuses its preceding runtime build and is not an independent cold CLI measurement.

Before adding DataFusion, the Arrow-only edit snapshot changed Harmos's workspace manifest by +6/-6 lines and its lock by +12/-24. Dokime's manifest changed +4/-4, the schema compatibility source +3/-3, and its lock +37/-41; ten lock removals were local Harmos source entries from the integration override, rather than Arrow changes. No usable paired Arrow-60 cold measurement completed before the owner's version decision changed, so no separate Arrow-version timing effect is claimed. The comparison above isolates the engine migration on Arrow 59.3.2.

## Costs And Recommendation

**Adopt Partially.** Use the shared relational engine where it can replace complete kernel modules. Do not merge this narrow prototype as a default replacement for the whole runtime query layer: it currently adds code, and its speedup combines batching/spooling improvements with DataFusion execution.

Migration order:

1. Consolidate projected IPC batch scanning/catalog admission into the shared provider, add a native-batch-only control, and validate Linux parity before production adoption.
2. Move Harmos filter/project/sort/group/histogram execution onto DataFusion, removing the superseded kernels. Retain explicitly deterministic sums/moments and exact percentile kernels where their API semantics require them.
3. Migrate Dokime sample and record relational plans, then plot grouping/projection/spill plumbing. Retain domain histories, snapshots, pagination, plot contracts, and specialized analysis algorithms.

The minimal-feature engine adds 55 packages to Dokime's workspace resolution and 98 to Harmos's independent resolution, including additional Arrow components and CSV/JSON/object-store infrastructure. Arrow must move in lock-step across both repositories: this release requires the measured downgrade from 60.1.0 to 59.3.2; retaining Arrow 60 would require a later compatible DataFusion release.

Single-partition execution constrains future parallelism. The custom numeric UDAFs cannot merge partial states or provide general aggregate spilling; high-cardinality queries may hit the bounded pool. Projection/output allocations and percentile scratch also need stronger end-to-end accounting before claiming a hard whole-query memory ceiling. Spill files need operational disk budgeting and deadline hardening. Cross-version reads of pre-existing Arrow-60 files and Linux execution were not measured; fixtures on both sides were written/read with Arrow 59.3.2. These remain production-hardening tasks.

## Reproduction And Preserved Evidence

The tracked benchmark is `crates/dokime-runtime/benches/integration/samples.rs`. Build the focused executable with:

```powershell
mise run build:rust:focused -- bench --locked -p dokime-runtime --features testing --bench integration-runtime --profile bench --no-run
```

Run that task's `integration_runtime-*.exe` directly with `--bench spike_samples --noplot --save-baseline after`, holding the machine lock through `python -m dokime_tools.build.machine -- <executable> <arguments>`. Set `CRITERION_HOME` to `target/benchmarks/datafusion/criterion` to retain native Criterion output. The local evidence folder preserves the `before`/`after` estimates and samples, stdout result fingerprints, RSS-probe logs, cold-build logs/JSON, and both measured executable copies. Evidence and executables are ignored build artifacts; this report and the benchmark/parity sources are committed.

Focused correctness commands:

```powershell
# Dokime
mise run build:rust:focused -- test --locked -p dokime-runtime --features testing --test integration -- query_ --test-threads=1
# Harmos
mise run build:rust:focused -- test --locked -p harmos-storage --features datafusion --test integration -- exact_quantile_spill --test-threads=1
mise run build:rust:focused -- test --locked -p harmos-storage --features datafusion --test integration -- query:: --test-threads=1
```

For final publication, Dokime pins the pushed Harmos revision `1a1089266651bfaaa733005cab55b11db135799c`. Its implementation commit is `b3c54121debc0b857036de342b11b372cbaf7f09`; the follow-up only orders module declarations for rustfmt. The ignored local override used for measurements is removed from the active configuration, and the committed lockfile resolves published git packages. Both branches are named `spike/datafusion`; no PR or merge is part of this spike.

The final published-pin run passes all ten selected Dokime tests. Together with Harmos's three targeted tests, focused proof totals 13 passing tests (11 existing and two added). Changed Rust files pass rustfmt, the workstation Python changes pass Ruff and the lock serialization/crash-recovery probe, and whitespace/required-term diff scans are clean. Validation is limited to this Windows spike; full project CI and Linux lanes were not run.
