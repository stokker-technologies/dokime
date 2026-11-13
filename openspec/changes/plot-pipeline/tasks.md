# Plot Pipeline Tasks

## 1. Contract

- [ ] 1.1 Record a baseline on Windows and Linux for a windowed live plot and a zoom over the three-day, thirty-plot workload: query time, bytes sent and redraw time.
  - Measurement owner: `crates/dokime-server/benches/plots/`, target `integration-plots`. Build with `mise run build -- --artifact bench --binary integration-plots`, then run `mise run bench -- --language rust --layer integration --binary integration-plots --refresh-baseline`. Refresh seeds native Criterion `dev` files for this implementation; it does not create historical before-change evidence.
  - Fixture: 259,200 rows at 1 Hz from `2026-01-01T00:00:00Z`, one stream with thirty numeric columns and thirty distinct time-series requests (7,776,000 values). Values and identifiers are deterministic. Live viewport: the last six hours; zoom: thirty seconds ending two minutes before the final sample; both use a 960-pixel panel. Fixture ingestion and flushing are outside the measurements. Default operation policies and storage settings are unchanged. Criterion uses ten flat samples, following the existing heavy storage/runtime benchmarks; each query sample includes all thirty requests.
  - Query measurements cover one sequential pass of thirty independent live `Watch` requests through their drawing snapshots, including request cloning, local gRPC transport, response encoding and client decode. The timer stops at snapshot arrival; watch closing is outside it. Exact statistics use the production independent path and are not awaited. Reported byte counts are drawing-frame protobuf bodies, excluding pending/statistics/control frames and gRPC/HTTP/2 framing. Interface decode covers protobuf decode plus the production native-frame adapter across thirty separate watch snapshots. Delta application calls the production retained-frame owner for thirty late-sample bucket replacements captured one plot at a time, replaying the same delta against the original snapshot each iteration. Vite/Node startup and pipe round trips are outside these interface timings. After query measurements, each separate watch receives one additional fixed late sample at a half-second offset; its snapshot frontier includes the earlier plots' late samples. These measure interface frame processing in Node; canvas redraw is not measured.
  - A single thirty-plot unary reply at this viewport exceeds the default transport limit. The benchmark measures thirty independent live watch requests, preserving the panel width, data density and all operation limits. The unary grouped-reply limitation is not a timing result.
  - [x] Measure the after-change workload on Windows, including request latency, drawing-frame bytes, interface decode and retained bucket-delta application.
  - [x] Fixed-cost repairs measured again on the same Windows workstation, with the same fixture, windows, 960-pixel panels, ten flat Criterion samples and default policies.
- [x] 1.2 Lock the source inventory: `crates/dokime-runtime/src/{plot,runtime/query/plots,storage/sqlite/data/telemetry}`, runtime signal kernels and their evidence, query and report consumers, the plot-bucket format check in `crates/dokime-runtime/src/storage/sqlite/data.rs`, plot contracts in `crates/dokime-domain/src/model/plots`, `crates/dokime-server/src/grpc/{plots,plots.rs}`, `crates/dokime-gui/src/{plots,components/document/PlotGroupGrid.tsx,stores/summary-window-store.ts,lib/api/native-plots.ts,lib/types}`, generated plot bindings, their focused tests and benches, Cargo dependencies, focused feature selection in `tools/python/` and `docs/tooling.md`, and affected plot documentation in `README.md` and `docs/`. Implement stored buckets, viewport queries, watch deltas, kernels, columnar wire and interface in that order. Preserve other workers' branding and remote access ownership. Final validation remains separate.

## 2. Backend

- [x] 2.1 Extend the stored previews to time buckets that answer a window, with first, last, minimum, maximum and running moments.
- [x] 2.2 Accept the visible range and panel width in a plot request and bucket by time to that width.
- [x] 2.3 Send changed buckets from the plot watch after its snapshot, with late samples replacing their bucket, and deliver exact statistics separately.
- [x] 2.4 The kernels stay hand-maintained and move into Dokime; `tsdownsample` was tried and dropped because it needs nightly Rust. Own the FFT, statistics and downsampling kernels privately, remove the dependency and add focused Cargo feature selection with tests and tooling documentation.

## 3. Wire and Interface

- [x] 3.1 Send one columnar frame with a shared time column and delete the duplicate point-series representation.
- [x] 3.2 Send the viewport and width from the interface, request detail on zoom and apply bucket deltas to the retained frame.

## 4. Final Validation

- [ ] 4.1 Run focused plot query, watch and interface tests, and repeat the baseline of task 1.1 with the results recorded beside it.
  - [x] Windows focused proof: runtime and server window-summary/bucket-delta tests, interface plot tests and the four-case after-change benchmark.
  - [x] Run formatting, Rust and repository checks, and focused Python tooling proof.
  - [x] Windows fixed-cost follow-up: exact Rust selections, deterministic SQLite work and decoder-reuse guards, format, Rust and repository checks.
  - [ ] Complete Linux proof and the before-change comparison run. Interface decode and retained-frame update are measured through the existing Vite harness; canvas redraw needs a browser measurement and is outside this bench.

## Workflow Follow-Up

- `plot-kinds` owns kind cleanup, two-dimensional contour binning, grid frames and playback with their report/SDK/plugin consumers; the completed transport work here remains its dependency.
- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- Run only focused tasks until combined source closure; final native proof remains separate.
