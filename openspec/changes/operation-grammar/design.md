# Operation Grammar Design

Status: locked by the owner on 2026-11-12
Boundary, managers and nested resources locked by the owner on 2026-11-12
Naming rule amended by the owner on 2026-11-12: bare verbs on the service's resource
Separation by audience locked by the owner on 2026-11-12
Three visibility levels locked by the owner on 2026-11-12
Paging, SDK packaging, transfers and durable lag amended by the owner on 2026-11-13

The paging scope and default unary transfer limits in the operation contracts
were amended by the owner on 2026-11-13.

Target decisions are Owner.
Current-source observations are not target decisions.
See proposal.md for the decision register and motivation.

## Contract and Ownership

The maintained architecture is `docs/architecture.md`; current operation identities,
caller mapping and writable values are in `docs/execution-host.md`. Those source
inventories replace the historical face tables and fixed method counts here.
The protocol roots in this tree are `proto/dokime/{model,host,plugin}/v1/`.
Model holds values; the executor owns each operation declaration. Host callers are
CLIENTS, PLUGINS or ALL, independently of Public/Protected/Private visibility.
There is one declaration when both clients and plugins call the same operation.

`operation-boundary` owns all remaining direct Store/SQL consumers, native scoped
cuts, compound publication, watches, transforms, typed facade, product wrappers,
dependency fences and composition-file deletion. Harmos host operations are
in-process only in the target; product operations declare the remote surface.
`protocol/composition.toml` and internal manager uses still exist in source and
are removals in that change, not permanent architecture.

`plugin-boundary` owns product plugin contracts, registration/admission, SDKs,
registry/settings, bundles, shared generator adoption and carrier composition.
`plugin-lifecycle` owns lifecycle verbs/hooks and template prompt/record ownership.
Plugin analyses are removed. Generated output remains tracked and is regenerated
through project tasks. Neither four-language authoring ownership nor registry
policy is transferred back to the library by this change.

## Goals and Boundaries

Define one naming and behavior contract shared by native, client and plugin
interfaces. Preserve durability, scoped authority, bounded physical storage
work, finalization, publication and cooperative cancellation where promised.
The library owns generic machinery under its managers; a product owns its storage
definition, objects, policy and operations. Product lifecycle and evidence selection
remain product-owned. No arbitrary time travel, second scheduler, automatic
streaming projection or storage reset is proposed. No accepted-data snapshot is built.
The measured durable lag is accepted, amended by the owner on 2026-11-13.
plugin-boundary supersedes the earlier library-owned author SDK decision: Dokime
owns its author SDKs and contracts over generic Harmos carrier/runtime helpers.

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
  Each nested service lives below its executor package in
  `<executor-package>.<lowercase-parent>`, with the part as the service name: `dokime.host.v1.plugins.Settings/Get`. One proto file per
  service lives in a folder per parent, for example `dokime/v1/plugins/settings.proto`.
  Messages in that package are `<Service><Method>Request/Response`, such as
  `SettingsGetRequest`; parent services retain names such as `ExecutionsStartRequest`.
  Replies are always wrappers. Operation IDs derive mechanically from package,
  service and method, with no second authored alias. A nested service may have a
  different owner from its parent: `dokime.host.v1.plugins.Settings` intentionally
  sits beside `harmos.host.v1.Plugins` on the host application's endpoint.
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
  passes the durable position from that write's receipt as `at`. The measured durable
  lag is accepted: small payloads about 99 ms, large and mixed up to several hundred
  milliseconds at p95. Report p50/p95/max without a failing threshold; no
  accepted-data snapshot. Amended by the owner on 2026-11-13.
- **Paging (Owner, amended by the owner on 2026-11-13):** paging is only for data
  rows that can grow unbounded in size. Small bounded sets return whole in one
  response, without page controls or continuation. Unbounded rows use `page_size`, `max_bytes`, `page_token`, `next_page_token`,
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
  Configuration bounds chunk and range sizes: default chunk 4 MiB, default message
  limit 8 MiB. Remove streaming file transfers. Throughput remains a benchmark
  measurement, without a deletion gate. Amended by the owner on 2026-11-13.
- **Streaming (Owner):** never generate twins automatically. Server streaming is
  reserved for Watch semantics (including noun-only streamed parts); the plugin protocol retains its own carriers. A managed
  stream exists only for an actual cooperative final-result promise after
  cancellation, replacing the ordinary gRPC method. Native gRPC cancellation abandons
  observation and cannot promise that final response; admitted durable work
  survives observer loss. Downloads, documents and diagnostics use bounded unary
  replies, with content bytes read through ranged Storage.GetFileBytes.

### Audience Faces and Visibility

Executor-owned model/host/plugin roots and explicit CLIENTS/PLUGINS/ALL callers
replace the former face layout and duplicate per-caller declarations. Visibility
remains independent: Private is local-only, Protected admits the declaring plugin
and host, Public allows declared peers/clients through host admission. The final
plugin registrar, contracts and SDK ownership are in plugin-boundary; lifecycle
names and hooks are in plugin-lifecycle.

### Built-In Visibility Mapping

This mapping is decided, closing the earlier built-in mapping question. Levels
apply to host-declared operations too; audience, requirements and product policy
still constrain a Public operation. Public context declarations do not become
client services. The earlier `grpc-plugins` use of Private for host requests is
superseded by Protected here; its files are outside this documentation delivery.

| Existing Built-In or Family | Locked Level | Reason |
| --- | --- | --- |
| Plugin TestCases.Materialize, Executions.Initialize/Cleanup, Steps.Execute, Criteria.Evaluate, including generated template-local bindings | Protected | Only the host application may request plugin lifecycle work; these must cross the carrier and cannot be Private. |
| Observer control and listener callbacks | Protected | The host drives plugin lifecycle; peers and clients cannot drive it. |
| Storage file internals: abandon_file, append_file, capture_file, create_file, delete_file, finish_file, follow_file, follow_production, hash_file, list_directory, read_file, resume_file, watch_directory, write_file | Protected | Host-owned file machinery is not an exposed operation or a plugin/client upload API. |
| Storage follow_objects, follow_stream, storage_status, write_schema | Protected | Internal observation, schema and storage administration belong to the host. |
| Storage capabilities, compare_exchange, write_objects, write_records, write_streams, follow_records, query, scan_arrow, scan_index, register_index, register_projection, advance_projection, inspect_projection, read_projection, open_snapshot, release_snapshot | Public | Generic storage capabilities remain usable by admitted callers, subject to product policy and scope; scans, projections and conditional updates stay distinct. |
| Host catalog, invocation status/cancel/watch, Time.Get/Sleep | Public | Discovery, observation, invocation control and time are intended caller capabilities, constrained by face and authorization. |
| New Storage transfer declarations on context/client faces: CreateFileProduction, GetFileProduction, CreateFileWriter, AppendFileBytes, PublishFileProduction, DeleteFileProduction, GetFileBytes, WatchFileBytes | Public | Explicit exposed transfer API, distinct from Protected file internals; each face has its own contract and authority. |
| Product host context: Prompts.Create; Records.Append/List; Evidence.Statistics; Artifacts.Create; Progress.Get/Set; Logs.Append; Traceability.Get/Set; Documents.AppendParts; Files.Capture; Captures.Start/Stop; Backend.Environment; Effects.Create/Get/Set; library HostValues.Get | Public | Plugins must call these host operations during an admitted invocation, with inherited scope; context audience still refuses clients. Current settings/secret reads map to HostValues.Get. |
| Implementation-only helpers inside the host or plugin, with no external caller | Private | Keep local implementation callable under ordinary policy without advertising it or adding a wire path. |

The mapping names operation declarations, not helpers such as storage registration,
exchange and snapshot support modules. No default visibility is introduced.

### Current SDK Dispatch Audit

The pinned protocol includes Protected and all four SDKs have adopted the current
carrier. Remaining product routing/local-Private/refusal proof stays in tasks.md;
old always-wire dispatch observations are not current upstream blockers.

### Generated Client Names

All four SDK roots derive the hierarchy from executor-owned descriptors, filtering
host callers before projection. Clients and plugins have separate resource roots
without a second declaration for shared capabilities. Product declarations are
remotely callable; Harmos host operations are in-process behind product wrappers.
plugin-boundary owns the shared generator adoption and public SDK surface.

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
`harmos/v1/access/listener.proto`, package `harmos.host.v1.access`, service Listener,
Get/Set with `ListenerGetRequest/Response` and `ListenerSetRequest/Response`.
Access.Pairings.Create and Access.Certificates.Create allocate lifecycle resources;
their resource-specific operations live in the access sub-package.

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

Harmos owns generic operation/stream/storage/transport machinery. Dokime owns
storage definitions/backend, policy, plugin registration/registry/settings/bundles
and author SDKs. operation-boundary owns its in-process facade and remote product
wrappers; plugin-boundary owns plugin carrier/contract adoption. The execution,
prompt and report managers own live product work. Cycles group execution tasks
without a separate scheduler. Pure rendering and storage reads do not acquire
invented managers. Media capture needs a live owner only for retained recordings;
its current handlers are unimplemented and grpc-plugins retains that obligation.

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

The measured durable lag is accepted as it is: small payloads about 99 ms,
large and mixed payloads up to several hundred milliseconds at the 95th percentile.
The reusable Nightly benchmark reports p50, p95 and maximum for small,
multi-megabyte and mixed loads, including a consumer's independent backend.
There is no failing lag threshold and no accepted-data snapshot. Correctness
proof checks positions, durability and configured byte splitting deterministically,
never elapsed time. Amended by the owner on 2026-11-13.

### Paging Eligibility

Paging is only for data rows that can grow unbounded in size. The same rule
applies for every declared caller. Amended by the owner on 2026-11-13.

| Set | Shape | Reason |
| --- | --- | --- |
| Catalog entries, plugins and pairings (library-owned) | Whole | The admitted configuration is a small bounded set, not accumulated evidence. |
| Catalog suites, record/stream schemas and prompt definitions | Whole | Each is a finite declaration set from admitted catalogs or one frozen execution. |
| Secrets metadata and storage sources | Whole | These describe configured values/endpoints, not an accumulated evidence history. |
| Cases and pending prompts | Whole | These are the bounded current plan and active operator questions. |
| Executions, cycles and retained prompt history | Paged, position-resolved | Successive runs and asked questions accumulate without a fixed bound. |
| Parameters and frozen steps | Whole | These are a finite case/execution definition, not appended evidence. |
| Records, samples, log/health lines, files, artifacts and notes | Paged, position-resolved | Evidence can accumulate indefinitely within selected histories. |
| Plot snapshots | Paged, position-resolved | Evaluated rows can grow with the selected evidence. |

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
the handler is shared. Append admits plugin callers; List admits its declared callers so a
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

### Transfers and Plugin Entries

The final product Files resource provides Get/Read/Watch/Delete and nested Uploads
Create/Get/Resume/Append/Publish/Delete over Harmos in-process storage operations.
Preserve total length/digest, bounded ranges, ordered identical replay, changed-byte
conflict, fenced resume, explicit expiry and publication. Product metadata and
scope authorization remain product policy; no streaming transfer twin survives.
Default chunks remain 4 MiB and message limits 8 MiB unless explicitly overridden.
Plugin registry/settings/install policy and bundle ownership live in plugin-boundary,
not generic Harmos management; template lifecycle names live in plugin-lifecycle.

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

## Current Operation Inventory

The historical face counts and source-to-target tables are superseded by the
executor/caller map in `docs/execution-host.md` and generated protocol descriptors.
Recount service/operation/method/wrapper coverage after the three owning cutovers;
never claim the old table's totals as current implementation evidence.

## Remaining Acceptance

The old source-audit counts, missing-API compiler probes and face tables are
superseded by the current owners and descriptors. The executor/caller split,
Protected enum, generated four-language clients, runtime managers and desktop
workspace restoration are landed. Canonical storage and plugin/lifecycle cutovers
are not complete; their own tasks retain the source and focused proof obligations.

tasks.md retains product Public peer/local Private admission, opaque scope,
callee-policy, receipt/frontier/restart, schema-evolution and failure/gap stories.
Generic upstream conformance alone does not prove the product composition.
Unimplemented Effects, aggregate/media/capture capabilities and remaining CLI/
benchmark restoration are in grpc-plugins. Do not delete their requirements merely
because a current caller is disabled. Source inspection never establishes final
native Windows/Linux proof; final checks, authorized delivery and hosted results
remain separate unchecked obligations.
