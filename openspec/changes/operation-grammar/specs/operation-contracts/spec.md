# Operation Contracts Delta

## Purpose

Give product callers one predictable operation grammar and consistent resource,
position, mutation and upload behavior across clients and plugins.
Status: locked by the owner on 2026-11-12
Boundary, managers and nested resources locked by the owner on 2026-11-12
Naming rule amended by the owner on 2026-11-12: bare verbs on the service's resource
Separation by audience locked by the owner on 2026-11-12
Three visibility levels locked by the owner on 2026-11-12
Paging, SDK packaging, transfers and durable lag amended by the owner on 2026-11-13

These are target requirements, not implemented behavior.

## ADDED Requirements

### Requirement: Audience Faces and Package Boundaries

Each service SHALL have exactly one audience face, visible in its package:
plugin protocol in `harmos.plugin.v1` (Instance only); product catalog operations
in `dokime.plugin.v1` on Instance.Invoke; host context in `harmos.context.v1` or
`dokime.context.v1` through child calls; host client API in `harmos.v1` or
`dokime.v1`. Nested resources SHALL stay within that face. Only the host SHALL
call instances; clients SHALL NOT call context. Plugins SHALL remain client management.

#### Scenario: Client Cannot Call Context

- **WHEN** a connected client attempts `dokime.context.v1.executions.Progress/Set` or `harmos.context.v1.HostValues/Get`
- **THEN** the host refuses the call with an error naming the action; client access does not authorize an invocation-context operation
- **AND** no context service is registered on the client endpoint or exposed by its generated client root

#### Scenario: Client Cannot Call an Instance Directly

- **WHEN** a client tries to open Instance.Get, Invoke, Samples or Stop directly on a plugin instance
- **THEN** the plugin protocol refuses the caller because only the host may open those calls
- **AND** Public catalog visibility does not authorize a direct instance connection

#### Scenario: Plugin Cannot Call Client-Only Management

- **WHEN** a plugin attempts client-only `harmos.v1.Plugins/Delete` through its context
- **THEN** the host refuses it with an error naming the action and leaves the entry unchanged
- **AND** plugin context authority does not grant a client-only operation, even when a handler is shared elsewhere

#### Scenario: Host Calls an Instance

- **WHEN** the host connects to a running plugin and invokes one of its admitted product operations
- **THEN** it reads identity/catalog through `harmos.plugin.v1.Instance/Get` and opens Instance.Invoke for the `dokime.plugin.v1` catalog operation
- **AND** Instance.Samples and Instance.Stop retain publication and cleanup semantics; Plugins remains a distinct host-served client management service

### Requirement: Explicit Catalog Visibility and Admission

Faces SHALL NOT replace visibility. Every SDK and host built-in SHALL explicitly
choose Private, Protected or Public with no default, following the decided mapping
in design.md. Private SHALL be local-only, Protected SHALL admit only its plugin
and host over their carrier, and Public SHALL admit declared peers and clients
through the host. Admission SHALL retain digest, fingerprint, requirement and
provider checks in one registry.
#### Scenario: Private Is Local and Absent from the Wire

- **WHEN** a plugin invokes its own explicitly Private operation in Rust, Python, C# or C++
- **THEN** the SDK runs its handler locally with the same policy, retry, deadline and cancellation handling as any operation, without a host call
- **AND** catalog/descriptor export omits the operation and no gRPC method can be associated with it
- **AND** a host, another plugin or a client cannot discover or invoke it on any wire

#### Scenario: Protected Host Invocation and Peer Refusal

- **WHEN** the host requests an admitted plugin's Protected operation
- **THEN** it invokes the operation over Instance.Invoke and the declaring plugin may also invoke it
- **AND** another plugin and a client are refused even if they declare the requirement, and a client-facing gRPC binding is rejected

#### Scenario: Explicit Visibility and Invalid Admission

- **WHEN** an author omits visibility in any SDK, or an incoming catalog contains unspecified visibility, Private, or a Protected client-facing gRPC binding
- **THEN** declaration or admission fails explicitly without registering a callable wire operation
- **AND** Private local metadata remains available only to its declaring SDK's dispatcher

#### Scenario: Host Built-Ins Use the Same Levels

- **WHEN** the host declares built-ins or generates plugin lifecycle bindings
- **THEN** each uses the decided design.md mapping with no default; implementation-only Private work stays local and host-requested plugin lifecycle work is Protected
- **AND** Public context built-ins remain callable by plugins but client credentials cannot call that context face

### Requirement: Visibility and Host-Mediated Plugin Invocation

The host SHALL route declared plugin requirements as child calls through one
operation registry to admitted providers via Instance.Invoke, checking visibility
and fingerprints. Clients SHALL reach Public plugin operations through client
Invocations.Invoke. Plugins and clients SHALL NOT connect directly to an instance.
Scope SHALL be inherited unchanged; results, failures and cancellation SHALL
propagate through the host. A requirement SHALL NOT grant access.

#### Scenario: Plugin Invokes Another Plugin's Public Operation

- **WHEN** a plugin makes a context child call for a declared requirement matching another admitted plugin's Public operation
- **THEN** the host checks visibility and fingerprint in its one registry, calls the providing plugin's Instance.Invoke and returns its result to the caller
- **AND** only the host opens the instance call; no client-only service authority is required by the calling plugin

#### Scenario: Protected Cross-Plugin Operation Is Refused

- **WHEN** a plugin makes the same child call to another plugin's Protected operation
- **THEN** the host refuses it with an error naming the action and does not invoke the provider
- **AND** the declaring plugin and host remain permitted callers; a declared requirement alone does not grant another plugin access

#### Scenario: Client Invokes a Public Plugin Operation

- **WHEN** a client invokes an admitted Public plugin-served operation through `harmos.v1.Invocations/Invoke`
- **THEN** the host checks visibility and fingerprint, resolves the provider in the same registry, invokes its instance and returns the result
- **AND** Private and Protected operations remain unavailable to that client

#### Scenario: Scope Is Inherited Across the Provider Hop

- **WHEN** a plugin child call is routed by the host to another plugin
- **THEN** the providing invocation inherits the caller's scope unchanged
- **AND** the host does not infer authority from payload fields or let the child widen its scope

#### Scenario: Cancelling the Caller Cancels the Child

- **WHEN** a caller is cancelled while its child operation is running in a providing plugin
- **THEN** the host cancels the child and propagates cancellation to the provider's Instance.Invoke
- **AND** cancellation/failure and required cleanup propagate through the existing invocation settlement contract

#### Scenario: Child Failure Propagates Back

- **WHEN** a Public provider operation fails after the host routes a child call
- **THEN** the caller receives its structured failure, action and preserved cause through the host
- **AND** the failure is not converted into an empty success

#### Scenario: Missing Requirement or Provider and Fingerprint Mismatch

- **WHEN** a plugin omits the required declaration, declares a mismatched fingerprint or targets an operation without an admitted provider
- **THEN** admission or dispatch refuses the call explicitly and the provider is not invoked
- **AND** a client call with an incompatible fingerprint or absent admitted provider is also refused

### Requirement: Independent Declarations per Face

A capability needed on context and client faces SHALL have separate IDs,
fingerprints, requests and responses per face, even with a shared handler.
Evolution and authorization SHALL be independent. The 15 overlapping capabilities
SHALL match design.md's per-face mapping. Context-only mutations SHALL NOT be
added to client services for symmetry.

#### Scenario: Shared Time and Produced Data

- **WHEN** a plugin and a client read time, transfer bytes or read produced records/statistics
- **THEN** each calls its own face's typed declaration with that face's authorization and scope rules
- **AND** a shared handler does not make the client request/response a context contract

#### Scenario: Context Revision Mirror

- **WHEN** a plugin gets the value/revision needed for context Progress.Set or Traceability.Set
- **THEN** it uses the context Get request/response, separate from the client's observation Get
- **AND** the client gains no progress or traceability Set through this duplication

### Requirement: Generate Only Required Faces

Each language SHALL ship one SDK package containing both plugin-development
interfaces and backend-client connection/functionality, with separate face roots.
Python SHALL use `dokime-sdk` and the library's single `harmos-sdk`; no
audience-specific builds SHALL remain. A codebase MAY install itself via `main.py`.
The API SHALL be simple to consume. Amended by the owner on 2026-11-13.

#### Scenario: One Package for Plugin and Client

- **WHEN** an author installs the language SDK package
- **THEN** it supplies plugin protocol/context interfaces and a backend client root
- **AND** client roots do not expose context or Instance authority; a host may consume all faces

### Requirement: Predictable Operation Names

Services SHALL name their resource with a plural noun, or a singular noun for
one thing. Methods acting on that resource SHALL be bare verbs and SHALL NOT
repeat its noun. Mixed-resource services SHALL keep distinguishing resource nouns,
without repeating the service name, and SHALL NOT be split merely to shorten names.

#### Scenario: Mixed-Resource Service

- **WHEN** Access allocates a pairing or regenerates a certificate
- **THEN** Access.Pairings.Create and Access.Certificates.Create state those lifecycle actions; Access.Pairings.List/Delete inspect and revoke independent pairings
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

A nested resource SHALL be a service namespace in package
`<face-package>.<lowercase-parent>`,
with the part as its service name, one proto file per service and a folder per
parent. Its messages SHALL use the local service prefix `<Service><Method>Request`
and `<Service><Method>Response`. Parent and nested resource MAY have different
library/product owners; composition SHALL preserve both namespaces.

#### Scenario: Product Settings Beside Library Plugins

- **WHEN** the host application serves library Plugins and product plugin settings
- **THEN** the product declares `dokime.v1.plugins.Settings` in `dokime/v1/plugins/settings.proto`, beside library Plugins, and the full nested Get path is `dokime.v1.plugins.Settings/Get`
- **AND** its wrappers are SettingsGetRequest/Response and SettingsSetRequest/Response, not flat PluginSettings wrappers or Plugins.GetSettings

### Requirement: Generated Method Messages and Clients

Messages SHALL use local ServiceMethodRequest/Response names and response
wrappers. In all four SDKs a build-time generator SHALL derive hierarchy within
each selected face from the shared descriptor package/service table, without
hand-written per-operation code. Composition SHALL merge distinct resource
children within one face and reject collisions. Language casing SHALL preserve
hierarchy; context and client roots SHALL remain separate.

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
- **THEN** the catalog carries declarations and `harmos.context.v1.HostValues/Get` calls the registered product resolver with its policy and stores
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
- **THEN** all 132 ordinary rows and their Run twins are accounted for once, and the original mapped client subset has 39 services (23 nested), 120 methods and 240 method wrappers
- **AND** library mapped-client totals are 7 services (3 nested), 27 operations/methods and 27 request/response pairs; product client totals are 32 services (20 nested), 93 operations/methods and 93 pairs
- **AND** product Content.ExecutionFiles.Get resolves metadata for library Storage.WatchFileBytes growth/replacement/truncation; Content.ExecutionFiles.Watch follows product metadata/membership

#### Scenario: Complete Per-Face Operation Census

- **WHEN** product plugin/context declarations and mapped library dependencies are included, counting shared capabilities once per face
- **THEN** the plugin face has 5 logical namespaces, 9 operations, 4 gRPC entry methods and 9 request/response pairs, with Instance its one wire service
- **AND** context has 15 logical namespaces, 29 operations, zero independent gRPC entry methods and 29 request/response pairs, carried as child calls
- **AND** the completed client face has 44 services, 128 operations/gRPC methods and 128 pairs, including seven generic library companions beyond the original 120-method mapping
- **AND** product-only totals are 48 namespaces, 116 declarations, 93 gRPC methods and 116 pairs; mapped library dependencies are 14 namespaces, 49 declarations, 38 gRPC methods and 49 pairs
- **AND** all-face totals are 64 logical namespaces, 45 wire services, 166 operations, 132 gRPC entry methods and 332 method wrappers; these are not the whole library protocol's totals
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

### Requirement: Measured Durable Lag

Latest-durable/no-wait reads SHALL NOT wait for unsaved accepted data. The measured
lag is accepted: small payloads about 99 ms, large and mixed up to several hundred
milliseconds at p95. No accepted-data snapshot SHALL be built. The Nightly benchmark
SHALL report p50, p95 and maximum without a failing threshold. Amended by the owner
on 2026-11-13.

#### Scenario: Nightly Measurement Across Payload Sizes

- **WHEN** the reusable durable-lag benchmark measures small, multi-megabyte and mixed loads
- **THEN** it reports the distributions for each load without a failing lag threshold
- **AND** correctness tests verify durable positions and configured byte splitting deterministically, without elapsed-time assertions

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

Paging SHALL apply only to data rows that can grow unbounded in size. Small
bounded sets SHALL return whole in one response, with no page controls or
continuation. This includes catalog entries, plugins, pairings, suites and schemas.
Executions, records, samples, log lines, files and notes SHALL remain paged and
position-resolved.

#### Scenario: Bounded Declaration List

- **WHEN** a caller lists a suite catalog or its schema declarations
- **THEN** the response contains the whole declaration set with no continuation
- **AND** evidence lists still use bounded pages at a resolved position

### Requirement: Position-Resolved Page Continuation

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

The library storage manager SHALL own one unary transfer mechanism behind separate
context/client declarations: Storage.CreateFileProduction, GetFileProduction,
AppendFileBytes, CreateFileWriter, PublishFileProduction and DeleteFileProduction.

#### Scenario: Context and Client Uploads

- **WHEN** a plugin or client uploads a file
- **THEN** its face's separate declarations use the library storage manager's one unary transfer mechanism
- **AND** the mechanism provides Storage.CreateFileProduction, GetFileProduction, AppendFileBytes, CreateFileWriter, PublishFileProduction and DeleteFileProduction

### Requirement: Bounded Replay-Safe Upload Chunks

Append SHALL accept bounded ordered chunks, replay identical bytes and refuse
conflicts.

#### Scenario: Resume After Lost Reply

- **WHEN** an upload chunk reply is lost and the caller reconnects
- **THEN** Storage.GetFileProduction reports the accepted offset and an identical Storage.AppendFileBytes replays safely
- **AND** CreateFileWriter fences the old writer when resume needs a new writer generation, while different bytes at an accepted offset conflict without changing content

#### Scenario: Ordered and Bounded Chunks

- **WHEN** a new chunk starts after the next accepted offset or exceeds the configured chunk bound
- **THEN** Storage.AppendFileBytes refuses it without changing session data
- **AND** Storage.GetFileProduction exposes the accepted offset, length, digest and session state to resume

### Requirement: Verified Upload Publication and Abandonment

Publish SHALL verify complete bytes, length and digest before visibility; Delete SHALL
abandon. Expiry SHALL be explicit.

#### Scenario: Partial Upload and Expiry

- **WHEN** publication is requested with missing bytes, a wrong digest or an expired session
- **THEN** it fails explicitly without publishing partial content or silently creating a replacement session

### Requirement: Default Unary Transfer Limits

Default chunks SHALL be 4 MiB and the default
message limit SHALL be 8 MiB.

#### Scenario: Default Transfer Configuration

- **WHEN** the default chunk size and message limit are used
- **THEN** chunks are 4 MiB and the message limit is 8 MiB

### Requirement: Shared Unary Ranged Downloads

The library storage manager SHALL serve separate client/context declarations of
Storage.GetFileBytes with independent request/response types and authority. Clients and plugins
SHALL download through repeated unary Storage.GetFileBytes calls with
offset and maximum bytes. Each reply SHALL state total length, content digest
and resolved_at. Configuration SHALL bound range sizes. Streaming download gRPC methods
SHALL be removed in favor of ranged Get reads.

#### Scenario: Download Resume and Bounds

- **WHEN** a client or plugin resumes a content download at its next offset with a maximum byte count
- **THEN** the unary response returns a configuration-bounded range with total length, content digest and resolved_at
- **AND** subsequent ranges preserve the content representation and returned cut

### Requirement: Unary Transfer Replacement

Streaming file transfers SHALL be removed in favor of unary sessions and ranged
reads. Throughput SHALL remain a Nightly benchmark measurement without a deletion
gate or correctness-test threshold. Amended by the owner on 2026-11-13.

#### Scenario: Unary Replacement

- **WHEN** the composed transfer capability replaces a streaming file transfer
- **THEN** callers use unary upload sessions or bounded ranged reads
- **AND** the benchmark reports throughput without gating removal

### Requirement: Shared Failures and Retry Policy

Interfaces SHALL use one Failure vocabulary and one retry-policy vocabulary,
preserving causes and canonical gRPC status/details. Budgets SHALL use Duration,
instants Timestamp, and total attempts SHALL include the first. Retry SHALL require
explicit replay safety; invocation correlation alone SHALL NOT deduplicate.

#### Scenario: Child Failure and Per-Call Override

- **WHEN** a child operation fails under a per-call retry override
- **THEN** code, classification, action, safe details and cause survive propagation
- **AND** a subsequent call without the override uses the declaration policy unchanged

#### Scenario: Unset Operation Policy

- **WHEN** an operation has no configured policy or per-call override
- **THEN** the unset policy means no deadline, zero cancellation grace and no retries
- **AND** callers do not invent budgets or retries to make a scenario pass

#### Scenario: Accepted Is Not Durable

- **WHEN** a mutation is accepted but not saved
- **THEN** its receipt cannot claim a durable position and a transport failure cannot be reported as proof that the mutation was not applied

#### Scenario: Lost Client Call

- **WHEN** a call fails on the client and its outcome is unknown
- **THEN** resending is the client's responsibility, and the same request identifier and unchanged payload return the original receipt
- **AND** a changed payload under that same identifier is a conflict

### Requirement: Opaque Inherited Invocation Scope

Scope SHALL be opaque caller-bound context carried and inherited unchanged by the
library, never interpreted as product identity, authorization or a grant. Only
product policy SHALL interpret it or require its presence. Payload and correlation
identifiers SHALL NOT establish or widen scope or authority.

#### Scenario: Non-Product Scope and Child Hop

- **WHEN** an admitted caller invokes a provider and a child provider with an opaque scope unknown to the library
- **THEN** both receive exactly that scope and the library routes without interpreting it
- **AND** a payload claiming another scope cannot widen authority; missing scope is refused only where the operation requires it

### Requirement: Distinct Storage Guarantees

Saved finite reads and follows of durable progress SHALL remain distinct.
Scans, indexes and projections SHALL retain their own selection, physical-work,
revision and frontier guarantees; an index or projection SHALL NOT silently claim
coverage beyond its materialized frontier. Bulk ingestion SHALL NOT replace an
atomic conditional update. Configured durability/synchronization SHALL remain
separate from acceptance, buffering and flush cadence.

#### Scenario: Saved Read Versus Follow

- **WHEN** accepted unsaved data exists and a caller reads saved data or opens a durable follow
- **THEN** the finite read resolves its durable cut while the follow delivers subsequent durable progress according to its own continuation contract
- **AND** neither claims an unsaved admission tail is saved or silently fills a retention gap

#### Scenario: Scan and Projection Frontiers

- **WHEN** an index or projection trails the selected saved frontier
- **THEN** its read identifies its coverage or explicitly refuses the unavailable selection, and physical scan work remains bounded separately from output size
- **AND** using an index cannot change the logical saved-read selection or hide a missing projection frontier

#### Scenario: Atomic Conditional Update Versus Bulk Ingestion

- **WHEN** competing writers use the same expected revision for a conditional update
- **THEN** at most one update commits and the other conflicts without partial effects
- **AND** a bulk append cannot be used as evidence of that atomic comparison guarantee

#### Scenario: Configured Durability and Controlled Recovery

- **WHEN** a write is accepted before its configured durable synchronization completes
- **THEN** its receipt does not claim durability; the durable acknowledgement is emitted only after that configured boundary
- **AND** controlled failure/recovery verifies the guarantee without timing or memory assertions

### Requirement: Deterministic Behavioral Proof

Correctness tests SHALL be deterministic, story-shaped and event/controlled-clock
based. They SHALL NOT assert wall-clock duration, machine load, sampled memory,
authored prose or a single repository constant. Measurements SHALL live in
benchmarks. Operation policies and configuration SHALL own deadlines, retries,
delays, budgets and limits; none SHALL be weakened or tuned merely to pass proof.

#### Scenario: Retry and One-Call Override

- **WHEN** a replay-safe operation is exercised with a configured one-call override and then without it
- **THEN** controlled-clock/events prove the actual attempts, deadlines and cancellation, and the later call uses the declaration policy unchanged
- **AND** replay-unsafe retry is refused without increasing the policy or limits to pass

#### Scenario: Performance Workload

- **WHEN** a scenario measures throughput, durable lag, elapsed execution or memory consumption
- **THEN** it runs as a benchmark and correctness tests instead verify deterministic state, bounds, splitting or event ordering
- **AND** an independent hang guard remains a deadlock safeguard, not an assertion about performance

### Requirement: Durable External Effect Restoration

The product SHALL durably prepare an owner-qualified inverse before external
writes, retain the earliest inverse per scope/resource and give each prepare a
unique provider-bound confirmation ticket. Cleanup SHALL restore resources once
in reverse first-prepare order while capture stays open. Startup SHALL restore
pending inverses before exposing plugins or recovering interrupted runs.
#### Scenario: Repeated Writes and Later Rollback

- **WHEN** multiple prepares join the same resource and a later ticket rolls back or remains unconfirmed
- **THEN** the earliest restoration state survives and final cleanup restores the original state once
- **AND** each ticket retains its provider, call evidence and restoration-ticket link without duplicating inverses

#### Scenario: Confirmation and Cross-Execution Exclusion

- **WHEN** a provider confirms a ticket or a different execution tries to prepare the same resource
- **THEN** confirmation requires that ticket's provider and the other execution is excluded until restoration
- **AND** a sole first-call rollback retires its inverse only if no later prepare joined; later rollback never removes the earliest inverse

#### Scenario: Recovery Cannot Restore

- **WHEN** startup finds an unapplied inverse whose provider is missing or restoration fails
- **THEN** startup refuses, keeps the inverse pending and exposes no recovered runs/plugins
- **AND** a later successful restoration can complete recovery without losing earlier evidence

### Requirement: Captured Evidence Survives Plugin Evolution

Stream and record shape identities SHALL be derived and compared only for equality.
Saved evidence SHALL retain its schema definitions across plugin evolution or
removal; neither SHALL invalidate a root. Product-owned unreadable persistent
state SHALL be refused with backup/fresh-root or explicit reset guidance and
without automatic reset, migration or deletion; development baselines stay v1.

#### Scenario: Plugin Changes or Is Absent

- **WHEN** a plugin's stream/record schema changes or the plugin is absent after an execution seals and the root reopens
- **THEN** historical evidence and reports use captured definitions and remain readable without requiring the old plugin
- **AND** schema identities are not manually incremented behavior/release versions

#### Scenario: Unreadable Product State

- **WHEN** the product cannot decode its own persistent state or encounters an incompatible database layout
- **THEN** startup refuses that root with backup and fresh-root/explicit-reset guidance while preserving existing data
- **AND** the caller must explicitly choose replacement storage; no silent reset occurs

### Requirement: Explicit Missing Evidence and Failure Causes

Every interface SHALL preserve structured failure classification, action and causes.
Missing evidence, unavailable cuts and Watch gaps SHALL NOT become empty successes;
retention gaps SHALL require explicit resynchronization. Abandoning observation
SHALL NOT cancel already admitted durable work; active invocation cancellation
SHALL preserve cleanup and evidence.

#### Scenario: Provider Failure and Unavailable Cut

- **WHEN** a provider fails or a requested cut/evidence is unavailable
- **THEN** the caller receives a structured failure preserving classification/action/cause or the explicit resynchronization outcome, never an empty success
- **AND** a transport failure leaves mutation outcome uncertain until resolved through its receipt contract

#### Scenario: Observer Leaves While Work Continues

- **WHEN** an observer closes its Watch while durable execution work is admitted
- **THEN** that work continues, whereas explicit active cancellation interrupts its invocation and still closes streams, runs cleanup and seals attributed evidence

### Requirement: Defined Receipt Replay Lifetime

Semantic mutation deduplication SHALL declare its retention lifetime. Expired
request keys SHALL fail explicitly rather than silently repeating irreversible
work; invocation correlation SHALL NOT replace the semantic key.

#### Scenario: Resend After Receipt Expiry

- **WHEN** a caller resends a semantic request key whose retained receipt has expired
- **THEN** the operation reports explicit expiry without applying the effect again
- **AND** unknown transport outcome is not converted into an assumption of nonapplication

### Requirement: Authoritative Host Clock

Time reads, waits and operation-policy scheduling SHALL use the authoritative
host clock. SDK waits SHALL remain context child calls rather than independent
SDK timers, preserving the same behavior under a controlled clock.

#### Scenario: Plugin Wait Under a Controlled Clock

- **WHEN** a plugin requests Time.Sleep during an invocation while the host clock is controlled
- **THEN** the host owns completion of the wait and its cancellation under that clock
- **AND** no SDK wall-clock timer independently completes the wait or advances evidence time
