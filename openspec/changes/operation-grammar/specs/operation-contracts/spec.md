# Operation Contracts Delta

## Purpose

Give product callers one predictable operation grammar and consistent resource,
position, mutation and upload behavior across clients and plugins.
This is a proposed target awaiting owner lock, not implemented behavior.

## ADDED Requirements

### Requirement: Predictable Operation Names

Operations SHALL use symmetric `VerbResource` names, `<Method>Request` requests
and `<Method>Response` wrappers. Ordinary verbs SHALL be limited to Get, List,
Watch, Set, Create, Delete and Append. A domain verb SHALL document one side
effect that an ordinary verb would hide. Clock waiting SHALL be named Sleep.

#### Scenario: Contract Review

- **WHEN** a public method and its operation declaration are reviewed
- **THEN** the resource name and verb agree across interfaces, the request and response use the method name, and the response is a wrapper even for a single resource
- **AND** Update, Read, Browse, Put, Edit, Add, Remove, Follow, Scan, Inspect and Register are absent as ordinary operation verbs

#### Scenario: Domain Action

- **WHEN** an operation performs Start, Stop, Cancel, Restart, Seal, Publish, Import, Export, Invoke, Advance, Capture, Finish, Abandon or a plugin lifecycle action
- **THEN** its declaration states the single lifecycle or side-effect meaning and does not offer a redundant generic Set-status route

### Requirement: Position-Resolved Resource Reads

Each data kind SHALL have one read path with optional `at` and response
`resolved_at`. Omission SHALL resolve once to latest durable without waiting for
accepted unsaved data. Explicit positions SHALL be previously returned compatible
positions or the seal; unavailable or arbitrary earlier positions SHALL fail.

#### Scenario: Live Read with Unsaved Writes

- **WHEN** accepted writes extend beyond the durable frontier and a read omits `at`
- **THEN** it returns the latest durable cut and its `resolved_at` without requesting a flush barrier or constructing an accepted-data snapshot
- **AND** later durable writes cannot alter that selected cut

#### Scenario: Sealed Execution

- **WHEN** an execution has a durable seal and a read omits `at`
- **THEN** the ordinary resource read resolves to the seal and its frozen evidence/context without calling a checkpoint-specific operation

#### Scenario: Read Your Write

- **WHEN** a caller reads with the compatible durable position returned in its write receipt
- **THEN** that cut includes the write and never silently advances to another cut
- **AND** an accepted-only position does not imply saved visibility

#### Scenario: Caller-Invented Earlier Position

- **WHEN** a caller supplies an earlier sequence that is neither a returned compatible position nor the seal
- **THEN** the read fails explicitly rather than offering arbitrary historical selection

### Requirement: Snapshot Then Watch

Watch SHALL use the same selection and position rule as its corresponding read,
open with a bounded snapshot/boundary at `resolved_at`, then emit durable changes
strictly after it. Gaps SHALL require explicit resynchronization. Explicit `at`
SHALL pin the opening snapshot, not freeze the subscription forever.

#### Scenario: Watch from a Returned Cut

- **WHEN** a caller opens WatchRecords with the compatible position from ListRecords
- **THEN** the opening identifies that cut and subsequent changes advance strictly after it with no duplicated or omitted matching records
- **AND** filtered-out durable progress can advance the resume position

#### Scenario: Retention Gap

- **WHEN** a watch's required continuation has expired or its history is unavailable
- **THEN** it reports resynchronization and does not pretend a fresh latest snapshot is continuous history

### Requirement: One Bounded Paging Shape

Paged reads SHALL use `page_size`, `max_bytes`, `page_token`, `next_page_token`
and `resolved_at`. Tokens SHALL bind cut, selection, ordering and schema/projection
revision. Continuation SHALL be exclusive in stable order, with row and byte
bounds independent of physical work limits.

#### Scenario: Collection Changes Between Pages

- **WHEN** a caller continues a page while later writes change the collection
- **THEN** it receives the next page at the original cut in stable order with identifier tie-breaking
- **AND** changing the selection or using an expired token fails explicitly

### Requirement: Masked Set with Revision Guard

Set SHALL accept an optional field mask; an absent mask SHALL replace the whole
writable value. Resources with revisions SHALL enforce the expected revision.
Read-only fields SHALL remain runtime-owned. An invalid or explicitly empty mask
SHALL fail without mutation.

#### Scenario: Whole Value and Partial Value

- **WHEN** a caller Sets a value without a mask and with the correct expected revision
- **THEN** all writable fields are replaced consistently
- **AND** when a nonempty valid mask is supplied only those writable fields change

#### Scenario: Revision Conflict

- **WHEN** another mutation has changed the expected revision
- **THEN** Set refuses the stale write with the shared conflict failure and leaves the value unchanged

### Requirement: Append Ordered Immutable Data

Ordered immutable records, samples, log lines and document contributions SHALL
use Append with mutation receipts. Create SHALL allocate lifecycle resources.
Retrying an Append with the same mutation key and normalized payload SHALL replay
the original outcome; changed payload SHALL conflict.

#### Scenario: Replayed Record Append

- **WHEN** a plugin retries an acknowledged record Append after losing its reply
- **THEN** the same receipt is returned and only one ordered record is retained
- **AND** the receipt distinguishes accepted position from durable position

#### Scenario: Different Payload for One Key

- **WHEN** a caller repeats the key with a changed record or log payload
- **THEN** the operation fails rather than silently appending a second value

### Requirement: Explicit Streaming Contracts

The protocol SHALL NOT automatically generate streaming twins. Durable work SHALL
use unary operations with resource Get/Watch. Only a method promising a cooperative
final result after cancellation SHALL use a canonical managed stream replacing
its ordinary route. Bounded response streaming SHALL NOT imply another route.

#### Scenario: Getter Projection

- **WHEN** contracts for GetExecution or ListRecords are generated
- **THEN** no RunGetExecution or RunListRecords route is generated

#### Scenario: Promised Cooperative Settlement

- **WHEN** a reviewed method promises a typed final result after cooperative cancellation
- **THEN** one managed method preserves that settlement and no parallel ordinary route is generated
- **AND** cancelling an ordinary observer does not cancel already admitted durable resource work

### Requirement: Execution Stop and Cancel

CancelExecution SHALL interrupt only an execution with a running invocation.
Queued cancellation SHALL be refused with an error naming the action.
StopExecution SHALL finish at a safe scheduling boundary. Both SHALL preserve
required cleanup and evidence; cancelling observation SHALL be separate.

#### Scenario: Queued Cancellation

- **WHEN** CancelExecution targets an execution that has not started
- **THEN** it fails with an action-named refusal and does not skip or cancel that queued execution

#### Scenario: Active Invocation

- **WHEN** CancelExecution targets a running invocation
- **THEN** interruption reaches that invocation and required cleanup remains scheduled
- **AND** StopExecution instead prevents further scheduling at a safe boundary

### Requirement: One-Execution Rename

Changing an execution's label SHALL affect only that execution, preserving
other attempts of its case and the case identity. Clients SHALL describe this
scope consistently.

#### Scenario: Sibling Attempt

- **WHEN** a caller Sets the label of one of two executions for the same case
- **THEN** only the selected execution's label changes and the sibling label remains unchanged

### Requirement: Shared Unary Upload Sessions

Clients and plugins SHALL use the same resumable unary upload sessions. Sessions
SHALL expose creation, status, chunk setting, deletion and publication. Publication
SHALL verify complete bytes, length and digest before immutable content becomes
available to artifact or plugin creation.

#### Scenario: Resume After Lost Reply

- **WHEN** an upload chunk reply is lost and the caller reconnects
- **THEN** GetUpload reports committed ranges and an identical SetUploadChunk can replay safely
- **AND** conflicting bytes for the same range fail

#### Scenario: Partial Upload and Expiry

- **WHEN** publication is requested with missing bytes, a wrong digest or an expired session
- **THEN** it fails explicitly without publishing partial content or silently creating a replacement session

### Requirement: Shared Failures and Retry Policy

Interfaces SHALL use one Failure vocabulary and one retry-policy vocabulary,
preserving causes and canonical gRPC status/details. Budgets SHALL use Duration,
instants Timestamp, and total attempts SHALL include the first. Retry SHALL require
explicit replay safety; invocation correlation alone SHALL NOT deduplicate.

#### Scenario: Child Failure and Per-Call Override

- **WHEN** a child operation fails under a per-call retry override
- **THEN** code, classification, action, safe details and cause survive propagation
- **AND** a subsequent call without the override uses the declaration policy unchanged

#### Scenario: Accepted Is Not Durable

- **WHEN** a mutation is accepted but not saved
- **THEN** its receipt cannot claim a durable position and a transport failure cannot be reported as proof that the mutation was not applied
