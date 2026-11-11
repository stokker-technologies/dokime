# Ingestion Storage Redesign Design

Status: proposed on 2026-11-08; deferred until the owner schedules it.

## Context

Measured on an idle Ryzen 9 5950X with NVMe, dev `2bfdef6c..76508f99`. The
reference column is sourced unless marked as an inference.

| Dimension | Dokime Now | Well-Built Rust Reference | Gap |
|---|---|---|---|
| Sample ingestion, durable | 29k/s in the SQLite sample store; about 50k/s through the sidecar Arrow path | 0.2 to 1M/s per ingestion core (inference from arrow-rs and bulk-store figures); durable SQLite alone 319k rows/s at 1,000 rows per transaction | 6 to 30x |
| Per-batch write cost | About 340 commits per million rows, about 34 ms of sync each, per-row indexed inserts | One multi-row statement per batch, one covering fsync per group of batches; NVMe fsync 0.3 to 0.6 ms | Dozens of ms per batch versus well under 1 ms |
| Row path | Postcard bytes decoded for admission, re-encoded, decoded again in the writer | Decode once into typed columnar builders; wire bytes kept only as evidence | About 1.2 s per 100k rows wasted |
| Schema identity | Derived revision recomputed about 6 times per record | Computed once per schema and cached | 109 of 150 µs per record |
| Operation call overhead | 16.7 ms per call, dominated by attempt-log file reopen | Sub-millisecond logging through an open appender | About 60 operations/s cap |
| Hot data queries | Arrow IPC files plus SQLite catalog; 1M-sample window 720 ms, exact median 324 ms | InfluxDB IOx 5-minute sum over 738,300 rows 382.70 ms; QuestDB hourly maxima over about 2.765M values 5.37 ms | Within an order of magnitude |
| Durability contract | Per-batch acknowledged durability | Group commit behind one covering sync; same promise, higher acknowledgement latency | Contract right, mechanism expensive |
| Seal of 213 members | 9.8 s here, over 60 s on Linux; about 5,100 flushes | Batched finalization, one barrier per batch | Fsync count |

Million-row workload, loaded then idle, accepted/durable: Dokime 31.2/37.1 s
then 33.89/34.48 s; the same-machine storage library 5.2/6.1 s then
5.38/5.28 s. The idle rerun found no cargo, rustc or uv processes and 24 Node
processes at zero CPU, so machine load does not explain the gap. A 4,096-row
probe spent 39.7 ms in indexed inserts and 42.3 ms in commits, 26.6 ms of it
syncing; million-row totals were 11.85 s inserts and 17.81 s commits, 11.48 s
syncing. The profiled scalar sidecar writer spent 2.31 s, of which 1.24 s was
decode, 0.44 s Arrow, 0.17 s catalog and 0.18 s commits.

The engineering target of 0.2 to 1M samples/s per ingestion core is an
inference, not a published ceiling; no source establishes a power-safe
single-core rate for this workload.

## Evidence That Drove the Decision

Vendor-published means the publisher sells the measured system or a competitor;
project documentation is the maintainers' own specification.

| Finding | Figure | Source and Date | Vendor-Published |
|---|---|---|---|
| SQLite WAL, 1,000 rows per transaction | FULL 319,398 rows/s; NORMAL 393,045 rows/s, 1.23x | [sqlite-bench](https://github.com/benbjohnson/sqlite-bench), undated, accessed 2026-11-08 | No |
| SQLite `synchronous` semantics | FULL syncs each commit; NORMAL can lose acknowledged transactions after power failure | [SQLite pragma specification](https://www3.sqlite.org/matrix/pragma.html), undated, accessed 2026-11-08 | Project documentation |
| NVMe sync, Windows 11 | `fsync` 388 µs, `fdatasync` 326 µs with cache enabled | [pgsql-hackers](https://postgrespro.com/list/id/CA%2BhUKG%2Ba-7r4GpADsasCnuDBiqC1c31DAQQco2FayVtB9V3sQw@mail.gmail.com), 2022-08-10 | No |
| Consumer NVMe sync, Linux | 0.49 ms, 3.8 ms and 9.6 ms per write/sync loop on three drives | [Percona](https://www.percona.com/blog/fsync-performance-storage-devices/), 2018-02-08 | No |
| PostgreSQL group commit | Delay 0 to 1,000 µs: 1,576 to 2,738 transactions/s, 1.74x, durable acknowledgement retained | [CYBERTEC](https://www.cybertec-postgresql.com/en/commit_delay-performance-postgresql-benchmark/), 2025-01, updated 2025-02-11 | No |
| RocksDB group commit | Concurrent writes share one WAL write and sync; maximum group 1 MB | [RocksDB WAL performance](https://github.com/facebook/rocksdb/wiki/WAL-Performance), 2020-10-15 | Project documentation |
| arrow-rs IPC writer | Derived about 955M/923M rows/s stream/file into RAM; no disk or sync | [arrow-rs PR 10833](https://github.com/apache/arrow-rs/pull/10833), 2026-08-25 | No |
| arrow-rs Parquet encoding | 412.6 MB/s nullable, 751.6 MB/s non-null primitives | [arrow mailing list](https://www.mail-archive.com/github@arrow.apache.org/msg436588.html), 2026-10-07 | No |
| DuckDB appender to Parquet | 1.742M durable rows/s versus 1.934M relaxed | [Mirexa](https://www.mirexa.dev/benchmarks/index.html), undated, accessed 2026-11-08 | Yes, competitor |
| Parquet row groups | 100k to 1M rows recommended, default 122,880; 960-row groups 8.77 s versus 122,880 at 0.87 s on read | [DuckDB file formats](https://duckdb.org/docs/1.4/guides/performance/file_formats), [Parquet tips](https://www.duckdb.org/docs/lts/data/parquet/tips), undated, accessed 2026-11-08 | Project documentation |
| Batch size | 10M rows in 131 s versus 46 s as batches grow from 10k to 1M rows; parallelism also changes | [ClickHouse](https://clickhouse.com/blog/clickhouse-input-format-matchup-which-is-fastest-most-efficient), 2025-02-26 | Yes |
| InfluxDB 1.x | WAL plus queryable cache, then immutable files; WAL sync precedes success | [Storage engine](https://docs.influxdata.com/influxdb/v1/concepts/storage_engine/), undated, accessed 2026-11-08 | Project documentation |
| InfluxDB 3 IOx | Queryable memory batches plus partitioned Parquet; queries combine both | [Durability](https://docs.influxdata.com/influxdb3/core/reference/internals/durability/), undated, accessed 2026-11-08 | Project documentation |
| IoTDB | `SYNC` waits for WAL sync; separate late-data path; chunk statistics accelerate aggregates | [SIGMOD paper](https://sxsong.github.io/doc/23sigmod-iotdb.pdf), June 2023 | No |
| VictoriaMetrics | Permits losing the latest buffered data after unclean shutdown | [Documentation](https://docs.victoriametrics.com/victoriametrics/index.html), undated, accessed 2026-11-08 | Project documentation |
| InfluxDB 3 IOx query | 382.70 ms sum over 738,300 rows, last 5 minutes, during ingestion | [InfluxData report](https://get.influxdata.com/rs/972-GDU-533/images/InfluxDB-3-0-vs-OSS-Benchmarks.pdf), August 2023 | Yes |
| QuestDB query | 5.37 ms hourly maxima over about 2.765M values, warm cache | [QuestDB comparison](https://questdb.com/blog/timescaledb-vs-questdb-comparison/), August 2026 | Yes |

Rejected options follow from the same evidence. DuckDB and QuestDB as system of
record move acknowledgement and recovery outside Dokime; QuestDB's embedded API
also needs a JVM. SQLite NORMAL changes the durability contract. Downsampling on
ingest loses raw evidence. A file per scalar series multiplies file management.
Early acknowledgement and timer-only persistence weaken the evidence promise.

## SQLite Decision

SQLite stays as the metadata store for catalog, criteria, cycles, journal
metadata and settings, with WAL and `synchronous=FULL`. Its durable bulk rate of
319k rows/s at 1,000 rows per transaction shows the engine is not the
bottleneck; per-row inserts and one sync per small batch are. Sample rows leave
SQLite, so its write rate falls to catalog and metadata volume, where FULL costs
nothing material. No engine replacement is planned. redb is considered only if a
build without C dependencies ever becomes a requirement. The open question is
the query engine, DataFusion, not the database.

## Durability Contract

- **At acknowledgement:** every acknowledged batch is in a journal segment whose
  bytes and, for a new segment, directory entry are covered by a completed sync.
  No acknowledgement precedes the sync that covers it. A failed sync fails every
  batch it covers and none of them is acknowledged.
- **After a power cut:** replay restores every acknowledged batch exactly once,
  in order. An unacknowledged batch is either restored completely or absent,
  never partial. A torn tail is truncated at the last record with a valid length
  and checksum.
- **Acknowledgement latency:** bounded by a sync group bound of time and bytes,
  to be chosen by the owner. Group commit raises acknowledgement latency under
  load; it never loses acknowledged data.
- **Assumption:** the filesystem and device honor sync barriers.

The journal stays the recovery authority until a segment's bytes, its namespace
entry and its catalog record are all durable; only then is the journal prefix
it covers reclaimable.

## Write Path

1. **Admission once:** decode the wire batch once, validate it against its
   cached shape identity and keep the decoded form; wire bytes are kept only as
   evidence.
2. **Typed builders:** append the decoded rows to bounded typed Arrow builders
   per execution and stream.
3. **Journal append:** append the batch as one checksummed journal record.
4. **Grouped sync:** one covering `sync_data` per group, bounded by time and
   bytes, for every batch queued since the previous sync.
5. **Acknowledgement:** acknowledge every batch the sync covered; the batch is
   now queryable in the live store.
6. **Segment flush:** in the background, when a builder reaches its row or byte
   bound, write it as an Arrow IPC segment and sync the file and its directory.
7. **Catalog update:** record the segment, its row and time ranges and the
   journal position it covers in one SQLite transaction per segment.
8. **Seal to Parquet:** at seal, write the execution's segments as Parquet with
   row-group statistics, sync it, record it in the catalog, then retire the IPC
   segments according to the retention decision.

## Crash and Recovery

1. Open SQLite and read the last durable segment record per stream and the
   journal position it covers.
2. Scan journal segments from that position, validating each record's length and
   checksum; truncate at the first invalid record.
3. Rebuild the live builders from the replayed records; they are acknowledged
   evidence and queryable again.
4. Discard IPC or Parquet files absent from the catalog; they were never
   published.
5. Resume background flush and sealing; an interrupted seal restarts from its
   catalogued segments.

## Query Path

A query takes one cutoff: the catalog snapshot plus the journal position the
live store has acknowledged. It reads catalogued Parquet and IPC segments up to
the segment boundary and live batches above it, so every row is counted once
even while a flush or seal publishes concurrently. DataFusion evaluates over the
same inputs when documents need joins, windows or cross-stream expressions; the
current numeric projection and exact-statistics behavior stay the baseline.

## Migration

Existing roots are not migrated. Under the development storage policy, a root in
the current layout needs a fresh data root; it is reported and preserved, never
silently reset or deleted. Plugin changes alone continue to keep a root.

## Open Questions for the Owner

1. Journal segment size and the sync group bound in time and bytes.
2. Parquet row-group size.
3. Whether DataFusion enters in this change or a later one.
4. Retention of Arrow IPC segments after the Parquet file is durable.
