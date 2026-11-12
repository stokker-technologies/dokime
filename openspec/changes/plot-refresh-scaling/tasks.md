# Plot Refresh Scaling Tasks

Implement after `plot-kinds` lands. Steps 1–7 run in order. Every step after 1
records before/after desktop-harness results on native Windows and Linux with
the same fixture and conditions; measurements belong in bench artifacts and
Nightly `benchmarks.json`, with the step 1 baseline recorded below, never in timing assertions.

## 1. Desktop Round-Trip Benchmark First

- [x] 1.1 Confirm `plot-kinds` has landed and resolve the proposal's source inventory to the current operation/storage owners; lock implementation, protocol/bindings, GUI, benchmark, tests, removals and affected README/guidance paths without starting other changes.
  - [x] Locked benchmark inventory: `crates/dokime-gui/benches/system/`, native harness exports in `crates/dokime-gui/e2e/native-desktop.mjs` and owned report-browser lifecycle in `e2e/report-browser.ts`, desktop bench target/dependency in `crates/dokime-gui/src-tauri/Cargo.toml`, bench build/report adapters in `tools/python/src/dokime_tools/{build/bench.py,build/rust/bench.py,test/run.py,test/desktop_bench.py}` and `tools/python/tests/integration/test_desktop_bench.py`, `docs/tooling.md` and this change. Current query/storage owners remain `crates/dokime-runtime/src/{runtime/query/plots,storage/sqlite/data/telemetry}`; transport remains `crates/dokime-server/src/grpc/plots{.rs,/}` and `crates/dokime-gui/src-tauri/src/rpc/calls.rs`; GUI owners remain `src/{lib/rpc/ipc.ts,lib/api/native-plots.ts,plots/live-data.ts,plots/live-frames.ts,components/document/PlotGroupGrid.tsx,plots/catalog}`. Observe their existing code through the bench build; no protocol/binding changes, limit changes, removals or README claims are needed for tasks 1.1–1.3. Plot-kind source and focused proof are present in the branch ancestry. Keep `integration-plots` unchanged. Windows measurement precedes optimization; Linux remains native Nightly proof.
- [ ] 1.2 Add the desktop-owned system bench under `crates/dokime-gui/benches/system/`, reuse `e2e/native-desktop.mjs` and existing bench tasks/reporting, and retain `integration-plots`. Instrument backend query, encoding/transport/native bridge, protobuf decode, retained-frame update, canvas draw and presentation separately.
  - [x] Add the native bench target, three-day fixture generator, bench-only frontend observation, native harness reuse and Nightly report collection. Preserve `integration-plots` and the 32-plot limit. `docs/tooling.md` records the task invocations, resource definitions and unavailable exclusive/physical stages.
  - [x] Run formatting, Python lint/type checking, TypeScript formatting/lint and deterministic stage/refusal/Nightly-report proof.
  - [ ] Rebuild the final fixture-publication identity refinement and complete native desktop case proof with C: above the owner's 50 GB minimum. The preceding optimized producer passed; native setup progressed through the Edge relaunch fix into actual fixture admission. No end-to-end benchmark pass is claimed.
- [ ] 1.3 Capture the baseline for 30/100 mixed plots over millions of sample rows and multi-day histories: cold/warm, live ingest, late samples, zoom, resize, navigation and stalled consumers on native Windows/Linux. Record stage p50/p95, first-visible/all-visible paint, CPU, peak memory, queue depth, bytes and frame misses into Nightly `benchmarks.json`; record refusals and missing stages explicitly, never infer paint from snapshot arrival.
  - [ ] Windows measurements: unavailable for both 30 and 100 plots in every condition. First-visible/all-visible paint, live-update/zoom p95, peak memory and all stage/resource quantiles remain unmeasured. The disk watcher reached 49.9 GiB during fixture admission, before any timed case; the owned run was stopped. Its disposable fixture occupied 14,483,394,015 bytes; automatic approval review rejected recursive cleanup with only `blocked by policy`. No native 100-plot refusal or paint result has been observed. The fixture defines 2,073,600 sample rows at 8 Hz across three days; documents contain respectively 21/91 time series plus one of every other kind.
  - [ ] Linux measurements: pending native Nightly execution. Software presentation opportunities, exclusive encoding/bridge attribution, physical display presentation, OS-cache-cold reads and native/server queue depth retain the documented measurement distinctions; missing stages must remain explicit in the actual baseline.

## 2. Plot-Local Changes and Visible Scheduling

- [ ] 2.1 Implement plot-local viewport/width/settings updates and resize debounce; prove one plot never restarts or requeries unaffected peers. Record before/after restart/query counts, cancelled work, zoom/resize latency and bytes using step 1.
- [ ] 2.2 Implement bounded visible-first work, cancellation of off-screen live interests and fresh snapshots on timed reentry. Support 100 plots and beyond without count-dependent density, replacing grouped message/paint barriers with cut-tagged independent frames. Preserve manual All batches/cuts and refresh progress. Record first/all-visible paint, density at equal width, queue depth, navigation cleanup, CPU/memory and 30/100-plot bytes before/after.

## 3. One Scoped Change Source

- [ ] 3.1 Consolidate plot/card invalidation on Harmos progress/follow facilities with scope, stream/schema or record dependencies, time range and readable frontier; separate accepted-live and durable progress, remove superseded broadcasters and broad state/stream wakeups, and recover gaps through frontier comparison. Prove out-of-scope, unselected-field and filter-excluded samples do not requery unaffected plots. Record wakeups, irrelevant queries, coalescing and CPU before/after.
- [ ] 3.2 Refresh affected visible cards from saved telemetry without a state transition, preserving prior content and frozen report cuts. Prove late samples, schema/record changes, sealing, reconnect and another client's edit; record sample-to-card freshness, refetch counts and frontier recovery latency before/after.

## 4. Coarser Preview Resolutions

- [ ] 4.1 Extend the existing preview owner with coarser stored resolutions, snapshot-consistent coverage and late-update replacement; prove raw parity at fixed cuts, partial edges, uncovered ranges and older cuts after newer writes. Preserve explicit telemetry bucket intervals. Record six-hour/three-day p50/p95, buckets decoded, raw rows/bytes read, ingest throughput/cost and memory before/after.

## 5. Measured Refresh Policy by Kind

- [ ] 5.1 Compare invalidation/refetch with pushed updates for every landed kind plus aggregates/statistics. Default cheap bounded frequency/XY/trajectory/contour/grid/aggregate/statistics results to visible-window refetch; keep time-series bucket deltas only where measured bandwidth justifies them. Record each kind's decision, CPU, bytes, update p50/p95, retained state/queue depth and complexity before/after, including late samples and reconnect. Preserve loaded-data-only playback and exact failures; remove superseded paths and redundant tests.

## 6. Shared Canonical Query Work

- [ ] 6.1 Share in-flight and reusable results by canonical query and identical readable cut across watches/clients, with bounded eviction, independent cancellation and per-client backpressure. Prove two clients cause one computation, cut/scope differences remain isolated, and a stalled/cancelled client cannot delay another. Record query counts, hit/eviction rates, memory, CPU, bytes and fast-client p50/p95 for 1/5/20 clients before/after.

## 7. Statistics and Bounded Fallback Reduction

- [ ] 7.1 Coalesce statistics only for affected visible plots independently of drawing. Prove unchanged/off-screen plots issue no statistics work and drawing never awaits it. Record statistics query counts, exact-median cost, summary age and drawing interference before/after.
- [ ] 7.2 Remove full-selection collection from spilled/stored fallback frame projections, using bounded streaming reducers and admitted exact spill where needed; migrate consumers and remove superseded collection paths. Prove exact raw parity, typed exhaustion and cancellation. Record peak memory versus increasing selected rows at fixed frame/buffer budgets, spill bytes, query p50/p95 and cancellation latency before/after.
- [ ] 7.3 Update affected README, product/query/tooling guidance and benchmark documentation for landed behavior; reconcile protocol/generated consumers and removals. Use step 1's final per-kind stage measurements to verify documented performance claims rather than introducing prose assertions.

## 8. Final Validation and Delivery

- [ ] 8.1 After all source items and targeted proof close, strictly validate all affected OpenSpec changes and run configured focused plot/card/query/GUI checks through project tasks; keep deterministic correctness proof separate from measured timing/resource observations.
- [ ] 8.2 After authorized delivery, obtain native Windows/Linux hosted and Nightly proof of the complete 30/100-plot matrix and `benchmarks.json`; compare every step with baseline, report regressions, failures and pending evidence, and repair affected source before claiming completion.
