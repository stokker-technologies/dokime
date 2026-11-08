# Evidence Durability Spec Delta (Proposed)

## ADDED Requirements

### Requirement: Acknowledged Batches Are Durable

The runtime SHALL acknowledge a telemetry batch only after a completed sync
covers its journal record and, for a new journal segment, its directory entry.
A failed sync MUST fail every batch it covers without acknowledging any of them.

#### Scenario: Sync Succeeds

- **WHEN** several batches are queued before one covering sync completes
- **THEN** all of them are acknowledged after that sync and none before it

#### Scenario: Sync Fails

- **WHEN** the covering sync fails
- **THEN** none of its batches is acknowledged and the producer receives the failure

### Requirement: Group Commit Within a Bound

The journal SHALL share one sync across the batches queued since the previous
sync, bounded by configured time and byte limits. Grouping MAY raise
acknowledgement latency and MUST NOT lose acknowledged data.

#### Scenario: Sustained Load

- **WHEN** batches arrive faster than one sync per batch
- **THEN** the number of syncs is bounded by the group limits rather than the batch count
- **AND** no acknowledgement waits longer than the configured time bound plus one sync

### Requirement: Replay After Power Loss

After a crash or power cut, recovery SHALL replay every acknowledged batch
exactly once, in order. An unacknowledged batch SHALL be restored completely or
be absent. A torn journal tail SHALL be truncated at the last record with a valid
length and checksum.

#### Scenario: Torn Tail

- **WHEN** power is lost while a journal record is partly written
- **THEN** recovery restores every acknowledged batch, drops the partial record and resumes ingestion

### Requirement: Journal Remains the Recovery Authority

A journal prefix SHALL be reclaimable only after the segment covering it, its
directory entry and its catalog record are durable.

#### Scenario: Crash During Segment Publication

- **WHEN** the process stops after writing a segment but before its catalog record commits
- **THEN** recovery ignores the uncatalogued segment and replays the covered batches from the journal

### Requirement: SQLite Metadata Durability

SQLite SHALL hold catalog, criteria, cycles, journal metadata and settings in WAL
mode with `synchronous=FULL` and SHALL NOT hold telemetry sample rows.

#### Scenario: Committed Metadata

- **WHEN** a catalog or criteria transaction commits and power is lost
- **THEN** the committed transaction is present after restart
