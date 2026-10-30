# Arrow IPC Telemetry Storage Design

## Context

This is a proposed experimental design, not landed behavior. Motivation is in `proposal.md`. Dokime's `crates/dokime-runtime/src/storage/sqlite/data/telemetry.rs` currently indexes encoded samples by signal; `crates/dokime-runtime/src/domains/exports/evidence/database.rs` builds partitioned export tables.

## Goals And Non-Goals

Retain durable evidence and live queries while eliminating per-signal writes and export payload conversion. No Harmos changes, Parquet, compatibility adapters, development-data migrations, silent storage resets, installers or hilti-hcl changes.

## Locked Decisions

1. Dokime owns Arrow inside `crates/dokime-runtime/src/storage`, implemented by `sqlite::Backend` under Harmos `StorageManager`; Harmos is unchanged. Arrow IPC files (Feather v2) are the only telemetry format.
2. One file per execution and stream: `<data root>/telemetry/<sha256(execution_id | stream)>.arrow`. Telemetry outside executions has a session-scoped file. Runtime names are hashes; export names are readable.
3. Write the header at start and append ZSTD record batches per flush. Both N-row and T-time thresholds come from configuration; either triggers a flush. Finalize writes only footer metadata from the block list, proportional to batch count, without scanning, decoding or rewriting payloads. Footerless crashes retain an embedded readable stream; recovery scans batch headers and skips bodies to rebuild the footer. A saved cursor advances only after batch fsync and committed catalog metadata; only flushed batches are durable.
4. Contract mapping: integer → Int64; unsigned → UInt64; number → Float64; bool → Boolean; string → Utf8; arrays → List of their mapped element; declared objects → Struct; undeclared objects → JSON Utf8; timestamps → Timestamp(µs, UTC). Field metadata includes unit, description and stream id. File metadata includes execution id, Dokime version and contract hash.
5. SQLite is the catalog. Rename the stored-files table to `files(id, kind, path, size_bytes, sha256, media_type)`, with kinds `artifact`, `telemetry`, `report`. `artifacts` references `files` for user/test files; telemetry is never an artifact. `streams` records execution, stream, file FK, rows, first/last times and batch offsets with row/time ranges. Retain incremental numeric summaries. Delete telemetry per-signal tables/indexes and telemetry payload-decoding reads.
6. GUI charts, queries and outputs combine only overlapping indexed flushed batches with the in-memory unflushed tail. Preserve the target of approximately 1 s per 1M samples or better.
7. Execution ZIP: `manifest.json` with path/size/SHA-256; `evidence.sqlite` with relational evidence and `files`, `artifacts`, `streams`, `stream_summaries`; `reports/execution-<id>.html`; `artifacts/<relative path>`; `telemetry/<stream>.arrow`. Preserve executions, steps, criteria, measurements, records, annotations and traceability. Copy Arrow bytes unchanged. Cycle ZIP uses `executions/<execution-id>/{artifacts,telemetry}/` and `reports/cycle-<id>.html`. Remove partition tables, column splitting and `stream_columns`.
8. Storage remains v1 with a clean break. Incompatible existing data requires a fresh root; never silently delete or reset it.
9. On Windows keep files open during runs; copies for export are allowed, but rename/delete only after close.

## Approved Footer Amendment (2026-10-30)

[Arrow IPC File Format](https://arrow.apache.org/docs/format/Columnar.html#ipc-file-format) requires a footer containing schema plus offsets and sizes for every batch. The [normative footer schema](https://github.com/apache/arrow/blob/main/format/File.fbs) declares `recordBatches: [Block]`; each block stores its offset and metadata/body lengths.

For B appended batches, that vector has B entries. The owner approved O(B) metadata-only footer emission from the in-memory/catalog block list. Decision 3's O(1) means finalize never scans, decodes or rewrites telemetry payloads. Each block is approximately 24 bytes; a week at five-second flushing produces approximately 3 MB of footer metadata.

Keep the block list in memory and in the SQLite batch index. Finalize reads no payload; crash recovery scans batch headers and skips bodies to rebuild the footer. IPC, durability, byte-for-byte exports and all other locked decisions stand.

The IPC file starts with eight bytes of magic/padding before its embedded stream. A recovery stream reader skips that header. Preserve readable flushed batches and rebuild the footer without decoding their payloads.

## Ownership And Dependency Order

The single writer is the current chat in `feat/arrow-telemetry`; the initial tree was clean and tracked `origin/dev`. Tasks lock the source inventory. The change is activated through `stkr specification start arrow-telemetry-storage` with a native persistent goal.

Implement schema/configuration and catalog first, then writer/recovery, reader/durable cursor consumers, exports, documentation and benchmarks. Retire superseded code in the same feature. Keep existing managed Harmos lifecycle contracts and public operation declarations as the boundary.

## Validation And Delivery

Use discovered mise tasks with standard compiler preflight and `UV_PROJECT_ENVIRONMENT` absent; do not redirect Python or temporary environments. Run targeted proof during implementation. After complete source closure, run the requested workspace checks/format and the existing benchmark task exactly once with an origin/dev comparison; no installers or umbrella lanes. TypeScript check/lint applies only if GUI changes. The owner's explicit no-push instruction overrides ordinary delivery automation.

## Risks And Rollout

Cross-resource durability needs fault tests proving fsync → catalog commit → saved-cursor publication. Recovery must preserve existing storage and Windows handle ownership. New incompatible roots remain explicit operator choices; existing roots are never reset.
