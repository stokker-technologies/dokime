## ADDED Requirements

### Requirement: Post-gRPC Sequencing

This change SHALL start after `grpc-plugins` wave 5. Harmos SHALL design and implement the five generic storage capabilities before Dokime extracts storage, and Dokime SHALL pin their published revision before final validation.

#### Scenario: Extraction Starts

- **WHEN** Dokime begins extracting application persistence
- **THEN** the last gRPC wave is closed and the Harmos capabilities are available

### Requirement: Single Persistence Owner

`dokime-storage` SHALL merge both runtime storage folders and own application persistence through the Harmos storage interface. It SHALL own persisted shapes and depend on protocol, never runtime, operations, rendering or a domain crate.

#### Scenario: Host Persists Data

- **WHEN** an operation needs application persistence
- **THEN** it uses storage resources without concrete backend access or a reverse runtime dependency

### Requirement: Named Exceptions

Logs, support bundles, exports and report spools, plugin installation, desktop root lease and backup, client pairing and download files, and command-line/build tooling SHALL remain with their owners outside application storage.

#### Scenario: Local File Is Owned Elsewhere

- **WHEN** a report renderer creates a spool or a client writes a download
- **THEN** its owner retains local-file responsibility without depending on host storage

### Requirement: Native Columnar Scans

Harmos SHALL support bounded native columnar scans with projection, predicate/time pruning, position continuation and explicit fallback at a stable snapshot.

#### Scenario: Columnar Data Is Read

- **WHEN** a backend supports native scans for the requested columns and time range
- **THEN** the query returns bounded columnar pages without reconstructing row payloads

### Requirement: Indexed Seek Search

Harmos SHALL support indexed predicates, secondary/prefix keys and ordered seek pages with stable snapshot semantics, without product-specific history meanings.

#### Scenario: Search Continues

- **WHEN** a caller resumes an indexed search at the same snapshot
- **THEN** results follow the declared order without duplicates or omissions

### Requirement: Durable Projections

Harmos SHALL expose durable materialized projections with commit-coupled advancement, a readable frontier, replay/invalidation rules and snapshot-consistent reads.

#### Scenario: Projection Restarts

- **WHEN** a host restarts after projection advancement
- **THEN** reads expose the durable frontier and replay preserves the committed result

### Requirement: Time-Bucket Transform

Harmos SHALL provide time buckets as a query transform with late-sample replacement, changed keys/revisions, first/last/min/max, moments, source positions and snapshot visibility.

#### Scenario: Late Sample Arrives

- **WHEN** a sample changes an existing time bucket
- **THEN** live queries observe its replacement while saved cuts retain their snapshot result

### Requirement: Conditional Update

Harmos SHALL support atomic expected-revision check and replacement, including absence/conflict semantics and a durable receipt.

#### Scenario: Revision Conflicts

- **WHEN** a write's expected revision differs from the stored revision
- **THEN** it returns a conflict without overwriting the current value

### Requirement: Product Semantics Retained

Dokime SHALL retain execution/step statistics, history filters/tokens, finalization/seals, report capture, artifact inventory and fresh-root refusal above generic storage. Runtime SHALL retain transactions, state, cycles and effects; storage SHALL use explicit inputs/results rather than host callbacks.

#### Scenario: Root Is Incompatible

- **WHEN** startup cannot read Dokime-owned durable state
- **THEN** it refuses the root without deleting or resetting data

### Requirement: Recovery and Durability

Extraction SHALL preserve required backend/file capabilities, replay-idempotence, recovery data frontiers, query cuts and durability barriers. Independent backend methods SHALL NOT be treated as a cross-method transaction.

#### Scenario: Finalization Is Interrupted

- **WHEN** a host restarts during finalization
- **THEN** recovery respects durable intents and required frontiers without inventing a completed seal

### Requirement: Comparative Scale Proof

Before-and-after benchmarks SHALL compare post-gRPC baseline, upstream adoption and extraction on native Linux and Windows, including deterministic multi-day/million-sample data, concurrent reads/writes, restart and late samples. Results SHALL cover throughput, latency, stop/cancel/report time, peak memory, bytes read and durability cost.

#### Scenario: Performance Is Assessed

- **WHEN** extraction is evaluated
- **THEN** comparable measurements and deterministic correctness proof account for regressions
