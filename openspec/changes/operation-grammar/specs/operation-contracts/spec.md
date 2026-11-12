# Operation Contracts Delta

## Purpose

Give product callers one predictable operation grammar and consistent resource,
position, mutation and upload behavior across clients and plugins.
Status: locked by the owner on 2026-11-12. These are target requirements,
not implemented behavior.

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
- **THEN** its declaration states the single lifecycle or side-effect meaning and does not offer a redundant generic Set-status gRPC method

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

### Requirement: Payload-Size-Independent Durable Lag

The newest durable position SHALL trail the newest accepted data by no more than
250 ms at the 95th percentile, independent of payload size, proven by the
Nightly benchmark case under small, multi-megabyte and mixed loads.
Latest-durable/no-wait reads SHALL NOT wait for unsaved accepted data.
The flush cadence alone SHALL NOT count as proof.

#### Scenario: Nightly Measurement Across Payload Sizes

- **WHEN** the durable-lag benchmark case runs in the Nightly benchmark set under the stable case naming convention used by the storage benches
- **THEN** it reports p50, p95 and maximum durable lag separately for small samples at a high rate, multi-megabyte batches and a mix, comparing each p95 with the 250 ms p95 bound
- **AND** a consumer can reuse the case against its own backend, including the independent product store, without wall-clock assertions in correctness tests

#### Scenario: Deterministic Flush Bounds

- **WHEN** input exceeds the configured flush size and the flush mechanism is tested deterministically
- **THEN** the input is split into byte-bounded flush pieces and no single commit exceeds that configured size
- **AND** the test verifies byte bounds and splitting without asserting elapsed time

#### Scenario: Measured Bound Is Missed

- **WHEN** any benchmark load misses the 250 ms p95 bound
- **THEN** smaller flush pieces and flushing as soon as a size threshold is reached, rather than only on the cadence, are tried first and measured again
- **AND** if those measures are insufficient, reads resolving against a snapshot of accepted data are the named fallback; that fallback is not built now

### Requirement: Library-Owned Language SDKs

The library SHALL retain its language SDKs because other products will host
plugins on it. Product SDKs SHALL keep building on them. The library protocol,
SDKs and conformance suite SHALL remain self-sufficient with no product
vocabulary. Remaining SDK decisions SHALL concern only plugin parity across the
four languages and generic managed-client coverage in all four or Rust only.

#### Scenario: Independent Plugin Host

- **WHEN** another product hosts plugins on the library
- **THEN** it can build on the library language SDKs without reimplementing the plugin protocol in every language
- **AND** the library can prove its protocol end to end through its own SDKs and conformance suite without product vocabulary

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

### Requirement: Whole-Value Set with Revision Guard

Set SHALL carry the whole writable value and the expected revision returned by
its matching Get. Get SHALL return that same writable value and revision,
separately from runtime-owned facts. If whole-value replacement is unreasonable,
the writable part SHALL be split into separately settable sub-resources, each
with its own Get, Set and revision. Read-only fields SHALL remain runtime-owned.

#### Scenario: Exact Get and Set Mirrors

- **WHEN** a caller Gets an execution and Sets its whole small writable value with that revision, changing only the label
- **THEN** label, pause and archive values are replaced together under that revision guard and runtime-owned facts remain unchanged
- **AND** the response is a SetExecutionResponse wrapper with the new writable value, revision and receipt

#### Scenario: Revision Conflict

- **WHEN** another mutation has changed the revision returned by Get
- **THEN** Set refuses the stale write with the shared conflict failure and leaves the value unchanged
- **AND** the failure names the action: get again and retry

#### Scenario: Large Writable Resource

- **WHEN** plugin settings or traceability would require an unreasonable whole-value replacement
- **THEN** the resource is split by identified setting or entry into separately revisioned Get/Set sub-resources

### Requirement: Append Ordered Immutable Data

Ordered immutable records, samples, log lines, upload bytes and document contributions SHALL
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
its ordinary gRPC method. Ordinary server streaming SHALL be reserved for Watch;
the plugin protocol SHALL retain its own carriers. Content transfers SHALL use
bounded unary ranges, never streaming download gRPC methods.

#### Scenario: Getter Projection

- **WHEN** contracts for GetExecution or ListRecords are generated
- **THEN** no RunGetExecution or RunListRecords gRPC method is generated

#### Scenario: Promised Cooperative Settlement

- **WHEN** a reviewed method promises a typed final result after cooperative cancellation
- **THEN** one managed method preserves that settlement and no parallel ordinary gRPC method is generated
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

### Requirement: Shared Unary Resumable Uploads

Clients and plugins SHALL share unary CreateUpload, AppendUploadBytes, GetUpload,
PublishUpload and DeleteUpload. Append SHALL accept ordered chunks at stated
offsets; identical replay SHALL succeed and different bytes at an accepted offset
SHALL conflict. Configuration SHALL bound chunk sizes. Publish SHALL verify
complete bytes, length and digest before visibility; Delete SHALL abandon.

#### Scenario: Resume After Lost Reply

- **WHEN** an upload chunk reply is lost and the caller reconnects
- **THEN** GetUpload reports the accepted offset and an identical AppendUploadBytes replays safely
- **AND** different bytes at an accepted offset conflict without changing content

#### Scenario: Partial Upload and Expiry

- **WHEN** publication is requested with missing bytes, a wrong digest or an expired session
- **THEN** it fails explicitly without publishing partial content or silently creating a replacement session

#### Scenario: Ordered and Bounded Chunks

- **WHEN** a new chunk starts after the next accepted offset or exceeds the configured chunk bound
- **THEN** AppendUploadBytes refuses it without changing session data
- **AND** GetUpload exposes the accepted offset, length, digest and session state to resume

### Requirement: Shared Unary Ranged Downloads

Clients and plugins SHALL download through repeated unary GetContent calls with
offset and maximum bytes. Each reply SHALL state total length, content digest
and resolved_at. Configuration SHALL bound range sizes. Streaming download gRPC methods
SHALL be removed in favor of ranged Get reads.

#### Scenario: Download Resume and Bounds

- **WHEN** a client or plugin resumes a content download at its next offset with a maximum byte count
- **THEN** the unary response returns a configuration-bounded range with total length, content digest and resolved_at
- **AND** subsequent ranges preserve the content representation and returned cut

### Requirement: Transfer Throughput Benchmark Before Removal

Large-file upload and download throughput SHALL be measured by a benchmark case
in the Nightly set before superseded streaming transfers are deleted. Throughput
SHALL NOT be a correctness-test assertion.

#### Scenario: Measure Before Streaming Removal

- **WHEN** replacement transfers are ready for deletion of the old streaming paths
- **THEN** the Nightly benchmark case measures large-file upload and download throughput for the shared client/plugin mechanism before those paths are deleted
- **AND** throughput remains a benchmark measurement rather than a correctness-test assertion

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
