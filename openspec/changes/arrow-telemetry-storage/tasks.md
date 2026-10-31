# Arrow IPC Telemetry Storage Tasks

## 1. Contract And Activation

- [x] 1.1 Record all nine owner-locked decisions, the source inventory and the impossibility evidence in proposal/design/spec artifacts; verify OpenSpec structure with strict validation.
- [x] 1.2 Record the owner's 2026-10-30 amendment approving metadata-only O(batch count) footer emission without payload scans, decoding or rewriting; verify against the Arrow IPC footer schema.
- [x] 1.3 Activate `arrow-telemetry-storage` through the Hub and create/resume the native persistent goal after contract resolution; verify the marker names this branch and change.

## 2. Configuration, Schema And Catalog

- [x] 2.1 In runtime `src/config/storage.rs`, `src/config/storage/defaults.rs` and configuration consumers, add configured row/time flush thresholds and validation; verify focused policy tests without literals at call sites.
- [x] 2.2 In runtime `src/storage` and Cargo dependency/lock owners, implement contract-to-Arrow mapping and metadata; verify every value type including nested arrays/objects, unsigned boundaries, nullability and UTC microseconds in round-trip tests.
- [x] 2.3 Replace stored-file ownership with `files`/`artifacts`, add streams/batch indexes and retain numeric summaries in runtime SQLite schema/file/projection owners; verify catalog joins, telemetry exclusion from artifacts and refusal of incompatible roots without data loss.

## 3. Writer, Durability And Recovery

- [x] 3.1 Implement per-execution/stream and session file ownership, ZSTD batches and row/time flush scheduling under runtime `src/storage`; verify both triggers, durable flush and Windows close-before-rename/delete behavior.
- [x] 3.2 Integrate batch fsync/catalog commit/saved-cursor ordering with runtime execution evidence and transaction owners; verify failure injection never acknowledges uncommitted or unflushed rows.
- [x] 3.3 Implement finalize and footerless recovery under runtime storage lifecycle; verify recovered complete batches, partial-write handling and the owner-approved finalize complexity without payload rewriting.

## 4. Consumers And Removal

- [x] 4.1 Complete overlapping indexed Arrow reads plus unflushed-tail queries, with projected numeric decoding and bounded exact median reduction. Focused proofs cover windows, out-of-order times, pagination, saved/live visibility, cache switching and identical statistics/envelopes; final latency acceptance remains in 7.2.
- [x] 4.2 Delete obsolete telemetry payload read/write paths, per-signal tables and indexes, retaining unrelated evidence and summaries; verify source searches and focused storage/durability/recovery tests.
- [x] 4.3 Review domain/operation/client and GUI consumers; migrate all affected bindings/examples if their contracts change and read `docs/gui.md` before GUI edits; verify focused consumer tests and generated-contract consistency where affected.

## 5. Exports And Documentation

- [x] 5.1 Replace runtime `src/domains/exports/evidence/{database,archive,snapshot}.rs` and cycle export projection with relational catalogs and original Arrow file copies; delete partition tables, column splitting and `stream_columns`; verify execution/cycle paths, manifest hashes and byte identity.
- [x] 5.2 Update runtime/server export fixtures and add `packages/dokime-client/tests/integration/client/arrow_archive.py`; verify exported Arrow with project Polars and evidence.sqlite joins through the focused Rust export task. Review the SDK's opaque-download `test_exports.py` and retain its unchanged protocol tests.
- [x] 5.3 Update `docs/product-guide.md`, README claims/examples if affected, export-related remote guides and `docs/exports.md` if present in the delivery base; verify documentation matches final behavior by review. The initial origin/dev has no `docs/exports.md`.

- [x] 5.4 Refresh `crates/dokime-runtime/dist-report/report.html` through `prepare:report` after the required resource check found it stale; review the generated-only difference and rerun `check:rust`.

## 6. Benchmark Source And Closure

- [x] 6.3 Profile the landed read path before optimization; own query-only changes in runtime `plot/stored.rs`, `storage/ipc{,/projection}.rs` and `storage/sqlite/data/telemetry{,/reader.rs,/tests.rs}`, plus deterministic plot/storage parity fixtures and dependency declarations only if needed. Preserve the file format, write/finalize/export paths and exact statistics/envelopes; remove superseded unconditional full-batch numeric decoding and repeated median materialization within the bounded cache. Review README/product-guide impact.
- [x] 6.4 Prove query equivalence on deterministic fixtures covering projected fields, nulls, numeric types, scope/time/frontier selection, pagination, saved/tail merging, exact statistics and downsampling. Migrate the existing report fixture from removed full-history XY/histogram rendering to the already-locked bounded-report contract; retain live typed-integer precision and live XY/histogram coverage. Replace global-progress race assertions with event-driven saved-execution barriers and explicit query frontiers.

- [x] 6.1 Extend the existing runtime bench harness (`benches/storage` and related unit/integration/system owners as appropriate) to compare origin/dev with Arrow writes, 1M-sample window query, finalize and export; verify equivalent inputs, give the existing storage query fixture paired increasing time bounds, wait for both terminal run receipt and saved execution before export, and retain reproducible before/after output without running the benchmark yet.
- [x] 6.2 Close every implementation item and targeted proof in the combined branch, including removal and consumer inventory; only then declare SOURCE COMPLETE. Verify no unimplemented contract or unhanded-off writer remains.

## 7. Final Validation And Local Handback

- [x] 7.1 After query source closure, run the requested workspace check and formatting through mise on the final tree; report actual native results without umbrella lanes, release builds or installers. No GUI changes are planned.
- [ ] 7.2 Run only the existing Arrow telemetry million-row window query once at the end through `build:rust:focused`; compare with the recorded landed-Arrow baseline. The owner authorized query-only revalidation; retain the completed write/finalize/export measurements without repeating them.
- [ ] 7.3 Validate the complete OpenSpec change strictly, commit owned changes with Conventional Commits and prepare a handback with profile findings, changes, before/after query latency and tests in at most six lines; do not push or merge.

## Workflow Follow-Up

- The owner resolved the footer blocker; implementation proceeds under the metadata-only footer amendment.
- Preserve this tracked change until the feature is complete; archive through the Hub only when all required tasks are actually closed.
- Experimental integration requires the owner's go; no remote delivery is authorized.
