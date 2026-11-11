# Ingestion Storage Redesign

Status: proposed on 2026-11-08; deferred until the owner schedules it.

## Why

Durable sample ingestion is 6 to 30 times below a well-built Rust reference. A
million-row run through the SQLite sample store takes 33.89 s accepted and
34.48 s durable on an idle Ryzen 9 5950X with NVMe, about 29k samples/s; the
sidecar Arrow path reaches about 50k/s. The same-machine storage library takes
5.38 s and 5.28 s for the same workload, about 6 times faster. Load is ruled
out: the idle rerun found no cargo, rustc or uv processes and 24 Node processes
at zero CPU, and it was no faster than the loaded run (31.2 s and 37.1 s).

The cost is in the mechanism, not the contract. A million rows spend 11.85 s
in per-row indexed inserts and 17.81 s in about 340 commits, 11.48 s of which
is syncing. Sealing 213 members takes 9.8 s here and over 60 s on Linux through
about 5,100 flushes. Per-batch acknowledged durability is right; one sync per
small batch and sample rows in SQLite are what make it expensive.

## What Changes

- Add a write-ahead journal with group commit: per-execution segments, one
  covering sync acknowledges every batch queued since the previous sync, and no
  acknowledgement precedes its sync. Evidence replays from the journal.
- Make bounded typed Arrow record batches the live, queryable store. Flush them
  to Arrow IPC segments in the background and update the SQLite catalog once per
  segment, not per batch. Samples leave SQLite.
- Store sealed evidence as Parquet with row-group statistics. Arrow IPC stays on
  the hot path and is retained until the Parquet file is durable.
- Use DataFusion as the query engine over live batches and IPC and Parquet files
  when documents need joins, windows or cross-stream expressions.
- Reject: DuckDB or QuestDB as system of record for samples, SQLite
  `synchronous=NORMAL`, downsampling on ingest, a file per scalar series, early
  acknowledgement and timer-only persistence.

## What Stays

- Per-batch acknowledged durability is the promise; only its mechanism changes.
- Arrow IPC remains the hot-path format.
- SQLite remains the metadata store for catalog, criteria, cycles, journal
  metadata and settings, with WAL and `synchronous=FULL`. No engine replacement.
- Data shape identities remain derived.
- Data roots survive plugin changes.

## Out of Scope

The in-flight write-path fixes land independently and are prerequisites of this
change: the schema revision cache, decoded admission reuse, batched sample
inserts, the attempt-log appender and batched seal finalization. This change
neither repeats nor depends on their internal shape, only on their landed
behavior.

## Capabilities

### New Capabilities

- `telemetry-segment-storage`: Live typed batches, background segment flush,
  per-segment catalog, Parquet sealing and queries with a consistent cutoff.
- `evidence-durability`: Journal group commit, the acknowledgement contract and
  replay after a crash or power cut.

### Modified Capabilities

None until scheduled. The landed `telemetry-acknowledgement`, `stream-ingest`
and `evidence-finalization` requirements are revisited when the change is
scheduled, and the open `arrow-telemetry-storage` change owns the current IPC
layout until then.

## Impact

Runtime storage, admission, the telemetry writer, the SQLite catalog, sealing,
evidence exports, query evaluation and their benchmarks. Existing development
roots need a fresh root under the development storage policy.

Sources: D:/repositories/stokker-technologies/orchestrator/research/ingestion-synthesis.md,
D:/repositories/stokker-technologies/orchestrator/codex/r-ingestperf.result.md,
D:/repositories/stokker-technologies/orchestrator/codex/r-industrystore.result.md,
D:/repositories/stokker-technologies/orchestrator/codex/r-industrytsdb.result.md.
