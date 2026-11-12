# Operation Grammar Design

Status: locked by the owner on 2026-11-12. Target decisions are Owner.
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

- **Names (Owner):** symmetric, predictable `VerbResource`; singular for one
  resource and plural for collections. Requests are `<Method>Request`, replies
  are `<Method>Response`, always the response wrapper, never a bare resource.
  Operation IDs are namespaced and derive mechanically from the method;
  their namespace/group follows the owning service, with no second authored alias.
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
  outcome; changed payload conflicts. Declare deduplication lifetime; expired keys
  fail rather than silently repeat irreversible work.
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
  at an accepted offset conflict. Download by repeated GetContent calls with
  offset and maximum bytes; each response states total length and content digest.
  Configuration bounds chunk and range sizes. Measure large-file throughput in a
  Nightly benchmark case before deleting streaming transfers.
- **Streaming (Owner):** never generate twins automatically. Server streaming is
  reserved for Watch; the plugin protocol retains its own carriers. A managed
  stream exists only for an actual cooperative final-result promise after
  cancellation, replacing the ordinary gRPC method. Native gRPC cancellation abandons
  observation and cannot promise that final response; admitted durable work
  survives observer loss. Downloads, documents and diagnostics use bounded unary
  replies, with content bytes read through ranged GetContent.

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

**Owner:** SetExecution of a label renames only that execution. CancelExecution
addresses only a running execution; queued cancellation fails with an action-named
error. Stop waits for a safe scheduling boundary; Cancel interrupts the active
invocation, retaining required cleanup. Neither is a generic writable status.
**Owner:** merge pause/resume, label, archive and cycle policy into whole-value
Set of each resource's small writable part, retaining revision conflicts and
lifecycle admission rules. Get returns that part separately from runtime facts.

```proto
message GetExecutionRequest { string name = 1; Position at = 2; }
message ExecutionWritable { string label = 1; bool paused = 2; bool archived = 3; }
message GetExecutionResponse {
  Execution execution = 1; Position resolved_at = 2;
  ExecutionWritable value = 3; bytes revision = 4;
}
message SetExecutionRequest {
  string name = 1;
  ExecutionWritable value = 2;
  bytes expected_revision = 3;
  string request_id = 4;
}
message SetExecutionResponse { ExecutionWritable value = 1; bytes revision = 2; Receipt receipt = 3; }
message CancelExecutionRequest { string name = 1; string request_id = 2; }
message CancelExecutionResponse { Execution execution = 1; Receipt receipt = 2; }
service Executions {
  rpc GetExecution(GetExecutionRequest) returns (GetExecutionResponse);
  rpc SetExecution(SetExecutionRequest) returns (SetExecutionResponse);
  rpc CancelExecution(CancelExecutionRequest) returns (CancelExecutionResponse);
}
```

### Representative Resource 2: Records

**Owner:** plugin publication becomes AppendRecords, not CreateRecord. Its
acknowledgement retains the existing promised stored-record semantics; the receipt
must state the durable position when that promise completes. Records and samples
retain produced/captured ownership, source time and schema identity.

```proto
message ListRecordsRequest {
  string parent = 1; Position at = 2; RecordFilter filter = 3; PageRequest page = 4;
}
message ListRecordsResponse { repeated Record records = 1; PageInfo page = 2; }
message WatchRecordsRequest {
  string parent = 1; Position at = 2; RecordFilter filter = 3; PageRequest page = 4;
}
message WatchRecordsResponse {
  Position resolved_at = 1;
  oneof event {
    RecordSnapshot snapshot = 2;
    Record appended = 3;
    ResyncRequired resync_required = 4;
    Position progress = 5;
  }
}
message AppendRecordsRequest {
  string parent = 1; repeated Record records = 2; string request_id = 3;
}
message AppendRecordsResponse { Receipt receipt = 1; }
service Evidence {
  rpc ListRecords(ListRecordsRequest) returns (ListRecordsResponse);
  rpc WatchRecords(WatchRecordsRequest) returns (stream WatchRecordsResponse);
  rpc AppendRecords(AppendRecordsRequest) returns (AppendRecordsResponse);
}
```

### Representative Resource 3: Transfers

**Owner:** CreateUpload, GetUpload, AppendUploadBytes, DeleteUpload and
PublishUpload are unary for every caller. AppendUploadBytes appends ordered bytes
at the stated next offset. Replaying an identical accepted chunk succeeds;
different bytes at an accepted offset conflict; gaps and out-of-order new chunks
are refused. GetUpload reports the accepted offset, length, digest and session
state to resume. PublishUpload verifies complete bytes, length and digest before
exposing immutable content. CreateArtifact or CreatePlugin then references it.
Sessions are caller-bound, bounded and expire explicitly; expiry cannot silently
start another upload. DeleteUpload abandons it. Transport loss never publishes
partial content. Chunk and range maxima are configuration-owned limits.

GetContent returns a bounded range, repeated by the caller to resume downloads.
The content identity and digest bind every range to the same representation,
including a scoped evidence archive at a returned cut. There is no separate
streaming archive download gRPC method. Measure large-file upload/download throughput
in the Nightly benchmark set before deleting the superseded streaming transfers;
throughput is a benchmark measurement, never a correctness-test assertion.

```proto
message CreateUploadRequest {
  string parent = 1; uint64 size_bytes = 2; bytes sha256 = 3; string request_id = 4;
}
message CreateUploadResponse { Upload upload = 1; Receipt receipt = 2; }
message GetUploadRequest { string name = 1; Position at = 2; }
message GetUploadResponse { Upload upload = 1; Position resolved_at = 2; }
message AppendUploadBytesRequest {
  string name = 1; uint64 offset = 2; bytes data = 3; string request_id = 4;
}
message AppendUploadBytesResponse { Upload upload = 1; Receipt receipt = 2; }
message PublishUploadRequest { string name = 1; string request_id = 2; }
message PublishUploadResponse { PublishedContent content = 1; Receipt receipt = 2; }
message DeleteUploadRequest { string name = 1; string request_id = 2; }
message DeleteUploadResponse { Receipt receipt = 1; }
message GetContentRequest {
  string name = 1; Position at = 2; uint64 offset = 3; uint64 max_bytes = 4;
}
message GetContentResponse {
  bytes data = 1; uint64 offset = 2; uint64 total_length = 3;
  bytes sha256 = 4; Position resolved_at = 5;
}
service Content {
  rpc CreateUpload(CreateUploadRequest) returns (CreateUploadResponse);
  rpc GetUpload(GetUploadRequest) returns (GetUploadResponse);
  rpc AppendUploadBytes(AppendUploadBytesRequest) returns (AppendUploadBytesResponse);
  rpc PublishUpload(PublishUploadRequest) returns (PublishUploadResponse);
  rpc DeleteUpload(DeleteUploadRequest) returns (DeleteUploadResponse);
  rpc GetContent(GetContentRequest) returns (GetContentResponse);
}
```

### Representative Resource 4: Plugin

**Owner:** one entry describes managed or remote placement. Availability is a
state, not another entry family. RestartPlugin replaces an activation; it does not
mean Set status. The existing origin-dependent interruption, finalization and
cleanup contract remains with `grpc-plugins`, not with the name change.

```proto
message CreatePluginRequest {
  string request_id = 1;
  oneof placement { RemotePlugin remote = 2; PublishedContent bundle = 3; }
}
message CreatePluginResponse { Plugin plugin = 1; Receipt receipt = 2; }
message RestartPluginRequest { string name = 1; string request_id = 2; }
message RestartPluginResponse { Plugin plugin = 1; Receipt receipt = 2; }
```

### Product-Specific Actions and Preserved Distinctions

Plugin lifecycle verbs each have one locked meaning: MaterializeTestCase resolves
parameters into a case without running it; InitializeExecution acquires suite
resources; ExecuteStep runs one frozen step; EvaluateCriterion judges captured
evidence; CleanupExecution releases suite resources. Host operation IDs remain
template-local where necessary; one method name is not one shared handler instance.
Files.CopyFiles becomes CaptureFiles, freezing selected external files into
evidence without deleting the originals. The effect journal's PrepareEffect and
ConfirmEffect become CreateEffect and SetEffect: allocate a durable inverse ticket,
then replace its small writable application state under the revision from GetEffect.
GetEffect returns that same writable state and revision. Neither performs the
physical write; earliest-inverse retention, ownership and restoration order remain.

CreateCases preserves validate-only preview, one-selection materialization and
atomic replay-protected plan registration. CreateCycle with `source_cycle`
preserves rerun notices and new identities. CreateExecutions allocates mutable
lifecycle resources; it is not Append of immutable evidence. CreatePromptAnswer
and CreateExecutionCommand allocate identified lifecycle resources with retained
receipts. SetExecutionProgress replaces a current value; AppendLogs and
AppendDocumentParts add ordered immutable evidence.

GetDocument retains evaluated output, budgets, repeat instances and failures;
it does not publish a report. CreateReport renders a preview at a fixed cut,
GetReport reads an existing published report, PublishReport commits a selected
report and seal. GetContent delivers bytes; downloads never publish. Secret
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
| `Appearance.GetAppearance` | `Backend.GetAppearance` | Merged: Move appearance ownership. | G |
| `Backend.CollectDiagnostics` | `Backend.GetDiagnostics` | Renamed: Read bounded diagnostic output. | G |
| `Backend.GetAvailability` | `Backend.GetAvailability` | Kept: Retain availability. | CS |
| `Backend.GetHealth` | `Backend.GetHealth` | Merged: One health owner. | CS |
| `Backend.GetIdentity` | `Backend.GetIdentity` | Merged: One identity owner. | GCS |
| `Backend.WatchLogs` | `Backend.WatchLogs` | Kept: Durable log watch. | T |
| `Backend.WatchTrace` | `Backend.WatchTraceEvents` | Renamed: Name trace event resource. | G |
| `Catalog.GetRecordSchema` | `Catalog.GetRecordSchema` | Merged: Catalog or frozen execution parent. | N |
| `Catalog.GetSuite` | `Catalog.GetSuite` | Kept: Retain suite definition. | GCS |
| `Catalog.GetTelemetrySchema` | `Catalog.GetStreamSchema` | Renamed: Use stream vocabulary. | N |
| `Catalog.ListRecordSchemas` | `Catalog.ListRecordSchemas` | Merged: Catalog or frozen execution parent. | N |
| `Catalog.ListSuites` | `Catalog.ListSuites` | Kept: Retain suite catalog. | GCS |
| `Catalog.ListTelemetrySchemas` | `Catalog.ListStreamSchemas` | Merged: Use stream vocabulary. | N |
| `Content.DownloadAppearanceAsset` | `Content.GetContent` | Merged: Bounded bytes by content identity. | G |
| `Content.DownloadArtifact` | `Content.GetArtifact` + `Content.GetContent` | Merged: Metadata then bounded bytes. | GC |
| `Content.DownloadEvidence` | `Content.GetContent` | Merged: Ranged archive content; preserve scope, cut and digest verification. | C |
| `Content.DownloadExecutionFile` | `Content.GetExecutionFile` + `Content.GetContent` | Merged: Metadata/range then bytes. | C |
| `Content.FollowExecutionFile` | `Content.WatchExecutionFile` | Renamed: Retain growth/replacement semantics. | G |
| `Content.ListArtifacts` | `Content.ListArtifacts` | Merged: Retain bounded artifact selection. | N |
| `Content.ListExecutionFiles` | `Content.ListExecutionFiles` | Merged: Retain bounded file selection. | G |
| `Content.ReadExecutionFile` | `Content.GetExecutionFile` | Merged: Same file resource, text/byte ranges. | G |
| `Content.UploadArtifact` | `Content.CreateUpload` + `Content.GetUpload` + `Content.AppendUploadBytes` + `Content.DeleteUpload` + `Content.PublishUpload` + `Content.CreateArtifact` | Merged: Shared unary resume lifecycle. | GC |
| `Documents.CaptureCurrentDocument` | `Documents.GetDocument` | Merged: Evaluated output without publication. | G |
| `Documents.UpdatePlotSettings` | `Plots.SetPlotSettings` | Renamed: Whole small writable plot value and expected revision from GetPlotSettings. | G |
| `Evidence.GetCapturedRecordSchema` | `Catalog.GetRecordSchema` | Merged: Execution parent selects frozen schema. | N |
| `Evidence.ListRecords` | `Evidence.ListRecords` | Merged: One selector/cut contract. | GC |
| `Evidence.WatchRecords` | `Evidence.WatchRecords` | Kept: Snapshot then durable changes. | GC |
| `Execution.AppendExecutions` | `Executions.CreateExecutions` | Renamed: Allocate mutable execution resources. | GC |
| `Execution.CancelExecution` | `Executions.CancelExecution` | Kept: Running only, queued refusal names action. | G |
| `Execution.CreateCycle` | `Cycles.CreateCycle` | Merged: Preserve new-cycle admission. | GCS |
| `Execution.DeleteCase` | `Cases.DeleteCase` | Kept: Preserve lifecycle admission. | G |
| `Execution.GetCase` | `Cases.GetCase` | Kept: Retain planned case. | G |
| `Execution.GetCycle` | `Cycles.GetCycle` | Merged: One operational/saved read. | GC |
| `Execution.GetExecution` | `Executions.GetExecution` | Merged: One operational/saved read with seal. | GCS |
| `Execution.GetExecutionCommand` | `Executions.GetExecutionCommand` | Kept: Retained command receipt. | C |
| `Execution.ListCaseParameters` | `Cases.ListCaseParameters` | Kept: Pinned definition values. | G |
| `Execution.ListCases` | `Cases.ListCases` | Kept: Retain ordered cases. | G |
| `Execution.ListCycles` | `Cycles.ListCycles` | Merged: Operational/query/history selection. | G |
| `Execution.ListExecutionParameters` | `Executions.ListExecutionParameters` | Kept: Frozen execution values. | G |
| `Execution.ListExecutionSteps` | `Executions.ListExecutionSteps` | Kept: Frozen steps and state. | G |
| `Execution.ListExecutions` | `Executions.ListExecutions` | Merged: Operational/query/history selection. | GC |
| `Execution.PauseCycle` | `Cycles.SetCycle` | Merged: Whole small writable value and Get revision; preserve pause admission. | S |
| `Execution.PauseExecution` | `Executions.SetExecution` | Merged: Whole small writable value and Get revision; preserve pause admission. | GC |
| `Execution.RenameCycle` | `Cycles.SetCycle` | Merged: Whole small writable value and Get revision, including label. | G |
| `Execution.RenameExecution` | `Executions.SetExecution` | Merged: Whole small writable value and Get revision; rename only this execution. | G |
| `Execution.RerunCycle` | `Cycles.CreateCycle` | Merged: source_cycle preserves rerun notices. | GC |
| `Execution.ResumeCycle` | `Cycles.SetCycle` | Merged: Whole small writable value and Get revision, with paused=false. | S |
| `Execution.ResumeExecution` | `Executions.SetExecution` | Merged: Whole small writable value and Get revision, with paused=false. | GC |
| `Execution.SealCycle` | `Cycles.SealCycle` | Kept: Preserve durable finalization. | GC |
| `Execution.SetCycleArchived` | `Cycles.SetCycle` | Merged: Whole small writable value and Get revision, including archived. | G |
| `Execution.SetExecutionArchived` | `Executions.SetExecution` | Merged: Whole small writable value and Get revision, including archived. | G |
| `Execution.StartExecution` | `Executions.StartExecution` | Kept: Activate selected execution. | GC |
| `Execution.StopExecution` | `Executions.StopExecution` | Kept: Stop scheduling at safe boundary. | GC |
| `Execution.SubmitExecutionCommand` | `Executions.CreateExecutionCommand` | Renamed: Interactive Next/RunStep/Finish, replay receipt. | C |
| `Execution.UpdateCyclePolicy` | `Cycles.SetCycle` | Merged: Whole small writable cycle value and Get revision, including policy. | G |
| `Health.ListHealth` | `Evidence.ListHealthEvents` | Merged: Name evidence collection. | GS |
| `History.BrowseCycles` | `Cycles.ListCycles` | Merged: Retain saved filtering and ordering. | G |
| `History.BrowseExecutions` | `Executions.ListExecutions` | Merged: Retain live/saved availability. | G |
| `History.DeleteExecution` | `Executions.DeleteExecution` | Kept: Keep seal precondition and replay protection. | G |
| `History.DownloadCheckpointArtifact` | `Content.GetArtifact` + `Content.GetContent` | Merged: Same reads at returned cut/seal. | G |
| `History.GetCheckpointContext` | `Executions.GetExecution` + `Cycles.GetCycle` + `Documents.GetDocument` + `Prompts.ListPrompts` + `Evidence.ListHealthEvents` + `Evidence.ListNotes` + `Evidence.GetTraceability` + `Backend.GetAppearance` | Merged: Position-matched frozen context, no checkpoint DTO. | GC |
| `History.GetCheckpointDocument` | `Documents.GetDocument` | Merged: Same read at returned cut/seal. | GC |
| `History.GetCheckpointEvidence` | `Content.ListArtifacts` + `Content.ListExecutionFiles` + `Catalog.ListRecordSchemas` + `Catalog.ListStreamSchemas` | Merged: One cut, preserve frozen schemas/availability. | G |
| `History.GetExecutionCheckpoint` | `Executions.GetExecution` | Merged: Return position and seal in ordinary read. | C |
| `History.ListCheckpointRecords` | `Evidence.ListRecords` | Merged: Same selector at returned cut/seal. | G |
| `History.ListCheckpointTelemetry` | `Evidence.ListSamples` | Merged: Same selector at returned cut/seal. | G |
| `History.ReadCheckpointOutput` | `Content.GetExecutionFile` | Merged: Same bounded file read at cut/seal. | G |
| `Media.AudioRecording` | `Media.CreateMediaCapture` + `Media.WatchMediaCapture` + `Media.StopMediaCapture` | Merged: Owned recording lifecycle, retained result. | G |
| `Media.CameraRecording` | `Media.CreateMediaCapture` + `Media.WatchMediaCapture` + `Media.StopMediaCapture` | Merged: Keep explicit unsupported-kind refusal. | N |
| `Media.Photo` | `Media.CreateMediaCapture` | Merged: Typed photo source. | G |
| `Media.ScreenRecording` | `Media.CreateMediaCapture` + `Media.WatchMediaCapture` + `Media.StopMediaCapture` | Merged: Keep explicit unsupported-kind refusal. | N |
| `Media.Screenshot` | `Media.CreateMediaCapture` | Merged: Typed screenshot source. | G |
| `Notes.CreateNote` | `Evidence.CreateNote` | Kept: Allocate identified note. | G |
| `Notes.EditNote` | `Evidence.SetNote` | Renamed: Whole note writable value and revision from GetNote. | G |
| `Notes.ListNotes` | `Evidence.ListNotes` | Merged: Retain note collection. | G |
| `Observation.GetRuntimeShell` | `Backend.GetOverview` | Renamed: Runtime overview resource. | G |
| `Observation.GetTraceability` | `Evidence.GetTraceability` | Merged: Retain evidence context. | G |
| `Observation.WatchExecutionProgress` | `Executions.WatchExecutionProgress` | Kept: Paired progress read/watch. | GS |
| `Observation.WatchRuntime` | `Backend.WatchOverview` | Renamed: Same overview selection. | G |
| `Planning.MaterializeTemplate` | `Cases.CreateCases` | Merged: Single materialized selection. | GCS |
| `Planning.PreviewSelection` | `Cases.CreateCases` | Merged: validate_only, no registration. | GC |
| `Planning.RegisterSelection` | `Cases.CreateCases` | Merged: Atomic plan with replay key. | C |
| `Plots.ListAnalyses` | `Catalog.ListAnalyses` | Kept: Move immutable declarations. | N |
| `Plots.QuerySnapshots` | `Plots.ListPlotSnapshots` | Renamed: Bounded evaluated snapshots. | G |
| `Plots.Watch` | `Plots.WatchPlotSnapshots` | Renamed: Same plot selector/cut. | G |
| `Plugins.GetInstallation` | `Plugins.GetPlugin` | Renamed: One managed/remote entry. | G |
| `Plugins.Install` | `Content.CreateUpload` + `Content.GetUpload` + `Content.AppendUploadBytes` + `Content.DeleteUpload` + `Content.PublishUpload` + `Plugins.CreatePlugin` | Merged: Shared upload then managed admission. | GC |
| `Plugins.ListAdmittedPlugins` | `Plugins.ListPlugins` | Merged: state=READY filter. | G |
| `Plugins.ListInstallations` | `Plugins.ListPlugins` | Merged: One entry selection. | G |
| `Plugins.Reload` | `Plugins.RestartPlugin` | Renamed: Replace activation, retain finalization rules. | GS |
| `Plugins.Remove` | `Plugins.DeletePlugin` | Renamed: Remove entry, migrate old payload. | G |
| `Plugins.SetEnabled` | `Plugins.SetPlugin` | Renamed: Whole small plugin writable value and GetPlugin revision, including enabled. | G |
| `Plugins.List` | `Plugins.ListPlugins` | Merged: One managed/remote entry list. | S |
| `Plugins.Add` | `Plugins.CreatePlugin` | Merged: Remote placement admission. | S |
| `Prompts.GetDefinition` | `Catalog.GetPromptDefinition` | Renamed: Separate definition from asked prompt. | S |
| `Prompts.GetPrompt` | `Prompts.GetPrompt` | Kept: Retain asked prompt state. | GCS |
| `Prompts.ListDefinitions` | `Catalog.ListPromptDefinitions` | Renamed: Immutable definitions. | S |
| `Prompts.ListPending` | `Prompts.ListPrompts` | Merged: state=PENDING filter. | GCS |
| `Prompts.Respond` | `Prompts.CreatePromptAnswer` | Renamed: One answer resource and replay receipt. | GCS |
| `Query.Aggregate` | `Queries.GetAggregate` | Renamed: Complete read-only aggregate. | T |
| `Query.AggregateSamples` | `Evidence.GetStatistics` | Renamed: Retain stream statistics selectors. | T |
| `Query.ListCycles` | `Cycles.ListCycles` | Merged: Preserve projection/filter semantics. | T |
| `Query.ListExecutions` | `Executions.ListExecutions` | Merged: Preserve projection/filter semantics. | TS |
| `Query.ListRecords` | `Evidence.ListRecords` | Merged: One canonical evidence read. | T |
| `Query.ListSamples` | `Evidence.ListSamples` | Merged: One canonical sample read. | T |
| `Remote.CreatePairing` | `Access.CreatePairing` | Kept: Preserve access material lifecycle. | G |
| `Remote.ReadListener` | `Access.GetListener` | Renamed: Derived listener state. | G |
| `Remote.RegenerateCertificate` | `Access.CreateCertificate` | Renamed: Allocate replacement certificate. | G |
| `Remote.UpdateListener` | `Access.SetListener` | Renamed: Whole listener writable settings and revision from GetListener. | G |
| `Reports.PublishExecutionReport` | `Reports.PublishReport` | Renamed: Explicit durable publication, never implicit download. | S |
| `Reports.DownloadCycleReport` | `Reports.GetReport` + `Content.GetContent` + `Reports.CreateReport` | Merged: Published bytes or explicit preview creation. | GCS |
| `Reports.DownloadExecutionReport` | `Reports.GetReport` + `Content.GetContent` + `Reports.CreateReport` | Merged: Published bytes or explicit preview creation. | GCS |
| `Reports.MaterializeReport` | `Reports.CreateReport` | Merged: Render only, no publication. | N |
| `Runtime.GetHealth` | `Backend.GetHealth` | Removed: Remove duplicate health gRPC method. | C |
| `Runtime.GetIdentity` | `Backend.GetIdentity` | Removed: Remove duplicate identity gRPC method. | C |
| `Secrets.Export` | `Secrets.ExportSecrets` | Renamed: Existing Base64 JSON transfer, not encryption. | G |
| `Secrets.Import` | `Secrets.ImportSecrets` | Merged: Preserve choices and generation conflict. | G |
| `Secrets.List` | `Secrets.ListSecrets` | Renamed: Bounded metadata, not secret values. | G |
| `Secrets.Lookup` | `Secrets.GetSecret` | Renamed: Granted identifier only. | N |
| `Secrets.Preview` | `Secrets.ImportSecrets` | Merged: validate_only, no writes. | G |
| `Secrets.Put` | `Secrets.SetSecret` | Renamed: Whole writable secret value and revision from GetSecret. | G |
| `Secrets.Remove` | `Secrets.DeleteSecret` | Renamed: Preserve keyring/catalog cleanup. | G |
| `Settings.GetPluginSettings` | `Settings.GetPluginSettings` | Kept: Saved values, not UI declarations. | G |
| `Settings.GetSettings` | `Settings.GetSettingsSchema` | Renamed: UI declarations/cards/bootstrap metadata. | G |
| `Settings.ListStorageSources` | `Settings.GetStorageSources` | Renamed: One ordered value. | G |
| `Settings.ReadApplicationSettings` | `Settings.GetApplicationSettings` | Renamed: Revisioned application values. | G |
| `Settings.ReorderStorageSources` | `Settings.SetStorageSources` | Merged: Whole small ordered value and revision from GetStorageSources. | G |
| `Settings.SetStorageSourceEnabled` | `Settings.SetStorageSources` | Merged: Whole small ordered value and Get revision, including enabled state. | G |
| `Settings.UpdateApplicationSettings` | `Settings.SetApplicationSettings` | Renamed: Whole writable value and matching Get revision. | G |
| `Settings.UpdatePluginSettings` | `Settings.SetPluginSettings` | Renamed: Whole writable value and matching Get revision; if large, use separately revisioned setting sub-resources with GetPluginSettings/SetPluginSettings. | G |
| `Telemetry.ListSamples` | `Evidence.ListSamples` | Merged: One canonical sample read. | CS |
| `Telemetry.WatchSamples` | `Evidence.WatchSamples` | Kept: Same source selector and durable boundary. | C |

## Locked Client Inventory and Counts

| Locked Service | Count | Operations |
| --- | ---: | --- |
| Backend | 11 | `GetIdentity`, `GetHealth`, `GetAvailability`, `GetAppearance`, `GetOverview`, `WatchOverview`, `ListLogs`, `WatchLogs`, `ListTraceEvents`, `WatchTraceEvents`, `GetDiagnostics` |
| Catalog | 9 | `GetSuite`, `ListSuites`, `GetRecordSchema`, `ListRecordSchemas`, `GetStreamSchema`, `ListStreamSchemas`, `GetPromptDefinition`, `ListPromptDefinitions`, `ListAnalyses` |
| Plugins | 7 | `GetPlugin`, `ListPlugins`, `WatchPlugins`, `CreatePlugin`, `SetPlugin`, `DeletePlugin`, `RestartPlugin` |
| Cases | 6 | `CreateCases`, `GetCase`, `ListCases`, `WatchCases`, `DeleteCase`, `ListCaseParameters` |
| Cycles | 6 | `GetCycle`, `ListCycles`, `WatchCycles`, `CreateCycle`, `SetCycle`, `SealCycle` |
| Executions | 17 | `GetExecution`, `ListExecutions`, `WatchExecutions`, `CreateExecutions`, `SetExecution`, `StartExecution`, `StopExecution`, `CancelExecution`, `DeleteExecution`, `ListExecutionParameters`, `WatchExecutionParameters`, `ListExecutionSteps`, `WatchExecutionSteps`, `GetExecutionCommand`, `CreateExecutionCommand`, `GetExecutionProgress`, `WatchExecutionProgress` |
| Evidence | 14 | `ListRecords`, `WatchRecords`, `GetStatistics`, `GetTraceability`, `WatchTraceability`, `ListNotes`, `WatchNotes`, `GetNote`, `CreateNote`, `SetNote`, `ListHealthEvents`, `WatchHealthEvents`, `ListSamples`, `WatchSamples` |
| Content | 14 | `CreateUpload`, `GetUpload`, `AppendUploadBytes`, `DeleteUpload`, `PublishUpload`, `GetContent`, `GetArtifact`, `CreateArtifact`, `ListArtifacts`, `WatchArtifacts`, `GetExecutionFile`, `ListExecutionFiles`, `WatchExecutionFiles`, `WatchExecutionFile` |
| Documents | 2 | `GetDocument`, `WatchDocument` |
| Plots | 4 | `ListPlotSnapshots`, `WatchPlotSnapshots`, `GetPlotSettings`, `SetPlotSettings` |
| Prompts | 4 | `GetPrompt`, `ListPrompts`, `WatchPrompts`, `CreatePromptAnswer` |
| Settings | 7 | `GetSettingsSchema`, `GetApplicationSettings`, `SetApplicationSettings`, `GetPluginSettings`, `SetPluginSettings`, `GetStorageSources`, `SetStorageSources` |
| Access | 4 | `GetListener`, `SetListener`, `CreatePairing`, `CreateCertificate` |
| Secrets | 7 | `ListSecrets`, `WatchSecrets`, `GetSecret`, `SetSecret`, `DeleteSecret`, `ExportSecrets`, `ImportSecrets` |
| Reports | 3 | `GetReport`, `CreateReport`, `PublishReport` |
| Media | 4 | `CreateMediaCapture`, `GetMediaCapture`, `WatchMediaCapture`, `StopMediaCapture` |
| Queries | 1 | `GetAggregate` |

The 132 ordinary mapping rows classify as 24 kept, 35 renamed,
71 merged and 2 duplicate gRPC methods removed. All 132 Run gRPC methods are
additionally removed. The locked client set has 17 services and 120 operations/gRPC methods,
compared with 23 services and 264 gRPC methods today: 144 fewer gRPC methods. This is a
locked target inventory, not a claim that every gRPC method is already served. Watches,
shared uploads and split content/report resources account for added destinations.

The mapping reaches 100 distinct destinations; 20 added companion operations
complete read/watch/resource coverage: `Backend.ListLogs`, `Backend.ListTraceEvents`, `Cases.WatchCases`, `Content.WatchArtifacts`, `Content.WatchExecutionFiles`, `Cycles.WatchCycles`, `Documents.WatchDocument`, `Evidence.GetNote`, `Evidence.WatchHealthEvents`, `Evidence.WatchNotes`, `Evidence.WatchTraceability`, `Executions.GetExecutionProgress`, `Executions.WatchExecutionParameters`, `Executions.WatchExecutionSteps`, `Executions.WatchExecutions`, `Media.GetMediaCapture`, `Plots.GetPlotSettings`, `Plugins.WatchPlugins`, `Prompts.WatchPrompts`, `Secrets.WatchSecrets`.

The 120 locked client methods require 240 named request/response messages. Removing
264 twin envelopes does not determine the final message total: rendered graphs,
authoring definitions and unreachable payloads still require field-level review.
All content downloads are bounded unary GetContent ranges; document/diagnostic
output is bounded unary, and only Watch uses ordinary server streaming. A proven
managed-settlement exception replaces its ordinary gRPC method, retaining the same
operation and gRPC method count.

## Plugin-Facing Mapping

These operations are logically unary over the duplex library invocation carrier;
publication has its own carrier. Counts are separate from client gRPC methods and do
not multiply dynamic step/criterion instances or count protocol control frames.

| Present Operation or Family | Locked Operation | Disposition and Preserved Behavior |
| --- | --- | --- |
| `<template>.materialize` | MaterializeTestCase | Renamed; pure materialization, request/response wrapper. |
| `<template>.initialize` | InitializeExecution | Renamed; acquire execution-local suite resources. |
| `Step.operation` | ExecuteStep | Renamed family; frozen step identity and retry evidence. |
| `Criterion.operation` | EvaluateCriterion | Renamed family; captured-evidence judgment. |
| `<template>.cleanup` | CleanupExecution | Renamed; required cleanup after initialization. |
| `dokime.execution.prompt` | Prompts.CreatePrompt | Renamed; wait for answer, preserve scope and cancellation. |
| `dokime.settings.read` | Settings.GetPluginSettings | Merged with caller-bound resource read. |
| `dokime.secrets.read` | Secrets.GetSecret | Merged with granted-identifier read. |
| `harmos.clock.sleep` | Clock.Sleep | Kept meaning; normalized carriers, library-owned clock. |
| `harmos.clock.now` | Clock.GetTime | Renamed; library-owned clock. |
| `dokime.execution.record` | Evidence.AppendRecords | Renamed; ordered immutable data, durable receipt. |
| `dokime.execution.records` | Evidence.ListRecords | Merged; preserve count/last selection and bounded reads. |
| `dokime.execution.statistics` | Evidence.GetStatistics | Merged; stream/field/step selectors and complete statistics. |
| `dokime.execution.artifact` | Content upload lifecycle + CreateArtifact | Merged; same unary sessions as clients, acknowledged bytes. |
| `dokime.execution.progress` | Executions.SetExecutionProgress | Renamed; whole small progress value and GetExecutionProgress revision. |
| `dokime.execution.log` | Evidence.AppendLogs | Renamed; ordered immutable log lines. |
| `dokime.execution.traceability` | Evidence.SetTraceability | Renamed; whole small writable value, GetTraceability revision and scope; large writable parts split into separately revisioned entry Get/Set sub-resources. |
| `dokime.execution.document` | Documents.AppendDocumentParts | Renamed; ordered immutable document contributions. |
| `dokime.files.copy_files` | Files.CaptureFiles | Renamed; freeze file copies, preserve originals. |
| `dokime.files.start_capture` | Files.StartFileCapture | Renamed; start capture lifecycle. |
| `dokime.files.stop_capture` | Files.StopFileCapture | Renamed; finish capture at safe boundary. |
| `dokime.execution.environment` | Backend.GetEnvironment | Renamed; derived environment read. |
| `dokime.execution.effect.prepare` | Effects.CreateEffect | Renamed; durable inverse ticket, no hardware write. |
| `dokime.execution.effect.applied` | Effects.SetEffect | Renamed; whole writable application state and GetEffect revision; preserve earliest inverse. |

All 17 product context bindings in `sdk/rust/src/host.rs` are covered, plus two
library clock operations and five dynamic lifecycle families: 24 inventory rows.
GetExecutionProgress and GetTraceability are shared with clients. Add plugin-only
Effects.GetEffect as the exact mirror of SetEffect; it is a companion, not a 25th
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
  WaitTime becomes Sleep; CreateRecord/CreateLog become AppendRecords/AppendLogs.
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
