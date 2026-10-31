# Query API Design

## Context

This records the agreed API direction. `tasks.md` tracks implementation and proof; motivation and scope are in `proposal.md`.

| Verified Baseline | Source |
| --- | --- |
| Python lookup by ID; execution records/samples replay and follow; cycle readers stop at a current frontier; occurrence IDs filter locally | `packages/dokime-client/src/dokime_client/client/execution/{handle,cycle}.py`, `client.py` |
| Immutable saved checkpoint document/context/schema/artifact/record/sample/download access | `packages/dokime-client/src/dokime_client/client/history.py` |
| Existing exported `Aggregate` vocabulary, without high-level query methods | `packages/dokime-client/src/dokime_client/client/aggregate.py` |
| Server execution browse already filters status/template/cycle/start time; cycles have saved cursor paging | `crates/dokime-operations/src/history/browse_{executions,cycles}.rs`, `crates/dokime-domain/src/model/history/wire.rs` |
| Record list/watch filters type/source/severity/scope; sample list/watch already supports start/end, schema/source/signal filters | `crates/dokime-operations/src/{evidence,telemetry}/`, `crates/dokime-domain/src/model/evidence/wire.rs` |
| Catalog aggregate already supports typed column predicates/grouping over cycles/cases/executions/steps/records, not telemetry; runtime folds saved rows | `crates/dokime-operations/src/query/aggregate.rs`, `crates/dokime-runtime/src/runtime/storage/aggregate{,/fields}.rs` |
| Arrow IPC per execution/stream/schema, SQLite stream/batch time and position indexes, projected numeric reads | `crates/dokime-runtime/src/storage/sqlite/data/telemetry/{catalog,reader}.rs` |
| Exact numeric ranks/moments and bounded preview envelopes exist; reports use a captured evidence boundary | `crates/dokime-runtime/src/storage/sqlite/data/telemetry/{aggregates,previews}.rs`, `crates/dokime-runtime/src/report/materialize.rs` |

Thus time filtering, history search and aggregates are extensions, not wholly missing primitives. No dependency on unfinished sibling live-report work is assumed.

## Goals And Non-Goals

Provide finite, reproducible selection with memory bounded independently of history length. Query reductions may scan selected evidence; Stop/report materialization must never invoke them. Exclusions are in the proposal and capability deltas; saved evidence formats remain v1.

## Decisions

### Ownership And Operations

Shared messages/validation belong in `dokime-domain`; actual operation declarations live in `dokime-operations` (the architecture skill's older domain-operation location is not the landed layout). Runtime owns planning/storage execution; server, CLI and clients translate the same contracts. Do not move existing ownership in this feature.

```text
ID                              Request -> Response (unary)
dokime.query.list_executions     ListExecutions{selection, page, at?} -> Page<ExecutionSummary>
dokime.query.list_cycles         ListCycles{selection, page, at?} -> Page<CycleSummary>
dokime.query.list_records        ListRecords{scope, record_type, versions?, where, between?, page, at?} -> Page<Record>
dokime.query.list_samples        ListSamples{execution_id, stream, versions?, fields?, where, between?, bucket?, page, at?} -> Page<Sample|Bucket>
dokime.query.aggregate_samples   AggregateSamples{execution_id, stream, versions?, where, between?, outputs, at?} -> Statistics
Page<T>{rows, schema, snapshot, limits, next?, complete}  PageRequest{rows?, bytes?, after?}
Statistics{schema, values, quality_counts, snapshot, complete=true}
```

Each row operation owns one request type; the sample response is a tagged raw/bucket variant, never an untyped table. Separate query operations avoid changing live/tail browse cursors, inclusive legacy time bounds or report preview semantics. Reuse their internal readers/metadata projection, not a competing store. Existing `dokime.query.aggregate` remains unchanged; the new statistics facade uses `Aggregate` values.

Query configuration composes canonical operation policies and existing storage/schema budgets. Static Harmos policies remain outer ceilings; configured active-work deadlines must fit within them. Query-specific budgets have one owner, validated at composition, and no request can override policy. This avoids a repository-wide dynamic operation-policy redesign.

### Python Surface

```python
from dokime_client import Aggregate, Predicate

runs = client.executions(template="example.demo.tracking", since="2026-10-01", status="failed")
async for run in runs:  # summary with ID, status, frozen metadata and availability
    consume(run)
ex = await client.execution(execution_id)
recs = ex.query_records("example.measurement", where={"status": "failed"}, between=(t0, t1))
async for record in recs:
    consume(record)
df = await ex.telemetry("host.system", fields=["cpu.load", "memory.used"],
                        between=(t0, t1), every="1s", agg=Aggregate.mean()).to_polars(max_rows=100_000)
stats = await ex.telemetry("host.system").aggregate("cpu.load", [Aggregate.mean(), Aggregate.percentile(95), Aggregate.max()])
# Same evidence cut for another finite query after the first page was read:
related = ex.query_records("example.measurement", at=recs.snapshot)
# Typed ordered predicates; maps above are equality-only shorthand:
filtered = ex.telemetry("host.system", where=[Predicate("cpu.load", "gte", 0.5)])
```

Factories are synchronous/lazy, avoiding an implicitly unbounded `await client.executions(...)` list. Terminal methods are awaited; `pages()` and item iteration provide backpressure. Readers expose `snapshot` after the first response and accept `at=` for reuse; no new snapshot-session service is needed. Use existing aggregate objects instead of a parallel string vocabulary (`"p95", "mean"`). Contract field IDs such as `cpu.load` are literal IDs, not implicit object traversal.

`client.cycles(...)`, `cycle.query_records(...)` and execution readers share selection rules. `collect(max_rows=..., max_bytes=...)` and dataframe methods are explicit finite materializations. New readers do not start, stop, seal or follow an execution. See `specs/python-client/spec.md` for lifecycle/type details.

### Planning And Snapshots

See `query-selection` and `query-snapshots`. Capture an admitted evidence cut and matching object/schema revisions, flush only through that cut, then query it; never chase a moving tail or stop the run. Reuse report capture/barrier primitives where applicable, without publishing a report or pretending a live query token is a terminal checkpoint. History captures already durable revisions, without globally flushing all running work.

Opaque authenticated tokens contain scope, source/store identity, frontier and expiry; continuation also binds canonical selection and last scanned key. They use seek paging, not SQL offsets or a held SQLite transaction. Historical revision indexes must answer the captured cut after later updates; expired, deleted or inaccessible cuts fail explicitly. No unbounded retained row set or revision vector is embedded in a token.

### Arrow Execution And Exact Statistics

Prune execution/stream/schema/time batches with the SQLite catalog, then project/filter Arrow columns once per admitted batch. Share scans across requested statistics. Buckets keep configured bounded accumulator state; return a typed budget error rather than silently coarsening `every`. Existing exact indexes are usable only when scope, filter, window and frontier match; latest mutable summaries cannot answer an older cut. Preview envelopes never substitute for raw query data.

Mean/min/max/count/sum/std/rms use bounded accumulators. Exact percentiles use matching rank indexes or bounded multipass radix selection over the selected numeric projection; do not sort a million Python objects or silently approximate. Extra passes count toward scan/work limits. Numeric aggregate projections follow existing Harmos f64 semantics; raw integer values remain lossless. Arbitrary grouping/joins and general query-engine dependencies add unnecessary scope.

## Risks And Trade-Offs

- Fixed snapshots need retained historical catalog revisions → index versioned projections and reject unavailable cuts, never substitute current metadata.
- Exact filtered percentiles may exceed an interactive budget → typed failure, narrower window; the one-second baseline excludes multipass quantiles.
- Operation ceilings and query settings can conflict → validate configured deadlines at construction; do not lengthen existing ceilings.
- Optional dataframe libraries vary in dtype support → explicit conversion errors, preserved schema metadata, dependency-free core readers.

## Migration Plan

Implement in the task order; regenerate all contract consumers from canonical declarations. Existing listeners/checkpoint APIs remain. If new catalog projections require an incompatible layout, require a fresh storage root, keep v1 and preserve the old root; no migration, reset or silent deletion. Rollback uses the previous binary with its compatible root. README impact was reviewed: no shipped claims change in this spec-only commit; implementation updates affected README/guide examples with their owning phase.

## Open Questions

- Which reference machines should calibrate the ~1 s/1M scan target? Recommended: record one representative native Windows and Linux machine with release profile, local SSD and warm cache; this does not change query semantics.
- Which deployment budget values should ship? Recommended: start from existing storage/schema/message budgets, then set query concurrency, scan, bucket and materialization defaults using those measurements; limits remain configuration-owned.
