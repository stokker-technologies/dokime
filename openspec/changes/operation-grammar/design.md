# Operation Grammar Design

Status: locked by the owner on 2026-11-12
Boundary, managers and nested resources locked by the owner on 2026-11-12
Naming rule amended by the owner on 2026-11-12: bare verbs on the service's resource
Separation by audience locked by the owner on 2026-11-12

Target decisions are Owner.
Current-source observations are not target decisions.
See proposal.md for the decision register and motivation.

## Context

The read-only product and library research reports supplied for this change are
inputs, not contracts. Current product protocols are in `proto/dokime/v1/`;
the new host and handlers are in `crates/dokime-runtime-next/` and
`crates/dokime-operations-next/`. The existing loop and desktop still coexist
with them. `grpc-plugins/design.md` owns that eventual atomic cutover. This
proposal adds no implementation, compatibility layer or lifecycle activation.

## Goals and Boundaries

Define one naming and behavior contract shared by native, client and plugin
interfaces. Preserve durability, scoped authority, bounded physical storage
work, finalization, publication and cooperative cancellation where promised.
The library owns generic machinery under its managers; a product owns its storage
definition, objects, policy and operations. Product lifecycle and evidence selection
remain product-owned. No arbitrary time travel, second scheduler, automatic
streaming projection or storage reset is proposed. An accepted-data snapshot is
the named fallback if bounded durable lag cannot be achieved, not work to build now.
The owner decided on 2026-11-12 that language SDKs stay in the library and product
SDKs build on them: other products will also host plugins on this library. Its
protocol, SDKs and conformance suite remain self-sufficient, with no product vocabulary.

## Shared Grammar and Vocabulary

The following shared text uses the supplied decisions and both reports' Shared
Vocabulary sections. Product-specific meanings belong in the next section.

- **Names (Owner):** a service names its resource with a plural noun, or a
  singular noun for one thing. Methods acting on that resource are bare verbs;
  never repeat its noun. A part read in exactly one way uses its noun alone,
  whether streamed, paged or returned whole. Rule 3 is replaced: a part with
  multiple read or change paths is a nested resource with its own bare verbs,
  such as Plugins.Settings.Get/Set and Executions.Progress.Get/Watch. Mixed-resource
  services retain distinguishing nouns for other actions. Flat compound settings
  services and verb-plus-part methods for multiple-path parts are superseded.
  Each nested service lives below its face package in
  `<face-package>.<lowercase-parent>`, with the part as the service name: `dokime.v1.plugins.Settings/Get`. One proto file per
  service lives in a folder per parent, for example `dokime/v1/plugins/settings.proto`.
  Messages in that package are `<Service><Method>Request/Response`, such as
  `SettingsGetRequest`; parent services retain names such as `ExecutionsStartRequest`.
  Replies are always wrappers. Operation IDs derive mechanically from package,
  service and method, with no second authored alias. A nested service may have a
  different owner from its parent: `dokime.v1.plugins.Settings` intentionally
  sits beside `harmos.v1.Plugins` on the host application's endpoint.
- **Ordinary verbs (Owner):** the closed set is Get, List, Watch, Set, Create,
  Delete and Append. Get observes one identified or derived value; List enumerates
  one bounded selection; Watch opens that selection and follows changes; Set
  establishes writable value fields; Create admits a lifecycle resource; Delete
  removes or releases it; Append adds ordered immutable data such as records,
  samples, log lines and upload bytes. Update, Read, Browse, Put, Edit, Add,
  Remove, Follow, Scan, Inspect and Register are not verbs in this grammar.
- **Set (Owner):** Set carries the whole writable value and the expected revision
  returned by its matching Get. Get returns that same writable value and revision;
  stale writes fail with a conflict naming the action: get again and retry.
  Field masks were rejected because they add path validation, nested-path rules, per-language helpers and subtle errors for small writable parts; a revision guard already prevents lost updates.
  If whole-value replacement is unreasonable, split the writable part into
  separately settable sub-resources, each with its own Get, Set and revision.
  Read-only and runtime-owned facts do not acquire a Set merely for symmetry.
- **Domain verbs (Owner):** use a domain verb only where a standard verb would
  hide a side effect, and state its one meaning. Start activates; Stop finishes at
  a safe boundary; Cancel interrupts; Restart replaces an activation; Seal closes
  an evidence lifecycle; Publish commits prepared visibility; Import admits an
  external representation; Export produces an external representation; Invoke
  calls a declared operation; Advance materializes a projection frontier; Capture
  freezes external data; Finish completes a production; Abandon terminates a
  production with different retained-data guarantees. Plugin lifecycle verbs keep
  their distinct meanings. The clock wait keeps the name Sleep.
- **Operation and invocation (Owner):** an operation is a declaration, an
  invocation is one execution of it, and an attempt is one retry attempt.
- **Identifiers and scope (Owner):** opaque resource and `*_id` strings;
  `name` identifies a full resource, `parent` its ownership. Invocation and child
  IDs are correlation identities, never authorization or deduplication tokens.
  Scope is optional opaque caller-bound context, inherited unchanged; neither a
  grant nor a payload field from which authority is inferred. Requiring it is an
  operation's decision.
- **Position (Owner):** `(history, sequence)` within one ordered history.
  History includes store identity and epoch. Never compare positions from different
  histories. A position is not wall time, a process epoch or a page token.
- **Cut (Owner):** an atomic read selection, potentially containing several
  positions/revisions. A cut is not a timestamp, page token, process epoch, snapshot
  lease or seal digest. A scalar sequence alone cannot represent a composite cut.
- **Reads (Owner):** one read path per kind of data; optional position `at`;
  absent resolves to latest; the reply states `resolved_at`; paging and watches
  follow the same rule. Accept `at` only as a position a previous reply returned
  for consistency, or a product-defined terminal boundary. Arbitrary earlier
  positions are not offered. The product decides what latest and its terminal
  boundary mean; the library does not define product lifecycles.
- **Latest and durability (Owner):** latest
  resolves once, at read/watch opening, to the latest durable position without
  waiting for accepted data that is not yet saved. A caller that needs its own write visible
  passes the durable position from that write's receipt as `at`. Durable lag must
  stay within 250 ms at the 95th percentile independent of payload size.
  Benchmark small, multi-megabyte and mixed loads;
  accepted-data snapshots are the fallback if smaller, size-triggered flushes
  cannot meet the bound, not work to build now.
- **Paging (Owner):** `page_size`, `max_bytes`, `page_token`, `next_page_token`,
  `resolved_at`; tokens bind selection, ordering, cut and schema/projection revision.
  Stable ordering has an identifier tie-breaker and exclusive continuation. Bound
  rows and bytes; work bounds remain separate. Expired or mismatched tokens fail
  explicitly, never restart silently at latest.
- **Watching (Owner):** opening returns a boundary/snapshot at the resolved
  cut, then changes strictly after it. Explicit `at` fixes the starting snapshot,
  not an eternal frozen subscription; gap detection and explicit resynchronization
  are mandatory. Filters can advance progress without emitting matching rows.
- **Time (Owner):** protobuf Timestamp for instants and Duration for elapsed
  budgets; host evidence time and optional source time remain distinct. Acceptance
  and saved time are separate. Source-time windows are half-open filters, separate
  from ingestion positions.
- **Failures (Owner):** one Failure vocabulary: classification, stable code,
  message, action, operation, plugin, cause and safe details; preserve causes and
  map consistently to canonical gRPC status/details. Diagnostics remain separate.
  Missing evidence, gaps and unavailable positions cannot become empty success.
- **Receipts (Owner):** admission, application and durability are separate
  facts. Receipts identify the semantic mutation and accepted/durable positions;
  accepted is not durable. Same key and same normalized request replay the original
  receipt; changed payload conflicts. When a call fails on the client, resending
  is the client's responsibility: the same request identifier returns the original
  receipt, and a changed payload under that identifier is a conflict. Declare
  deduplication lifetime; expired keys fail rather than silently repeat irreversible work.
- **Retry policy (Owner):** one policy vocabulary for invocation budget,
  per-attempt budget, cancellation grace, total attempts, backoff and replay safety.
  Total attempts includes the first attempt. Backoff has one fixed/exponential
  representation with an explicit jitter choice. Retry only explicitly replay-safe
  operations; correlation IDs do not deduplicate. A per-call override changes only
  that call. Declarations own field defaults; per-call overrides do not change them.
- **Transfers (Owner):** uploads and downloads are unary, ranged and resumable,
  using one mechanism for clients and plugins. Create an upload session, Append
  ordered chunks at stated offsets, Get status to resume, Publish to make content
  visible, and Delete to abandon. Identical chunk replay succeeds; different bytes
  at an accepted offset conflict. Download by repeated Storage.GetFileBytes calls with
  offset and maximum bytes; each response states total length and content digest.
  Configuration bounds chunk and range sizes. Measure large-file throughput in a
  Nightly benchmark case before deleting streaming transfers.
- **Streaming (Owner):** never generate twins automatically. Server streaming is
  reserved for Watch semantics (including noun-only streamed parts); the plugin protocol retains its own carriers. A managed
  stream exists only for an actual cooperative final-result promise after
  cancellation, replacing the ordinary gRPC method. Native gRPC cancellation abandons
  observation and cannot promise that final response; admitted durable work
  survives observer loss. Downloads, documents and diagnostics use bounded unary
  replies, with content bytes read through ranged Storage.GetFileBytes.

### Audience Faces and Visibility

**Owner, locked 2026-11-12:** every service belongs to exactly one face. Its
package identifies who serves it and who may call it on the wire. This adds a
boundary to the existing grammar; bare verbs, nested resources and local
`<Service><Method>Request/Response` wrappers remain unchanged within each face.
Here `library` means the actual package prefix `harmos`, and `product` means
`dokime`; examples using these placeholders preserve the same package layout.

| Face | Package Root | Server and Permitted Caller | Transport |
| --- | --- | --- | --- |
| Plugin Protocol | `harmos.plugin.v1` | Running plugin; only the host calls its instance | One service, Instance.Get/Invoke/Samples/Stop |
| Product Plugin Operations | `dokime.plugin.v1` | Running plugin; only the host opens the carrier | Catalog operation declarations carried by Instance.Invoke, never separate client services |
| Context | `harmos.context.v1`, `dokime.context.v1` | Host; only a plugin handling an invocation calls through its context object | Typed child calls on the active invocation; no independent context gRPC entry method |
| Client API | `harmos.v1`, `dokime.v1` | Host; applications, CLI tools and scripts | Client gRPC services, including generic invocation of Public plugin operations |

Nested resources remain sub-packages of their parent *within that face*, such as
`dokime.context.v1.executions.Progress` and `dokime.v1.executions.Progress`.
`Instance` names one running instance; client entry management keeps `Plugins`.
There is no second singular management service. The protocol has exactly one
wire service; TestCases, Executions, Steps and Criteria in the product plugin
package group catalog operations and messages, not additional wire services.

A capability needed on two faces is declared once **per face**, with its own
operation ID, fingerprint, request and response, even when both declarations
call the same handler. Each face evolves and is authorized separately. Do not
reuse a client request/response as a context contract or generate a context
client by exposing all client services. The duplicate declarations required by
this product are listed below the per-face inventories.

Faces do not replace visibility. A face controls the wire caller; visibility is
explicit on **each operation a plugin serves**, in its catalog carried by
Instance.Get and invoked through Instance.Invoke. **Private means the declaring
plugin and the host. Public means everyone connected: other plugins and the
host's clients.** All four SDKs require an explicit visibility argument with no
default; unspecified visibility is rejected at admission. The host keeps one
operation registry, checking catalog descriptors/digest, visibility, declared
requirement fingerprints and provider admission; no registry per face is added.

A plugin may invoke another plugin's Public operation that it declared as a
requirement. The path is calling plugin -> host -> providing plugin: its context
sends a child request identifying the required operation and fingerprint; the
host resolves that entry in its one registry, checks visibility, the declared
fingerprint and that a provider is admitted, and opens Instance.Invoke on that
provider. The caller's scope is inherited unchanged across the hop. Cancellation
cancels the child and reaches the provider; its result or structured failure
propagates back through the host. Private cross-plugin calls fail with an error
naming the action; declaring a requirement does not grant access. A client uses
`harmos.v1.Invocations.Invoke` for the same host-mediated Public-operation path.
Other plugins and clients never connect directly to an instance, even for a
Public operation. Client credentials do not authorize context calls, and plugin
identity does not authorize client-only management operations.

The plugin SDK generates the plugin protocol to serve and context contracts to
call (including product plugin operation declarations). A client generates only
the client API. A host generates all three. A distribution containing both SDK
roles keeps their generation inputs and roots separate. Generic child-call
requirements can reference Public plugin operation descriptors without generating
a client-only management surface. Plugin SDK visibility remains required in
Rust, Python, C# and C++.

Illustrative protocol shape, in `harmos/plugin/v1/instance.proto` (supporting
frames and payloads omitted; wrapper names apply to carriers as well):

```proto
syntax = "proto3";
package harmos.plugin.v1;
service Instance {
  rpc Get(InstanceGetRequest) returns (InstanceGetResponse);
  rpc Invoke(stream InstanceInvokeRequest) returns (stream InstanceInvokeResponse);
  rpc Samples(stream InstanceSamplesRequest) returns (stream InstanceSamplesResponse);
  rpc Stop(InstanceStopRequest) returns (InstanceStopResponse);
}
message OperationContract {
  string id = 1; string request = 2; string response = 3;
  Visibility visibility = 4; // Required explicit choice; unspecified is invalid.
  OperationPolicy policy = 5; bytes fingerprint = 6;
}
enum Visibility {
  VISIBILITY_UNSPECIFIED = 0;
  VISIBILITY_PRIVATE = 1; // Declaring plugin and host.
  VISIBILITY_PUBLIC = 2;  // Everyone connected, through the host.
}
// InstanceGetResponse.catalog contains operations, declared requirements,
// streams, settings declarations, descriptors and digest for admission.
```

For example `dokime/plugin/v1/steps.proto` declares StepsExecuteRequest and
StepsExecuteResponse for the catalog operation
`dokime.plugin.v1.Steps/Execute`, carried by Instance.Invoke; it declares no
`service Steps` RPC. Context wrappers similarly describe child payloads rather
than new network entry methods. Counts below distinguish these logical service
namespaces from wire gRPC services and exclude supporting payload/control frames.

### Generated Client Names

All four SDK client roots expose the hierarchy mechanically from descriptor-set
packages and service names, filtered by face before building roots. A small
generator in each SDK build consumes the same package/service table: remove the complete face prefix (owner plus optional
plugin/context plus version), use the remaining package segments as parent resources and append the service name as the leaf. Top-level
services supply their own root leaf. Apply only each language's standard casing;
no hand-written per-operation code or aliases. Compose the library and product
roots by resource path within one face, merging distinct children and rejecting
collisions. Never merge context and client roots because their resource paths match.

```python
client.plugins.get(request)                 # Library service
client.plugins.settings.get(request)        # Product nested service
client.executions.start(request)
client.executions.progress.get(request)
client.executions.progress.watch(request)
client.plots.settings.set(request)
```

Progress is runtime-owned: Executions.Progress.Get returns its current value,
Executions.Progress.Watch streams it, and Executions.Get includes that same value
at its resolved cut. A plugin produces progress through a host operation it calls,
`dokime.context.v1.executions.Progress/Set`, with its context Get/revision
contract. Set is not a client-facing RPC or part of the client counts; its Get
is a separate declaration from the client progress Get. Parameters and
steps each retain their sole paged noun-only read.

### Nested Wire Examples

`dokime/v1/plugins/settings.proto` intentionally extends a library-owned parent
resource from the product namespace. Settings payload declarations are illustrative.

```proto
syntax = "proto3";
package dokime.v1.plugins;
message SettingsGetRequest { string name = 1; }
message SettingsGetResponse { SettingsValue value = 1; bytes revision = 2; }
message SettingsSetRequest {
  string name = 1; SettingsValue value = 2; bytes expected_revision = 3;
  string request_id = 4;
}
message SettingsSetResponse { SettingsValue value = 1; bytes revision = 2; Receipt receipt = 3; }
service Settings {
  rpc Get(SettingsGetRequest) returns (SettingsGetResponse);
  rpc Set(SettingsSetRequest) returns (SettingsSetResponse);
}
```

`dokime/v1/executions/progress.proto` serves only client observation:

```proto
syntax = "proto3";
package dokime.v1.executions;
message ProgressGetRequest { string name = 1; Position at = 2; }
message ProgressGetResponse { ExecutionProgress value = 1; bytes revision = 2; Position resolved_at = 3; }
message ProgressWatchRequest { string name = 1; Position at = 2; }
message ProgressWatchResponse { ProgressEvent event = 1; Position resolved_at = 2; }
service Progress {
  rpc Get(ProgressGetRequest) returns (ProgressGetResponse);
  rpc Watch(ProgressWatchRequest) returns (stream ProgressWatchResponse);
}
// The plugin producer calls dokime.context.v1.executions.Progress/Set as a child
// call. Its distinct context Get/Set wrappers are declared in the context package.
```

Library listener configuration follows the same grammar:
`harmos/v1/access/listener.proto`, package `harmos.v1.access`, service Listener,
Get/Set with `ListenerGetRequest/Response` and `ListenerSetRequest/Response`.
Access.CreatePairing and Access.CreateCertificate allocate lifecycle resources;
their single creation paths keep distinguishing nouns on the mixed-resource parent.

### Shared Shape Sketch

Snippets illustrate the locked public shapes, not generated code or assigned wire
numbers. Supporting payload definitions are omitted. Message presence distinguishes
absent `at`. Every Set has a matching Get of the whole writable value and revision.

```proto
syntax = "proto3";
import "google/protobuf/duration.proto";
import "google/protobuf/timestamp.proto";

message History { string store = 1; string epoch = 2; }
message Position { History history = 1; uint64 sequence = 2; }
message PageRequest {
  uint32 page_size = 1;
  uint64 max_bytes = 2;
  string page_token = 3;
}
message PageInfo {
  string next_page_token = 1;
  Position resolved_at = 2;
}
message Receipt {
  string request_id = 1;
  Position accepted_at = 2;
  Position durable_at = 3;
  bool completed = 4;
}
// durable_at is absent until saved; accepted_at never implies durability.
message Failure {
  string classification = 1;
  string code = 2;
  string message = 3;
  string action = 4;
  string operation = 5;
  string plugin = 6;
  Failure cause = 7;
  map<string, string> details = 8;
}
```

## Ownership and Managers

**Boundary (Owner, locked 2026-11-12):** if the library has a manager for generic
machinery, that machinery belongs in the library. A product defines its storage
layer (schemas, indexes, projections and the meaning of latest), its own objects
and operations, and policy about who may do what. The host application composes
and serves the library services directly on its endpoint; it does not redeclare,
wrap or forward their public operations in product services.

A product MUST NOT rebuild generic machinery on the protocol crate alone. That
approach previously produced a second copy of plugin connection, dispatch,
streams and process launch. Replace those copies with the library runtime owners.
Library text uses generic vocabulary: a product or the host application.

A manager exists where there is something live to own: changing state, background
work or a lifecycle. Plain operations over storage need no manager. This is the
single manager-creation rule for both owners.

| Library Manager | Services or Owned Boundary | Responsibility |
| --- | --- | --- |
| PluginManager | Plugins | Managed/remote entries; verified bundles, persistence, state/failures, connection and process lifecycle; already being implemented in the library. |
| GrpcManager | Access, Access.Listener; transport for composed services | Listener Get/Set, pairing creation, certificate regeneration and key rotation. |
| StorageManager | Storage | One resumable unary upload/ranged-read mechanism and generic storage; the host application supplies its storage definition. |
| ServiceManager | Status, Status.Logs, Status.TraceEvents | Host identity, health, availability, logs, trace events and diagnostics archive; narrow registration accepts host-application version and component-health contributions. |
| OperationManager | CatalogEntries, Invocations, context HostValues; Instance invocation carrier | Declarations, dispatch, invocations, retries/cancellation and plugin host-operation routing. |
| StreamManager | Context Samples (outside this mapped subset), Instance.Samples | Stream lifecycle, observation and delivery. |
| RecordManager | Generic record operations persisted through Storage; no new public service | Record capture lifecycle. |
| StateManager | Generic state operations; no new public service | Live state and observation. |

These manager types already exist in the runtime, including the generic operations
manager's internal `Manager` and `WeakOperationManager`; their target public-service
coverage is not a claim of landed implementation. Time remains library-owned
(separate client/context `Time.Get`, `Time.Sleep` declarations), under
ServiceManager with the authoritative runtime clock; no new clock manager is required. The plugin protocol, SDKs, operations/invocations, storage
and streams remain library-owned. Their pre-existing full inventories outside
the explicitly mapped dependencies are not counted as new services of this
product mapping.

Settings and secrets are product-owned: stores, schemas, UI declarations,
archives and use of an operating-system credential store are too opinionated
for the library. This closes the per-plugin settings/secrets question. The
library supplies only the generic host-operation mechanism by which a host
application serves values to a plugin, plus the settings-declaration section of
the plugin catalog. The generic context operation is
`harmos.context.v1.HostValues/Get`; a product registers its value resolver rather
than introducing library settings stores. Product Plugins.Settings is
intentionally separate from library Plugins. Product-only status reads remain in Backend; product facts
contributed to Status use registration, not duplicate identity/health services.

| Product Owner | Responsibility and Service Routing |
| --- | --- |
| Execution manager | Queueing, lifecycle, retries, Stop/Cancel, recovery; Executions, Cases and Cycles operations call it where lifecycle is involved. Decide whether cycles belong here or in a small cycle manager at the split, using the simplest complete option. |
| Prompt manager | Pending operator questions; Prompts operations. |
| Report manager | Atomic publication; Reports.Publish. Rendering is a pure function with no manager. |
| Capture manager, only if recordings remain | Media.Captures recording lifecycle; preserve currently supported capture behavior and explicit unsupported-kind refusals. |
| Storage operations, no manager | Evidence, Catalog, notes, Aggregates, Plots and Documents; product settings/secrets and metadata operations also use storage unless an existing live owner is involved. |

Split `crates/dokime-runtime-next/src/host.rs::Host/Inner` and
`services/state.rs::State` along those owners. Composition assembles handles;
it is not another orchestration owner. Every service method stays thin: one
operation file performs the work by calling its manager or storage. Delete the
product's plugin connection, dispatch, streams, process launch, entry management,
listener/pairing, upload handling and host-status copies as composition replaces
them; preserve product policy and storage definitions.

## Product Decisions and Public Shapes

### Read Selection and Durability

**Owner:** latest for an execution is the live frontier while it runs and its
seal after sealing. Only a previously returned position or the seal is accepted
as `at`; no caller-invented earlier sequence is a historical-read API. Remove the
checkpoint method family and carry seal, frozen context and evidence availability
through the same resource reads. A cut must still bind execution state, seal,
evidence frontier, mutable presentation, membership, schemas and document revisions
atomically. Retaining a returned live cut for paging is not arbitrary time travel.

**Owner:** the live frontier is the latest durable frontier. Reads do not wait
for accepted unsaved writes. A durable write receipt can supply `at` for
read-your-write; an accepted-only receipt cannot. Resource Get/Watch must expose
the later durable receipt without inventing an implicit read barrier. Watches use
the same durable visibility rule for subsequent changes, not an accepted-data tail.

The newest durable position must trail the newest accepted data by no more than
250 ms at the 95th percentile, independent of payload size. A live view may
therefore lag accepted data by a small interval, including for large payloads.

Read-only library source inspection confirms `crates/harmos-storage/src/config.rs::FLUSH_INTERVAL` is
100 ms; `worker.rs` requests periodic flushes and `sqlite/mod.rs` sets
`PRAGMA synchronous=FULL` before transactional persistence. Flush time is added
to the cadence and grows with the bytes written: batches target 1 MB and are capped
at 16 MB, each committed synchronously. Nothing measures durable lag today, so
100 ms is not proof of the required bound. The new product `dokime-storage` is
independent of `harmos-storage`; the library cadence is not proof of the new host's
latency. The product store supplies the product durable frontier; the library
supplies its own.

Add a durable-lag benchmark case to the Nightly benchmark set using the stable
case naming convention of the storage benches. Report p50, p95 and maximum lag
under small samples at a high rate, multi-megabyte batches and a mix. Make the
case reusable by a consumer against its own backend, including the independent
product store. Compare measured p95 for each load with the 250 ms p95 bound.
These are benchmark measurements, never wall-clock assertions in tests. Test the
mechanism deterministically: flush work is bounded in bytes and large input is
split so no single commit exceeds the configured size.

If the bound is missed, first try smaller flush pieces and flushing as soon as a
size threshold is reached, rather than only on the cadence, then remeasure.
If that is insufficient, the named fallback is reads resolving against a snapshot
of accepted data; do not build that fallback now. A per-read durability barrier
was considered and not chosen because it puts flush/commit latency on reads.
An accepted-data snapshot adds snapshot/retention ownership and unsaved-data
semantics, but remains the fallback rather than a rejected alternative. This
owner decision replaces the reports' accepted-frontier and
per-read-barrier suggestions, not a claim that current code already meets it.

### Representative Resource 1: Execution

**Owner:** Executions.Set of a label renames only that execution. Executions.Cancel
addresses only a running execution; queued cancellation fails with an action-named
error. Stop waits for a safe scheduling boundary; Cancel interrupts the active
invocation, retaining required cleanup. Neither is a generic writable status.
**Owner:** merge pause/resume, label, archive and cycle policy into whole-value
Set of each resource's small writable part, retaining revision conflicts and
lifecycle admission rules. Get returns that part separately from runtime facts.

```proto
message ExecutionsGetRequest { string name = 1; Position at = 2; }
message ExecutionWritable { string label = 1; bool paused = 2; bool archived = 3; }
message ExecutionsGetResponse {
  Execution execution = 1; Position resolved_at = 2;
  ExecutionWritable value = 3; bytes revision = 4;
  ExecutionProgress progress = 5; // Same current value as Progress.Get at resolved_at.
}
message ExecutionsSetRequest {
  string name = 1;
  ExecutionWritable value = 2;
  bytes expected_revision = 3;
  string request_id = 4;
}
message ExecutionsSetResponse { ExecutionWritable value = 1; bytes revision = 2; Receipt receipt = 3; }
message ExecutionsCancelRequest { string name = 1; string request_id = 2; }
message ExecutionsCancelResponse { Execution execution = 1; Receipt receipt = 2; }
service Executions {
  rpc Get(ExecutionsGetRequest) returns (ExecutionsGetResponse);
  rpc Set(ExecutionsSetRequest) returns (ExecutionsSetResponse);
  rpc Cancel(ExecutionsCancelRequest) returns (ExecutionsCancelResponse);
}
```

### Representative Resource 2: Records

**Owner:** plugin publication becomes Evidence.Records.Append, not CreateRecord. Its
acknowledgement retains the existing promised stored-record semantics; the receipt
must state the durable position when that promise completes. Records and samples
retain produced/captured ownership, source time and schema identity.

The nested service lives in `dokime/v1/evidence/records.proto`:

```proto
syntax = "proto3";
package dokime.v1.evidence;
message RecordsListRequest {
  string parent = 1; Position at = 2; RecordFilter filter = 3; PageRequest page = 4;
}
message RecordsListResponse { repeated Record records = 1; PageInfo page = 2; }
message RecordsWatchRequest {
  string parent = 1; Position at = 2; RecordFilter filter = 3; PageRequest page = 4;
}
message RecordsWatchResponse {
  Position resolved_at = 1;
  oneof event {
    RecordSnapshot snapshot = 2;
    Record appended = 3;
    ResyncRequired resync_required = 4;
    Position progress = 5;
  }
}
service Records {
  rpc List(RecordsListRequest) returns (RecordsListResponse);
  rpc Watch(RecordsWatchRequest) returns (stream RecordsWatchResponse);
}
```

The context declaration lives in `dokime/context/v1/evidence/records.proto`.
Its List request/response are distinct from the client List wrappers even when
the handler is shared. Append is context-only; List exists on both faces so a
plugin can read previously produced records within its inherited scope.

```proto
syntax = "proto3";
package dokime.context.v1.evidence;
message RecordsAppendRequest {
  string parent = 1; repeated Record records = 2; string request_id = 3;
}
message RecordsAppendResponse { Receipt receipt = 1; }
message RecordsListRequest {
  string parent = 1; Position at = 2; RecordFilter filter = 3; PageRequest page = 4;
}
message RecordsListResponse { repeated Record records = 1; PageInfo page = 2; }
// Evidence.Records.Append/List are child-call declarations, not client RPCs.
```

Progress Get/Set follows the same separation in
`dokime/context/v1/executions/progress.proto`, with context-local
ProgressGetRequest/Response and ProgressSetRequest/Response. The context Get
returns the exact writable value and revision consumed by context Set.

### Representative Resource 3: Transfers

**Owner:** Storage.CreateFileProduction, Storage.GetFileProduction, Storage.AppendFileBytes, Storage.DeleteFileProduction and
Storage.PublishFileProduction are unary for every caller. Storage.AppendFileBytes appends ordered bytes
at the stated next offset. Replaying an identical accepted chunk succeeds;
different bytes at an accepted offset conflict; gaps and out-of-order new chunks
are refused. Storage.GetFileProduction reports the accepted offset, length, digest and session
state to resume. Storage.PublishFileProduction verifies complete bytes, length and digest before
exposing immutable content. Content.Artifacts.Create or Plugins.Create then references it.
Sessions are caller-bound, bounded and expire explicitly; expiry cannot silently
start another upload. Storage.DeleteFileProduction abandons it. Transport loss never publishes
partial content. Chunk and range maxima are configuration-owned limits.

Storage.GetFileBytes returns a bounded range, repeated by the caller to resume downloads.
The content identity and digest bind every range to the same representation,
including a scoped evidence archive at a returned cut. There is no separate
streaming archive download gRPC method. Measure large-file upload/download throughput
in the Nightly benchmark set before deleting the superseded streaming transfers;
throughput is a benchmark measurement, never a correctness-test assertion.

The library's existing `Storage` service owns this mechanism; add no transfer
service alias. The client declarations below live in `harmos/v1/storage.proto`. Separate
context declarations with their own wrappers live in
`harmos/context/v1/storage.proto`; plugins call those through child frames:

```proto
syntax = "proto3";
package harmos.v1;
message StorageCreateFileProductionRequest {
  string parent = 1; uint64 size_bytes = 2; bytes sha256 = 3; string request_id = 4;
}
message StorageCreateFileProductionResponse { FileProduction production = 1; Receipt receipt = 2; }
message StorageGetFileProductionRequest { string name = 1; Position at = 2; }
message StorageGetFileProductionResponse { FileProduction production = 1; Position resolved_at = 2; }
message StorageAppendFileBytesRequest {
  string name = 1; uint64 offset = 2; bytes data = 3; string request_id = 4;
}
message StorageAppendFileBytesResponse { FileProduction production = 1; Receipt receipt = 2; }
message StorageCreateFileWriterRequest { string name = 1; string request_id = 2; }
message StorageCreateFileWriterResponse { FileWriter writer = 1; Receipt receipt = 2; }
message StoragePublishFileProductionRequest { string name = 1; string request_id = 2; }
message StoragePublishFileProductionResponse { PublishedContent content = 1; Receipt receipt = 2; }
message StorageDeleteFileProductionRequest { string name = 1; string request_id = 2; }
message StorageDeleteFileProductionResponse { Receipt receipt = 1; }
message StorageGetFileBytesRequest {
  string name = 1; Position at = 2; uint64 offset = 3; uint64 max_bytes = 4;
}
message StorageGetFileBytesResponse {
  bytes data = 1; uint64 offset = 2; uint64 total_length = 3;
  bytes sha256 = 4; Position resolved_at = 5;
}
message StorageWatchFileBytesRequest { string name = 1; Position at = 2; }
message StorageWatchFileBytesResponse { FileBytesEvent event = 1; Position resolved_at = 2; }
service Storage {
  rpc CreateFileProduction(StorageCreateFileProductionRequest) returns (StorageCreateFileProductionResponse);
  rpc GetFileProduction(StorageGetFileProductionRequest) returns (StorageGetFileProductionResponse);
  rpc AppendFileBytes(StorageAppendFileBytesRequest) returns (StorageAppendFileBytesResponse);
  rpc CreateFileWriter(StorageCreateFileWriterRequest) returns (StorageCreateFileWriterResponse);
  rpc PublishFileProduction(StoragePublishFileProductionRequest) returns (StoragePublishFileProductionResponse);
  rpc DeleteFileProduction(StorageDeleteFileProductionRequest) returns (StorageDeleteFileProductionResponse);
  rpc GetFileBytes(StorageGetFileBytesRequest) returns (StorageGetFileBytesResponse);
  rpc WatchFileBytes(StorageWatchFileBytesRequest) returns (stream StorageWatchFileBytesResponse);
}
```

GetFileProduction reports the accepted offset, length, digest and writer generation;
CreateFileWriter resumes with a new fenced writer where needed. WatchFileBytes
preserves growth/replacement/truncation observation, distinct from the product's
execution-file metadata collection watch. All six production/session operations
use the existing library mechanism with distinct declarations on the two faces,
not new product transfer handlers.

Product artifact/file metadata lives in `dokime.v1.content.Artifacts` and
`dokime.v1.content.ExecutionFiles`, while bytes use library Storage directly.
Storage stays a mixed-resource service under the library's existing contract;
its file methods name distinct production, writer, byte and hash resources. Do
not add parallel Content/Uploads services or a second upload engine. Product
`Storage.Sources` intentionally lives beside library Storage in another package.

### Representative Resource 4: Plugin

**Owner:** one entry describes managed or remote placement. Availability is a
state, not another entry family. Plugins.Restart replaces an activation; it does not
mean Set status. The existing origin-dependent interruption, finalization and
cleanup contract remains with `grpc-plugins`, not with the name change.

```proto
message PluginsCreateRequest {
  string request_id = 1;
  oneof placement { RemotePlugin remote = 2; PublishedContent bundle = 3; }
}
message PluginsCreateResponse { Plugin plugin = 1; Receipt receipt = 2; }
message PluginsRestartRequest { string name = 1; string request_id = 2; }
message PluginsRestartResponse { Plugin plugin = 1; Receipt receipt = 2; }
```

### Product-Specific Actions and Preserved Distinctions

Product plugin catalog operations in `dokime.plugin.v1` each have one locked meaning: TestCases.Materialize resolves
parameters into a case without running it; Executions.Initialize acquires suite
resources; Steps.Execute runs one frozen step; Criteria.Evaluate judges captured
evidence; Executions.Cleanup releases suite resources. Plugin-served operation
bindings remain template-local where necessary; one method name is not one shared handler instance.
Context Files.CopyFiles becomes Files.Capture, freezing selected external files into
evidence without deleting the originals. The effect journal's PrepareEffect and
ConfirmEffect become Effects.Create and Effects.Set: allocate a durable inverse ticket,
then replace its small writable application state under the revision from Effects.Get.
Effects.Get returns that same writable state and revision. Neither performs the
physical write; earliest-inverse retention, ownership and restoration order remain.

Cases.Create preserves validate-only preview, one-selection materialization and
atomic replay-protected plan registration. Cycles.Create with `source_cycle`
preserves rerun notices and new identities. Executions.Create allocates mutable
lifecycle resources; it is not Append of immutable evidence. Prompts.Answers.Create
and Executions.Commands.Create allocate identified lifecycle resources with retained
receipts. Executions.Progress.Set replaces a current value; Evidence.Logs.Append and
Documents.AppendParts add ordered immutable evidence.

Documents.Get retains evaluated output, budgets, repeat instances and failures;
it does not publish a report. Reports.Create renders a preview at a fixed cut,
Reports.Get reads an existing published report, Reports.Publish commits a selected
report and seal. Storage.GetFileBytes delivers bytes; downloads never publish. Secret
Import/Export preserve current transfer protection and conflict checks; the
report's description of an encrypted transfer is corrected below.

## Complete Mapping from Present Product Client Operations

The table below inventories every ordinary method in current `client_*.proto`.
Each also has `Run<Method>`: all 132 twins are **removed** as separate gRPC methods and
map to the same destination as their ordinary method. This explicit rule covers
all 264 present gRPC methods, including twins of watches and uploads. No current product
twin demonstrates a cooperative final-result promise; see `client/conversation.rs`.

Disposition: **kept** retains method meaning/name (service moves are stated);
**renamed** changes a name or shape without another inventoried method collapsing
into it; **merged** joins overlapping gRPC methods, whole writable values, selectors or resource workflows;
**removed** applies to obsolete gRPC methods, not lost capability. Every row retains
the report's caller evidence by category; dynamic and generated use also exists.
`G` = desktop/native bridge, `C` = CLI/client, `S` = SDK convenience/new-host story,
`T` = authored integration story, `N` = no named production caller found, not dead.
Every destination in this table is on the client face. The service inventory
below assigns its exact package, including mixed-owner workflows.
Detailed caller owners: `crates/dokime-gui/src/lib/api/native-*.ts`,
`src/lib/rpc/appearance.ts`, `src-tauri/src/`, `crates/dokime-client/src/`,
`crates/dokime-cli/src/commands/`, `sdk/rust/src/client/host.rs`,
`sdk/python/src/dokime/client.py`, server query/authorization stories and new-host
`tests/system/`. Wire declarations and handler code win over report assumptions.

| Present Operation | Locked Destination on the Client Face | Destination Owner | Disposition and Reason | Callers |
| --- | --- | --- | --- | --- |
| `Appearance.GetAppearance` | `Backend.Appearance` | Product | Merged: Move appearance ownership. | G |
| `Backend.CollectDiagnostics` | `Status.Diagnostics` | Library | Renamed: Read bounded diagnostic output. | G |
| `Backend.GetAvailability` | `Status.Availability` | Library | Renamed: Retain availability. | CS |
| `Backend.GetHealth` | `Status.Health` | Library | Merged: One health owner. | CS |
| `Backend.GetIdentity` | `Status.Identity` | Library | Merged: One identity owner. | GCS |
| `Backend.WatchLogs` | `Status.Logs.Watch` | Library | Kept: Durable log watch. | T |
| `Backend.WatchTrace` | `Status.TraceEvents.Watch` | Library | Renamed: Name trace event resource. | G |
| `Catalog.GetRecordSchema` | `Catalog.RecordSchemas.Get` | Product | Merged: Catalog or frozen execution parent. | N |
| `Catalog.GetSuite` | `Catalog.Suites.Get` | Product | Kept: Retain suite definition. | GCS |
| `Catalog.GetTelemetrySchema` | `Catalog.StreamSchemas.Get` | Product | Renamed: Use stream vocabulary. | N |
| `Catalog.ListRecordSchemas` | `Catalog.RecordSchemas.List` | Product | Merged: Catalog or frozen execution parent. | N |
| `Catalog.ListSuites` | `Catalog.Suites.List` | Product | Kept: Retain suite catalog. | GCS |
| `Catalog.ListTelemetrySchemas` | `Catalog.StreamSchemas.List` | Product | Merged: Use stream vocabulary. | N |
| `Content.DownloadAppearanceAsset` | `Storage.GetFileBytes` | Library | Merged: Bounded bytes by content identity. | G |
| `Content.DownloadArtifact` | `Content.Artifacts.Get` + `Storage.GetFileBytes` | Product + Library | Merged: Metadata then bounded bytes. | GC |
| `Content.DownloadEvidence` | `Evidence.Archive` + `Storage.GetFileBytes` | Product + Library | Merged: Product selects scoped evidence/archive at the cut and returns its content reference; library serves bounded bytes with digest verification. | C |
| `Content.DownloadExecutionFile` | `Content.ExecutionFiles.Get` + `Storage.GetFileBytes` | Product + Library | Merged: Metadata/range then bytes. | C |
| `Content.FollowExecutionFile` | `Content.ExecutionFiles.Get` + `Storage.WatchFileBytes` | Product + Library | Renamed: Product resolves file metadata; library watches byte growth/replacement/truncation. Product collection metadata Watch remains separate. | G |
| `Content.ListArtifacts` | `Content.Artifacts.List` | Product | Merged: Retain bounded artifact selection. | N |
| `Content.ListExecutionFiles` | `Content.ExecutionFiles.List` | Product | Merged: Retain bounded file selection. | G |
| `Content.ReadExecutionFile` | `Content.ExecutionFiles.Get` | Product | Merged: Same file resource, text/byte ranges. | G |
| `Content.UploadArtifact` | `Storage.CreateFileProduction` + `Storage.GetFileProduction` + `Storage.CreateFileWriter` + `Storage.AppendFileBytes` + `Storage.DeleteFileProduction` + `Storage.PublishFileProduction` + `Content.Artifacts.Create` | Library + Product | Merged: Shared unary resume lifecycle. | GC |
| `Documents.CaptureCurrentDocument` | `Documents.Get` | Product | Merged: Evaluated output without publication. | G |
| `Documents.UpdatePlotSettings` | `Plots.Settings.Set` | Product | Renamed: Whole small writable plot value and expected revision from Plots.Settings.Get. | G |
| `Evidence.GetCapturedRecordSchema` | `Catalog.RecordSchemas.Get` | Product | Merged: Execution parent selects frozen schema. | N |
| `Evidence.ListRecords` | `Evidence.Records.List` | Product | Merged: One selector/cut contract. | GC |
| `Evidence.WatchRecords` | `Evidence.Records.Watch` | Product | Kept: Snapshot then durable changes. | GC |
| `Execution.AppendExecutions` | `Executions.Create` | Product | Renamed: Allocate mutable execution resources. | GC |
| `Execution.CancelExecution` | `Executions.Cancel` | Product | Renamed: Running only, queued refusal names action. | G |
| `Execution.CreateCycle` | `Cycles.Create` | Product | Merged: Preserve new-cycle admission. | GCS |
| `Execution.DeleteCase` | `Cases.Delete` | Product | Renamed: Preserve lifecycle admission. | G |
| `Execution.GetCase` | `Cases.Get` | Product | Renamed: Retain planned case. | G |
| `Execution.GetCycle` | `Cycles.Get` | Product | Merged: One operational/saved read. | GC |
| `Execution.GetExecution` | `Executions.Get` | Product | Merged: One operational/saved read with seal. | GCS |
| `Execution.GetExecutionCommand` | `Executions.Commands.Get` | Product | Renamed: Retained command receipt. | C |
| `Execution.ListCaseParameters` | `Cases.Parameters` | Product | Renamed: Pinned definition values. | G |
| `Execution.ListCases` | `Cases.List` | Product | Renamed: Retain ordered cases. | G |
| `Execution.ListCycles` | `Cycles.List` | Product | Merged: Operational/query/history selection. | G |
| `Execution.ListExecutionParameters` | `Executions.Parameters` | Product | Renamed: Frozen execution values. | G |
| `Execution.ListExecutionSteps` | `Executions.Steps` | Product | Renamed: Frozen steps and state. | G |
| `Execution.ListExecutions` | `Executions.List` | Product | Merged: Operational/query/history selection. | GC |
| `Execution.PauseCycle` | `Cycles.Set` | Product | Merged: Whole small writable value and Get revision; preserve pause admission. | S |
| `Execution.PauseExecution` | `Executions.Set` | Product | Merged: Whole small writable value and Get revision; preserve pause admission. | GC |
| `Execution.RenameCycle` | `Cycles.Set` | Product | Merged: Whole small writable value and Get revision, including label. | G |
| `Execution.RenameExecution` | `Executions.Set` | Product | Merged: Whole small writable value and Get revision; rename only this execution. | G |
| `Execution.RerunCycle` | `Cycles.Create` | Product | Merged: source_cycle preserves rerun notices. | GC |
| `Execution.ResumeCycle` | `Cycles.Set` | Product | Merged: Whole small writable value and Get revision, with paused=false. | S |
| `Execution.ResumeExecution` | `Executions.Set` | Product | Merged: Whole small writable value and Get revision, with paused=false. | GC |
| `Execution.SealCycle` | `Cycles.Seal` | Product | Renamed: Preserve durable finalization. | GC |
| `Execution.SetCycleArchived` | `Cycles.Set` | Product | Merged: Whole small writable value and Get revision, including archived. | G |
| `Execution.SetExecutionArchived` | `Executions.Set` | Product | Merged: Whole small writable value and Get revision, including archived. | G |
| `Execution.StartExecution` | `Executions.Start` | Product | Renamed: Activate selected execution. | GC |
| `Execution.StopExecution` | `Executions.Stop` | Product | Renamed: Stop scheduling at safe boundary. | GC |
| `Execution.SubmitExecutionCommand` | `Executions.Commands.Create` | Product | Renamed: Interactive Next/RunStep/Finish, replay receipt. | C |
| `Execution.UpdateCyclePolicy` | `Cycles.Set` | Product | Merged: Whole small writable cycle value and Get revision, including policy. | G |
| `Health.ListHealth` | `Evidence.HealthEvents.List` | Product | Merged: Name evidence collection. | GS |
| `History.BrowseCycles` | `Cycles.List` | Product | Merged: Retain saved filtering and ordering. | G |
| `History.BrowseExecutions` | `Executions.List` | Product | Merged: Retain live/saved availability. | G |
| `History.DeleteExecution` | `Executions.Delete` | Product | Renamed: Keep seal precondition and replay protection. | G |
| `History.DownloadCheckpointArtifact` | `Content.Artifacts.Get` + `Storage.GetFileBytes` | Product + Library | Merged: Same reads at returned cut/seal. | G |
| `History.GetCheckpointContext` | `Executions.Get` + `Cycles.Get` + `Documents.Get` + `Prompts.List` + `Evidence.HealthEvents.List` + `Evidence.Notes.List` + `Evidence.Traceability.Get` + `Backend.Appearance` | Product | Merged: Position-matched frozen context, no checkpoint DTO. | GC |
| `History.GetCheckpointDocument` | `Documents.Get` | Product | Merged: Same read at returned cut/seal. | GC |
| `History.GetCheckpointEvidence` | `Content.Artifacts.List` + `Content.ExecutionFiles.List` + `Catalog.RecordSchemas.List` + `Catalog.StreamSchemas.List` | Product | Merged: One cut, preserve frozen schemas/availability. | G |
| `History.GetExecutionCheckpoint` | `Executions.Get` | Product | Merged: Return position and seal in ordinary read. | C |
| `History.ListCheckpointRecords` | `Evidence.Records.List` | Product | Merged: Same selector at returned cut/seal. | G |
| `History.ListCheckpointTelemetry` | `Evidence.Samples.List` | Product | Merged: Same selector at returned cut/seal. | G |
| `History.ReadCheckpointOutput` | `Content.ExecutionFiles.Get` | Product | Merged: Same bounded file read at cut/seal. | G |
| `Media.AudioRecording` | `Media.Captures.Create` + `Media.Captures.Watch` + `Media.Captures.Stop` | Product | Merged: Owned recording lifecycle, retained result. | G |
| `Media.CameraRecording` | `Media.Captures.Create` + `Media.Captures.Watch` + `Media.Captures.Stop` | Product | Merged: Keep explicit unsupported-kind refusal. | N |
| `Media.Photo` | `Media.Captures.Create` | Product | Merged: Typed photo source. | G |
| `Media.ScreenRecording` | `Media.Captures.Create` + `Media.Captures.Watch` + `Media.Captures.Stop` | Product | Merged: Keep explicit unsupported-kind refusal. | N |
| `Media.Screenshot` | `Media.Captures.Create` | Product | Merged: Typed screenshot source. | G |
| `Notes.CreateNote` | `Evidence.Notes.Create` | Product | Kept: Allocate identified note. | G |
| `Notes.EditNote` | `Evidence.Notes.Set` | Product | Renamed: Whole note writable value and revision from Evidence.Notes.Get. | G |
| `Notes.ListNotes` | `Evidence.Notes.List` | Product | Merged: Retain note collection. | G |
| `Observation.GetRuntimeShell` | `Backend.Overview.Get` | Product | Renamed: Runtime overview resource. | G |
| `Observation.GetTraceability` | `Evidence.Traceability.Get` | Product | Merged: Retain evidence context. | G |
| `Observation.WatchExecutionProgress` | `Executions.Progress.Watch` | Product | Renamed: Paired progress read/watch. | GS |
| `Observation.WatchRuntime` | `Backend.Overview.Watch` | Product | Renamed: Same overview selection. | G |
| `Planning.MaterializeTemplate` | `Cases.Create` | Product | Merged: Single materialized selection. | GCS |
| `Planning.PreviewSelection` | `Cases.Create` | Product | Merged: validate_only, no registration. | GC |
| `Planning.RegisterSelection` | `Cases.Create` | Product | Merged: Atomic plan with replay key. | C |
| `Plots.ListAnalyses` | `Catalog.Analyses` | Product | Renamed: Move immutable declarations. | N |
| `Plots.QuerySnapshots` | `Plots.Snapshots.List` | Product | Renamed: Bounded evaluated snapshots. | G |
| `Plots.Watch` | `Plots.Snapshots.Watch` | Product | Renamed: Same plot selector/cut. | G |
| `Plugins.GetInstallation` | `Plugins.Get` | Library | Renamed: One managed/remote entry. | G |
| `Plugins.Install` | `Storage.CreateFileProduction` + `Storage.GetFileProduction` + `Storage.CreateFileWriter` + `Storage.AppendFileBytes` + `Storage.DeleteFileProduction` + `Storage.PublishFileProduction` + `Plugins.Create` | Library | Merged: Shared upload then managed admission. | GC |
| `Plugins.ListAdmittedPlugins` | `Plugins.List` | Library | Merged: state=READY filter. | G |
| `Plugins.ListInstallations` | `Plugins.List` | Library | Merged: One entry selection. | G |
| `Plugins.Reload` | `Plugins.Restart` | Library | Renamed: Replace activation, retain finalization rules. | GS |
| `Plugins.Remove` | `Plugins.Delete` | Library | Renamed: Remove entry, migrate old payload. | G |
| `Plugins.SetEnabled` | `Plugins.Set` | Library | Renamed: Whole small plugin writable value and Plugins.Get revision, including enabled. | G |
| `Plugins.List` | `Plugins.List` | Library | Merged: One managed/remote entry list. | S |
| `Plugins.Add` | `Plugins.Create` | Library | Merged: Remote placement admission. | S |
| `Prompts.GetDefinition` | `Catalog.PromptDefinitions.Get` | Product | Renamed: Separate definition from asked prompt. | S |
| `Prompts.GetPrompt` | `Prompts.Get` | Product | Renamed: Retain asked prompt state. | GCS |
| `Prompts.ListDefinitions` | `Catalog.PromptDefinitions.List` | Product | Renamed: Immutable definitions. | S |
| `Prompts.ListPending` | `Prompts.List` | Product | Merged: state=PENDING filter. | GCS |
| `Prompts.Respond` | `Prompts.Answers.Create` | Product | Renamed: One answer resource and replay receipt. | GCS |
| `Query.Aggregate` | `Aggregates.Get` | Product | Renamed: Complete read-only aggregate. | T |
| `Query.AggregateSamples` | `Evidence.Statistics` | Product | Renamed: Retain stream statistics selectors. | T |
| `Query.ListCycles` | `Cycles.List` | Product | Merged: Preserve projection/filter semantics. | T |
| `Query.ListExecutions` | `Executions.List` | Product | Merged: Preserve projection/filter semantics. | TS |
| `Query.ListRecords` | `Evidence.Records.List` | Product | Merged: One canonical evidence read. | T |
| `Query.ListSamples` | `Evidence.Samples.List` | Product | Merged: One canonical sample read. | T |
| `Remote.CreatePairing` | `Access.CreatePairing` | Library | Kept: Preserve access material lifecycle. | G |
| `Remote.ReadListener` | `Access.Listener.Get` | Library | Renamed: Derived listener state. | G |
| `Remote.RegenerateCertificate` | `Access.CreateCertificate` | Library | Renamed: Allocate replacement certificate. | G |
| `Remote.UpdateListener` | `Access.Listener.Set` | Library | Renamed: Whole listener writable settings and revision from GetListener. | G |
| `Reports.PublishExecutionReport` | `Reports.Publish` | Product | Renamed: Explicit durable publication, never implicit download. | S |
| `Reports.DownloadCycleReport` | `Reports.Get` + `Storage.GetFileBytes` + `Reports.Create` | Product + Library | Merged: Published bytes or explicit preview creation. | GCS |
| `Reports.DownloadExecutionReport` | `Reports.Get` + `Storage.GetFileBytes` + `Reports.Create` | Product + Library | Merged: Published bytes or explicit preview creation. | GCS |
| `Reports.MaterializeReport` | `Reports.Create` | Product | Merged: Render only, no publication. | N |
| `Runtime.GetHealth` | `Status.Health` | Library | Removed: Remove duplicate health gRPC method. | C |
| `Runtime.GetIdentity` | `Status.Identity` | Library | Removed: Remove duplicate identity gRPC method. | C |
| `Secrets.Export` | `Secrets.Export` | Product | Renamed: Existing Base64 JSON transfer, not encryption. | G |
| `Secrets.Import` | `Secrets.Import` | Product | Merged: Preserve choices and generation conflict. | G |
| `Secrets.List` | `Secrets.List` | Product | Renamed: Bounded metadata, not secret values. | G |
| `Secrets.Lookup` | `Secrets.Get` | Product | Renamed: Granted identifier only. | N |
| `Secrets.Preview` | `Secrets.Import` | Product | Merged: validate_only, no writes. | G |
| `Secrets.Put` | `Secrets.Set` | Product | Renamed: Whole writable secret value and revision from Secrets.Get. | G |
| `Secrets.Remove` | `Secrets.Delete` | Product | Renamed: Preserve keyring/catalog cleanup. | G |
| `Settings.GetPluginSettings` | `Plugins.Settings.Get` | Product | Renamed: Saved values, not UI declarations. | G |
| `Settings.GetSettings` | `Settings.Schema` | Product | Renamed: UI declarations/cards/bootstrap metadata. | G |
| `Settings.ListStorageSources` | `Storage.Sources.Get` | Product | Renamed: One ordered value. | G |
| `Settings.ReadApplicationSettings` | `Settings.Get` | Product | Renamed: Revisioned application values. | G |
| `Settings.ReorderStorageSources` | `Storage.Sources.Set` | Product | Merged: Whole small ordered value and revision from Storage.Sources.Get. | G |
| `Settings.SetStorageSourceEnabled` | `Storage.Sources.Set` | Product | Merged: Whole small ordered value and Get revision, including enabled state. | G |
| `Settings.UpdateApplicationSettings` | `Settings.Set` | Product | Renamed: Whole writable value and matching Get revision. | G |
| `Settings.UpdatePluginSettings` | `Plugins.Settings.Set` | Product | Renamed: Whole writable value and matching Get revision; if large, use separately revisioned setting sub-resources with Plugins.Settings.Get/Plugins.Settings.Set. | G |
| `Telemetry.ListSamples` | `Evidence.Samples.List` | Product | Merged: One canonical sample read. | CS |
| `Telemetry.WatchSamples` | `Evidence.Samples.Watch` | Product | Kept: Same source selector and durable boundary. | C |

## Locked Inventories and Counts by Face

Counts below cover **this product's complete target and its explicitly composed
library dependencies**, not the sibling library's entire protocol. Superseded
whole-library counts from before the audience lock do not describe the new
faces and are removed here; its other storage/provider/native operations remain
with that repository's owner. This product does not redeclare those operations.
Counts are operation templates, not dynamic provider or resource instances.

A logical service is an operation namespace. Only Instance and client services
are independent wire gRPC services here. Product plugin operation groups and
context groups have typed declarations carried by Instance.Invoke; they add no
independent gRPC entry methods. Thus a service count always states whether it is
logical or wire. Every operation/carrier has one request and one response
wrapper in its own package, including logical child-call operations. Supporting
payloads, shared primitives and nested carrier control/event frames are excluded;
implementation must census them separately. No automatic twins are counted.

| Face / Owner | Logical Services | Wire Services | Operations | gRPC Entry Methods | Requests | Responses | Method Messages |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Plugin Protocol / Library | 1 | 1 | 4 | 4 | 4 | 4 | 8 |
| Plugin Operations / Product | 4 | 0 | 5 | 0 | 5 | 5 | 10 |
| Plugin Face Combined | 5 | 1 | 9 | 4 | 9 | 9 | 18 |
| Context / Library Dependencies | 3 | 0 | 11 | 0 | 11 | 11 | 22 |
| Context / Product | 12 | 0 | 18 | 0 | 18 | 18 | 36 |
| Context Combined | 15 | 0 | 29 | 0 | 29 | 29 | 58 |
| Client API / Library Dependencies | 10 | 10 | 34 | 34 | 34 | 34 | 68 |
| Client API / Product | 32 | 32 | 93 | 93 | 93 | 93 | 186 |
| Client API Combined | 42 | 42 | 127 | 127 | 127 | 127 | 254 |
| All Faces, Each Declaration Counted | 62 | 43 | 165 | 131 | 165 | 165 | 330 |

The product declares 48 logical services and 116 operations/pairs across the
three faces (32/93 client, 12/18 context, 4/5 plugin). The mapped library
contributes 14 logical services and 49 operations/pairs (10/34 client, 3/11
context, 1/4 plugin protocol), with 38 gRPC entry methods. These are declaration
counts: a capability on two faces counts twice, and a shared handler counts
neither as a third declaration nor as a second registry.

### Client Mapping and Companion Coverage

The 132 ordinary mapping rows remain 7 kept, 52 renamed, 71 merged and 2 duplicate
gRPC methods removed. All 132 automatic Run twins are removed. Their mapped
client surface is still 39 services (23 nested), 120 methods and 240 wrappers:
7 library services/27 methods plus 32 product services/93 methods. It is 144 fewer
methods than the 264 current product methods. Seven library client companions
complete the generic client access needed here: CatalogEntries.List,
Invocations.Get/Watch/Cancel/Invoke and Time.Get/Sleep. This yields 42 client
services (23 nested), 127 methods and 254 wrappers; the 132 source rows are not
rewritten as if these companions had present product callers.

Each row below is one service on the **client face**. Dots express hierarchy;
namespace-only parents are not empty services. The package column is exact.

| Owner | Client Package | Locked Service | Nested | Count | Operations |
| --- | --- | --- | --- | ---: | --- |
| Library | `harmos.v1` | Plugins | No | 7 | `Get`, `List`, `Watch`, `Create`, `Set`, `Delete`, `Restart` |
| Library | `harmos.v1` | Access | No | 2 | `CreatePairing`, `CreateCertificate` |
| Library | `harmos.v1.access` | Access.Listener | Yes | 2 | `Get`, `Set` |
| Library | `harmos.v1` | Status | No | 4 | `Identity`, `Health`, `Availability`, `Diagnostics` |
| Library | `harmos.v1.status` | Status.Logs | Yes | 2 | `List`, `Watch` |
| Library | `harmos.v1.status` | Status.TraceEvents | Yes | 2 | `List`, `Watch` |
| Library | `harmos.v1` | Storage | No | 8 | `CreateFileProduction`, `GetFileProduction`, `CreateFileWriter`, `AppendFileBytes`, `DeleteFileProduction`, `PublishFileProduction`, `GetFileBytes`, `WatchFileBytes` |
| Product | `dokime.v1` | Backend | No | 1 | `Appearance` |
| Product | `dokime.v1.backend` | Backend.Overview | Yes | 2 | `Get`, `Watch` |
| Product | `dokime.v1` | Catalog | No | 1 | `Analyses` |
| Product | `dokime.v1.catalog` | Catalog.Suites | Yes | 2 | `Get`, `List` |
| Product | `dokime.v1.catalog` | Catalog.RecordSchemas | Yes | 2 | `Get`, `List` |
| Product | `dokime.v1.catalog` | Catalog.StreamSchemas | Yes | 2 | `Get`, `List` |
| Product | `dokime.v1.catalog` | Catalog.PromptDefinitions | Yes | 2 | `Get`, `List` |
| Product | `dokime.v1` | Cases | No | 6 | `Create`, `Get`, `List`, `Watch`, `Delete`, `Parameters` |
| Product | `dokime.v1` | Cycles | No | 6 | `Get`, `List`, `Watch`, `Create`, `Set`, `Seal` |
| Product | `dokime.v1` | Executions | No | 11 | `Get`, `List`, `Watch`, `Create`, `Set`, `Start`, `Stop`, `Cancel`, `Delete`, `Parameters`, `Steps` |
| Product | `dokime.v1.executions` | Executions.Commands | Yes | 2 | `Get`, `Create` |
| Product | `dokime.v1.executions` | Executions.Progress | Yes | 2 | `Get`, `Watch` |
| Product | `dokime.v1` | Evidence | No | 2 | `Statistics`, `Archive` |
| Product | `dokime.v1.evidence` | Evidence.Records | Yes | 2 | `List`, `Watch` |
| Product | `dokime.v1.evidence` | Evidence.Traceability | Yes | 2 | `Get`, `Watch` |
| Product | `dokime.v1.evidence` | Evidence.Notes | Yes | 5 | `Get`, `List`, `Watch`, `Create`, `Set` |
| Product | `dokime.v1.evidence` | Evidence.HealthEvents | Yes | 2 | `List`, `Watch` |
| Product | `dokime.v1.evidence` | Evidence.Samples | Yes | 2 | `List`, `Watch` |
| Product | `dokime.v1.content` | Content.Artifacts | Yes | 4 | `Get`, `Create`, `List`, `Watch` |
| Product | `dokime.v1.content` | Content.ExecutionFiles | Yes | 3 | `Get`, `List`, `Watch` |
| Product | `dokime.v1` | Documents | No | 2 | `Get`, `Watch` |
| Product | `dokime.v1.plots` | Plots.Snapshots | Yes | 2 | `List`, `Watch` |
| Product | `dokime.v1.plots` | Plots.Settings | Yes | 2 | `Get`, `Set` |
| Product | `dokime.v1` | Prompts | No | 3 | `Get`, `List`, `Watch` |
| Product | `dokime.v1.prompts` | Prompts.Answers | Yes | 1 | `Create` |
| Product | `dokime.v1` | Settings | No | 3 | `Get`, `Set`, `Schema` |
| Product | `dokime.v1.plugins` | Plugins.Settings | Yes | 2 | `Get`, `Set` |
| Product | `dokime.v1.storage` | Storage.Sources | Yes | 2 | `Get`, `Set` |
| Product | `dokime.v1` | Secrets | No | 7 | `List`, `Watch`, `Get`, `Set`, `Delete`, `Export`, `Import` |
| Product | `dokime.v1` | Reports | No | 3 | `Get`, `Create`, `Publish` |
| Product | `dokime.v1.media` | Media.Captures | Yes | 4 | `Create`, `Get`, `Watch`, `Stop` |
| Product | `dokime.v1` | Aggregates | No | 1 | `Get` |
| Library | `harmos.v1` | CatalogEntries | No | 1 | `List` |
| Library | `harmos.v1` | Invocations | No | 4 | `Get`, `Watch`, `Cancel`, `Invoke` |
| Library | `harmos.v1` | Time | No | 2 | `Get`, `Sleep` |

The original mapping reaches 102 distinct destinations; these 18 companions
complete that 120-method surface: Cases.Watch, Content.Artifacts.Watch,
Content.ExecutionFiles.Watch, Cycles.Watch, Documents.Watch,
Evidence.HealthEvents.Watch, Evidence.Notes.Get/Watch, Evidence.Traceability.Watch,
Executions.Progress.Get, Executions.Watch, Media.Captures.Get, Plots.Settings.Get,
Plugins.Watch, Prompts.Watch, Secrets.Watch, Status.Logs.List and
Status.TraceEvents.List. The seven generic companions above are additional.
Invocations.Invoke names the generic client invocation path required by this
lock; Get/Watch/Cancel retain invocation observation/control, and CatalogEntries.List
provides authorized discovery. It is a client operation, not another plugin
carrier or a second operation registry.

Every present method has one mapping row. A `+` cell is a required workflow,
not an alternative or compatibility RPC. CreateFileWriter is a fenced-resume
branch and DeleteFileProduction an abandon branch, not mandatory before Publish.
Content.ExecutionFiles.Get resolves metadata for Storage.WatchFileBytes;
Content.ExecutionFiles.Watch follows product membership/metadata. Cases.Parameters
and Executions.Parameters/Steps remain sole paged noun reads. Client progress
Get/Watch and Executions.Get expose observation; no client progress Set exists.
Evidence.Archive selects/assembles scoped product evidence at a cut and returns
immutable content references for library ranged reads. Archive layout and policy
stay product-owned storage operations, without an archive manager.

### Plugin Protocol and Product Catalog Operations

| Package | Logical Service | Operations | Count | gRPC Entry Methods |
| --- | --- | --- | ---: | ---: |
| `harmos.plugin.v1` | Instance | Get, Invoke, Samples, Stop | 4 | 4 |
| `dokime.plugin.v1` | TestCases | Materialize | 1 | 0 |
| `dokime.plugin.v1` | Executions | Initialize, Cleanup | 2 | 0 |
| `dokime.plugin.v1` | Steps | Execute | 1 | 0 |
| `dokime.plugin.v1` | Criteria | Evaluate | 1 | 0 |

Instance.Get returns identity and the catalog, including explicit operation
visibility, requirements, fingerprints, descriptors, digest, streams and settings
declarations. Invoke carries the five product lifecycle families and arbitrary
other catalog operations; Samples preserves demand/publication semantics; Stop
preserves shutdown and cleanup. Product operations retain template-local provider
bindings; their ID and wrapper package is the plugin face. No product lifecycle
service is registered on the client endpoint.

### Context Inventory

Every row is served by the host to a plugin through its active context object as
a child call. Package-qualified identity distinguishes it from any client method
with the same local name. All 29 operations have typed request/response wrappers
and zero added independent gRPC entry methods.

| Context Package | Logical Service | Operations | Count |
| --- | --- | --- | ---: |
| `harmos.context.v1` | Time | Get, Sleep | 2 |
| `harmos.context.v1` | HostValues | Get | 1 |
| `harmos.context.v1` | Storage | CreateFileProduction, GetFileProduction, CreateFileWriter, AppendFileBytes, DeleteFileProduction, PublishFileProduction, GetFileBytes, WatchFileBytes | 8 |
| `dokime.context.v1` | Prompts | Create | 1 |
| `dokime.context.v1.evidence` | Records | Append, List | 2 |
| `dokime.context.v1` | Evidence | Statistics | 1 |
| `dokime.context.v1.content` | Artifacts | Create | 1 |
| `dokime.context.v1.executions` | Progress | Get, Set | 2 |
| `dokime.context.v1.evidence` | Logs | Append | 1 |
| `dokime.context.v1.evidence` | Traceability | Get, Set | 2 |
| `dokime.context.v1` | Documents | AppendParts | 1 |
| `dokime.context.v1` | Files | Capture | 1 |
| `dokime.context.v1.files` | Captures | Start, Stop | 2 |
| `dokime.context.v1` | Backend | Environment | 1 |
| `dokime.context.v1` | Effects | Create, Get, Set | 3 |

The six nested product context namespaces are Evidence.Records, Content.Artifacts,
Executions.Progress, Evidence.Logs, Evidence.Traceability and Files.Captures.
HostValues.Get routes declared setting/secret requests to the product's resolver;
it does not expose the client settings/secrets stores. Effects.Get is an added
companion. Progress.Get and Traceability.Get are separate context Get/revision
mirrors for context Set; they cannot borrow the client declaration. Large writable
parts split into identified, separately revisioned Get/Set resources when needed.

### Capabilities Declared on Both Context and Client Faces

Each row below denotes **two declarations** with independent authorization,
request/response types and fingerprints; shared handlers/storage mechanisms are
allowed. The context side always carries inherited invocation scope. This is the
complete overlap in the mapped inventory, not permission to expose all client
operations to plugins.

| Capability on Each Face | Context Package | Client Package | Count Per Face | Reason |
| --- | --- | --- | ---: | --- |
| Time.Get, Time.Sleep | `harmos.context.v1` | `harmos.v1` | 2 | Both callers need authoritative time and host-clock waits. |
| Storage.CreateFileProduction/GetFileProduction/CreateFileWriter/AppendFileBytes/DeleteFileProduction/PublishFileProduction/GetFileBytes/WatchFileBytes | `harmos.context.v1` | `harmos.v1` | 8 | Plugins and clients upload, resume and read produced bytes through one storage mechanism, with separate caller authority. |
| Evidence.Records.List | `dokime.context.v1.evidence` | `dokime.v1.evidence` | 1 | Plugins read produced records; clients inspect the same evidence under their own selection policy. |
| Evidence.Statistics | `dokime.context.v1` | `dokime.v1` | 1 | Plugin decisions and client views both need statistics over produced samples. |
| Content.Artifacts.Create | `dokime.context.v1.content` | `dokime.v1.content` | 1 | Either caller attaches published bytes as product metadata; scope/admission differ. |
| Executions.Progress.Get | `dokime.context.v1.executions` | `dokime.v1.executions` | 1 | Plugin obtains its Set revision; client observes current progress. |
| Evidence.Traceability.Get | `dokime.context.v1.evidence` | `dokime.v1.evidence` | 1 | Plugin obtains its Set value/revision; client reads traceability. |

There are 15 duplicated capabilities: 10 library and 5 product, adding 15 distinct
face declarations compared with counting each capability once. Product unique
capabilities remain 111 (= 116 declarations minus 5 overlaps); this does not
preserve the old 111-pair census because pairs now belong to faces. Context-only
Append is not added to a client service for symmetry. A client's Public plugin
invocation and a plugin's Public child call target the same provider operation;
its visibility remains catalog metadata, not a duplicate client/context provider
operation or a direct connection to the instance.

### Complete Mapping of Present Plugin and Context Calls

All 17 product context bindings, two library clock calls and five dynamic
lifecycle families are covered below: 24 present rows. The first five belong to
the product plugin protocol; all remaining rows are context calls. Qualified
operation IDs use the same package/service/method derivation as client methods.
All 132 client rows above remain exclusively on the client face.

| Present Operation or Family | Face | Locked Qualified Operation | Owner | Disposition and Preserved Behavior |
| --- | --- | --- | --- | --- |
| `<template>.materialize` | Plugin Protocol | `dokime.plugin.v1.TestCases/Materialize` | Product | Renamed; pure materialization, request/response wrapper. |
| `<template>.initialize` | Plugin Protocol | `dokime.plugin.v1.Executions/Initialize` | Product | Renamed; acquire execution-local suite resources. |
| `Step.operation` | Plugin Protocol | `dokime.plugin.v1.Steps/Execute` | Product | Renamed family; frozen step identity and retry evidence. |
| `Criterion.operation` | Plugin Protocol | `dokime.plugin.v1.Criteria/Evaluate` | Product | Renamed family; captured-evidence judgment. |
| `<template>.cleanup` | Plugin Protocol | `dokime.plugin.v1.Executions/Cleanup` | Product | Renamed; required cleanup after initialization. |
| `dokime.execution.prompt` | Context | `dokime.context.v1.Prompts/Create` | Product | Renamed; wait for answer, preserve scope and cancellation. |
| `dokime.settings.read` | Context | `harmos.context.v1.HostValues/Get` | Library | Generic declared-value delivery; product resolver owns caller-bound settings storage and policy. |
| `dokime.secrets.read` | Context | `harmos.context.v1.HostValues/Get` | Library | Generic declared-value delivery; product resolver owns granted-identifier secret storage and policy. |
| `harmos.clock.sleep` | Context | `harmos.context.v1.Time/Sleep` | Library | Kept meaning; normalized carriers, library-owned clock. |
| `harmos.clock.now` | Context | `harmos.context.v1.Time/Get` | Library | Renamed; library-owned clock. |
| `dokime.execution.record` | Context | `dokime.context.v1.evidence.Records/Append` | Product | Renamed; ordered immutable data, durable receipt. |
| `dokime.execution.records` | Context | `dokime.context.v1.evidence.Records/List` | Product | Merged; preserve count/last selection and bounded reads. |
| `dokime.execution.statistics` | Context | `dokime.context.v1.Evidence/Statistics` | Product | Merged; stream/field/step selectors and complete statistics. |
| `dokime.execution.artifact` | Context | `harmos.context.v1.Storage/CreateFileProduction` + `harmos.context.v1.Storage/GetFileProduction` + `harmos.context.v1.Storage/CreateFileWriter` + `harmos.context.v1.Storage/AppendFileBytes` + `harmos.context.v1.Storage/DeleteFileProduction` + `harmos.context.v1.Storage/PublishFileProduction` + `dokime.context.v1.content.Artifacts/Create` | Library + Product | Merged; separately declared context face of the shared unary session mechanism, acknowledged bytes. |
| `dokime.execution.progress` | Context | `dokime.context.v1.executions.Progress/Set` | Product | Renamed; context-only producer operation, whole small progress value and Executions.Progress.Get revision; no client Set. |
| `dokime.execution.log` | Context | `dokime.context.v1.evidence.Logs/Append` | Product | Renamed; ordered immutable log lines. |
| `dokime.execution.traceability` | Context | `dokime.context.v1.evidence.Traceability/Set` | Product | Renamed; whole small writable value, Evidence.Traceability.Get revision and scope; large writable parts split into separately revisioned entry Get/Set sub-resources. |
| `dokime.execution.document` | Context | `dokime.context.v1.Documents/AppendParts` | Product | Renamed; ordered immutable document contributions. |
| `dokime.files.copy_files` | Context | `dokime.context.v1.Files/Capture` | Product | Renamed; freeze file copies, preserve originals. |
| `dokime.files.start_capture` | Context | `dokime.context.v1.files.Captures/Start` | Product | Renamed; start capture lifecycle. |
| `dokime.files.stop_capture` | Context | `dokime.context.v1.files.Captures/Stop` | Product | Renamed; finish capture at safe boundary. |
| `dokime.execution.environment` | Context | `dokime.context.v1.Backend/Environment` | Product | Renamed; derived environment read. |
| `dokime.execution.effect.prepare` | Context | `dokime.context.v1.Effects/Create` | Product | Renamed; durable inverse ticket, no hardware write. |
| `dokime.execution.effect.applied` | Context | `dokime.context.v1.Effects/Set` | Product | Renamed; whole writable application state and Effects.Get revision; preserve earliest inverse. |

The old host/plugin-only union added 18 operations to client namespaces. The
new inventory instead declares five lifecycle operations on the plugin face
and 18 product context operations, including five separately declared client
capabilities. Library context transfer wrappers are also separate from client
wrappers. This removes the mixed-face namespaces without changing required
behavior or introducing a second transfer engine.

## Source Verification and Report Corrections

Verified on the current product tree and by read-only inspection of the library
checkout named by its report:

- `proto/dokime/v1/client_*.proto`: 23 services, 132 ordinary methods and 132 Run
  twins. Counting `message` declarations, including nested declarations, across
  `proto/dokime/v1/*.proto` gives 953. This confirms the reports' correction of
  131/262. PublishExecutionReport is present. Remove 264 twin-envelope messages;
  the final payload-message total remains unknown; neither subtraction nor the
  330 scoped method messages establishes it.
- `crates/dokime-runtime-next/src/services/mod.rs`: 13 registered client services,
  not complete serving of all 23. `dokime-operations-next/src/lib.rs` declares and
  dispatches exactly log, progress, record, records, statistics, prompt and library
  Sleep. The 17 context bindings are real requirements, not obsolete methods.
- `crates/dokime-gui/src/lib/rpc/generated/client.ts`: 129 distinct canonical
  product paths; Plugins.Add/List and Reports.PublishExecutionReport are absent.
  `native-execution.ts` still consumes the older projection. Regeneration alone
  cannot fix its installation-shaped Remove and older DTO assumptions.
- `client/history.rs` requires the existing saved seal and exact checkpoint token;
  `client/paging.rs` hashes current rows and rejects changed cursors. They do not
  implement retained position paging for live mutable metadata. A product cut
  cannot be replaced by an unqualified sequence or a process epoch.
- `client/conversation.rs` sends CANCELLED and drops its unary work on Cancel;
  this confirms the blanket twins do not provide the library's typed cooperative
  settlement. `execution_rename_execution.rs` changes only the selected execution.
  Preserve Stop/Cancel distinction from `runtime-next/src/run_control.rs`.
- **Correction:** the product report describes Secret Import/Export as encrypted.
  `client_secrets.proto` and `client_types.proto` carry archive bytes; `docs/product-guide.md` / Shared
  Plugin Secrets explicitly describes Base64-encoded JSON, not encryption.
  Preserve the actual contract; adding encryption is a separate owner decision.
- **Decision overrides, not source discrepancies:** both reports' latest-read
  suggestions are replaced by owner-decided latest-durable/no-wait with
  a payload-size-independent 250 ms p95 durable-lag bound; whole-value Set with
  exact Get/revision symmetry is retained and bare replies become wrappers;
  WaitTime becomes Sleep; CreateRecord/CreateLog become Evidence.Records.Append/Evidence.Logs.Append.
  No arbitrary earlier cut API is carried forward.

## Defects to Fix Regardless of the Grammar Lock

The new host must serve every host operation plugins already declare: settings,
secrets, artifact upload, traceability, document parts, file copy/capture, effect
prepare/confirm, environment and clock-now. Only the seven dispatches listed
above exist now. Do not hide these gaps by deleting SDK requirements. The desktop
must move from its older generated contracts to the canonical new host, including
plugin entry/remove shapes and published-report support, once in the combined
cutover. The service-level named UNIMPLEMENTED list in `grpc-plugins/tasks.md`
remains an integration obligation, independent of this naming proposal.

Sibling-library findings to reconcile with its owner, not implement here: Python
defaults visibility to Public while the other SDKs use Private; the locked fix
is explicit visibility in all four SDKs with no default. C# lacks an
invocation-local resource store and cancellation reason; static Storage services
expose a subset; retry policy has two shapes; the plugin proto's draft/not-implemented
header is stale. Generated availability is not proof of application consumption.

## Open Questions During Implementation

Settings/secrets ownership and all four hierarchical client roots are locked,
not open questions. Cycle-manager ownership is intentionally chosen at the manager
split by the simplest complete option; whether a capture manager is needed follows
whether recordings remain. The following protocol questions remain open.
Settle each when its implementation is reached;
they do not block the rest of the locked work. Snippets and counts do not settle
these contracts or claim implementation.

1. **SDK coverage:** visibility has no default in all four SDKs and is locked.
   Settle remaining plugin parity across Rust, Python, C# and C++, including
   C# resource/cancellation parity and generic managed-client coverage in all four languages versus Rust only. Library ownership is locked.
2. **Retention and restart:** how long does a returned position stay valid and
   which positions/cuts survive restart? Define retained mutable context and
   explicit unavailability without arbitrary earlier positions.
3. **Watches:** settle exact snapshot/follow, durable progress, gaps/resync,
   bounded delivery and actual feed coverage. The shared `at`/`resolved_at`
   rule and durable visibility are locked.
4. **Deletion:** settle deletion while readers hold data across backends,
   including in-progress ranged downloads and retained-cut invalidation.
5. **Managed cancellation:** identify which operations actually promise a
   cooperative final result and require one replacement managed gRPC method. Preserve
   the plugin protocol's own carriers and proven settlement; current product
   twins establish no such promise.

### SDK Ownership and Remaining Coverage Facts

Ownership is decided; size is a review aid for the remaining coverage work.
Counts below are current tracked `.rs`, `.py`, `.cs`, `.cpp`, `.hpp`, `.h` source under each `sdk/<language>/`,
including source tests/examples, excluding directories named `generated` or
`_generated` and Python `_pb2`/`_pb2_grpc` files. Counts include comments/blank lines;
they exclude manifests, proto files and build artifacts. These are source sizes,
not package sizes or promised implementation savings.

| Repository | Language | Files | Lines | Bytes |
| --- | --- | ---: | ---: | ---: |
| Product | Rust | 34 | 5,188 | 161,409 |
| Product | Python | 22 | 3,035 | 104,027 |
| Product | C# | 23 | 1,441 | 67,539 |
| Product | C++ | 19 | 1,893 | 63,260 |
| Library | Rust | 72 | 10,643 | 349,939 |
| Library | Python | 16 | 1,527 | 46,396 |
| Library | C# | 10 | 1,113 | 44,042 |
| Library | C++ | 12 | 1,535 | 54,670 |

Library SDKs contain plugin declarations, descriptors/catalogs, operation handlers,
requirements, child calls, scope/cancellation, streams/observers/publishers and
transport lifecycle. Rust also contains derives/encoding integration, optional
WebAssembly authoring and the generic managed application client; the other
languages expose generated Storage clients rather than equivalent generic clients.
The static Storage proto has 11 methods while 34 storage operations are declared
by the library operations crate, a separate coverage defect.

Product SDKs add suites/templates, materialization and step/criterion/cleanup
bindings, execution-local resources, evidence/plot/document builders and typed host
context conveniences; they also carry generated product clients and lifecycle
conveniences. Rust `sdk/rust/src/client.rs` uses the library generic client while
`client/host.rs` uses generated product stubs. Python is pure Python with generated
services and thin async clients. C#/C++ provide plugin authoring and generated client
surfaces. Product manifests depend on the library SDKs and will keep building on
them. Moving SDKs into a product was considered and not chosen because a second
plugin-hosting product would have to reimplement the plugin protocol in every
language, and the library could not be proven end to end by itself. Library
examples and SDK stories consume their own SDKs; product SDK stories and new-host
system stories exercise all four languages.

The library conformance tool under `tools/conformance/` needs real managed/remote
plugin endpoints, authoritative descriptors, registration/digest and admission
rules, invocation/child scope and failure propagation, cancellation settlement,
clock ticks, publication and stop cleanup. It needs conforming implementations
and fixtures within the library's self-sufficient protocol, SDK and conformance
suite, with no product vocabulary. Product equivalence stories add materialization,
prompts, evidence, seals and restart reads; protocol conformance does not replace
them. Preserve both kinds of proof and their independent consumers.

## Risks and Migration Plan

Durable lag is currently unmeasured: a 100 ms cadence plus synchronous commits
of growing batches may miss the payload-size-independent p95 bound. Prove small,
multi-megabyte and mixed loads in the reusable Nightly storage benchmark; first
reduce flush pieces and add size-triggered flushing if needed. If that is
insufficient, reads against accepted-data snapshots are the named fallback,
with additional snapshot/retention ownership and unsaved-data semantics; they
are not built now. SDK ownership is settled, but four-language plugin parity and
generic managed-client coverage still need agreement and proof.

Pinned live cuts require retained mutable context; row hashes cannot supply that
guarantee. Settle retention when implementing returned-position reads; independent
work continues. Upload sessions add calls
but permit resume and one plugin/client path. Removing twins loses a real library
capability unless promised settlement is explicitly retained; ordinary product
watches and byte delivery do not need it. Replacing IDs changes fingerprints,
requirements, manifests, fixtures and SDK constants together.

For implementation, first establish the three package roots and move declarations,
imports and generation inputs to their final faces. Then prepare library services
under their managers, narrow host-fact registration and descriptor-derived four-language client roots. Publish
and pin the exact library revision. Then declare product services with their final
nested names, compose the library services directly, split the product managers
and migrate product operations with focused proof. Remove replaced generic copies
as their consumers move, keeping the old desktop projection building until its
single cutover. Then migrate SDKs and CLI, and finally move the
desktop to the new host and grammar in one cutover. Delete old host paths, duplicate
types and obsolete stories only after their capabilities are covered. Keep each
intermediate tree building; do not ship adapters to obsolete APIs. Implementation
updates README, product/query/SDK/GUI guidance and affected OpenSpec owners together.
No data is deleted or reset; incompatible development storage uses an explicitly
selected fresh root, retaining the v1 baseline. Rollback before delivery uses the
previous coherent binary/client set and its storage root, not aliases or silent
schema migration. Separate targeted proof from final native Windows/Linux and
hosted proof; this document claims none of that implementation validation.
