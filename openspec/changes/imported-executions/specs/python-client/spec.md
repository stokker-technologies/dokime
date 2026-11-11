## MODIFIED Requirements

### Requirement: Owned Cycle Lifecycle

The client SHALL offer flat cycle creation and public explicit close() without requiring a context manager. Closing SHALL stop owned work, seal the cycle and drain final outcomes with cancellation-shielded cleanup. Cleanup failures SHALL remain grouped with any original failure.

#### Scenario: Any Context Exit

- **WHEN** a caller creates a cycle and closes it after completion, failure or cancellation
- **THEN** owned work stops, the cycle seals and final outcomes drain
- **AND** cleanup failures remain grouped with any original failure

### Requirement: Lazy Finite Query Readers
The client SHALL expose the synchronous lazy query factories in `design.md` as single-consumer async readers. Factories SHALL perform no network I/O; consumption SHALL fetch on demand. Omitted filters SHALL be unconstrained within scope. Existing lookups, records/samples, explicit resource lifecycles, checkpoint and lifecycle semantics SHALL remain unchanged.

#### Scenario: Factory Arguments
- **WHEN** callers build execution/cycle discovery or evidence readers
- **THEN** executions accepts template/cycle/status/since/until/archive/at and cycles accepts phase/since/until/archive/at
- **AND** execution/cycle query_records accepts record_type/where/between/versions/at; cycle queries can narrow to a member execution ID
- **AND** execution telemetry accepts stream/fields/where/between/versions/every/agg/at

#### Scenario: Historical Selection Without Follow
- **WHEN** a caller iterates `client.executions(template="example.demo.tracking", since="2026-10-01", status="failed")`
- **THEN** it receives typed execution summaries across matching cycles and ends at the fixed snapshot
- **AND** it does not await terminal execution completion or allocate the whole history

## ADDED Requirements

### Requirement: Flat Python API

Public Python Dokime and Harmos API usage SHALL have a form without with or async with. Value APIs SHALL compose a value and pass it to one call. Resources SHALL offer explicit close(), or existing reader aclose(), without requiring magic-method calls.

#### Scenario: Client and Reader Release

- **WHEN** a caller connects a client, consumes a finite reader and releases both explicitly
- **THEN** the reader stops fetching and the client releases local resources without a context-manager block or stopping unrelated backend work
