# Storage Interface Tasks

## 1. Prerequisites and Baseline

- [ ] 1.1 After `grpc-plugins` wave 5 closes, inventory surviving producers, consumers, persisted types, concrete-state escapes, configuration, tests, benchmarks, README and guidance against the proposal; lock ownership and source scope.
- [ ] 1.2 Establish native Linux and Windows before-change baselines on the post-gRPC host path for storage, telemetry, plots, documents, finalization and reports; replace obsolete sidecar-ingest workloads and add deterministic multi-day/million-sample datasets.
- [ ] 1.3 Agree the upstream Harmos capability contracts, snapshot/frontier/fallback semantics and backend support matrix before implementing them in Harmos's own change.

## 2. Harmos Extensions First

- [ ] 2.1 Implement bounded native columnar scans with projection, pruning, continuation and explicit fallback in Harmos; prove parity and cancellation.
- [ ] 2.2 Implement indexed search with stable ordered seek pages and predicate/prefix support; prove snapshot and continuation behavior.
- [ ] 2.3 Implement durable materialized projections with commit-coupled advancement, readable frontier, replay and invalidation; prove restart/snapshot consistency.
- [ ] 2.4 Implement time buckets as a query transform with late-sample replacement, changed keys/revisions, moments and source positions; prove live and saved-cut parity.
- [ ] 2.5 Implement atomic conditional update with expected-revision conflict/absence behavior and durability receipts.
- [ ] 2.6 Preserve required existing backend/state/file/receipt capabilities, run focused upstream proof, publish the upstream revision and pin it in Dokime before final validation; do not expose Dokime-specific APIs in Harmos.

## 3. Break Reverse Dependencies

- [ ] 3.1 Move preview index, batch/checkpoint and ingestion-coupled projection advancement below plot request/context dependencies.
- [ ] 3.2 Relocate history `CaseHeader`, `CaseProjection`, status projection and evidence selections to protocol definitions or storage-local read shapes as specified; migrate all callers.
- [ ] 3.3 Split persistence adapters from `RuntimeHandle`, transactions, capture admission and report-operation calls using explicit owned inputs/results; keep transactions, state, cycles and effects together in runtime.
- [ ] 3.4 Remove storage/document definition cycles: use protocol messages, keep expression evaluation with applying operations, move the storage-used fold beside aggregation and pass explicit storage settings.
- [ ] 3.5 Relocate the proposal's remaining `ExecutionView`/`CycleView`, capture/acquisition, artifact, query, metadata, plot and moments types to their stated owners; replace all runtime re-exports and concrete-state escapes without rebuilding a domain layer.

## 4. Extract Application Persistence

- [ ] 4.1 Merge `crates/dokime-runtime/src/storage/` and `src/runtime/storage/` into `dokime-storage`, including backend/codecs, persisted shapes, settings/metadata, snapshots, reads and durable projections through the Harmos interface.
- [ ] 4.2 Migrate operations/runtime query consumers to storage resources; retain Dokime execution/step statistics, history filters/tokens, finalization/seals, report capture, artifact inventory and fresh-root refusal on generic primitives.
- [ ] 4.3 Preserve required durability barriers, recovery frontiers, replay-idempotence, bounded files/paging and conditional settings updates; never assume independent backend methods form a transaction.
- [ ] 4.4 Verify each named exception stays with its owner: logs, support bundles, exports/report spools, plugin installation, desktop root lease/backup, client pairing/downloads and command-line/build tooling.
- [ ] 4.5 Delete superseded modules/adapters/accessors and migrate meaningful tests, manifests, examples, README and guides; enforce storage's absence of runtime, operations, rendering and domain dependencies.

## 5. Focused Proof

- [ ] 5.1 Prove query parity, native columnar pruning, ordered seek pagination, saved-cut/live-tail semantics, late-sample replacement and exact execution/step-range statistics with deterministic data.
- [ ] 5.2 Prove projection replay/frontiers, settings conflicts, capture/artifact ownership, interrupted finalization/seals, report dependencies and fresh-root refusal without silent resets.
- [ ] 5.3 Compare before/adoption/extraction benchmarks on Linux and Windows, including multi-day data, concurrent ingestion/read, restart and late samples; report throughput, first-page latency, stop/cancel/report time, peak memory, bytes read and durability cost, and resolve regressions.

## 6. Final Validation

- [ ] 6.1 Close every source item and focused proof, validate this change strictly, and complete combined native/platform and hosted validation through project tasks when authorized; keep actual failures and pending proof explicit.
- [ ] 6.2 Confirm final dependency boundaries, exception ownership, upstream revision pin and before/after multi-day benchmark evidence before retiring the change.
