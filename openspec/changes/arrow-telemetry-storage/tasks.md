# Arrow IPC Telemetry Storage Tasks

## 1. Contract And Activation

- [x] 1.1 Record all nine owner-locked decisions, the source inventory and the impossibility evidence in proposal/design/spec artifacts; verify OpenSpec structure with strict validation.
- [ ] 1.2 Obtain owner resolution of decision 3's O(1) footer conflict; verify the approved wording against the Arrow IPC footer schema before any implementation.
- [ ] 1.3 Activate `arrow-telemetry-storage` through the Hub and create/resume the native persistent goal after contract resolution; verify the marker names this branch and change.

## 2. Configuration, Schema And Catalog

- [ ] 2.1 In runtime `src/config/storage.rs`, `src/config/storage/defaults.rs` and configuration consumers, add configured row/time flush thresholds and validation; verify focused policy tests without literals at call sites.
- [ ] 2.2 In runtime `src/storage` and Cargo dependency/lock owners, implement contract-to-Arrow mapping and metadata; verify every value type including nested arrays/objects, unsigned boundaries, nullability and UTC microseconds in round-trip tests.
- [ ] 2.3 Replace stored-file ownership with `files`/`artifacts`, add streams/batch indexes and retain numeric summaries in runtime SQLite schema/file/projection owners; verify catalog joins, telemetry exclusion from artifacts and refusal of incompatible roots without data loss.

## 3. Writer, Durability And Recovery

- [ ] 3.1 Implement per-execution/stream and session file ownership, ZSTD batches and row/time flush scheduling under runtime `src/storage`; verify both triggers, durable flush and Windows close-before-rename/delete behavior.
- [ ] 3.2 Integrate batch fsync/catalog commit/saved-cursor ordering with runtime execution evidence and transaction owners; verify failure injection never acknowledges uncommitted or unflushed rows.
- [ ] 3.3 Implement finalize and footerless recovery under runtime storage lifecycle; verify recovered complete batches, partial-write handling and the owner-approved finalize complexity without payload rewriting.

## 4. Consumers And Removal

- [ ] 4.1 Replace runtime storage/query/output/report telemetry readers with overlapping batch reads plus unflushed tail; verify windows, out-of-order times, pagination, no duplicates and saved-versus-live visibility.
- [ ] 4.2 Delete obsolete telemetry payload read/write paths, per-signal tables and indexes, retaining unrelated evidence and summaries; verify source searches and focused storage/durability/recovery tests.
- [ ] 4.3 Review domain/operation/client and GUI consumers; migrate all affected bindings/examples if their contracts change and read `docs/gui.md` before GUI edits; verify focused consumer tests and generated-contract consistency where affected.

## 5. Exports And Documentation

- [ ] 5.1 Replace runtime `src/domains/exports/evidence/{database,archive,snapshot}.rs` and cycle export projection with relational catalogs and original Arrow file copies; delete partition tables, column splitting and `stream_columns`; verify execution/cycle paths, manifest hashes and byte identity.
- [ ] 5.2 Update runtime/server export fixtures and `packages/dokime-client/tests/integration/client/test_exports.py`; verify exported Arrow with available project polars or pyarrow and evidence.sqlite joins through focused mise tasks.
- [ ] 5.3 Update `docs/product-guide.md`, README claims/examples if affected, export-related remote guides and `docs/exports.md` if present in the delivery base; verify documentation matches final behavior by review. The initial origin/dev has no `docs/exports.md`.

## 6. Benchmark Source And Closure

- [ ] 6.1 Extend the existing runtime bench harness (`benches/storage` and related unit/integration/system owners as appropriate) to compare origin/dev with Arrow writes, 1M-sample window query, finalize and export; verify equivalent inputs and reproducible before/after output without running the benchmark yet.
- [ ] 6.2 Close every implementation item and targeted proof in the combined branch, including removal and consumer inventory; only then declare SOURCE COMPLETE. Verify no unimplemented contract or unhanded-off writer remains.

## 7. Final Validation And Local Handback

- [ ] 7.1 Run requested workspace checks and formatting through mise after source closure, plus TypeScript check/lint if GUI changed; record actual results without umbrella lanes or installers.
- [ ] 7.2 Run the existing project bench task once at the end with before/after comparison; report write throughput, 1M-sample window latency, finalize and export numbers, including unresolved failures.
- [ ] 7.3 Validate the complete OpenSpec change strictly, commit owned changes in small Conventional Commits and hand back status, added/deleted lines, benchmark results, tests and open issues in at most ten lines; do not push or merge.

## Workflow Follow-Up

- Implementation has not started: task 1.2 blocks subsequent implementation under the owner's explicit impossibility rule.
- Preserve this tracked change until the feature is complete; archive through the Hub only when all required tasks are actually closed.
- Experimental integration requires the owner's go; no remote delivery is authorized.
