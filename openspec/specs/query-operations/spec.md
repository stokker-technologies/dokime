# query-operations Specification

## Purpose
Define the canonical read-only query operations, their configuration, authorization, typed errors and CLI parity.

## Requirements

### Requirement: Canonical Read Operations
Dokime SHALL expose the five unary query operations in `design.md`, each with one canonical request in `dokime-operations`, domain response types and generated contracts. Runtime SHALL own query execution. Queries SHALL be read-only and idempotent at a supplied snapshot.

#### Scenario: Initial Live Query
- **WHEN** a query creates a durable cut during a running execution
- **THEN** it may flush admitted evidence but cannot change the execution lifecycle
- **AND** transport adapters use the runtime semantics without redeclaration

#### Scenario: Contract Discovery
- **WHEN** an authorized caller discovers the operation catalog
- **THEN** each operation has a dotted ID, one request schema, response schema and declared Harmos policy
- **AND** native, gRPC, generated clients and generic CLI calls use that same declaration

### Requirement: Configuration-Owned Policies And Limits
A runtime-owned immutable query configuration SHALL compose canonical Harmos policies and storage/schema/message budgets. It SHALL own query defaults and ceilings, reject invalid settings at composition, and disallow request policy overrides. Page sizes SHALL clamp to effective ceilings; excessive complexity/work SHALL fail explicitly.

#### Scenario: Configuration Composition
- **WHEN** the application constructs its query subsystem
- **THEN** configuration bounds page rows/encoded bytes, decoded working bytes, scanned rows/bytes, active time, concurrency, predicate terms, IN members, fields, buckets, aggregate outputs and snapshot lifetime
- **AND** zero/invalid budgets and work deadlines exceeding declared operation ceilings are rejected
- **AND** omitted request limits use configured defaults; callers may reduce page budgets but cannot supply SQL, timeout extensions or policy overrides

#### Scenario: Restricted Server
- **WHEN** configuration reduces page bytes and scan work below a caller's requested size
- **THEN** page responses advertise effective limits and respect the smaller allocation/encoding budget
- **AND** exceeding scan work produces a typed failure rather than a larger timeout or silent partial answer

### Requirement: Authorization And Cancellation
Queries SHALL use existing caller authorization, including request-dependent execution/cycle/stream read scopes. Each continuation SHALL recheck access; tokens grant no authority. Admission SHALL obey configured concurrency with bounded waiting/cancellation. Cancellation SHALL release query resources without changing execution or evidence.

#### Scenario: Retried Request
- **WHEN** a caller retries an initial query without `at`
- **THEN** it may receive a newer cut, while retrying with the same snapshot preserves the original selection

#### Scenario: Access Revoked Between Pages
- **WHEN** an authorized first page is followed by a continuation after access is revoked
- **THEN** the continuation fails with PermissionDenied without returning additional evidence

#### Scenario: Reader Disconnected
- **WHEN** a client cancels or disconnects during a reduction
- **THEN** the reduction cooperatively stops within the existing operation cancellation policy and releases owned resources
- **AND** the running test continues unchanged

### Requirement: Typed Readable Errors
Query failures SHALL use the existing Harmos failure path with stable typed details: reason, operation, safe message and optional field/operator, snapshot identity and effective limit. Python SHALL preserve status/details/cause in query exceptions; CLI SHALL fail readably. Missing evidence SHALL NOT become an empty success.

#### Scenario: Failure Classification
- **WHEN** a query fails during validation or execution
- **THEN** invalid selection/type/window maps to InvalidArgument, denied access to PermissionDenied, absent scope/contract to NotFound, and incompatible schema or expired/unavailable cuts to FailedPrecondition
- **AND** work/resources map to ResourceExhausted, deadlines to DeadlineExceeded, corruption to DataLoss and unavailable storage to Unavailable
- **AND** remote details omit raw SQL, credentials and host storage paths

#### Scenario: Invalid Field And Lost Storage
- **WHEN** a request names an undeclared field, or a later page loses its backing evidence
- **THEN** the first fails with the field and InvalidArgument and the second with an unavailable-snapshot or storage error
- **AND** neither silently omits rows or substitutes a different snapshot

### Requirement: Cheap CLI Parity
The existing `dokime operation call` command SHALL support the five canonical request/response contracts, including snapshot and continuation fields, without a new parser or bespoke query command tree. JSON replies SHALL retain schema, snapshot and completeness metadata. Documentation SHALL show a first request and continuation using the returned token; dedicated flags and dataframe exports are outside scope.

#### Scenario: CLI Continuation
- **WHEN** a caller invokes `dokime operation call dokime.query.list_records --input request.json --output page.json`
- **THEN** the saved response contains the same typed rows and continuation metadata as the Python page interface

### Requirement: Cycle Work At The Snapshot
Cycle query rows SHALL expose the declared cycle work status and running execution count from the query cut. Status SHALL be captured from the cycle revision; running count SHALL count only Running member executions at that same cut. Existing phase and paging semantics SHALL remain unchanged.

#### Scenario: Work Changes After Capture
- **WHEN** cycle lifecycle or execution status changes after the first page
- **THEN** continuation and snapshot reuse retain the captured status and running count
- **AND** a fresh query observes the newer work state

#### Scenario: Native And Remote Discovery
- **WHEN** native and remote callers query the same snapshot
- **THEN** their cycle status and running count agree

#### Scenario: Running Work Before A Checkpoint
- **WHEN** an execution commits its Running transition before its next checkpoint
- **THEN** a fresh query reports the Running execution and its cycle has a running count of one
- **AND** lifecycle projections commit with the state transition; later transitions and checkpoints preserve earlier query cuts
