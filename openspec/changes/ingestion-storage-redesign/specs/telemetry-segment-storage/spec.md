# Telemetry Segment Storage Spec Delta (Proposed)

## ADDED Requirements

### Requirement: Single-Decode Admission

Admission SHALL decode each wire batch once into bounded typed Arrow builders,
validated against its cached derived shape identity. Wire bytes SHALL be
retained only as evidence.

#### Scenario: Batch Admission

- **WHEN** a sidecar publishes a batch
- **THEN** the writer consumes the decoded columns without decoding or re-encoding the wire bytes again

### Requirement: Live Typed Batches

Acknowledged batches SHALL be queryable from bounded typed Arrow batches in
memory until their segment is catalogued.

#### Scenario: Read After Acknowledgement

- **WHEN** a batch is acknowledged and no segment has flushed yet
- **THEN** a query over its window returns its rows

### Requirement: Background Segment Flush

The runtime SHALL flush live batches to Arrow IPC segments in the background
when a row or byte bound is reached, and SHALL record each segment with its row
range, time range and covered journal position in one catalog transaction per
segment.

#### Scenario: Many Small Batches

- **WHEN** many small batches fill one segment
- **THEN** the catalog receives one transaction for that segment, not one per batch

### Requirement: Consistent Query Cutoff

A query SHALL read catalogued segments and live batches at one cutoff so that
every row is returned exactly once while flushes or seals publish concurrently.

#### Scenario: Flush During Query

- **WHEN** a segment is catalogued while a window query is running
- **THEN** the query returns each row in the window exactly once

### Requirement: Parquet Sealed Evidence

Sealing SHALL write an execution's telemetry as Parquet with row-group
statistics and SHALL retain its IPC segments until the Parquet file, its
directory entry and its catalog record are durable.

#### Scenario: Crash During Seal

- **WHEN** the process stops after writing Parquet but before its catalog record commits
- **THEN** the IPC segments remain the evidence and sealing restarts from them

### Requirement: Fresh Root for the New Layout

A data root in the previous telemetry layout SHALL be reported and preserved, and
the new layout SHALL require a fresh data root. Plugin changes alone SHALL NOT
require a fresh root.

#### Scenario: Previous Layout

- **WHEN** startup opens a root with sample rows in SQLite
- **THEN** it names the fresh-root action and leaves the existing data intact
