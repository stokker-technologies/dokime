# Query API Foundations

The finite query declarations are under `crates/dokime-operations/src/query/`;
shared messages are under `crates/dokime-domain/src/model/query/`.
Indexed local history selection is implemented under runtime query/storage ownership.
Record and telemetry query execution share that ownership.
The five operations are callable through native and gRPC transports; generated
contracts and high-level client integration follow their owning tasks.
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
storage reader. Both record and sample execution use this planner.

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
never grants access; each transport invocation rechecks the existing caller authorization.
Expired, tampered, foreign-store, deleted and unavailable cuts fail explicitly.
History projections and captured contracts survive retirement of superseded native
object versions. Destructive file deletion advances a separate query generation
in the deletion transaction; affected tokens then become unavailable.

Continuations bind the canonical selection/projection/reduction and last scanned
key. Changing row/byte sizes preserves the selection; changing predicates or
projection does not. History and record handlers reserve response metadata and continuation
bytes before admitting rows. No SQLite transaction, client lock, member vector
or result set is retained between pages. Record, raw sample and bucket execution consume evidence seek coordinates.

## Arrow Sample Pages

`runtime/query/samples.rs` plans captured versions and the union of projected,
predicate and envelope columns. The SQLite reader prunes stream/capture, durable
position and time batches before projected Arrow decoding. A bounded candidate
map merges interleaved stream files by durable position; raw values retain their
signed/unsigned integer representation and original source/scope/version.
Window matching uses original timestamp precision and excludes the upper bound.

Compressed blocks and decoded buffer sizes are admitted before allocation or
decompression. Oversized layouts fail explicitly; no full-history object list,
preview envelope or current numeric summary supplies raw rows. Response schema,
snapshot and continuation metadata count toward page byte limits. Each page
rechecks the cut and releases its reader transaction. Later arrivals are excluded
by the physical frontier even when their timestamps fall inside the window.
Captured contracts include retained Arrow versions from the selected execution or cycle, within the schema budget. For streams already present in that scope, persisted Arrow contracts take precedence over the mutable active catalog; a newer plugin version in another scope cannot enlarge an older execution’s schema. Conflicting contracts that reuse one version fail explicitly. Sparse raw scans advance through bounded durable-position spans and can return empty continuation pages after excluded rows.

## Exact Sample Statistics

`runtime/query/statistics.rs` shares the durable numeric moments accumulator
with telemetry output storage. One selection pass computes all requested moments,
row counts and missing/null/non-finite quality counts. Count outputs are Uint64
and dimensionless; other outputs are nullable Float64 with their input unit.
Aliases and output schema follow request order. Non-finite requested arithmetic
fails explicitly.

Filtered percentiles and medians use the existing ordered-f64 rank representation
with bounded radix counters over repeated projected scans. Every pass uses the
same captured contracts, predicates, window and physical frontier, and counts
against the same scan/time budget. Interpolation uses exact integer rank positions
and the fractional remainder of `(count - 1) * rank / 100`. Current mutable numeric
indexes and report previews never answer a historical or filtered selection.
Exhaustion returns a typed failure; statistics are never partial estimates.

## Epoch Bucket Pages

`runtime/query/buckets.rs` reduces each selected execution/stream/source partition
across compatible captured versions. Positive `every_nanos` intervals use UTC
Unix-epoch floor alignment, including timestamps before the epoch. Windows clip
input rows and keep natural bucket bounds. Empty intervals are omitted; populated
all-null buckets retain their sample count, finite-field count zero and a null
numeric value (Count returns zero).

Count/min/max use shared bounded moments. Sum/mean use bounded exponent bins
with integer mantissa accumulation and a fixed evaluation order, so late or
out-of-order arrival cannot change results. Each bucket reports the versions
actually present, units and reducer. Count fields are dimensionless Uint64;
other bucket values are nullable Float64.

Partition, bucket and working-state limits apply before adding accumulators.
Excessive cardinality returns BucketLimit with guidance to narrow the window or
increase every; no interval is silently coarsened. Output pages contain complete
buckets ordered by start and partition identity. Seek continuations bind the
selection and reducer; each page rescans the fixed cut within its work budget,
without retaining a transaction or sample history between calls. Schema,
snapshot and the longest required continuation count against page byte limits.

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

## Transport Access And CLI Paging

Native and gRPC calls use the runtime's five canonical handlers. Existing remote
keys grant full product read access; native-only operations keep their separate
restrictions. Catalog authorization runs on every call, including continuations.
Paired-client revocation cancels an admitted read with PermissionDenied. Shared
configured active/pending limits bound admission; cancellation drops the reader
and its permits without starting, stopping or sealing any execution.

Failures use the operation failure path with the declared status and a status
envelope containing standard `ErrorInfo`: domain `dokime.query`, an uppercase
canonical reason, and `metadata.query` containing the canonical JSON `Failure`
from `model/query/error.rs`. This retains reason, operation, field, operator,
snapshot and effective limit without a second error declaration.

The generic CLI discovers the same protobuf JSON contract; no query parser or
query command tree is required. Save `request.json` for a first cycle page:

```json
{"selection":{},"page":{"rows":1}}
```

```sh
dokime operation call dokime.query.list_cycles --backend lab --input request.json --output page.json
```

The output is an event array. Its unary completion's `value` contains `rows`,
`schema`, `snapshot`, `limits`, `next` and `complete`. When `next` is present,
copy it unchanged into `page.after` in `next-request.json`, retaining the original
selection. For example, substitute the actual returned token here:

```json
{"selection":{},"page":{"rows":1,"after":"RETURNED_NEXT_TOKEN"}}
```

```sh
dokime operation call dokime.query.list_cycles --backend lab --input next-request.json --output next-page.json
```

Repeat until `complete=true` and no `next` is returned, including after empty
pages. The continuation already binds the original snapshot. To start another
compatible query at that cut, use `at: {"token": "RETURNED_SNAPSHOT_TOKEN"}` with
the token from the completion's `snapshot`. `operation show` exposes each exact
request shape; requests cannot supply timeouts or operation policy overrides.
