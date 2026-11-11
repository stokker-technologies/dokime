## REMOVED Requirements

### Requirement: Owned Cycle Lifecycle

**Reason**: A required async context conflicts with the flat SDK lifecycle.
**Migration**: Create cycles as values/handles and explicitly finish or stop them and close resources.

## ADDED Requirements

### Requirement: Explicit Cycle Lifecycle

The Python client SHALL offer flat cycle creation, finish, stop and close operations.
Callers SHALL be able to await completion and drain saved outcomes without a context manager.
Cleanup failures SHALL retain their original causes alongside execution failures.

#### Scenario: Explicit Completion

- **WHEN** a script creates a cycle, starts its cases, finishes it and closes the client
- **THEN** work settles, the cycle seals and final outcomes remain readable

#### Scenario: Explicit Stop

- **WHEN** a script stops its owned cycle after an exception or cancellation
- **THEN** active runs clean up and their executions close without depending on block exit

## MODIFIED Requirements

### Requirement: Lazy Finite Query Readers

The client SHALL expose synchronous lazy query factories as single-consumer async readers.
Factories SHALL perform no network I/O; consumption SHALL fetch on demand. Omitted filters
SHALL be unconstrained within scope. Existing lookup, evidence, checkpoint and lifecycle
semantics SHALL remain, with flat lifecycle and explicit close available everywhere.

#### Scenario: Factory Arguments

- **WHEN** callers build execution/cycle discovery or evidence readers
- **THEN** discovery accepts typed identity/status/time/archive/snapshot selections and evidence accepts record/stream, predicate, time, version and snapshot selections
- **AND** cycle evidence can narrow to a member execution without assuming every origin is a runnable plugin case

#### Scenario: Historical Selection Without Follow

- **WHEN** a caller iterates a finite execution query by template, start time and status
- **THEN** it receives typed summaries at a fixed snapshot and terminates without waiting for running work or materializing all history

### Requirement: Typed Rows and Page Control

Readers SHALL expose async items, pages, a snapshot after the first response and explicit
`aclose()`. Concurrent/mixed consumption SHALL fail. Decoding SHALL preserve captured types;
bucket/history rows SHALL use distinct models. Closing/cancelling SHALL stop page requests
without execution lifecycle calls. No reader SHALL require an async context manager.

#### Scenario: Page and Row Metadata

- **WHEN** a reader receives a page
- **THEN** it exposes effective limits, schema, rows, snapshot, continuation and completion, preserving original versions, envelope identity, scalar types, nulls and units
- **AND** compatible snapshots/checkpoints can be reused without fabricating terminal readiness; page sizes obey host limits

#### Scenario: Reuse a Cut and Close Early

- **WHEN** a caller consumes a record page, passes its snapshot to a telemetry query and explicitly closes the record reader
- **THEN** telemetry uses the same cut and the closed reader fetches no more pages
- **AND** the execution continues and server page resources are released

### Requirement: Typed Work Status

The client SHALL export typed cycle and execution status values. Execution status SHALL
represent open/closed/sealed evidence; queued/initializing/stepping/evaluating/cleaning-up
SHALL belong only to manager live state. Summaries SHALL preserve attributed outcomes
and cycle running counts. Filters SHALL accept declared typed status values.

#### Scenario: Inspect Busy Work

- **WHEN** a caller queries active, paused or stopping cycles
- **THEN** their summaries retain cycle state and running count without execution lifecycle calls

#### Scenario: Decode Statuses

- **WHEN** the server returns a declared execution or cycle state
- **THEN** its typed summary preserves that state and attributed outcome
- **AND** unsupported replies fail with a client-facing protocol error
