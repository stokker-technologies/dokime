# Plot Pipeline Tasks

## 1. Contract

- [ ] 1.1 Record a baseline on Windows and Linux for a windowed live plot and a zoom over the three-day, thirty-plot workload: query time, bytes sent and redraw time.
  - The pre-change baseline was never taken. The first pipeline implementation commit is `4ae27bfb4` (`feat(plots): persist indexed window buckets and running moments`); its parent `ad4373ce6e143d480a8e138c50cc33e8dccbc1f9` is the comparison revision. A before-change run must adapt the benchmark to that revision's request and frame contracts while preserving the fixture and windows.
  - Measurement owner: `crates/dokime-server/benches/plots/`, target `integration-plots`. Build with `mise run build -- --artifact bench --binary integration-plots`, then run `mise run bench -- --language rust --layer integration --binary integration-plots --refresh-baseline`. Refresh seeds native Criterion `dev` files for this implementation; it does not create historical before-change evidence.
  - Fixture: 259,200 rows at 1 Hz from `2026-01-01T00:00:00Z`, one stream with thirty numeric columns and thirty distinct time-series requests (7,776,000 values). Values and identifiers are deterministic. Live viewport: the last six hours; zoom: thirty seconds ending two minutes before the final sample; both use a 960-pixel panel. Fixture ingestion and flushing are outside the measurements. Default operation policies and storage settings are unchanged. Criterion uses ten flat samples, following the existing heavy storage/runtime benchmarks; each query sample includes all thirty requests.
  - Query measurements cover one sequential pass of thirty independent live `Watch` requests through their drawing snapshots, including request cloning, local gRPC transport, response encoding and client decode. The timer stops at snapshot arrival; watch closing is outside it. Exact statistics use the production independent path and are not awaited. Reported byte counts are drawing-frame protobuf bodies, excluding pending/statistics/control frames and gRPC/HTTP/2 framing. Interface decode covers protobuf decode plus the production native-frame adapter across thirty separate watch snapshots. Delta application calls the production retained-frame owner for thirty late-sample bucket replacements captured one plot at a time, replaying the same delta against the original snapshot each iteration. Vite/Node startup and pipe round trips are outside these interface timings. After query measurements, each separate watch receives one additional fixed late sample at a half-second offset; its snapshot frontier includes the earlier plots' late samples. These measure interface frame processing in Node; canvas redraw is not measured.
  - A single thirty-plot unary reply at this viewport exceeds the default transport limit. The benchmark measures thirty independent live watch requests, preserving the panel width, data density and all operation limits. The unary grouped-reply limitation is not a timing result.
  - [x] Measure the after-change workload on Windows, including request latency, drawing-frame bytes, interface decode and retained bucket-delta application. Run: 2026-11-11, Windows 11 Home 10.0.26300, AMD Ryzen 9 5950X (16 cores), Rust 1.99.0 (`b940084d7`), Node 26.11.1. Production source: `f2975e4c633ecdace5b319ccd8f9d4b9086132a3` plus the benchmark added here. The focused bench task exited successfully with four native results; estimates below are Criterion medians with 95% confidence intervals, per thirty-plot pass.

    | Measurement | Median | 95% Confidence Interval | Drawing-Frame Protobuf Bytes |
    | --- | ---: | ---: | ---: |
    | Six-Hour Live Snapshot | 6.537 s | 6.500–6.597 s | 1,596,235 |
    | Thirty-Second Zoom Snapshot | 3.283 s | 3.258–3.316 s | 40,887 |
    | Interface Snapshot Decode | 10.746 ms | 10.699–11.302 ms | Uses Captured Snapshots |
    | Retained Bucket-Delta Application | 7.047 ms | 6.976–7.154 ms | 19,484 Across Thirty Deltas |

  - These are after-change measurements on Windows only. Native estimates are under `target/benchmarks/criterion/` for `integration-plots`, group `plot_pipeline_three_days_thirty_plots`, cases `live_snapshot_21600s`, `zoom_snapshot_30s`, `decode` and `apply_delta`. Earlier local after-change calibration runs are not a pre-change baseline; native comparison percentages are not evidence of a pipeline speedup. Linux, the before-change run at the revision above and actual canvas redraw remain unmeasured, so task 1.1 remains open.
  - Fixed-cost investigation, Windows, 2026-11-11: one live and one zoom request were profiled through the same benchmark fixture and local gRPC client. These are diagnostic single-request durations, not Criterion medians. Native product spans are in `target/benchmarks/plot-profile-before/`; the scope is the client call through the first drawing snapshot. The thirty-second viewport includes both endpoints (31 fixture samples).

    | Request Stage | Six-Hour Live (ms) | Thirty-Second Zoom (ms) |
    | --- | ---: | ---: |
    | Stream Viewing Lease Setup | 0.030 | 0.013 |
    | Saved/Admission Snapshot Cut | 0.002 | 0.002 |
    | State Context Capture | 0.026 | 0.004 |
    | Stream/Stored Schema Resolution | 0.626 | 0.475 |
    | Latest-Time Preview Decode | 0.186 | 0.179 |
    | Admission Tail Check | 0.541 | 0.390 |
    | Preview Width Discovery, Twice | 101.256 | 95.414 |
    | Window Bucket SQLite Read and Zlib/JSON Decode | 113.692 | 0.252 |
    | Raw-Range Index Lookup, Excluding Width Discovery | 9.852 | 0.085 |
    | Raw Edge/Detail Arrow Read and Sample Decode | 11.560 | 4.691 |
    | Remaining Window Assembly, Reduction and Internal Awaits | 3.307 | 0.300 |
    | Snapshot Frame Conversion and Protobuf Size Checks | 0.381 | 0.050 |
    | Snapshot Enqueue | 0.076 | 0.014 |
    | Remaining Dispatch, Encoding, Transport and Client Decode | 2.745 | 1.315 |
    | **Client Call Through Drawing Snapshot** | **244.280** | **103.184** |

  - The fixed cost was two `SELECT DISTINCT width` queries over every retained bucket for the selected execution/stream/field. The zoom was scanning its preview index history, not reading the entire Arrow stream. Actual detail reads were time-pruned by the Arrow batch catalog. Live drawing also decodes the 21,600 closed one-second buckets inside its requested window; this is window-proportional work. Exact statistics are spawned after drawing is enqueued and overlap client delivery; they are not awaited by the snapshot path and are excluded from the critical-path stage sums above. Encoding and transport are bounded by the drawing frame; the remaining row includes the final transport encoder and client decoder, which run outside the plot handler spans.
  - Width discovery now seeks the next resolution using the existing `(execution,stream,field,width,bucket)` primary key. A deterministic regression counts SQLite VM instructions through both production window lookups while retained history grows 64-fold; it does not assert elapsed time. No schema, persisted bytes, limits or deadlines change.
  - First repair, Windows Criterion medians per thirty-plot pass: width seeks reduce live snapshots from 6.537 s to 4.060 s (95% CI 4.031–4.131 s), and zoom snapshots from 3.283 s to 92.800 ms (90.600–105.845 ms). Drawing bodies remain 1,596,239 and 40,885 bytes respectively; timestamp text lengths can change these totals by a few bytes. Native estimates are preserved in `target/benchmarks/plot-fixed-cost-indexed/` and the original estimates in `target/benchmarks/plot-fixed-cost-before/`. Optional product tracing now starts only after all four timing cases: set `DOKIME_PLOT_PROFILE=1` and `DOKIME_TEST_TRACE_DIR` to an owned directory when running the same bench task. The trace and grouped-watch diagnostic add no work to any timing case. These final diagnostics run after the fixture's late updates; the original attribution above was captured before late updates.
  - Desktop transport audit: `crates/dokime-gui/src/plots/live-data.ts` calls finite `QuerySnapshots` once per plot, so the oversized thirty-plot unary reply is not its finite document path. Timed plots do share one `Watch`. The server divides the revision budget between its plots and fits each frame, preserving an explicit reduction warning. The benchmark's thirty-plot live watch succeeds with 596,679 drawing-body bytes and 21,421 retained points, with thirty warnings; density is lower than the independent-watch measurements. This is the existing transport-fitting behavior, not a transport-limit exception. No limits or client calls were changed.
  - [x] Fixed-cost repairs measured again on the same Windows workstation, with the same fixture, windows, 960-pixel panels, ten flat Criterion samples and default policies. The second repair reuses one inflater and JSON buffer per window instead of allocating them for every stored bucket; stored Zlib/JSON bytes are unchanged. The final run has tracing disabled and performs its grouped-watch diagnostic after every timed case.

    | Measurement, per Thirty Plots | Original Windows Run | Width Seeks, Windows | Final Windows Run | Final 95% Confidence Interval | Final Drawing-Frame Bytes |
    | --- | ---: | ---: | ---: | ---: | ---: |
    | Six-Hour Live Snapshot | 6.537 s | 4.060 s | 2.172 s | 2.167–2.183 s | 1,596,238 |
    | Thirty-Second Zoom Snapshot | 3.283 s | 92.800 ms | 76.018 ms | 75.286–76.761 ms | 40,888 |
    | Interface Snapshot Decode | 10.746 ms | 12.450 ms | 11.186 ms | 11.080–11.276 ms | Uses Captured Snapshots |
    | Retained Bucket-Delta Application | 7.047 ms | 8.115 ms | 7.230 ms | 6.991–7.290 ms | 19,484 Across Thirty Deltas |

  - Final drawing latency averages 72.406 ms per live plot and 2.534 ms per zoom plot across those median passes, versus 217.909 ms and 109.431 ms originally. Native final estimates are preserved under `target/benchmarks/plot-fixed-cost-final/`; the ordinary Criterion location also contains the final estimates. The first decoder-reuse run, preserved under `target/benchmarks/plot-fixed-cost-reused/`, measured 2.370 s live and 80.724 ms zoom, with slower interface cases. Diagnostics were moved after all timed cases and the complete benchmark was rerun to isolate measurement activity. Interface timings are recorded as measured; no interface implementation changed and no interface speedup over the original Windows baseline is claimed. Both before and after measurements in this comparison are Windows only.
  - The current grouped-watch diagnostic runs after the thirty late updates and succeeds with 597,259 bytes, 21,428 retained points and thirty reduction warnings. `DocumentWorkspaceView.tsx` passes all visible document plots to `useLivePlotPayloads`; its timed path can therefore request thirty plots together. Its finite path still sends one plot per unary call. The grouped unary limit remains an explicit refusal for callers that choose that shape, while the desktop's grouped live path preserves the existing budget by reducing drawing density.
  - Remaining design cost: a wide window still reads every stored base-resolution bucket inside that window before reducing it to panel width. The fixed history-wide work is removed, but these measurements do not prove the general one-second-per-million-samples target. A concrete follow-up is to persist coarser summary resolutions under the existing preview owner and seek the resolution nearest the requested panel width, retaining raw reads for partial/newer buckets and exact statistics. That needs ingestion, late-update and fixed-cut coverage design and proof; it is not implemented here. Linux, the historical pre-pipeline comparison and actual canvas redraw remain open as stated above.
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
  - [x] Windows focused proof: runtime `domains::telemetry::plots::series_window_summaries` and server `grpc::plots::deltas::tests::bucket_deltas_replace_late_samples_and_expire_old_buckets` passed (two Rust tests); interface `tests/unit/plots` passed (seventeen tests in five files). The after-change bench above completed all four cases.
  - [x] `mise run format`, `mise run check -- --language rust` and `mise run check -- --repository` passed. Python tooling checks also passed, including ninety-five tooling tests. No full test lane was run. Rust checking emitted a nonfatal warning about the unreachable `tokio::process::Command::new` entry in the existing Clippy configuration.
  - [x] Windows fixed-cost follow-up: 34 exact Rust selections passed unchanged apart from two added guards: nine runtime unit tests, nineteen runtime integration tests, four server unit tests and two server integration tests. This includes every test in `domains::telemetry::plots`, `all_plot_previews`, `windowed_plot_zoom_and_late_sample`, `telemetry_watch_plot`, live plot cadence, columnar wire limits and bucket deltas. The new guards count SQLite VM work as retained history grows and read independent compressed checkpoints through a reused decoder, including reset after corruption. `mise run format`, `mise run check -- --language rust` and `mise run check -- --repository` passed. The statistics return type was named to repair the type-complexity finding from Rust checking. No full test lane was run.
  - Linux proof and the before-change comparison run are still required. Interface decode and retained-frame update are measured through the existing Vite harness; canvas redraw needs a browser measurement and is outside this bench.

## Workflow Follow-Up

- `plot-kinds` owns kind cleanup, two-dimensional contour binning, grid frames and playback with their report/SDK/plugin consumers; the completed transport work here remains its dependency.
- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- Run only focused tasks until combined source closure; final native proof remains separate.
