# Query API Foundations

The finite query declarations are under `crates/dokime-operations/src/query/`;
shared messages are under `crates/dokime-domain/src/model/query/`.
Indexed local history selection is implemented under runtime query/storage ownership.
Record query execution is implemented under the same ownership.
Operation registration, telemetry execution and generated clients remain pending.
These declarations are not yet callable through transports.
The existing catalog aggregate, browse, replay and follow operations retain their behavior.

| Operation | Request Selection | Response |
| --- | --- | --- |
| `dokime.query.list_executions` | `selection`, `page`, `at` | `ExecutionPage` |
| `dokime.query.list_cycles` | `selection`, `page`, `at` | `CyclePage` |
| `dokime.query.list_records` | `scope`, `record_type`, `versions`, `where`, `between`, `page`, `at` | `RecordPage` |
| `dokime.query.list_samples` | `execution_id`, `stream`, `versions`, `fields`, `where`, `between`, `bucket`, `page`, `at` | `SamplePage` |
| `dokime.query.aggregate_samples` | `execution_id`, `stream`, `versions`, `where`, `between`, `outputs`, `at` | `Statistics` |

Each operation declares one unary request and the ordinary operation policy.
Runtime active-work budgets must fit that policy. Requests cannot change policies.
Requests reject unknown JSON properties. Omitted optional values are unconstrained;
empty version/field lists select compatible captured versions/all declared fields.
History alternatives are exact and conjunctive across fields; omitted archive selects
unarchived rows. Execution ordering is start time then ID, missing times last;
cycles order by creation time then ID.

`where` is a flat conjunction of `{field, operator, values}`. Operators are
`eq`, `lt`, `lte`, `gt`, `gte`, `in`, `exists`. Comparison takes one tagged
scalar; IN takes a nonempty bounded list; exists takes one tagged Boolean.
Integer, Unsigned and Number operands are distinct and preserve their values.
Null operands are invalid: `exists(false)` matches absent or null fields.
Array/object fields support only exists; Boolean fields support eq/IN/exists.
Contract IDs containing dots select literal fields. Reserved envelope fields are
`@id`, `@timestamp`, `@received_at`, `@source`, `@schema_version` and records'
`@severity`. There is no expression language, nested traversal or SQL input.

`between: {start, end}` uses UTC `{seconds, nanos}` instants and `[start, end)`.
Invalid precision, reversed/equal bounds and missing times under a bounded
window are rejected/excluded respectively. Existing inclusive windows stay unchanged.

Pages carry `rows`, `schema`, `snapshot`, `limits`, `next` and `complete`.
`page: {rows?, bytes?, after?}` controls finite page budgets; configured ceilings
clamp positive requested limits. Schema and metadata count toward encoded size.
An empty page can have a continuation after its last scanned position.
Complete means exhausted with no continuation. Raw samples and buckets are tagged
`SampleRow` variants; raw rows retain their original types, versions and attribution.
Schema fields retain literal ID, declared type, nullability and unit. Integer
contracts permit signed/unsigned 64-bit values; concrete storage types can advertise
Int64/Uint64. No integer is converted to a floating value in raw results.

`at` is a tagged Token or saved Checkpoint reference. Returned snapshot tokens
are opaque, authenticated, bounded and scoped to the local retained store;
they never grant access or imply terminal readiness. Continuations bind the
snapshot, selection, projection/reduction and last scanned key, permitting only
page-budget changes. Token creation/validation uses the fixed-cut lifecycle below.

`bucket: {every_nanos, aggregate}` requires a positive fixed interval and a
count/sum/mean/min/max reducer. Buckets are epoch-aligned, clip inputs to the
window, omit empty intervals and preserve partition identity, bounds, units,
reducer, sample count and per-field finite count. Other statistics use ordered
unique `{alias, field, aggregate}` outputs. Empty-field Count counts rows;
other outputs require numeric fields. Percentile ranks are integral 0..100.
Statistics include typed values, schema, snapshot, selected-row and quality
counts, with `complete=true` only on successful full reduction. Empty numeric
selections return zero count and null for other statistics.

Failures carry stable reason, operation, safe message, optional field/operator,
snapshot and effective limit; incompatible schemas also list available versions.
Invalid input maps to InvalidArgument, missing scope/contract to NotFound,
access denial to PermissionDenied, incompatible/unavailable cuts to
FailedPrecondition, budgets to ResourceExhausted, active deadlines to
DeadlineExceeded, corruption to DataLoss and storage failure to Unavailable.
Failure messages must omit storage paths, credentials and internal query text.

Captured-field planning lives in
`crates/dokime-runtime/src/runtime/query/selection/`. `Selection::records` and
`Selection::samples` consume captured contracts and an immutable `QueryConfig`,
validate versions, projection, predicates and UTC windows, and return a schema
plus `required_fields`. Storage planning must select those versions and bind
`FieldRef::Declared` to actual captured columns; envelope references have their
own closed enum. The pure matcher accepts borrowed values and occurrence time,
so it applies no ingestion defaults or implicit object traversal. It opens no
storage reader. Record execution uses this planner; sample execution remains pending.

## Local History Storage

Finite history queries read only the connected server's local retained SQLite catalog.
They do not merge external adapters or other servers; the existing merged browse
operations retain their separate conflict and discovery semantics.

`storage/sqlite/history.rs` projects execution and cycle summaries in the same
transaction as each authoritative native object write. Revision indexes support
exact template/status/cycle/archive filters and normalized UTC time/ID seeks.
Later metadata and archive changes append projections; earlier catalog cuts
keep their original membership, values and ordering. Cycle membership is indexed
at the same revision. History capture reads already durable catalog state and
does not flush all active executions.

Archive intent remains journal-owned. A successful archive mutation reconciles
its catalog visibility atomically; startup repeats reconciliation before serving.
Interrupted writes expose only committed projections. Layouts lacking these
indexes are refused: select a fresh storage root explicitly, preserve the old
root, and keep the development schema/evidence baseline at v1. No migration or
automatic reset is performed.

## Fixed Query Cuts

`runtime/query/snapshots.rs` drains the existing fixed publication-ingest frontier,
then captures contracts, the journal frontier and admitted metadata cursor under
the existing admission gates. It drains records through that journal frontier
and flushes the admitted evidence cursor. Later record readers must apply both
the journal and physical frontiers; queued later records cannot enlarge the cut.
A flush that saves newer work cannot enlarge the returned cut. History capture
uses the already durable catalog without flushing active executions. Checkpoint
reuse keeps its original evidence boundary and separate metadata frontier.

Snapshots bind the backend identity, local source, store/epoch, catalog revision,
metadata/evidence frontiers, captured-contract identity and configured expiry.
Contract captures are bounded by the schema budget and stored by content hash;
sealed executions reuse their existing captured source. They contain schema
metadata, not evidence payloads. Versioned cycle membership and member revisions
are sought from the catalog, one bounded page at a time.

The local store persists one random signing key. Authenticated tokens survive
restart while their exact data and contract capture remain available. A token
never grants access; transport registration and access checks remain task 1.10.
Expired, tampered, foreign-store, deleted and unavailable cuts fail explicitly.
History projections and captured contracts survive retirement of superseded native
object versions. Destructive file deletion advances a separate query generation
in the deletion transaction; affected tokens then become unavailable.

Continuations bind the canonical selection/projection/reduction and last scanned
key. Changing row/byte sizes preserves the selection; changing predicates or
projection does not. History and record handlers reserve response metadata and continuation
bytes before admitting rows. No SQLite transaction, client lock, member vector
or result set is retained between pages. Arrow execution remains with its owning tasks.

## Finite Records And Follow

`RuntimeHandle::query_list_records` selects one record type within an execution
or cycle, optionally narrowed to a member execution. It reads the existing indexed
producer/capture links, returning each occurrence once in durable position order.
Captured observer records keep their original producer scope, source, occurrence
time, receive time and schema version; capture does not rewrite attribution.
Cycle membership uses the captured catalog revision, excluding later members.

Queries are finite reads at one retained cut. They never wait for execution
completion or future records, and never start, stop or seal work. Existing replay
and follow readers remain the interfaces for observing later evidence. A fresh
query may see newer evidence; passing the returned snapshot preserves the old cut.
The captured record journal epoch and position constrain records independently
of the physical storage boundary, including after restart. Checkpoints retain
their original physical boundary without a new journal cutoff.

Selection and output use captured record contracts, including compatible versions,
nullable fields and units. Each row includes only that version's declared details;
missing fields stay absent and nulls stay null, without applying defaults.
Unknown fields/contracts and incompatible versions fail before evidence scanning,
including when no rows match. Integer details remain lossless.

Each call scans at most the configured `max_page_rows` chunk within working and
scan budgets. A sparse chunk can return no rows with `complete=false` and a seek
continuation after its last scanned occurrence. Follow that continuation until
`complete=true`; an empty page alone does not mean exhaustion. Page row/byte limits
can change between calls. Schema, snapshot and continuation metadata count toward
the page ceiling. An indivisible oversized row or metadata, work exhaustion,
corruption or unavailable cut returns a typed failure, never a complete subset.
