# Operation Grammar Design

Status: locked by the owner on 2026-11-12
Boundary, managers and nested resources locked by the owner on 2026-11-12
Naming rule amended by the owner on 2026-11-12: bare verbs on the service's resource

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
  Each nested service lives in package `<owner>.v1.<lowercase-parent>`, with the
  part as the service name: `product.v1.plugins.Settings/Get`. One proto file per
  service lives in a folder per parent, for example `product/v1/plugins/settings.proto`.
  Messages in that package are `<Service><Method>Request/Response`, such as
  `SettingsGetRequest`; parent services retain names such as `ExecutionsStartRequest`.
  Replies are always wrappers. Operation IDs derive mechanically from package,
  service and method, with no second authored alias. A nested service may have a
  different owner from its parent: `product.v1.plugins.Settings` intentionally
  sits beside `library.v1.Plugins` on the host application's endpoint.
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

### Generated Client Names

All four SDK client roots expose the hierarchy mechanically from descriptor-set
packages and service names. A small generator in each SDK build consumes the same
package/service table: remove the owner/version prefix, use the remaining package
segments as parent resources and append the service name as the leaf. Top-level
services supply their own root leaf. Apply only each language's standard casing;
no hand-written per-operation code or aliases. Compose the library and product
roots by resource path, merging distinct children and rejecting collisions.

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
`Executions.Progress.Set`, with its matching Get/revision contract. Set is NOT a
client-facing RPC or part of the client method/message counts. Parameters and
steps each retain their sole paged noun-only read.

### Nested Wire Examples

`product/v1/plugins/settings.proto` intentionally extends a library-owned parent
resource from the product namespace. Settings payload declarations are illustrative.

```proto
syntax = "proto3";
package product.v1.plugins;
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

`product/v1/executions/progress.proto` serves only client observation:

```proto
syntax = "proto3";
package product.v1.executions;
message ProgressGetRequest { string name = 1; Position at = 2; }
message ProgressGetResponse { ExecutionProgress value = 1; bytes revision = 2; Position resolved_at = 3; }
message ProgressWatchRequest { string name = 1; Position at = 2; }
message ProgressWatchResponse { ProgressEvent event = 1; Position resolved_at = 2; }
service Progress {
  rpc Get(ProgressGetRequest) returns (ProgressGetResponse);
  rpc Watch(ProgressWatchRequest) returns (stream ProgressWatchResponse);
}
// The plugin producer calls the host operation Progress.Set over its invocation
// carrier; this client-facing service deliberately has no Set RPC.
```

Library listener configuration follows the same grammar:
`library/v1/access/listener.proto`, package `library.v1.access`, service Listener,
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
| OperationManager | CatalogEntries, Invocations, host-only HostValues; Plugin invocation carrier | Declarations, dispatch, invocations, retries/cancellation and plugin host-operation routing. |
| StreamManager | Samples, Plugin.Samples | Stream lifecycle, observation and delivery. |
| RecordManager | Generic record operations persisted through Storage; no new public service | Record capture lifecycle. |
| StateManager | Generic state operations; no new public service | Live state and observation. |

These manager types already exist in the runtime, including the generic operations
manager's internal `Manager` and `WeakOperationManager`; their target public-service
coverage is not a claim of landed implementation. Time remains library-owned
(`Time.Get`, `Time.Sleep`), under ServiceManager with the authoritative runtime
clock; no new clock
manager is required. The plugin protocol, SDKs, operations/invocations, storage
and streams remain library-owned. Their pre-existing full inventories are not
recounted as new services of this product mapping.

Settings and secrets are product-owned: stores, schemas, UI declarations,
archives and use of an operating-system credential store are too opinionated
for the library. This closes the per-plugin settings/secrets question. The
library supplies only the generic host-operation mechanism by which a host
application serves values to a plugin, plus the settings-declaration section of
the plugin catalog. The generic host operation is HostValues.Get; a product
registers its value resolver rather than introducing library settings stores. Product Plugins.Settings is intentionally separate from
library Plugins. Product-only status reads remain in Backend; product facts
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

The nested service lives in `product/v1/evidence/records.proto`:

```proto
syntax = "proto3";
package product.v1.evidence;
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
message RecordsAppendRequest {
  string parent = 1; repeated Record records = 2; string request_id = 3;
}
message RecordsAppendResponse { Receipt receipt = 1; }
service Records {
  rpc List(RecordsListRequest) returns (RecordsListResponse);
  rpc Watch(RecordsWatchRequest) returns (stream RecordsWatchResponse);
  // Append wrappers above belong to a host operation called by a plugin.
  // They do not add a client-facing RPC to this service.
}
```

Append's declaration is carried by the plugin host-operation transport rather
than served as a client RPC; its sketch shows the operation's typed wrappers.

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
service alias. These declarations share `library/v1/storage.proto`:

```proto
syntax = "proto3";
package library.v1;
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
are the existing library mechanism, not new product methods.

Product artifact/file metadata lives in `product.v1.content.Artifacts` and
`product.v1.content.ExecutionFiles`, while bytes use library Storage directly.
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

Plugin lifecycle verbs each have one locked meaning: TestCases.Materialize resolves
parameters into a case without running it; Executions.Initialize acquires suite
resources; Steps.Execute runs one frozen step; Criteria.Evaluate judges captured
evidence; Executions.Cleanup releases suite resources. Host operation IDs remain
template-local where necessary; one method name is not one shared handler instance.
Files.CopyFiles becomes Files.Capture, freezing selected external files into
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

## Complete Mapping from Present Product Operations

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
Detailed caller owners: `crates/dokime-gui/src/lib/api/native-*.ts`,
`src/lib/rpc/appearance.ts`, `src-tauri/src/`, `crates/dokime-client/src/`,
`crates/dokime-cli/src/commands/`, `sdk/rust/src/client/host.rs`,
`sdk/python/src/dokime/client.py`, server query/authorization stories and new-host
`tests/system/`. Wire declarations and handler code win over report assumptions.

| Present Operation | Locked Destination | Destination Owner | Disposition and Reason | Callers |
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

## Locked Client Inventory and Counts

The first table counts the client surface reached or completed by THIS repository,
grouped by declaration owner and served directly on the host application endpoint.
Existing generic library services beyond these destinations are not added a second
time; the full library inventory and plugin-only product union follow below.

| Owner in This Client Mapping | Services | Nested Services | Operations | gRPC Methods | Requests | Responses | Method Messages |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Library | 7 | 3 | 27 | 27 | 27 | 27 | 54 |
| Product | 32 | 20 | 93 | 93 | 93 | 93 | 186 |
| Combined Mapped Client Surface | 39 | 23 | 120 | 120 | 120 | 120 | 240 |

Each row below is one client service. Dots express hierarchy, not a flat compound
service name. Nested services count as separate wire services; namespace-only
parents do not count as empty services. The package owner is `library` or `product`
respectively; actual product declarations use `dokime`, for example
`dokime.v1.plugins.Settings` beside library Plugins.

| Owner | Locked Service | Nested | Count | Operations |
| --- | --- | --- | ---: | --- |
| Library | Plugins | No | 7 | `Get`, `List`, `Watch`, `Create`, `Set`, `Delete`, `Restart` |
| Library | Access | No | 2 | `CreatePairing`, `CreateCertificate` |
| Library | Access.Listener | Yes | 2 | `Get`, `Set` |
| Library | Status | No | 4 | `Identity`, `Health`, `Availability`, `Diagnostics` |
| Library | Status.Logs | Yes | 2 | `List`, `Watch` |
| Library | Status.TraceEvents | Yes | 2 | `List`, `Watch` |
| Library | Storage | No | 8 | `CreateFileProduction`, `GetFileProduction`, `CreateFileWriter`, `AppendFileBytes`, `DeleteFileProduction`, `PublishFileProduction`, `GetFileBytes`, `WatchFileBytes` |
| Product | Backend | No | 1 | `Appearance` |
| Product | Backend.Overview | Yes | 2 | `Get`, `Watch` |
| Product | Catalog | No | 1 | `Analyses` |
| Product | Catalog.Suites | Yes | 2 | `Get`, `List` |
| Product | Catalog.RecordSchemas | Yes | 2 | `Get`, `List` |
| Product | Catalog.StreamSchemas | Yes | 2 | `Get`, `List` |
| Product | Catalog.PromptDefinitions | Yes | 2 | `Get`, `List` |
| Product | Cases | No | 6 | `Create`, `Get`, `List`, `Watch`, `Delete`, `Parameters` |
| Product | Cycles | No | 6 | `Get`, `List`, `Watch`, `Create`, `Set`, `Seal` |
| Product | Executions | No | 11 | `Get`, `List`, `Watch`, `Create`, `Set`, `Start`, `Stop`, `Cancel`, `Delete`, `Parameters`, `Steps` |
| Product | Executions.Commands | Yes | 2 | `Get`, `Create` |
| Product | Executions.Progress | Yes | 2 | `Get`, `Watch` |
| Product | Evidence | No | 2 | `Statistics`, `Archive` |
| Product | Evidence.Records | Yes | 2 | `List`, `Watch` |
| Product | Evidence.Traceability | Yes | 2 | `Get`, `Watch` |
| Product | Evidence.Notes | Yes | 5 | `Get`, `List`, `Watch`, `Create`, `Set` |
| Product | Evidence.HealthEvents | Yes | 2 | `List`, `Watch` |
| Product | Evidence.Samples | Yes | 2 | `List`, `Watch` |
| Product | Content.Artifacts | Yes | 4 | `Get`, `Create`, `List`, `Watch` |
| Product | Content.ExecutionFiles | Yes | 3 | `Get`, `List`, `Watch` |
| Product | Documents | No | 2 | `Get`, `Watch` |
| Product | Plots.Snapshots | Yes | 2 | `List`, `Watch` |
| Product | Plots.Settings | Yes | 2 | `Get`, `Set` |
| Product | Prompts | No | 3 | `Get`, `List`, `Watch` |
| Product | Prompts.Answers | Yes | 1 | `Create` |
| Product | Settings | No | 3 | `Get`, `Set`, `Schema` |
| Product | Plugins.Settings | Yes | 2 | `Get`, `Set` |
| Product | Storage.Sources | Yes | 2 | `Get`, `Set` |
| Product | Secrets | No | 7 | `List`, `Watch`, `Get`, `Set`, `Delete`, `Export`, `Import` |
| Product | Reports | No | 3 | `Get`, `Create`, `Publish` |
| Product | Media.Captures | Yes | 4 | `Create`, `Get`, `Watch`, `Stop` |
| Product | Aggregates | No | 1 | `Get` |

The 132 ordinary mapping rows classify as 7 kept, 52 renamed, 71 merged
and 2 duplicate gRPC methods removed. All 132 Run twins are additionally removed.
Compared with 264 current gRPC methods, the composed mapped set has 120 methods,
144 fewer, and 240 method wrappers. The product declares 32 services (20 nested)
and 93 client methods; it does not redeclare the library destinations.

The mapping reaches 102 distinct destinations; 18 companions complete coverage: `Cases.Watch`, `Content.Artifacts.Watch`, `Content.ExecutionFiles.Watch`, `Cycles.Watch`, `Documents.Watch`, `Evidence.HealthEvents.Watch`, `Evidence.Notes.Get`, `Evidence.Notes.Watch`, `Evidence.Traceability.Watch`, `Executions.Progress.Get`, `Executions.Watch`, `Media.Captures.Get`, `Plots.Settings.Get`, `Plugins.Watch`, `Prompts.Watch`, `Secrets.Watch`, `Status.Logs.List`, `Status.TraceEvents.List`.

Library-owned client destinations in this mapping total 27: Plugins (7), access
(4), host status (8) and existing Storage transfer/session/byte methods (8).
The six session operations include CreateFileWriter for fenced resume. Identified
byte follow maps to Storage.WatchFileBytes; Content.ExecutionFiles.Watch follows
product metadata/membership and is not another byte-stream implementation.
All product artifact/file metadata stays under Content nested resources. Settings
owns application Get/Set and the noun-only Schema; Plugins.Settings, Plots.Settings
and Storage.Sources are revisioned nested resources. Prompts.Answers.Create allocates
an answer resource. Progress.Set is host-only; Get/Watch and Executions.Get carry
client observation. Cases.Parameters and Executions.Parameters/Steps stay paged
noun-only reads without automatic watch companions.

Each present method has exactly one mapping row. A `+` cell is one required
workflow, not alternative destinations. The owner column and service inventory
identify each destination owner, including mixed-owner workflows. CreateFileWriter
is the resume branch and DeleteFileProduction the abandon branch, not mandatory
calls before PublishFileProduction. No aggregate compatibility RPC is introduced.

Evidence.Archive selects/assembles product evidence at a cut and returns a bounded
immutable content reference, then library Storage.GetFileBytes serves ranges.
Archive layout, scope and evidence semantics stay product-owned operations over
storage with no manager. Diagnostics similarly return bounded archive references.
This product-specific selection cannot be delegated to generic byte reads.

### Complete Owner Totals, Including Host Operations

The library's locked sibling inventory has 13 logical services: Plugin, CatalogEntries,
Invocations, Time, Samples, Storage, Plugins, Access, Access.Listener, Status,
Status.Logs, Status.TraceEvents and HostValues. Eleven are exposed as gRPC services;
Samples and HostValues are host/native-only. These are target counts from its
own change, not implementation proof or extra services to implement in this repository.

| Owner / Boundary | Logical Services | Nested | Operations | gRPC Methods | Requests | Responses |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Library, Whole Locked Inventory | 13 | 3 | 67 | 61 | 67 | 67 |
| Library, Mapped Client Subset Above | 7 | 3 | 27 | 27 | 27 | 27 |
| Product, Client Subset | 32 | 20 | 93 | 93 | 93 | 93 |
| Product, Client and Host/Plugin Union | 39 | 22 | 111 | 93 | 111 | 111 |

The product union adds the 18 host/plugin-only templates listed below, shared
client methods counted once. The library union includes its Time.Get/Sleep and
HostValues.Get, not product settings/secrets stores. Six library operations are
native/host-only: Samples.Append, Storage.Info, Storage.SetSchema/GetSchema,
Storage.DeleteFile and HostValues.Get. Library wrappers total 134 (122 exposed,
12 native/host-only); product wrappers total 222 (186 client, 36 host/plugin-only).
Neither wrapper census includes domain payloads, shared primitives or carrier
control/event frames; field-level descriptor review must establish those totals.
A managed-settlement replacement changes no operation/method count.

## Plugin-Facing Mapping

These operations are logically unary over the duplex library invocation carrier;
publication has its own carrier. Counts are separate from client gRPC methods and do
not multiply dynamic step/criterion instances or count protocol control frames.

| Present Operation or Family | Locked Operation | Destination Owner | Disposition and Preserved Behavior |
| --- | --- | --- | --- |
| `<template>.materialize` | TestCases.Materialize | Product | Renamed; pure materialization, request/response wrapper. |
| `<template>.initialize` | Executions.Initialize | Product | Renamed; acquire execution-local suite resources. |
| `Step.operation` | Steps.Execute | Product | Renamed family; frozen step identity and retry evidence. |
| `Criterion.operation` | Criteria.Evaluate | Product | Renamed family; captured-evidence judgment. |
| `<template>.cleanup` | Executions.Cleanup | Product | Renamed; required cleanup after initialization. |
| `dokime.execution.prompt` | Prompts.Create | Product | Renamed; wait for answer, preserve scope and cancellation. |
| `dokime.settings.read` | HostValues.Get | Library | Generic declared-value delivery; product resolver owns caller-bound settings storage and policy. |
| `dokime.secrets.read` | HostValues.Get | Library | Generic declared-value delivery; product resolver owns granted-identifier secret storage and policy. |
| `harmos.clock.sleep` | Time.Sleep | Library | Kept meaning; normalized carriers, library-owned clock. |
| `harmos.clock.now` | Time.Get | Library | Renamed; library-owned clock. |
| `dokime.execution.record` | Evidence.Records.Append | Product | Renamed; ordered immutable data, durable receipt. |
| `dokime.execution.records` | Evidence.Records.List | Product | Merged; preserve count/last selection and bounded reads. |
| `dokime.execution.statistics` | Evidence.Statistics | Product | Merged; stream/field/step selectors and complete statistics. |
| `dokime.execution.artifact` | Storage.CreateFileProduction + Storage.GetFileProduction + Storage.CreateFileWriter + Storage.AppendFileBytes + Storage.DeleteFileProduction + Storage.PublishFileProduction + Content.Artifacts.Create | Library + Product | Merged; same unary sessions as clients, acknowledged bytes. |
| `dokime.execution.progress` | Executions.Progress.Set | Product | Renamed; host-only producer operation, whole small progress value and Executions.Progress.Get revision; no client Set. |
| `dokime.execution.log` | Evidence.Logs.Append | Product | Renamed; ordered immutable log lines. |
| `dokime.execution.traceability` | Evidence.Traceability.Set | Product | Renamed; whole small writable value, Evidence.Traceability.Get revision and scope; large writable parts split into separately revisioned entry Get/Set sub-resources. |
| `dokime.execution.document` | Documents.AppendParts | Product | Renamed; ordered immutable document contributions. |
| `dokime.files.copy_files` | Files.Capture | Product | Renamed; freeze file copies, preserve originals. |
| `dokime.files.start_capture` | Files.Captures.Start | Product | Renamed; start capture lifecycle. |
| `dokime.files.stop_capture` | Files.Captures.Stop | Product | Renamed; finish capture at safe boundary. |
| `dokime.execution.environment` | Backend.Environment | Product | Renamed; derived environment read. |
| `dokime.execution.effect.prepare` | Effects.Create | Product | Renamed; durable inverse ticket, no hardware write. |
| `dokime.execution.effect.applied` | Effects.Set | Product | Renamed; whole writable application state and Effects.Get revision; preserve earliest inverse. |

All 17 product context bindings plus two library time operations and five dynamic
lifecycle families are covered: 24 present rows. Settings/secrets host calls map
to library HostValues.Get with a product resolver, not library stores. The shared
upload workflow adds no product transfer engine; its library methods are counted
once. Effects.Get is an added companion, not a 25th present row. Progress.Set is
host-only, called by a plugin, with Get/revision symmetry and scoped authority;
there is no client Set. Large writable settings/traceability values split by
identified value rather than replace an unbounded aggregate.

| Product Host/Plugin-Only Namespace | Additional Methods | Count | New Logical Service |
| --- | --- | ---: | --- |
| TestCases | Materialize | 1 | Yes |
| Executions | Initialize, Cleanup | 2 | No |
| Steps | Execute | 1 | Yes |
| Criteria | Evaluate | 1 | Yes |
| Prompts | Create | 1 | No |
| Evidence.Records | Append | 1 | No |
| Executions.Progress | Set | 1 | No |
| Evidence.Logs | Append | 1 | Yes, nested |
| Evidence.Traceability | Set | 1 | No |
| Documents | AppendParts | 1 | No |
| Files | Capture | 1 | Yes |
| Files.Captures | Start, Stop | 2 | Yes, nested |
| Backend | Environment | 1 | No |
| Effects | Create, Set, Get | 3 | Yes |

These are 18 operations, 18 requests and 18 responses with zero added client RPCs,
adding seven logical service namespaces, two nested. Shared client operations are
counted once in the 111-operation product union. Dynamic step/criterion instances
and protocol control frames do not multiply method templates. The clock and generic
host-value operations belong only to the library inventory.

## Source Verification and Report Corrections

Verified on the current product tree and by read-only inspection of the library
checkout named by its report:

- `proto/dokime/v1/client_*.proto`: 23 services, 132 ordinary methods and 132 Run
  twins. Counting `message` declarations, including nested declarations, across
  `proto/dokime/v1/*.proto` gives 953. This confirms the reports' correction of
  131/262. PublishExecutionReport is present. Remove 264 twin-envelope messages;
  the final payload-message total remains unknown; neither subtraction nor the
  240 method messages establishes it.
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
defaults visibility to Public while the other SDKs use Private; C# lacks an
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

1. **SDK coverage:** settle plugin parity across Rust, Python, C# and C++, including
   visibility and C# resource/cancellation parity, and generic managed-client
   coverage in all four languages versus Rust only. Library ownership is locked.
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

For implementation, first prepare library services under their managers, narrow
host-fact registration and descriptor-derived four-language client roots. Publish
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
