## ADDED Requirements

### Requirement: One Evidence And Metadata Cut
The first evidence query without `at` SHALL capture admitted evidence and corresponding membership, revisions and schemas at one logical cut, flush only through it and return its identity. Later evidence SHALL be excluded. A supplied `at` SHALL select the original retained cut for compatible queries in scope.

#### Scenario: Capture Without Finalization
- **WHEN** a live evidence query freezes context and its evidence frontier
- **THEN** it needs no report publication, seal or provider stop, and a cycle cut freezes member revisions and membership
- **AND** initial history queries capture already durable catalog revisions without flushing all running executions

#### Scenario: Live Queries During Writes
- **WHEN** a first record page captures a running execution while sample batches, metadata updates and new records arrive
- **THEN** all subsequent pages and a telemetry query using the returned snapshot see only that cut's evidence/contracts/revisions
- **AND** writes continue and a fresh query can see later durable evidence

#### Scenario: New Cycle Member
- **WHEN** an execution is appended after a cycle snapshot is captured
- **THEN** that snapshot's record query excludes the new member's evidence

### Requirement: Snapshot Identity And Availability
Snapshot tokens SHALL be opaque, authenticated and bounded, identifying backend/store generation, local source, scope, metadata/evidence frontier and expiry. They SHALL contain no unbounded row/member list. No transaction or client-held lock SHALL span pages. Unavailable cuts SHALL fail explicitly without restarting at latest.

#### Scenario: Checkpoint Or Query Token
- **WHEN** a query uses a compatible saved checkpoint as its starting cut
- **THEN** it retains that checkpoint's evidence boundary; a live query token does not imply a terminal checkpoint
- **AND** access is checked each call and expiry, deletion, retention or generation changes fail explicitly

#### Scenario: Restart And Deletion
- **WHEN** a client resumes after restart with an unexpired token whose exact cut remains retained
- **THEN** it continues the original cut
- **AND** if the execution was deleted or its cut is unavailable it fails with a typed snapshot error

### Requirement: Deterministic Seek Pagination
Row responses SHALL contain typed rows/schema, snapshot, effective limits, optional continuation and complete. Complete SHALL mean the selection is exhausted with no continuation. Seek tokens SHALL bind snapshot, canonical selection/projection/reduction and last scanned key. Repeated identical requests SHALL return the same logical page at that cut.

#### Scenario: Stable Page Order And Size
- **WHEN** a caller pages raw evidence, history or buckets and changes only page row/byte sizes within limits
- **THEN** raw evidence orders by durable position then occurrence ID, history by history-search rules, and buckets by start then partition identity
- **AND** tokens track scanned keys including excluded rows, and empty pages may continue with progress

#### Scenario: Sparse Matches Across Pages
- **WHEN** a page scans its admitted chunk without matching any record and later chunks contain matches
- **THEN** it returns zero rows, complete=false and a continuation after the scanned chunk
- **AND** following all tokens returns every match once without offset scans, gaps or duplicates

#### Scenario: Continuation Reused With Another Predicate
- **WHEN** a caller changes a predicate or projection while reusing a continuation
- **THEN** the server rejects the continuation instead of restarting or changing the selection

### Requirement: Bounded Execution And Streaming Consumption
Server readers SHALL bound decoded batches, rows, accumulators, encoding and loans by configuration, independently of history/page count. Python SHALL pull pages on demand with bounded prefetch. Oversized rows/schemas and work exhaustion SHALL fail explicitly. Statistics SHALL never be partial; failed row iteration SHALL remain visibly incomplete.

#### Scenario: Encoding And Work Budgets
- **WHEN** a page admits data for encoding or hits its scan/time ceiling
- **THEN** schema and metadata count against budgets, and oversized rows/schemas fail without splitting, omission or infinite continuation
- **AND** work exhaustion raises a typed failure, not a successful partial statistic

#### Scenario: Slow Consumer And Oversized Row
- **WHEN** a slow reader consumes a long history and later reaches a row larger than the configured page-byte ceiling
- **THEN** server/client buffers stay bounded and the oversized row produces ResourceExhausted
- **AND** no successful complete collection is returned
