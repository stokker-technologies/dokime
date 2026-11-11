# Plot Refresh Scaling

## Why

Documents hold tens of plots over millions of samples, executions run for days,
and several clients watch one backend. Refresh must scale with changed, visible
dependencies rather than the whole document or selected history.

The read-only research report `r4-invalidation.md` (2026-11-11) and the measurements
in `../plot-pipeline/tasks.md`, task 1.1, establish the current behavior:

- One viewport, width or settings change restarts the workspace watch in
  `crates/dokime-gui/src/plots/live-data.ts`. All uses independent unary calls.
  The GUI and server refuse timed workspaces above 32 requests. Selected cards
  remain live off screen; there is no screen-intersection priority.
- The grouped 30-plot diagnostic delivered 597,259 drawing-body bytes, 21,428
  retained points and 30 reduction warnings. A shared message budget lowers
  density as plots are added, despite unchanged panel widths.
- Windows medians per 30 independent watches were 2.172 s / 1,596,238 bytes for
  six-hour snapshots and 76.018 ms / 40,888 bytes for 30-second zooms. Protobuf
  decode plus native-frame adaptation took 11.186 ms; retained delta application
  took 7.230 ms. The fixture has 259,200 sample rows and 7,776,000 numeric values,
  not millions of sample rows. Canvas paint, independent statistics completion,
  Linux and the historical pre-pipeline comparison remain unmeasured.
- Measurement exposed two repaired defects: history-wide resolution discovery
  and per-bucket inflater allocation. The original Windows snapshot medians
  were 6.537 s and 3.283 s respectively. Wide windows still decode every selected
  base-resolution bucket before reducing to panel width. Repeated measurement
  must expose such regressions before they become an operator report.
- `crates/dokime-runtime/src/runtime/query/plots/mod.rs` selects dirty plots by
  stream name before scope, fields and filters; statistics requery unchanged
  eligible plots. Deduplication stops at one grouped query, so separate watches
  and clients repeat query, comparison and encoding work.
- Document invalidation in `crates/dokime-gui/src/app/live/use-runtime-stream.ts`
  follows state changes with a five-second throttle, not saved telemetry. A
  sample-only interval can leave cards stale; query staleTime is not polling.
- `crates/dokime-runtime/src/plot/payload/spilled/{mod,series}.rs` and
  `crates/dokime-runtime/src/plot/stored.rs` can collect the full selection before
  frame reduction. A bounded wire frame does not bound query peak memory.

These are present-source observations, not claims that the pending gRPC crate
layout or cleaned kind catalog has landed. No definite navigation watch leak
was found. Existing bounded transport and per-watch backpressure remain useful.

## What Changes

Implement **after `plot-kinds` lands**, because both changes edit plot code.
The following order is locked; step 1 precedes every optimization, and steps
2–7 each record comparable before/after measurements from that harness.

1. Add a desktop-owned full round-trip benchmark at
   `crates/dokime-gui/benches/system/`, reusing
   `crates/dokime-gui/e2e/native-desktop.mjs` and the repository bench pipeline.
   Measure backend query, encoding/transport including the native bridge,
   protobuf decode, retained-frame update, canvas draw and presentation. Use
   30 and 100 mixed plots over millions of sample rows, cold/warm, live ingest,
   late samples, zoom, resize and navigation, including multi-day history.
   Report stage p50/p95, first-visible/all-visible paint, CPU, peak memory,
   queue depth and bytes on native Windows and Linux into Nightly
   `benchmarks.json`. Record baseline refusals, including the current 100-plot
   limit, rather than modifying limits to obtain a baseline. Keep
   `integration-plots` and its historical proof obligations.
2. Make viewport, width and settings changes plot-local, debounce resize,
   prioritize on-screen cards and stop maintaining off-screen live interests.
   Admit documents beyond 32 plots with bounded scheduling and independent
   per-plot frames; plot count never reduces drawing density. Replace the
   single-message dirty-set/animation-frame barrier explicitly: results retain
   a common captured cut where jointly invalidated, but complete independently.
3. Use one scoped, coalescing change source for plots and live document cards,
   built on Harmos progress and follow facilities. Carry scope, stream and
   schema or record dependency, affected time range and readable frontier;
   distinguish accepted-live from durable evidence. Filter dependencies before
   requerying, refresh saved-evidence cards without state transitions and recover
   notification gaps by frontier comparison. Do not add a parallel row listener
   history or per-plot raw callbacks.
4. Persist/query coarser time-bucket resolutions under the existing preview
   owner. Pick a useful resolution for wide windows, cover partial/uncovered
   edges exactly and update affected resolutions for late samples. Never read
   newer summaries as an older cut or silently coarsen an explicit query interval.
5. Default to invalidate then re-request the visible window for cheap bounded
   frequency, XY, trajectory, contour, grid chart, aggregate and statistics
   results. Evaluate every kind from the benchmark. Retain pushed bucket deltas
   for time series only where measured bandwidth justifies their state and CPU
   cost; record the decision per kind, including bar, pie, histogram and box plot.
6. Compute identical canonical queries at the same readable cut once across
   watches and clients, with bounded in-flight/result retention and eviction.
   Canonical identity includes effective selection, fields, filters, schema,
   resolution and exactness, not display plot ID. Cancellation is independent;
   a stalled client cannot delay another or retain unbounded shared state.
7. Schedule exact statistics only for affected visible plots, coalesced
   independently of drawing. Stream/reduce fallback projections without
   collecting all selected points; bound memory by the output frame and admitted
   working buffers, using bounded spill for exact operations where necessary.

## What Stays

Raw evidence stays authoritative. Exact results, typed failures, fixed-cut
queries, late-sample correctness, revision provenance, cancellation, bounded
transport and backpressure remain. Drawing reductions retain their specified
kind semantics; exact analyses never become approximations. Statistics cannot
delay drawing. Frozen reports stay at their sealed cut and never subscribe.

The viewer still owns its common time window; authored plots and reports gain no
window field. All remains manual: entry or Refresh authorizes one saved-cut
batch, which can be scheduled visible-first and filled lazily at that same cut;
visibility, zoom and telemetry do not start a new All batch. Timed plots resume
with a fresh bounded snapshot/frontier comparison when visible again. Playback
continues using loaded data only. Preserve completed card content while reading.

Storage and wire breaks are acceptable before 1.0. Keep development baselines at
v1, migrate consumers cleanly and preserve incompatible existing roots for an
explicit fresh-root choice; never silently reset data or add legacy adapters.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `plot-refresh`: desktop measurement, plot-local scheduling, independent timed
  delivery, dependency-scoped change source, per-kind refresh and statistics.
- `plot-window`: visible-first refresh ownership while preserving manual All.
- `telemetry-query`: exact coarser preview resolutions and bounded fallback
  frame reduction, without changing explicit bucket query intervals.
- `query-snapshots`: bounded sharing of canonical work at identical readable cuts.
- `document-query-evaluation`: cards follow scoped durable telemetry progress.
- `plot-pipeline`: replace the unconditional pushed-delta requirement with the
  measured per-kind policy. This delta applies after that capability lands.
  Archive `plot-pipeline` before archiving this change; until then OpenSpec can
  strictly validate this dependent delta but cannot fold it into the main specs.

## Impact and Related Ownership

Current source inventory: GUI `src/plots/{live-data,live-frames}.ts`,
`src/plots/CartesianPlot.tsx`, document workspace/grid/presentation components,
`src/app/live/use-runtime-stream.ts`, `src/lib/api/native-{plots,documents}.ts`,
native bridge `src-tauri/src/rpc/calls.rs`, desktop bench/harness; runtime
`runtime/query/plots/`, `plot/{preview,payload,stored.rs}`, telemetry
`storage/sqlite/data/telemetry/{flush,writer,previews}.rs`, document query/capture;
current server `src/grpc/plots{.rs,/cursor.rs,/deltas.rs,/wire.rs}` and documents;
protocol definitions/generated bindings, focused proof, bench reporting tasks,
Nightly reporting and affected README/product/query/tooling guidance. This change
authors only specification files; implementation must resolve relocated paths.

- `plot-pipeline` owns existing buckets, width-aware queries, columnar transport
  and private kernels. Its historical baseline and Linux obligations stay there.
  Coarser resolutions and refresh scaling move here; no duplicate implementation
  task is added there. This change supersedes its permissive selection-sized
  memory direction for fallback reductions and its unconditional delta policy.
- `plot-kinds` owns catalog, contour/grid frame semantics, SDK/report consumers
  and loaded-data playback. It lands first; this change changes refresh policy,
  not kind definitions, grid reduction, contour masking or playback behavior.
- `storage-interface` owns upstream generic storage capabilities and eventual
  extraction after gRPC wave 5. Reuse its progress, projection, cut and late-update
  contracts; do not create another storage owner or perform extraction here.
  Publish and pin any required Harmos extension before final validation.
- `grpc-plugins` owns operation/protocol migration and removal of the server
  adapter crate. Implement refresh in whichever owner has landed, migrating its
  consumers with it; the paths above are evidence, not a requirement to retain
  that crate. This change neither starts nor finishes those separate changes.

The report also found that clean watch EOF marks a plot stale without using the
error-reconnect path. Its intended terminal-versus-recoverable semantics were
not part of the agreed seven steps; no new EOF recovery contract is invented here.
