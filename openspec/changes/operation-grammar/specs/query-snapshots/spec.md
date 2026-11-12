# Query Snapshots Delta

## MODIFIED Requirements

### Requirement: One Evidence and Metadata Cut

The first evidence query without `at` SHALL resolve once to the latest durable
evidence and corresponding membership, revisions and schemas at one logical cut,
without waiting for unsaved accepted data. Later evidence SHALL be excluded.
A supplied `at` SHALL select a compatible previously returned retained cut or
the seal; arbitrary earlier positions SHALL be refused. Replies SHALL state
`resolved_at`. The product SHALL own the storage schemas, indexes, projections
and meaning of latest that define this cut; generic library services SHALL NOT
redefine those object semantics.

#### Scenario: Capture Without Finalization

- **WHEN** a live evidence query freezes context and its evidence frontier
- **THEN** it needs no report publication, seal, provider stop or durability barrier, and a cycle cut freezes member revisions and membership
- **AND** the selected journal/metadata frontier is already durable without checkpointing running executions
- **AND** latest-durable/no-wait reads follow the locked payload-size-independent 250 ms p95 durable-lag bound in the operation-contracts delta, proven by the reusable Nightly storage benchmark rather than wall-clock test assertions

#### Scenario: Live Queries During Writes

- **WHEN** a first record page captures a running execution while sample batches, metadata updates and new records arrive
- **THEN** all subsequent pages and a telemetry query using the returned position see only that cut's evidence/contracts/revisions
- **AND** writes continue and a fresh query can see later durable evidence
- **AND** accepted unsaved evidence is excluded without delaying the read for a flush

#### Scenario: New Cycle Member

- **WHEN** an execution is appended after a cycle snapshot is captured
- **THEN** that snapshot's record query excludes the new member's evidence

#### Scenario: Explicit Position Provenance

- **WHEN** a caller selects a compatible durable position from its write receipt, a previous read reply or the execution's seal
- **THEN** it receives that exact retained cut or an explicit unavailable-cut failure
- **AND** an arbitrary caller-chosen earlier sequence is refused

### Requirement: Snapshot Identity and Availability

Returned positions SHALL use `(history, sequence)` with history identifying store
and epoch. Bounded authenticated selection metadata SHALL bind scope, evidence and
metadata revisions, source and expiry without unbounded member lists. No transaction
or client-held lock SHALL span pages. Unavailable cuts SHALL fail without restarting
at latest; a composite cut SHALL NOT be reduced to an unqualified scalar sequence.

#### Scenario: Checkpoint or Query Token

- **WHEN** a query uses the execution's saved seal or a compatible previously returned position as its starting cut
- **THEN** the ordinary resource read retains that evidence boundary without a separate checkpoint operation family; a live returned cut does not imply a terminal seal
- **AND** access is checked each call and expiry, deletion, retention or history changes fail explicitly

#### Scenario: Restart and Deletion

- **WHEN** a client resumes after restart with an unexpired token whose exact cut remains retained
- **THEN** it continues the original cut
- **AND** if the execution was deleted or its cut is unavailable it fails with a typed snapshot error

### Requirement: Deterministic Seek Pagination

Row responses SHALL contain typed rows/schema, `resolved_at`, effective limits,
`next_page_token` and complete. Requests SHALL use `page_size`, `max_bytes` and
`page_token`. Complete SHALL mean exhaustion without continuation. Tokens SHALL
bind cut, canonical selection/projection/reduction, schema revision and last scanned
key. Repeated identical requests SHALL return the same logical page at that cut.

#### Scenario: Stable Page Order and Size

- **WHEN** a caller pages raw evidence, history or buckets and changes only page row/byte sizes within limits
- **THEN** raw evidence orders by durable position then occurrence ID, history by history-search rules with an identifier tie-breaker, and buckets by start then partition identity
- **AND** tokens track scanned keys including excluded rows, and empty pages may continue with progress

#### Scenario: Sparse Matches Across Pages

- **WHEN** a page scans its admitted chunk without matching any record and later chunks contain matches
- **THEN** it returns zero rows, complete=false and a continuation after the scanned chunk
- **AND** following all tokens returns every match once without offset scans, gaps or duplicates

#### Scenario: Continuation Reused with Another Predicate

- **WHEN** a caller changes a predicate or projection while reusing a continuation
- **THEN** the server rejects the continuation instead of restarting or changing the selection
