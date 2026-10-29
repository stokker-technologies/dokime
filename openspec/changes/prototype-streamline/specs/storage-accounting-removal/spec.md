# Storage Accounting Removal

## ADDED Requirements

### Requirement: Remove Memory Accounting Without Behavior Loss

Dokime and the paired Harmos storage interface SHALL remove memory-accounting claims while preserving the behavior listed below.

#### Scenario: Read Through The Storage Manager

- **WHEN** a read method previously took `&ReadSession`
- **THEN** it takes `&Manager` in the same argument position and invokes its ordinary read methods
- **AND** no dummy claim or no-op compatibility type replaces the removed reservation

#### Scenario: Telemetry Backpressure Survives

- **WHEN** `SampleAdmission` is removed
- **THEN** telemetry channel backpressure, saved frontiers and capture errors remain

#### Scenario: Locked Design Decisions Apply

- **WHEN** this requirement is implemented or reviewed
- **THEN** the following locked design decisions hold unchanged:

Replace `ReadSession` use with the existing storage manager and ordinary owned/borrowed payloads; remove `RetainedMemory`, `Held`, `Pool`, reservations and grow/retain estimators from Dokime paths. `ReadBatch<T>` remains a cheap shared immutable batch with slicing/iteration/serialization, not a charge carrier. Preserve fallible allocation and checked arithmetic where correctness needs them. Remove `SampleAdmission` but retain telemetry channel backpressure, saved frontiers and capture errors. Keep validation/authorization errors named Admission where unrelated to memory; do not globally erase admission concepts.

Locked Harmos interface transformation: in `harmos-storage/src/backend.rs`, remove every `RetainedMemory` parameter and `Inner::backend_memory`; keep every other parameter, return type, lifetime, cursor, limit and durability guarantee. Exact examples: `fn open(&self) -> StorageFuture<'_, StorageCursor>`; `fn read_records<'a>(&'a self, snapshot: &'a BackendSnapshot, selection: &'a RecordSelection, page: ScanPage) -> StorageFuture<'a, Vec<StoredRecord>>`. For Dokime payload constructors, remove session/claim-only arguments, retaining remaining order and return/error types. For methods that perform reads, replace `&ReadSession` with `&Manager` in the same argument position and invoke its ordinary read methods; for example `async fn read_file_bytes(storage: &Manager, file: &FileReference) -> Result<ReadBatch<Vec<u8>>>`. Never replace removed reservations with dummy claims or no-op compatibility types.

Preserve `read_opened_file(file: std::fs::File, range: ByteRange)` behavior and opened capture/follow lifecycle, including original inode, growth/truncation checks, acknowledged prefixes and cancellation settlement. Preserve one borrowed SQLite row encoding per transaction iteration, position-based record identity for repeated items, rollback, exact retry comparison, borrowed canonical/JSON serializers, Harmos waveform reduction without allocations, exact full-envelope `wire_len`, time-range filtering at storage query construction and live plot reads from saved storage. Delete only estimator/claim portions of mixed files; never revert whole files to HEAD.

### Requirement: Atomic Saved Evidence Frontiers

Native evidence rows, capture membership and their saved frontier SHALL commit together. Retrying a committed batch SHALL preserve its exact identities and membership without inserting duplicates.

#### Scenario: Writer Stops Before Commit

- **WHEN** the writer process stops after a row insert, during capture indexing or before publishing the saved frontier
- **THEN** reopening retains the previous durable frontier and exposes no membership for the interrupted batch
- **AND** retrying the same batch commits it exactly once

#### Scenario: Acknowledgement Is Lost After Commit

- **WHEN** the writer process stops after committing evidence but before acknowledging the caller
- **THEN** reopening exposes the committed rows, membership and frontier
- **AND** an exact retry preserves that evidence without duplication
