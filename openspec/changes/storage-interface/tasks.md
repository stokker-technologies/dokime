# Storage Interface Tasks

The extraction is landed; remaining backend proof is retained here.
[operation-boundary](../operation-boundary/tasks.md) owns the replacement of
direct Store access, compound mutations, the typed facade and all aggregation.
Checked supersession pointers record transferred scope, never passing proof.

## 1. Prerequisites and Baseline

- [ ] 1.1 Recheck surviving backend capabilities, consumers, configuration, tests and benchmarks after the landed extraction; operation-boundary owns the direct Store inventory and migration. Preserve remaining native durability and backend-contract proof.
- [ ] 1.2 Establish native Linux and Windows before-change baselines on the post-gRPC host path for storage, telemetry, plots, documents, finalization and reports; replace obsolete sidecar-ingest workloads and add deterministic multi-day/million-sample datasets.
- [ ] 1.3 Agree the upstream Harmos capability contracts, snapshot/frontier/fallback semantics and backend support matrix before implementing them in Harmos's own change.

## 2. Harmos Extensions First

- [x] 2.1 The adopted backend implements bounded columnar scans with projection and saved-position selection. Focused pruning, fallback, parity and cancellation proof remains in 5.1 and native final validation.
- [x] 2.2 The adopted backend implements index definitions and ordered entry scans at retained snapshots. Predicate/prefix, stable continuation and cancellation proof remains in 5.1 and native final validation.
- [x] 2.3 The adopted backend implements projection definitions, status/frontiers, output publication and retained projection reads. Commit-coupling/replay/invalidation proof remains in 5.2 and native final validation.
- [ ] 2.4 Superseded by [operation-boundary tasks 8–9](../operation-boundary/tasks.md) for bucket aggregation/merge ownership. Before closing this transfer, explicitly inventory late-sample replacement, changed bucket keys/revisions, moments/source positions and live-versus-saved parity in the replacing change; those detailed obligations remain owned here until then.
- [x] 2.5 The adopted backend implements object compare/exchange and persisted mutation receipts. Conflict/absence and durable restart proof remains in 5.2 and native final validation.
- [ ] 2.6 Preserve required existing backend/state/file/receipt capabilities, run focused upstream proof, publish the upstream revision and pin it in Dokime before final validation; do not expose Dokime-specific APIs in Harmos.

## 3. Break Reverse Dependencies

- [x] 3.1 Preview ingestion/checkpoint modules are under dokime-storage, which has no product runtime, operations or rendering dependency. Harmos bucket-summary adoption remains in operation-boundary 8–9.
- [x] 3.2 Retired domain history projections are replaced by protocol definitions and storage-local execution/history shapes; callers no longer import the deleted domain/runtime storage types.
- [x] 3.3 Persistence adapters and durable shapes live in dokime-storage; live execution, prompt and report managers stay in runtime. Direct Store access replacement remains owned by operation-boundary.
- [x] 3.4 Superseded by [operation-boundary tasks 8–9](../operation-boundary/tasks.md): expression bindings compose Harmos transforms; aggregation/folds belong in Harmos and product kernels retain only their product algorithms.
- [x] 3.5 Retired ExecutionView/CycleView and domain types are removed; protocol owns exchanged values and storage owns durable shapes. The remaining direct caller boundary is tracked in operation-boundary 11.

## 4. Extract Application Persistence

- [x] 4.1 Application persistence is extracted into dokime-storage with the Arrow/SQLite backend and protocol dependencies; retired runtime storage directories are gone. Native recovery proof and operation adoption remain open.
- [x] 4.2 Superseded by [operation-boundary tasks 3–11](../operation-boundary/tasks.md): operations and runtime reach stored product data only through in-process Harmos operations, preserving history, seals, reports, artifacts and fresh-root refusal.
- [x] 4.3 Superseded by [operation-boundary tasks 2 and 6](../operation-boundary/tasks.md): compound mutations provide the cross-method transaction; preserve barriers, recovery, receipts, paging and revision conflicts through those operations.
- [ ] 4.4 Verify each named exception stays with its owner: logs, support bundles, exports/report spools, plugin installation, desktop root lease/backup, client pairing/downloads and command-line/build tooling.
- [ ] 4.5 Delete superseded modules/adapters/accessors and migrate meaningful tests, manifests, examples, README and guides; enforce storage's absence of runtime, operations, rendering and domain dependencies.

## 5. Focused Proof

- [ ] 5.1 Prove query parity, native columnar pruning, ordered seek pages, saved-cut/live-tail semantics, late replacement and execution/step-range statistics through Harmos transforms, using deterministic data and the accepted Harmos f64 RMS/standard-deviation semantics.
- [ ] 5.2 Prove projection replay/frontiers, settings conflicts, capture/artifact ownership, interrupted finalization/seals, report dependencies and fresh-root refusal without silent resets.
- [ ] 5.3 Compare before/adoption/extraction benchmarks on Linux and Windows, including multi-day data, concurrent ingestion/read, restart and late samples; report throughput, first-page latency, stop/cancel/report time, peak memory, bytes read and durability cost, and resolve regressions.

## 6. Final Validation

- [ ] 6.1 Close every source item and focused proof, validate this change strictly, and complete combined native/platform and hosted validation through project tasks when authorized; keep actual failures and pending proof explicit.
- [ ] 6.2 Confirm final dependency boundaries, exception ownership, upstream revision pin and before/after multi-day benchmark evidence before retiring the change.
