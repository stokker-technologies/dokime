# Telemetry Acknowledgement Specification

## Purpose

Resolve saved telemetry receipts without holding the storage writer and preserve
durable close-out after transient reads or explicitly repaired flush failures.

## Requirements

### Requirement: Indexed Reader Acknowledgement

Saved sample acknowledgements SHALL resolve by indexed sample identity on a WAL
reader, constrained by stream, execution capture membership and saved frontier.
Replay SHALL advance a validated chunk from its known saved position without
re-resolving its final identity from history. Index records SHALL commit atomically
with the Arrow batch catalog. Older roots without the index SHALL be refused intact.

#### Scenario: Long History

- **WHEN** a receiver acknowledges chunks from multi-hour saved telemetry
- **THEN** acknowledgement performs bounded identity lookup work without decoding prior Arrow rows or acquiring the writer

#### Scenario: Capture And Durability

- **WHEN** an acknowledgement is outside an execution's captured rows or saved frontier, or its batch failed to commit
- **THEN** it is refused, including after reopening storage

#### Scenario: Execution Close-Out

- **WHEN** saved telemetry replays and the receiver confirms each final sample
- **THEN** replay advances in order and allows normal finish/seal
- **AND** any remaining upstream transient-read admission poisoning is documented with exact source locations without modifying Harmos

### Requirement: Read Failure Isolation

Snapshot-open and payload-read failures SHALL remain local to the requesting read;
they SHALL NOT poison subsequent write admission or telemetry sealing. Persistence
action failures SHALL retain their existing write-admission protection and deadlines.
SQLite close SHALL retain nontransient task errors and unsaved tail failures;
it MAY retire transient task errors once every tail position is durably saved.

#### Scenario: Transient Read Then Seal

- **WHEN** a payload read returns a transient I/O error and the underlying file becomes available again
- **THEN** a subsequent sample save and telemetry seal succeed without clearing a global persistence error

#### Scenario: Repaired Flush Then Close

- **WHEN** a transient telemetry flush task fails and an explicit retry saves every admitted tail position
- **THEN** SQLite close discards that stale transient task error after confirming tail durability
- **AND** outstanding unsaved rows and nontransient task failures still refuse close

At pinned Harmos revision `91b8b3a77c46f9da1dbf47cbe06ef9ebd1233158`,
`crates/harmos-storage/src/worker.rs:597-598` wraps snapshot-open errors in
`Completed::Opened`; `worker.rs:337-346` and `snapshot.rs:258-270` retain the error
only on that snapshot. Payload reads return errors directly at `snapshot.rs:161`,
`:184` and `:203`. Persistence-action failures, including snapshot-release failures,
can reach `worker.rs:474` and block writes at `manager/admission.rs:159-160`.
Dokime's snapshot release performs no backend I/O. These paths do not establish
that acknowledgement caused a particular failed execution without its failure trace.
