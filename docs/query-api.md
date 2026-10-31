# Query API Foundations

The finite query declarations are under `crates/dokime-operations/src/query/`;
shared messages are under `crates/dokime-domain/src/model/query/`.
Runtime handlers, storage execution and generated clients remain pending in
OpenSpec `query-api` tasks 1.4 onward. These declarations are not yet callable.
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
page-budget changes. Token creation/validation belongs to later snapshot work.

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
