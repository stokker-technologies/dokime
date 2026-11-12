# Operation Grammar Design

Status: locked by the owner on 2026-11-12
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
The library owns generic primitives; product lifecycle and evidence selection
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
  singular noun for one thing such as a plugin's own service. Methods acting on
  that resource are bare verbs; never repeat the service's resource noun.
  A part read in exactly one way uses its noun alone, whether streamed, paged or
  returned whole. A part with multiple read or change paths uses verb plus part;
  prefer a small resource service with bare verbs where this makes an exact
  Get/Set pair clearer. Mixed-resource services retain the affected resource
  nouns, without repeating the service's own name; do not split them merely to
  shorten methods. Messages use `<Service><Method>Request` and
  `<Service><Method>Response`, for example `ExecutionsStartRequest`, so repeated
  bare methods remain unique in the service namespace. Replies are always
  wrappers, never bare resources. Operation IDs derive mechanically from the
  owning service and method with no second authored alias. Generated clients
  take their names directly from the proto, giving short names in every language
  without a hand-written layer.
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
  at an accepted offset conflict. Download by repeated Content.Get calls with
  offset and maximum bytes; each response states total length and content digest.
  Configuration bounds chunk and range sizes. Measure large-file throughput in a
  Nightly benchmark case before deleting streaming transfers.
- **Streaming (Owner):** never generate twins automatically. Server streaming is
  reserved for Watch semantics (including noun-only streamed parts); the plugin protocol retains its own carriers. A managed
  stream exists only for an actual cooperative final-result promise after
  cancellation, replacing the ordinary gRPC method. Native gRPC cancellation abandons
  observation and cannot promise that final response; admitted durable work
  survives observer loss. Downloads, documents and diagnostics use bounded unary
  replies, with content bytes read through ranged Content.Get.

### Generated Client Names

For a resource whose progress part is read only as a stream, the naming rule gives
these generated calls directly, in every language's normal method casing:

```python
executions.start(request)
executions.cancel(request)
executions.get(request)
executions.progress(request)
```

In this product's preserved contract, progress also has a whole-value Get/Set
revision pair for plugin writes, so it is a genuine multiple-path part under rule 3:
`Executions.GetProgress`, `Executions.SetProgress` and `Executions.WatchProgress`.
Its actual generated stream call is `executions.watch_progress(request)`; the
stream-only `progress` illustration above is not a second method or client alias.
Keeping these paths together avoids introducing a separate progress resource solely
to shorten names. The Get is shared with clients and remains the exact writable
value/revision mirror for plugin Set. By contrast, parameters and steps have no
independent change path, so their sole reads are paged noun-only methods.

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

Library source confirms `crates/harmos-storage/src/config.rs::FLUSH_INTERVAL` is
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

**Owner:** plugin publication becomes Evidence.AppendRecords, not CreateRecord. Its
acknowledgement retains the existing promised stored-record semantics; the receipt
must state the durable position when that promise completes. Records and samples
retain produced/captured ownership, source time and schema identity.

```proto
message EvidenceListRecordsRequest {
  string parent = 1; Position at = 2; RecordFilter filter = 3; PageRequest page = 4;
}
message EvidenceListRecordsResponse { repeated Record records = 1; PageInfo page = 2; }
message EvidenceWatchRecordsRequest {
  string parent = 1; Position at = 2; RecordFilter filter = 3; PageRequest page = 4;
}
message EvidenceWatchRecordsResponse {
  Position resolved_at = 1;
  oneof event {
    RecordSnapshot snapshot = 2;
    Record appended = 3;
    ResyncRequired resync_required = 4;
    Position progress = 5;
  }
}
message EvidenceAppendRecordsRequest {
  string parent = 1; repeated Record records = 2; string request_id = 3;
}
message EvidenceAppendRecordsResponse { Receipt receipt = 1; }
service Evidence {
  rpc ListRecords(EvidenceListRecordsRequest) returns (EvidenceListRecordsResponse);
  rpc WatchRecords(EvidenceWatchRecordsRequest) returns (stream EvidenceWatchRecordsResponse);
  rpc AppendRecords(EvidenceAppendRecordsRequest) returns (EvidenceAppendRecordsResponse);
}
```

### Representative Resource 3: Transfers

**Owner:** Content.CreateUpload, Content.GetUpload, Content.AppendUploadBytes, Content.DeleteUpload and
Content.PublishUpload are unary for every caller. Content.AppendUploadBytes appends ordered bytes
at the stated next offset. Replaying an identical accepted chunk succeeds;
different bytes at an accepted offset conflict; gaps and out-of-order new chunks
are refused. Content.GetUpload reports the accepted offset, length, digest and session
state to resume. Content.PublishUpload verifies complete bytes, length and digest before
exposing immutable content. Content.CreateArtifact or Plugins.Create then references it.
Sessions are caller-bound, bounded and expire explicitly; expiry cannot silently
start another upload. Content.DeleteUpload abandons it. Transport loss never publishes
partial content. Chunk and range maxima are configuration-owned limits.

Content.Get returns a bounded range, repeated by the caller to resume downloads.
The content identity and digest bind every range to the same representation,
including a scoped evidence archive at a returned cut. There is no separate
streaming archive download gRPC method. Measure large-file upload/download throughput
in the Nightly benchmark set before deleting the superseded streaming transfers;
throughput is a benchmark measurement, never a correctness-test assertion.

```proto
message ContentCreateUploadRequest {
  string parent = 1; uint64 size_bytes = 2; bytes sha256 = 3; string request_id = 4;
}
message ContentCreateUploadResponse { Upload upload = 1; Receipt receipt = 2; }
message ContentGetUploadRequest { string name = 1; Position at = 2; }
message ContentGetUploadResponse { Upload upload = 1; Position resolved_at = 2; }
message ContentAppendUploadBytesRequest {
  string name = 1; uint64 offset = 2; bytes data = 3; string request_id = 4;
}
message ContentAppendUploadBytesResponse { Upload upload = 1; Receipt receipt = 2; }
message ContentPublishUploadRequest { string name = 1; string request_id = 2; }
message ContentPublishUploadResponse { PublishedContent content = 1; Receipt receipt = 2; }
message ContentDeleteUploadRequest { string name = 1; string request_id = 2; }
message ContentDeleteUploadResponse { Receipt receipt = 1; }
message ContentGetRequest {
  string name = 1; Position at = 2; uint64 offset = 3; uint64 max_bytes = 4;
}
message ContentGetResponse {
  bytes data = 1; uint64 offset = 2; uint64 total_length = 3;
  bytes sha256 = 4; Position resolved_at = 5;
}
service Content {
  rpc CreateUpload(ContentCreateUploadRequest) returns (ContentCreateUploadResponse);
  rpc GetUpload(ContentGetUploadRequest) returns (ContentGetUploadResponse);
  rpc AppendUploadBytes(ContentAppendUploadBytesRequest) returns (ContentAppendUploadBytesResponse);
  rpc PublishUpload(ContentPublishUploadRequest) returns (ContentPublishUploadResponse);
  rpc DeleteUpload(ContentDeleteUploadRequest) returns (ContentDeleteUploadResponse);
  rpc Get(ContentGetRequest) returns (ContentGetResponse);
}
```

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
lifecycle resources; it is not Append of immutable evidence. Prompts.CreateAnswer
and Executions.CreateCommand allocate identified lifecycle resources with retained
receipts. Executions.SetProgress replaces a current value; Evidence.AppendLogs and
Documents.AppendParts add ordered immutable evidence.

Documents.Get retains evaluated output, budgets, repeat instances and failures;
it does not publish a report. Reports.Create renders a preview at a fixed cut,
Reports.Get reads an existing published report, Reports.Publish commits a selected
report and seal. Content.Get delivers bytes; downloads never publish. Secret
Import/Export preserve current transfer protection and conflict checks; the
report's description of an encrypted transfer is corrected below.

## Complete Product Mapping

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

| Present Operation | Locked Destination | Disposition and Reason | Callers |
| --- | --- | --- | --- |
| `Appearance.GetAppearance` | `Backend.Appearance` | Merged: Move appearance ownership. | G |
| `Backend.CollectDiagnostics` | `Backend.Diagnostics` | Renamed: Read bounded diagnostic output. | G |
| `Backend.GetAvailability` | `Backend.Availability` | Renamed: Retain availability. | CS |
| `Backend.GetHealth` | `Backend.Health` | Merged: One health owner. | CS |
| `Backend.GetIdentity` | `Backend.Identity` | Merged: One identity owner. | GCS |
| `Backend.WatchLogs` | `Backend.WatchLogs` | Kept: Durable log watch. | T |
| `Backend.WatchTrace` | `Backend.WatchTraceEvents` | Renamed: Name trace event resource. | G |
| `Catalog.GetRecordSchema` | `Catalog.GetRecordSchema` | Merged: Catalog or frozen execution parent. | N |
| `Catalog.GetSuite` | `Catalog.GetSuite` | Kept: Retain suite definition. | GCS |
| `Catalog.GetTelemetrySchema` | `Catalog.GetStreamSchema` | Renamed: Use stream vocabulary. | N |
| `Catalog.ListRecordSchemas` | `Catalog.ListRecordSchemas` | Merged: Catalog or frozen execution parent. | N |
| `Catalog.ListSuites` | `Catalog.ListSuites` | Kept: Retain suite catalog. | GCS |
| `Catalog.ListTelemetrySchemas` | `Catalog.ListStreamSchemas` | Merged: Use stream vocabulary. | N |
| `Content.DownloadAppearanceAsset` | `Content.Get` | Merged: Bounded bytes by content identity. | G |
| `Content.DownloadArtifact` | `Content.GetArtifact` + `Content.Get` | Merged: Metadata then bounded bytes. | GC |
| `Content.DownloadEvidence` | `Content.Get` | Merged: Ranged archive content; preserve scope, cut and digest verification. | C |
| `Content.DownloadExecutionFile` | `Content.GetExecutionFile` + `Content.Get` | Merged: Metadata/range then bytes. | C |
| `Content.FollowExecutionFile` | `Content.WatchExecutionFile` | Renamed: Retain growth/replacement semantics. | G |
| `Content.ListArtifacts` | `Content.ListArtifacts` | Merged: Retain bounded artifact selection. | N |
| `Content.ListExecutionFiles` | `Content.ListExecutionFiles` | Merged: Retain bounded file selection. | G |
| `Content.ReadExecutionFile` | `Content.GetExecutionFile` | Merged: Same file resource, text/byte ranges. | G |
| `Content.UploadArtifact` | `Content.CreateUpload` + `Content.GetUpload` + `Content.AppendUploadBytes` + `Content.DeleteUpload` + `Content.PublishUpload` + `Content.CreateArtifact` | Merged: Shared unary resume lifecycle. | GC |
| `Documents.CaptureCurrentDocument` | `Documents.Get` | Merged: Evaluated output without publication. | G |
| `Documents.UpdatePlotSettings` | `PlotSettings.Set` | Renamed: Whole small writable plot value and expected revision from PlotSettings.Get. | G |
| `Evidence.GetCapturedRecordSchema` | `Catalog.GetRecordSchema` | Merged: Execution parent selects frozen schema. | N |
| `Evidence.ListRecords` | `Evidence.ListRecords` | Merged: One selector/cut contract. | GC |
| `Evidence.WatchRecords` | `Evidence.WatchRecords` | Kept: Snapshot then durable changes. | GC |
| `Execution.AppendExecutions` | `Executions.Create` | Renamed: Allocate mutable execution resources. | GC |
| `Execution.CancelExecution` | `Executions.Cancel` | Renamed: Running only, queued refusal names action. | G |
| `Execution.CreateCycle` | `Cycles.Create` | Merged: Preserve new-cycle admission. | GCS |
| `Execution.DeleteCase` | `Cases.Delete` | Renamed: Preserve lifecycle admission. | G |
| `Execution.GetCase` | `Cases.Get` | Renamed: Retain planned case. | G |
| `Execution.GetCycle` | `Cycles.Get` | Merged: One operational/saved read. | GC |
| `Execution.GetExecution` | `Executions.Get` | Merged: One operational/saved read with seal. | GCS |
| `Execution.GetExecutionCommand` | `Executions.GetCommand` | Renamed: Retained command receipt. | C |
| `Execution.ListCaseParameters` | `Cases.Parameters` | Renamed: Pinned definition values. | G |
| `Execution.ListCases` | `Cases.List` | Renamed: Retain ordered cases. | G |
| `Execution.ListCycles` | `Cycles.List` | Merged: Operational/query/history selection. | G |
| `Execution.ListExecutionParameters` | `Executions.Parameters` | Renamed: Frozen execution values. | G |
| `Execution.ListExecutionSteps` | `Executions.Steps` | Renamed: Frozen steps and state. | G |
| `Execution.ListExecutions` | `Executions.List` | Merged: Operational/query/history selection. | GC |
| `Execution.PauseCycle` | `Cycles.Set` | Merged: Whole small writable value and Get revision; preserve pause admission. | S |
| `Execution.PauseExecution` | `Executions.Set` | Merged: Whole small writable value and Get revision; preserve pause admission. | GC |
| `Execution.RenameCycle` | `Cycles.Set` | Merged: Whole small writable value and Get revision, including label. | G |
| `Execution.RenameExecution` | `Executions.Set` | Merged: Whole small writable value and Get revision; rename only this execution. | G |
| `Execution.RerunCycle` | `Cycles.Create` | Merged: source_cycle preserves rerun notices. | GC |
| `Execution.ResumeCycle` | `Cycles.Set` | Merged: Whole small writable value and Get revision, with paused=false. | S |
| `Execution.ResumeExecution` | `Executions.Set` | Merged: Whole small writable value and Get revision, with paused=false. | GC |
| `Execution.SealCycle` | `Cycles.Seal` | Renamed: Preserve durable finalization. | GC |
| `Execution.SetCycleArchived` | `Cycles.Set` | Merged: Whole small writable value and Get revision, including archived. | G |
| `Execution.SetExecutionArchived` | `Executions.Set` | Merged: Whole small writable value and Get revision, including archived. | G |
| `Execution.StartExecution` | `Executions.Start` | Renamed: Activate selected execution. | GC |
| `Execution.StopExecution` | `Executions.Stop` | Renamed: Stop scheduling at safe boundary. | GC |
| `Execution.SubmitExecutionCommand` | `Executions.CreateCommand` | Renamed: Interactive Next/RunStep/Finish, replay receipt. | C |
| `Execution.UpdateCyclePolicy` | `Cycles.Set` | Merged: Whole small writable cycle value and Get revision, including policy. | G |
| `Health.ListHealth` | `Evidence.ListHealthEvents` | Merged: Name evidence collection. | GS |
| `History.BrowseCycles` | `Cycles.List` | Merged: Retain saved filtering and ordering. | G |
| `History.BrowseExecutions` | `Executions.List` | Merged: Retain live/saved availability. | G |
| `History.DeleteExecution` | `Executions.Delete` | Renamed: Keep seal precondition and replay protection. | G |
| `History.DownloadCheckpointArtifact` | `Content.GetArtifact` + `Content.Get` | Merged: Same reads at returned cut/seal. | G |
| `History.GetCheckpointContext` | `Executions.Get` + `Cycles.Get` + `Documents.Get` + `Prompts.List` + `Evidence.ListHealthEvents` + `Evidence.ListNotes` + `Evidence.GetTraceability` + `Backend.Appearance` | Merged: Position-matched frozen context, no checkpoint DTO. | GC |
| `History.GetCheckpointDocument` | `Documents.Get` | Merged: Same read at returned cut/seal. | GC |
| `History.GetCheckpointEvidence` | `Content.ListArtifacts` + `Content.ListExecutionFiles` + `Catalog.ListRecordSchemas` + `Catalog.ListStreamSchemas` | Merged: One cut, preserve frozen schemas/availability. | G |
| `History.GetExecutionCheckpoint` | `Executions.Get` | Merged: Return position and seal in ordinary read. | C |
| `History.ListCheckpointRecords` | `Evidence.ListRecords` | Merged: Same selector at returned cut/seal. | G |
| `History.ListCheckpointTelemetry` | `Evidence.ListSamples` | Merged: Same selector at returned cut/seal. | G |
| `History.ReadCheckpointOutput` | `Content.GetExecutionFile` | Merged: Same bounded file read at cut/seal. | G |
| `Media.AudioRecording` | `MediaCaptures.Create` + `MediaCaptures.Watch` + `MediaCaptures.Stop` | Merged: Owned recording lifecycle, retained result. | G |
| `Media.CameraRecording` | `MediaCaptures.Create` + `MediaCaptures.Watch` + `MediaCaptures.Stop` | Merged: Keep explicit unsupported-kind refusal. | N |
| `Media.Photo` | `MediaCaptures.Create` | Merged: Typed photo source. | G |
| `Media.ScreenRecording` | `MediaCaptures.Create` + `MediaCaptures.Watch` + `MediaCaptures.Stop` | Merged: Keep explicit unsupported-kind refusal. | N |
| `Media.Screenshot` | `MediaCaptures.Create` | Merged: Typed screenshot source. | G |
| `Notes.CreateNote` | `Evidence.CreateNote` | Kept: Allocate identified note. | G |
| `Notes.EditNote` | `Evidence.SetNote` | Renamed: Whole note writable value and revision from GetNote. | G |
| `Notes.ListNotes` | `Evidence.ListNotes` | Merged: Retain note collection. | G |
| `Observation.GetRuntimeShell` | `Backend.GetOverview` | Renamed: Runtime overview resource. | G |
| `Observation.GetTraceability` | `Evidence.GetTraceability` | Merged: Retain evidence context. | G |
| `Observation.WatchExecutionProgress` | `Executions.WatchProgress` | Renamed: Paired progress read/watch. | GS |
| `Observation.WatchRuntime` | `Backend.WatchOverview` | Renamed: Same overview selection. | G |
| `Planning.MaterializeTemplate` | `Cases.Create` | Merged: Single materialized selection. | GCS |
| `Planning.PreviewSelection` | `Cases.Create` | Merged: validate_only, no registration. | GC |
| `Planning.RegisterSelection` | `Cases.Create` | Merged: Atomic plan with replay key. | C |
| `Plots.ListAnalyses` | `Catalog.Analyses` | Renamed: Move immutable declarations. | N |
| `Plots.QuerySnapshots` | `Plots.ListSnapshots` | Renamed: Bounded evaluated snapshots. | G |
| `Plots.Watch` | `Plots.WatchSnapshots` | Renamed: Same plot selector/cut. | G |
| `Plugins.GetInstallation` | `Plugins.Get` | Renamed: One managed/remote entry. | G |
| `Plugins.Install` | `Content.CreateUpload` + `Content.GetUpload` + `Content.AppendUploadBytes` + `Content.DeleteUpload` + `Content.PublishUpload` + `Plugins.Create` | Merged: Shared upload then managed admission. | GC |
| `Plugins.ListAdmittedPlugins` | `Plugins.List` | Merged: state=READY filter. | G |
| `Plugins.ListInstallations` | `Plugins.List` | Merged: One entry selection. | G |
| `Plugins.Reload` | `Plugins.Restart` | Renamed: Replace activation, retain finalization rules. | GS |
| `Plugins.Remove` | `Plugins.Delete` | Renamed: Remove entry, migrate old payload. | G |
| `Plugins.SetEnabled` | `Plugins.Set` | Renamed: Whole small plugin writable value and Plugins.Get revision, including enabled. | G |
| `Plugins.List` | `Plugins.List` | Merged: One managed/remote entry list. | S |
| `Plugins.Add` | `Plugins.Create` | Merged: Remote placement admission. | S |
| `Prompts.GetDefinition` | `Catalog.GetPromptDefinition` | Renamed: Separate definition from asked prompt. | S |
| `Prompts.GetPrompt` | `Prompts.Get` | Renamed: Retain asked prompt state. | GCS |
| `Prompts.ListDefinitions` | `Catalog.ListPromptDefinitions` | Renamed: Immutable definitions. | S |
| `Prompts.ListPending` | `Prompts.List` | Merged: state=PENDING filter. | GCS |
| `Prompts.Respond` | `Prompts.CreateAnswer` | Renamed: One answer resource and replay receipt. | GCS |
| `Query.Aggregate` | `Aggregates.Get` | Renamed: Complete read-only aggregate. | T |
| `Query.AggregateSamples` | `Evidence.Statistics` | Renamed: Retain stream statistics selectors. | T |
| `Query.ListCycles` | `Cycles.List` | Merged: Preserve projection/filter semantics. | T |
| `Query.ListExecutions` | `Executions.List` | Merged: Preserve projection/filter semantics. | TS |
| `Query.ListRecords` | `Evidence.ListRecords` | Merged: One canonical evidence read. | T |
| `Query.ListSamples` | `Evidence.ListSamples` | Merged: One canonical sample read. | T |
| `Remote.CreatePairing` | `Access.CreatePairing` | Kept: Preserve access material lifecycle. | G |
| `Remote.ReadListener` | `Access.GetListener` | Renamed: Derived listener state. | G |
| `Remote.RegenerateCertificate` | `Access.CreateCertificate` | Renamed: Allocate replacement certificate. | G |
| `Remote.UpdateListener` | `Access.SetListener` | Renamed: Whole listener writable settings and revision from GetListener. | G |
| `Reports.PublishExecutionReport` | `Reports.Publish` | Renamed: Explicit durable publication, never implicit download. | S |
| `Reports.DownloadCycleReport` | `Reports.Get` + `Content.Get` + `Reports.Create` | Merged: Published bytes or explicit preview creation. | GCS |
| `Reports.DownloadExecutionReport` | `Reports.Get` + `Content.Get` + `Reports.Create` | Merged: Published bytes or explicit preview creation. | GCS |
| `Reports.MaterializeReport` | `Reports.Create` | Merged: Render only, no publication. | N |
| `Runtime.GetHealth` | `Backend.Health` | Removed: Remove duplicate health gRPC method. | C |
| `Runtime.GetIdentity` | `Backend.Identity` | Removed: Remove duplicate identity gRPC method. | C |
| `Secrets.Export` | `Secrets.Export` | Renamed: Existing Base64 JSON transfer, not encryption. | G |
| `Secrets.Import` | `Secrets.Import` | Merged: Preserve choices and generation conflict. | G |
| `Secrets.List` | `Secrets.List` | Renamed: Bounded metadata, not secret values. | G |
| `Secrets.Lookup` | `Secrets.Get` | Renamed: Granted identifier only. | N |
| `Secrets.Preview` | `Secrets.Import` | Merged: validate_only, no writes. | G |
| `Secrets.Put` | `Secrets.Set` | Renamed: Whole writable secret value and revision from Secrets.Get. | G |
| `Secrets.Remove` | `Secrets.Delete` | Renamed: Preserve keyring/catalog cleanup. | G |
| `Settings.GetPluginSettings` | `PluginSettings.Get` | Renamed: Saved values, not UI declarations. | G |
| `Settings.GetSettings` | `SettingsSchema.Get` | Renamed: UI declarations/cards/bootstrap metadata. | G |
| `Settings.ListStorageSources` | `StorageSources.Get` | Renamed: One ordered value. | G |
| `Settings.ReadApplicationSettings` | `ApplicationSettings.Get` | Renamed: Revisioned application values. | G |
| `Settings.ReorderStorageSources` | `StorageSources.Set` | Merged: Whole small ordered value and revision from StorageSources.Get. | G |
| `Settings.SetStorageSourceEnabled` | `StorageSources.Set` | Merged: Whole small ordered value and Get revision, including enabled state. | G |
| `Settings.UpdateApplicationSettings` | `ApplicationSettings.Set` | Renamed: Whole writable value and matching Get revision. | G |
| `Settings.UpdatePluginSettings` | `PluginSettings.Set` | Renamed: Whole writable value and matching Get revision; if large, use separately revisioned setting sub-resources with PluginSettings.Get/PluginSettings.Set. | G |
| `Telemetry.ListSamples` | `Evidence.ListSamples` | Merged: One canonical sample read. | CS |
| `Telemetry.WatchSamples` | `Evidence.WatchSamples` | Kept: Same source selector and durable boundary. | C |

## Locked Client Inventory and Counts

| Locked Service | Count | Operations |
| --- | ---: | --- |
| Backend | 11 | `Identity`, `Health`, `Availability`, `Appearance`, `GetOverview`, `WatchOverview`, `ListLogs`, `WatchLogs`, `ListTraceEvents`, `WatchTraceEvents`, `Diagnostics` |
| Catalog | 9 | `GetSuite`, `ListSuites`, `GetRecordSchema`, `ListRecordSchemas`, `GetStreamSchema`, `ListStreamSchemas`, `GetPromptDefinition`, `ListPromptDefinitions`, `Analyses` |
| Plugins | 7 | `Get`, `List`, `Watch`, `Create`, `Set`, `Delete`, `Restart` |
| Cases | 6 | `Create`, `Get`, `List`, `Watch`, `Delete`, `Parameters` |
| Cycles | 6 | `Get`, `List`, `Watch`, `Create`, `Set`, `Seal` |
| Executions | 15 | `Get`, `List`, `Watch`, `Create`, `Set`, `Start`, `Stop`, `Cancel`, `Delete`, `Parameters`, `Steps`, `GetCommand`, `CreateCommand`, `GetProgress`, `WatchProgress` |
| Evidence | 14 | `ListRecords`, `WatchRecords`, `Statistics`, `GetTraceability`, `WatchTraceability`, `ListNotes`, `WatchNotes`, `GetNote`, `CreateNote`, `SetNote`, `ListHealthEvents`, `WatchHealthEvents`, `ListSamples`, `WatchSamples` |
| Content | 14 | `CreateUpload`, `GetUpload`, `AppendUploadBytes`, `DeleteUpload`, `PublishUpload`, `Get`, `GetArtifact`, `CreateArtifact`, `ListArtifacts`, `WatchArtifacts`, `GetExecutionFile`, `ListExecutionFiles`, `WatchExecutionFiles`, `WatchExecutionFile` |
| Documents | 2 | `Get`, `Watch` |
| Plots | 2 | `ListSnapshots`, `WatchSnapshots` |
| PlotSettings | 2 | `Get`, `Set` |
| Prompts | 4 | `Get`, `List`, `Watch`, `CreateAnswer` |
| SettingsSchema | 1 | `Get` |
| ApplicationSettings | 2 | `Get`, `Set` |
| PluginSettings | 2 | `Get`, `Set` |
| StorageSources | 2 | `Get`, `Set` |
| Access | 4 | `GetListener`, `SetListener`, `CreatePairing`, `CreateCertificate` |
| Secrets | 7 | `List`, `Watch`, `Get`, `Set`, `Delete`, `Export`, `Import` |
| Reports | 3 | `Get`, `Create`, `Publish` |
| MediaCaptures | 4 | `Create`, `Get`, `Watch`, `Stop` |
| Aggregates | 1 | `Get` |

The 132 ordinary mapping rows classify as 7 kept, 52 renamed,
71 merged and 2 duplicate gRPC methods removed. All 132 Run gRPC methods are
additionally removed. The locked client set has 21 services and 118 operations/gRPC methods,
compared with 23 services and 264 gRPC methods today: 146 fewer gRPC methods.
There are 118 request messages and 118 response messages: 236 method messages.
This is a locked target inventory, not a claim that every method is already served.

The mapping reaches 100 distinct destination methods; 18 added companions complete
read/watch/resource coverage: `Backend.ListLogs`, `Backend.ListTraceEvents`, `Cases.Watch`, `Content.WatchArtifacts`, `Content.WatchExecutionFiles`, `Cycles.Watch`, `Documents.Watch`, `Evidence.GetNote`, `Evidence.WatchHealthEvents`, `Evidence.WatchNotes`, `Evidence.WatchTraceability`, `Executions.GetProgress`, `Executions.Watch`, `MediaCaptures.Get`, `PlotSettings.Get`, `Plugins.Watch`, `Prompts.Watch`, `Secrets.Watch`.
The previous automatic parameter and step watch companions are removed:
`Executions.Parameters` and `Executions.Steps` are each one paged read, and
`Cases.Parameters` is likewise paged. This changes no present-operation mapping.

Regrouping gives PlotSettings, ApplicationSettings, PluginSettings and StorageSources
exact bare Get/Set pairs. SettingsSchema owns the one whole schema read; these four
settings services replace Settings. Media becomes MediaCaptures and Queries becomes
Aggregates to name their actual resources. Backend, Catalog, Evidence, Content and
Access keep their mixed-resource ownership; Content.Get reads content ranges, while
its upload, artifact and execution-file methods retain the distinguishing nouns.
Plugin lifecycle families now live under TestCases, Executions, Steps and Criteria;
Files retains its capture actions, and Effects uses bare Create/Get/Set.

Each present method has exactly one canonical mapping row. A cell with `+` is one
required resource workflow, whose constituent method destinations are listed in
order, not alternative mappings. Transfers, checkpoint context and report download
retain these workflows rather than inventing aggregate RPCs or compatibility layers.

Removing 264 twin envelopes does not determine the final payload-message total:
rendered graphs, authoring definitions and unreachable payloads require field-level
review. All content downloads are bounded unary Content.Get ranges; document and
diagnostic output is bounded unary. A proven managed-settlement exception replaces
its ordinary gRPC method, retaining the same operation and method count.

## Plugin-Facing Mapping

These operations are logically unary over the duplex library invocation carrier;
publication has its own carrier. Counts are separate from client gRPC methods and do
not multiply dynamic step/criterion instances or count protocol control frames.

| Present Operation or Family | Locked Operation | Disposition and Preserved Behavior |
| --- | --- | --- |
| `<template>.materialize` | TestCases.Materialize | Renamed; pure materialization, request/response wrapper. |
| `<template>.initialize` | Executions.Initialize | Renamed; acquire execution-local suite resources. |
| `Step.operation` | Steps.Execute | Renamed family; frozen step identity and retry evidence. |
| `Criterion.operation` | Criteria.Evaluate | Renamed family; captured-evidence judgment. |
| `<template>.cleanup` | Executions.Cleanup | Renamed; required cleanup after initialization. |
| `dokime.execution.prompt` | Prompts.Create | Renamed; wait for answer, preserve scope and cancellation. |
| `dokime.settings.read` | PluginSettings.Get | Merged with caller-bound resource read. |
| `dokime.secrets.read` | Secrets.Get | Merged with granted-identifier read. |
| `harmos.clock.sleep` | Clock.Sleep | Kept meaning; normalized carriers, library-owned clock. |
| `harmos.clock.now` | Clock.Time | Renamed; library-owned clock. |
| `dokime.execution.record` | Evidence.AppendRecords | Renamed; ordered immutable data, durable receipt. |
| `dokime.execution.records` | Evidence.ListRecords | Merged; preserve count/last selection and bounded reads. |
| `dokime.execution.statistics` | Evidence.Statistics | Merged; stream/field/step selectors and complete statistics. |
| `dokime.execution.artifact` | Content.CreateUpload + Content.GetUpload + Content.AppendUploadBytes + Content.DeleteUpload + Content.PublishUpload + Content.CreateArtifact | Merged; same unary sessions as clients, acknowledged bytes. |
| `dokime.execution.progress` | Executions.SetProgress | Renamed; whole small progress value and Executions.GetProgress revision. |
| `dokime.execution.log` | Evidence.AppendLogs | Renamed; ordered immutable log lines. |
| `dokime.execution.traceability` | Evidence.SetTraceability | Renamed; whole small writable value, Evidence.GetTraceability revision and scope; large writable parts split into separately revisioned entry Get/Set sub-resources. |
| `dokime.execution.document` | Documents.AppendParts | Renamed; ordered immutable document contributions. |
| `dokime.files.copy_files` | Files.Capture | Renamed; freeze file copies, preserve originals. |
| `dokime.files.start_capture` | Files.StartCapture | Renamed; start capture lifecycle. |
| `dokime.files.stop_capture` | Files.StopCapture | Renamed; finish capture at safe boundary. |
| `dokime.execution.environment` | Backend.Environment | Renamed; derived environment read. |
| `dokime.execution.effect.prepare` | Effects.Create | Renamed; durable inverse ticket, no hardware write. |
| `dokime.execution.effect.applied` | Effects.Set | Renamed; whole writable application state and Effects.Get revision; preserve earliest inverse. |

All 17 product context bindings in `sdk/rust/src/host.rs` are covered, plus two
library clock operations and five dynamic lifecycle families: 24 inventory rows.
Executions.GetProgress and Evidence.GetTraceability are shared with clients. Add plugin-only
Effects.Get as the exact mirror of Effects.Set; it is a companion, not a 25th
present mapping row. Any large plugin-settings or traceability writable part must
be split by identified setting or entry into sub-resources with their own Get/Set
and revision, rather than replacing an unbounded aggregate. The client inventory
counts method templates, not instances of these sub-resources.
The shared client upload workflow expands one producer call to several operations;
do not add 24 to the client total or treat this as a fixed generated-gRPC method count.

## Source Verification and Report Corrections

Verified on the current product tree and by read-only inspection of the library
checkout named by its report:

- `proto/dokime/v1/client_*.proto`: 23 services, 132 ordinary methods and 132 Run
  twins. Counting `message` declarations, including nested declarations, across
  `proto/dokime/v1/*.proto` gives 953. This confirms the reports' correction of
  131/262. PublishExecutionReport is present. Remove 264 twin-envelope messages;
  the final payload-message total remains unknown; neither subtraction nor the
  236 method messages establishes it.
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
  WaitTime becomes Sleep; CreateRecord/CreateLog become Evidence.AppendRecords/Evidence.AppendLogs.
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

Only these questions remain open. Settle each when its implementation is reached;
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

For implementation, coordinate the library primitives and exact pin, then
implement the
canonical product contracts/host behavior, then migrate SDKs and CLI, then move the
desktop to the new host and grammar in one cutover. Delete old host paths, duplicate
types and obsolete stories only after their capabilities are covered. Keep each
intermediate tree building; do not ship adapters to obsolete APIs. Implementation
updates README, product/query/SDK/GUI guidance and affected OpenSpec owners together.
No data is deleted or reset; incompatible development storage uses an explicitly
selected fresh root, retaining the v1 baseline. Rollback before delivery uses the
previous coherent binary/client set and its storage root, not aliases or silent
schema migration. Separate targeted proof from final native Windows/Linux and
hosted proof; this document claims none of that implementation validation.
