# Operation Contracts Delta

## Purpose

Give product callers one predictable operation grammar and consistent resource,
position, mutation and upload behavior across clients and plugins.
Status: locked by the owner on 2026-11-12
Boundary, managers and nested resources locked by the owner on 2026-11-12
Naming rule amended by the owner on 2026-11-12: bare verbs on the service's resource

These are target requirements, not implemented behavior.

## ADDED Requirements

### Requirement: Predictable Operation Names

Services SHALL name their resource with a plural noun, or a singular noun for
one thing. Methods acting on that resource SHALL be bare verbs and SHALL NOT
repeat its noun. Mixed-resource services SHALL keep distinguishing resource nouns,
without repeating the service name, and SHALL NOT be split merely to shorten names.

#### Scenario: Mixed-Resource Service

- **WHEN** Access allocates a pairing or regenerates a certificate
- **THEN** Access.CreatePairing and Access.CreateCertificate state those single lifecycle actions
- **AND** the separately readable/writable listener uses Access.Listener.Get/Set

### Requirement: Resource Part Names

A part read in exactly one way SHALL use its noun alone. A part with multiple read
or change paths SHALL be a nested resource with its own bare verbs, replacing the
earlier verb-plus-part rule and flat compound resource services.

#### Scenario: Single-Mode Part and Revisioned Settings

- **WHEN** execution parameters and steps are paged, and plugin or plot settings are read and changed
- **THEN** the methods are Executions.Parameters, Executions.Steps, Plugins.Settings.Get/Set and Plots.Settings.Get/Set, with no automatic parameter or step watches
- **AND** each settings Get returns the exact writable value and revision expected by Set

#### Scenario: Progress Observation and Production

- **WHEN** a client reads execution progress or follows its stream
- **THEN** Executions.Progress.Get returns the current value and Executions.Progress.Watch follows changes, and Executions.Get includes that same value at its resolved cut
- **AND** a plugin produces progress by calling a host operation; no client-facing Set is declared or generated

### Requirement: Nested Service Wire Layout

A nested resource SHALL be a service in package `<owner>.v1.<lowercase-parent>`,
with the part as its service name, one proto file per service and a folder per
parent. Its messages SHALL use the local service prefix `<Service><Method>Request`
and `<Service><Method>Response`. Parent and nested resource MAY have different
library/product owners; composition SHALL preserve both namespaces.

#### Scenario: Product Settings Beside Library Plugins

- **WHEN** the host application serves library Plugins and product plugin settings
- **THEN** the product declares `product.v1.plugins.Settings` in `product/v1/plugins/settings.proto`, beside library Plugins, and the full nested Get path is `product.v1.plugins.Settings/Get`
- **AND** its wrappers are SettingsGetRequest/Response and SettingsSetRequest/Response, not flat PluginSettings wrappers or Plugins.GetSettings

### Requirement: Generated Method Messages and Clients

Messages SHALL follow the service-prefixed method naming rule in their package.
Responses SHALL always be wrappers. All four SDKs SHALL derive their client root
hierarchy mechanically from service packages and names through a small build-time
generator consuming the same descriptor-derived table, with no hand-written
per-operation code. Composition SHALL merge distinct library/product resource
children and reject collisions; language casing SHALL NOT change hierarchy.

#### Scenario: Contract Review

- **WHEN** Executions.Start, Plugins.Get and product Plugins.Settings.Get are generated
- **THEN** their requests are ExecutionsStartRequest, PluginsGetRequest and SettingsGetRequest in their respective packages, and responses are method-specific wrappers
- **AND** each SDK exposes the equivalent of client.executions.start(request), client.plugins.get(request) and client.plugins.settings.get(request)

#### Scenario: Progress Hierarchy

- **WHEN** a client subscribes to execution progress through an SDK
- **THEN** the generated call is client.executions.progress.watch(request), using the Progress service in the executions sub-package
- **AND** no client progress Set or hand-written naming alias is generated

### Requirement: Library Machinery and Product Composition

The library SHALL own Plugins, access, transfers and host status under the
managers listed in design.md, alongside its existing generic runtime surfaces.
A product SHALL own its storage definition, objects, operations and policy, and
serve library services directly by composition without redeclaring or wrapping
them. It SHALL NOT rebuild generic machinery on the protocol crate alone.

#### Scenario: Direct Library Client Calls on a Product Endpoint

- **WHEN** a client manages a plugin, configures the listener, transfers bytes or reads host health on the host application's endpoint
- **THEN** it calls the library Plugins, Access/Access.Listener, Storage or Status services directly
- **AND** PluginManager owns Plugins; GrpcManager owns access, listener, pairing, certificates and key rotation; StorageManager owns transfers; ServiceManager owns host status
- **AND** the product has no second connection, dispatch, stream, process-launch, entry-management, listener/pairing, upload-handling or host-status implementation; the earlier protocol-only approach produced those duplicates
- **AND** product storage ownership includes schemas, indexes, projections and the meaning of latest

#### Scenario: Narrow Host-Fact Contribution

- **WHEN** a host application contributes its version and component-health facts
- **THEN** narrow registration supplies those facts to the library service manager's status services
- **AND** reads unique to the product remain on its own services rather than duplicating generic identity or health

### Requirement: Product Settings and Secrets

Settings and secrets SHALL remain product-owned, including stores, schemas,
UI declarations, archives and operating-system credential-store use. The library
SHALL supply only generic host-operation calls by which a host application serves
values to plugins, and each plugin's settings declarations as a catalog section.
The per-plugin settings/secrets ownership question SHALL be treated as closed.

#### Scenario: Plugin Requests a Value

- **WHEN** a plugin declares its settings and requests a setting or secret
- **THEN** the catalog carries declarations and library HostValues.Get calls the registered product resolver with its policy and stores
- **AND** the library introduces no opinionated settings/secret store or archive

### Requirement: Manager-Owned Orchestration

The product SHALL split Host/shared State into execution, prompt and report
managers, with capture only if recordings remain, under the single manager rule
in design.md. Each service method SHALL map to one operation file which calls a
manager or storage. Rendering SHALL remain pure. Evidence, catalog, notes,
aggregates, plots and documents SHALL remain storage operations without managers.

#### Scenario: Lifecycle Ownership Split

- **WHEN** Host and shared State are split
- **THEN** the execution manager owns queueing, lifecycle, retries, Stop/Cancel and recovery; the prompt manager owns pending questions; the report manager owns atomic publication
- **AND** cycles belong to that execution manager or a small cycle manager, chosen at the split by the simplest complete option; a capture manager exists only if recordings remain

#### Scenario: Publication Versus Rendering

- **WHEN** a report is rendered for preview and later published
- **THEN** the renderer is a pure function and atomic publication is owned by the report manager
- **AND** a service handler delegates through one operation file rather than retaining orchestration in shared State

#### Scenario: Storage-Only Read

- **WHEN** a caller reads evidence, catalog, notes, aggregates, plots or documents
- **THEN** its operation accesses storage without introducing a manager for that collection

### Requirement: Complete Composed Client Inventory

Implementation SHALL preserve all 132 ordinary mapping rows and account for
132 removed Run twins, with one destination/workflow per row and explicit
ownership. The client inventory SHALL match design.md's per-owner totals;
plugin carriers and the library's other existing surfaces SHALL remain separate.

#### Scenario: Mapping and Count Review

- **WHEN** the declarations and generated outputs are reviewed against design.md
- **THEN** all 132 ordinary rows and their Run twins are accounted for once, and the client inventory has 39 services (23 nested), 120 methods and 240 method wrappers
- **AND** library mapped-client totals are 7 services (3 nested), 27 operations/methods and 27 request/response pairs; product client totals are 32 services (20 nested), 93 operations/methods and 93 pairs
- **AND** product Content.ExecutionFiles.Get resolves metadata for library Storage.WatchFileBytes growth/replacement/truncation; Content.ExecutionFiles.Watch follows product metadata/membership

#### Scenario: Whole-Owner Operation Census

- **WHEN** host/plugin-only templates and the complete library protocol are included without double-counting shared operations
- **THEN** library totals are 13 logical services (3 nested), 67 operations, 61 gRPC methods and 67 request/response pairs; product totals are 39 logical services (22 nested), 111 operations, 93 gRPC methods and 111 pairs
- **AND** supporting payloads and protocol frames remain separate from method wrappers

### Requirement: Product Evidence Archive and Generic Delivery

Evidence.Archive SHALL select the product's evidence at a compatible cut and
return a bounded immutable content reference. Library Storage.GetFileBytes SHALL deliver
its ranges without interpreting product evidence selectors, policy or archive
layout. Archive assembly SHALL remain an operation over storage with no manager.

#### Scenario: Download Evidence

- **WHEN** an authorized caller downloads an execution's evidence archive
- **THEN** product Evidence.Archive binds the scope, cut and archive representation, and library Storage.GetFileBytes returns bounded ranges with total length and digest
- **AND** the library does not acquire product object semantics or duplicate the archive-selection operation

### Requirement: Closed Ordinary Verb Vocabulary

Ordinary verbs SHALL be limited to Get, List, Watch, Set, Create, Delete and Append.
A domain verb SHALL document one side effect that an ordinary verb would hide.
Clock waiting SHALL be named Sleep.

#### Scenario: Ordinary Verb Review

- **WHEN** a method uses an ordinary verb
- **THEN** Update, Read, Browse, Put, Edit, Add, Remove, Follow, Scan, Inspect and Register are absent as ordinary operation verbs

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

- **WHEN** a caller opens Evidence.Records.Watch with the compatible position from Evidence.Records.List
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
- **AND** the response is an ExecutionsSetResponse wrapper with the new writable value, revision and receipt

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

The protocol SHALL NOT generate streaming twins. Durable work SHALL use unary
operations with resource Get/Watch. Only a cooperative final-result promise after
cancellation SHALL use a managed stream replacing the ordinary method. Ordinary
server streaming SHALL serve Watch semantics, including noun-only streamed parts.
Plugin carriers SHALL remain. Content transfers SHALL use bounded unary ranges.

#### Scenario: Getter Projection

- **WHEN** contracts for Executions.Get or Evidence.Records.List are generated
- **THEN** no Executions.RunGet or Evidence.Records.RunList gRPC method is generated

#### Scenario: Promised Cooperative Settlement

- **WHEN** a reviewed method promises a typed final result after cooperative cancellation
- **THEN** one managed method preserves that settlement and no parallel ordinary gRPC method is generated
- **AND** cancelling an ordinary observer does not cancel already admitted durable resource work

### Requirement: Execution Stop and Cancel

Executions.Cancel SHALL interrupt only an execution with a running invocation.
Queued cancellation SHALL be refused with an error naming the action.
Executions.Stop SHALL finish at a safe scheduling boundary. Both SHALL preserve
required cleanup and evidence; cancelling observation SHALL be separate.

#### Scenario: Queued Cancellation

- **WHEN** Executions.Cancel targets an execution that has not started
- **THEN** it fails with an action-named refusal and does not skip or cancel that queued execution

#### Scenario: Active Invocation

- **WHEN** Executions.Cancel targets a running invocation
- **THEN** interruption reaches that invocation and required cleanup remains scheduled
- **AND** Executions.Stop instead prevents further scheduling at a safe boundary

### Requirement: One-Execution Rename

Changing an execution's label SHALL affect only that execution, preserving
other attempts of its case and the case identity. Clients SHALL describe this
scope consistently.

#### Scenario: Sibling Attempt

- **WHEN** a caller Sets the label of one of two executions for the same case
- **THEN** only the selected execution's label changes and the sibling label remains unchanged

### Requirement: Shared Unary Resumable Uploads

The library storage manager SHALL own the one client/plugin transfer mechanism:
unary Storage.CreateFileProduction, GetFileProduction, AppendFileBytes,
CreateFileWriter, PublishFileProduction and DeleteFileProduction. Append SHALL accept
bounded ordered chunks, replay identical bytes and reject conflicting bytes at
accepted offsets. Publish SHALL verify complete bytes, length and digest before
visibility; Delete SHALL abandon. Expiry SHALL be explicit.

#### Scenario: Resume After Lost Reply

- **WHEN** an upload chunk reply is lost and the caller reconnects
- **THEN** Storage.GetFileProduction reports the accepted offset and an identical Storage.AppendFileBytes replays safely
- **AND** CreateFileWriter fences the old writer when resume needs a new writer generation, while different bytes at an accepted offset conflict without changing content

#### Scenario: Partial Upload and Expiry

- **WHEN** publication is requested with missing bytes, a wrong digest or an expired session
- **THEN** it fails explicitly without publishing partial content or silently creating a replacement session

#### Scenario: Ordered and Bounded Chunks

- **WHEN** a new chunk starts after the next accepted offset or exceeds the configured chunk bound
- **THEN** Storage.AppendFileBytes refuses it without changing session data
- **AND** Storage.GetFileProduction exposes the accepted offset, length, digest and session state to resume

### Requirement: Shared Unary Ranged Downloads

The library storage manager SHALL serve Storage.GetFileBytes directly. Clients and plugins
SHALL download through repeated unary Storage.GetFileBytes calls with
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

#### Scenario: Lost Client Call

- **WHEN** a call fails on the client and its outcome is unknown
- **THEN** resending is the client's responsibility, and the same request identifier and unchanged payload return the original receipt
- **AND** a changed payload under that same identifier is a conflict
