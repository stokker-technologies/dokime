## ADDED Requirements

### Requirement: Execution And Cycle History Selection
History queries SHALL search the backend's local retained catalog across executions/cycles, including durable live revisions. Execution filters SHALL support template/cycle IDs, status and started_at windows; cycles SHALL support ID, phase and created_at windows. Filters SHALL be typed, exact and conjunctive, with bounded alternatives. Archive selection SHALL default to unarchived and allow archived/all.

#### Scenario: History Filter Boundaries
- **WHEN** typed history filters use `[since, until)` or failed execution status
- **THEN** start bounds exclude unstarted executions, while unbounded selection can include them
- **AND** failed means the declared status, not text inference; template/phase/status values are validated without SQL or glob matching

#### Scenario: Failed Template Executions Across Cycles
- **WHEN** a caller selects template `example.demo.tracking`, failed status and since `2026-10-01`
- **THEN** matching executions from every local retained cycle are returned using captured template identity and a UTC inclusive lower start bound
- **AND** an execution at until is excluded and an archived execution is excluded unless explicitly requested

### Requirement: Stable Typed Discovery
Typed summaries SHALL expose captured metadata and evidence availability without implying checkpoint readiness. Execution order SHALL be `(started_at ascending, execution_id ascending)`, missing times last; cycles SHALL order by `(created_at ascending, cycle_id ascending)`. Snapshot paging SHALL freeze membership and values.

#### Scenario: Summary Projection
- **WHEN** discovery returns execution and cycle summaries
- **THEN** executions include ID, cycle/case/template IDs, frozen label, status, start/finish times, result and evidence/checkpoint availability
- **AND** cycles include ID, label, phase, created/updated times and captured membership count, with missing metadata explicit
- **AND** execution IDs can be opened and compatible returned snapshots reused for finite evidence reads

#### Scenario: Status Changes After First Page
- **WHEN** a running execution completes or is relabeled after a history snapshot
- **THEN** remaining pages keep the original membership/status/order and a fresh search observes the later revision
- **AND** opening an uncheckpointed summary never fabricates terminal evidence readiness

### Requirement: SQLite Catalog Index Ownership
History SHALL use versioned SQLite catalog projections keyed by identity and revision/frontier. Plans SHALL seek latest eligible revisions without loading all documents or Arrow files. Projection visibility, deletion/archive state and authoritative metadata SHALL commit atomically. Recovery SHALL not expose incomplete indexes as complete.

#### Scenario: Catalog Index Construction
- **WHEN** history projections are created or recovered from retained metadata
- **THEN** execution indexes support template/status/cycle plus start-time/ID seeks, and cycle indexes phase plus creation-time/ID seeks, including archive selection
- **AND** recovery invents no evidence; incompatible layouts require an explicitly selected fresh v1 root with old data preserved

#### Scenario: Indexed Historical Search
- **WHEN** a selective template/status/time query runs over a large local history
- **THEN** it uses catalog index seeks and bounded page reads rather than scanning full execution documents
- **AND** later revisions beyond the snapshot cannot overwrite the older selected projection

#### Scenario: Interrupted Projection Update
- **WHEN** the process fails during a metadata/index write and restarts
- **THEN** the catalog exposes only committed matching metadata/index state or refuses queries until recovery completes

### Requirement: Explicit Local Scope
New query snapshots SHALL cover only the connected server's local catalog/store. External adapters and other servers SHALL NOT be silently merged into a supposedly atomic cut. Existing merged `History.BrowseExecutions`/`BrowseCycles` APIs SHALL remain available with their existing discovery issues/conflict semantics; they SHALL NOT be advertised as the new snapshot query contract. An explicit unsupported source request SHALL fail with a typed error.

#### Scenario: External Source Requested
- **WHEN** a caller requests a query snapshot spanning another backend or external adapter
- **THEN** the server rejects that selection and identifies local scope rather than returning an apparently complete subset
