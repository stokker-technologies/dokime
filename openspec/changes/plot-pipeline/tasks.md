# Plot Pipeline Tasks

## 1. Contract

- [ ] 1.1 Record a baseline on Windows and Linux for a windowed live plot and a zoom over the three-day, thirty-plot workload: query time, bytes sent and redraw time.
- [x] 1.2 Lock the source inventory: `crates/dokime-runtime/src/{plot,runtime/query/plots,storage/sqlite/data/telemetry}`, plot contracts in `crates/dokime-domain/src/model/plots`, `crates/dokime-server/src/grpc/plots`, `crates/dokime-gui/src/{plots,lib/api/native-plots.ts,lib/types/plot.ts}`, generated plot bindings, their focused tests and benches, Cargo dependencies, and affected plot documentation in `README.md` and `docs/`. Implement stored buckets, viewport queries, watch deltas, kernels, columnar wire and interface in that order. Preserve other workers' branding and remote access ownership. Final validation remains separate.

## 2. Backend

- [x] 2.1 Extend the stored previews to time buckets that answer a window, with first, last, minimum, maximum and running moments.
- [x] 2.2 Accept the visible range and panel width in a plot request and bucket by time to that width.
- [ ] 2.3 Send changed buckets from the plot watch after its snapshot, with late samples replacing their bucket, and deliver exact statistics separately.
- [ ] 2.4 Replace the downsampling kernels with `tsdownsample` and move FFT into Dokime.

## 3. Wire and Interface

- [ ] 3.1 Send one columnar frame with a shared time column and delete the duplicate point-series representation.
- [ ] 3.2 Send the viewport and width from the interface, request detail on zoom and apply bucket deltas to the retained frame.

## 4. Final Validation

- [ ] 4.1 Run focused plot query, watch and interface tests, and repeat the baseline of task 1.1 with the results recorded beside it.

## Workflow Follow-Up

- Try a query engine over Arrow in its own worktree once this lands; adopt it only if build times stay acceptable.
- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
