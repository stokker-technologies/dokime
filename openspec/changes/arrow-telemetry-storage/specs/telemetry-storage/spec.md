# Telemetry Storage Spec Delta

## Purpose

Define durable typed telemetry and portable evidence archives while preserving indexed live access and acknowledged storage progress.

## ADDED Requirements

### Requirement: Execution And Session Files
The backend SHALL store telemetry exclusively as Arrow IPC files, one per execution/stream at `<data root>/telemetry/<sha256(execution_id | stream)>.arrow`, with separate session-scoped files outside executions.

#### Scenario: Stream Ownership
- **WHEN** two executions publish the same stream and a session publishes outside execution
- **THEN** each execution/stream and the session own separate files, without treating telemetry as an artifact

### Requirement: Configured Durable Flush
The backend SHALL append ZSTD batches after a header and flush when either configured row or time threshold is reached. The saved cursor MUST advance only after batch fsync and its catalog commit.

#### Scenario: Threshold Or Commit Failure
- **WHEN** either threshold is reached, or fsync/catalog commit fails
- **THEN** the batch flush is attempted and saved progress includes it only after both durability steps succeed

### Requirement: Constant-Time Finalize And Recovery
The backend SHALL write the IPC footer at finalize in O(1). A footerless crashed file SHALL remain readable as a stream and recovery SHALL scan its batches to rebuild the footer. This locked requirement is currently blocked by the normative format conflict recorded in design.md.

#### Scenario: Finalize And Interrupted Run
- **WHEN** a run finalizes or restarts after a crash before footer completion
- **THEN** finalize is O(1), or recovery preserves readable flushed batches and rebuilds their valid IPC footer respectively

### Requirement: Typed Contracts And Metadata
Telemetry SHALL map integer/unsigned/number/bool/string to Int64/UInt64/Float64/Boolean/Utf8, arrays to List, declared objects to Struct, undeclared objects to JSON Utf8 and timestamps to Timestamp(µs, UTC). Fields SHALL retain unit, description and stream id; files SHALL retain execution id, Dokime version and contract hash.

#### Scenario: External Reader Round Trip
- **WHEN** a contract containing each value type is recorded and read using an Arrow reader
- **THEN** typed values, nested structure, units and identity metadata are preserved

### Requirement: Relational Catalog And Summaries
SQLite SHALL expose `files(id,kind,path,size_bytes,sha256,media_type)` with artifact/telemetry/report kinds, `artifacts` referencing files, streams with execution/stream/file identity, row counts, first/last times and indexed batch offsets/time/row ranges, and retained incremental numeric summaries. Telemetry payload storage and per-signal tables/indexes SHALL be removed.

#### Scenario: Catalog Joins
- **WHEN** an execution captures telemetry and user/test files
- **THEN** streams join to telemetry files, artifacts join only to captured artifact files, and numeric summaries remain available without per-signal rows

### Requirement: Indexed Live Windows
Charts, queries and outputs SHALL combine overlapping flushed batches selected through the batch index with the unflushed in-memory tail, retaining a query latency target of approximately one second per million samples or better.

#### Scenario: Window With Pending Rows
- **WHEN** a requested window overlaps saved batches and pending rows
- **THEN** the result contains matching rows from both exactly once and reads no nonoverlapping batch payloads

### Requirement: Portable Execution And Cycle Exports
Execution ZIPs SHALL contain manifest.json with path/size/SHA-256, evidence.sqlite, reports/execution-<id>.html, artifacts/<relative path> and telemetry/<stream>.arrow. Arrow bytes MUST be copied unchanged. Cycle ZIPs SHALL place files under executions/<execution-id>/{artifacts,telemetry}/ and include reports/cycle-<id>.html.

#### Scenario: Portable Archive
- **WHEN** an execution or cycle archive is exported
- **THEN** its layout and manifest match, Arrow files equal stored bytes, and an independent Python Arrow reader can read telemetry

### Requirement: Relational Export Evidence
The export database SHALL contain files, artifacts, streams, stream_summaries and relational executions, steps, criteria, measurements, records, annotations and traceability. Stream partition tables, column splitting and stream_columns SHALL be removed.

#### Scenario: Evidence Join
- **WHEN** an external SQLite reader joins execution evidence, stream catalogs and file catalogs
- **THEN** captured relationships and summaries are available and no partition discovery or stream_columns table is needed

### Requirement: Clean Development Break
Storage and evidence baselines SHALL remain v1 without migrations or legacy compatibility. Incompatible existing storage MUST require an explicitly selected fresh root and MUST NOT be silently reset or deleted.

#### Scenario: Incompatible Root
- **WHEN** startup encounters incompatible development storage
- **THEN** it reports the incompatibility and preserves all existing data

### Requirement: Windows File Lifetime
Telemetry files SHALL remain open during a run and permit export copies; rename and deletion MUST occur only after close.

#### Scenario: Export During Ownership
- **WHEN** an owned telemetry file is copied for export on Windows
- **THEN** copying respects the active handle and no rename/delete occurs before close
