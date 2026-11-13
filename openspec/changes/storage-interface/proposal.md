# Storage Interface

## Why

Application persistence is split between
`crates/dokime-runtime/src/storage/` and `src/runtime/storage/`, with concrete
SQLite access leaking into queries and product adapters. One storage owner must
preserve columnar performance, query cuts and durable finalization without
depending back on the runtime. The existing Harmos interface needs five generic
capabilities before Dokime can remove those escapes.

## What Changes

Start this separate change only after the last `grpc-plugins` wave (wave 5).
Design and implement the Harmos extensions first, publish and pin their revision,
then extract `dokime-storage`. No extension or extraction belongs to the gRPC
waves. Capture a post-gRPC baseline before changing storage and compare after
upstream adoption and after extraction.

```text
proto contracts --> dokime-protocol
                          ^
                    dokime-storage --> Harmos storage interface
                          ^
                    dokime-operations
                          ^
                    dokime-runtime --> harmos-runtime
                          ^
                     CLI / desktop
```

Arrows mean dependencies except the proto generation arrow. Earlier layers may
be consumed directly. There is no `dokime-domain` dependency and no storage
dependency on operations, runtime, rendering or a public SDK facade.

### Scope and Exceptions

`dokime-storage` merges both runtime storage folders and owns application
persistence: backend implementation, persisted shapes/codecs, settings and
metadata persistence, object/record/stream/file access, snapshots, history indexes,
durable preview projections, artifact inventory and persistence adapters.
Use Harmos storage admission, ordering, durability, query and file facilities;
physical SQLite/Arrow/IPC/compression remains behind the backend boundary.
Do not keep concrete `State` escapes in operations or runtime query consumers.

These named exceptions stay with their owners and do not require new generic
all-disk APIs: logs; support bundles; exports and report spools; plugin
installation; desktop root lease and backup; client pairing and download files;
command-line and build tooling. Their operations can use application storage
without moving their local-file ownership into it. In particular, report spool
files are distinct from durable report capture/intents, and plugin bundle
installation is distinct from persisted application plugin settings.

Execution/step-range statistics, history filters and authenticated continuation
tokens, evidence/metadata cuts, finalization/intents/seals, report capture,
artifact ownership/inventory and fresh-root refusal remain Dokime behavior on top
of generic capabilities. Runtime retains lifecycle policy, transactions, state,
cycle transitions and effects; storage receives explicit owned inputs and returns
durable results. It never invokes runtime transactions or report operations.
Development schemas stay at v1; refuse incompatible owned roots without resetting
or deleting data. Use a fresh root explicitly rather than adding migrations.

### Harmos Dependency

The upstream Harmos storage change must define and prove these capabilities
before Dokime adopts them. The following are capability contracts, not settled
Rust signatures:

| Capability | Required Semantics |
| --- | --- |
| Native columnar scans | Bounded Arrow pages with column projection, predicate/time pruning, stable position continuation and explicit fallback, avoiding row-payload reconstruction when supported. |
| Indexed search | Backend-pushed predicates, secondary/prefix keys and ordered seek pages at a stable snapshot; no Dokime cycle/status/archive meanings. |
| Durable materialized projections | Registration, commit-coupled advancement, readable durable frontier, replay/invalidation rules and snapshot-consistent reads. |
| Time buckets as a query transform | Late-sample replacement and changed keys/revisions; preserve first/last/min/max, moments, source positions and snapshot visibility. |
| Conditional update | Atomic expected-revision check and replacement, including conflict/absence semantics and a durable receipt. |

Preserve the existing `StorageBackend` lifecycle/history/cursors, replay-idempotent
writes, bounded reads, snapshots, retention and staged files, including optional
resume/delete/receipt/identity capabilities used by Dokime. Preserve `StateBackend`
journal and recovery snapshots with required data frontiers; query snapshots are
distinct. Startup currently composes a Dokime primary backend with Harmos's
`state::file::FileBackend`; extraction does not require inventing a Dokime state
backend. Independent backend writes are not one cross-method transaction.
Reuse existing file capture/follow, staged production and directory selection.
No generic execution-statistics, migration, scratch-workspace or all-disk API is
part of this change.

### Cycles to Break Before Extraction

Paths in this section are relative to `crates/dokime-runtime/src/`.

| Reverse Dependency | Replacement |
| --- | --- |
| `storage/sqlite/data/telemetry/writer.rs` → `plot/preview/index.rs` → plot request/context | Storage owns preview index, batch/checkpoint shapes and ingestion-coupled advancement. Plot consumers pass protocol requests and read projections. |
| `storage/sqlite/history.rs` → `runtime/query/cases.rs` | Move stored `CaseHeader`, `CaseProjection` and status projection beside history persistence; expose protocol views to callers. |
| `runtime/storage/` → `RuntimeHandle`, transactions and report operations | Split persistence from lifecycle control. Runtime/operations pass owned snapshots and commands through narrow resources; storage returns results without callbacks. |
| Storage → document definitions, `document/resolve::value_matches_operator`, `document/frame::Fold` | Use protocol definitions directly; expression evaluation belongs to its applying operation, while the storage-used fold belongs beside storage aggregation. |
| Storage → runtime configuration defaults | Pass explicit storage-owned settings; relocate persisted metadata and query limits without importing host configuration. |

### Types to Relocate

Protocol messages are the definition source. Persisted encodings are private
storage shapes, not a recreated domain layer. The inventory below names the
source types to replace or relocate, including all consumers and re-exports:

| Source Area | Types and Target |
| --- | --- |
| `crates/dokime-domain/src/model/` as migrated by gRPC wave 5 | `TestExecution`, `TestCycle`, `ExecutionCheckpoint`, `ExecutionSource`, `ExecutionOutput`, `ExecutionSeal`, `CycleSeal`, `TelemetryContract`, `TelemetrySample`, query selections/pages and settings snapshots: wire definitions in protocol, durable encodings in storage. No domain crate is retained for extraction. |
| `runtime/storage/` | `ExecutionView`, `CycleView`, `CapturedSource`, `ExecutionStorage`, `Acquisition`, `Captures`, `NativeOutput`, `ArtifactRecord`: persistence-only shapes/read handles in storage; capture admission and live lifecycle resources stay in runtime or their operations. Do not pull report spools into storage. |
| `runtime/query/`, `storage/sqlite/`, `config/` | `QueryConfig`, `ApplicationMetadata`, `CaseHeader`, `CaseProjection`, status projection: persisted metadata, history projections and storage query settings in storage; client-visible definitions in protocol. |
| `domains/evidence/selection.rs` | `ScopeSelection`, `TelemetrySelection`: protocol selections with storage-local resolved read plans; no import of host evidence domains. |
| `plot/`, `plot/preview/index.rs` | `PlotPayloadRequest`, `Preview`, `Limits`, `Cut`, `QueryRead`, `QueryWork`, `Moments`, preview `Batch`/`Checkpoint`: shared request/view definitions in protocol, index/aggregation/read plans in storage; rendering and operation policy stay with their owners. |

Remove superseded paths, concrete-state accessors, adapters, re-exports and tests
whose covered code disappears; migrate meaningful coverage and all consumers.
Retain numerical signal kernels. No presentation-crate extraction is included.

### Benchmark Proof

Use the same deterministic workloads and native platform conditions before and
after upstream adoption and extraction. Record actual results, not inferred
speedups from source size. Baselines start after gRPC wave 5 so loop removal is
not attributed to the storage change.

Cover `benches/storage/{query,contention,ingestion,files,filesystem,directories}.rs`,
`benches/integration/{telemetry,plots,documents,finalization}.rs` and system report
benchmarks. Migrate the old sidecar-ingest benchmark to the new host path. Retain
million-row datasets and add deterministic multi-day datasets with millions of
samples; the existing 1/4-hour finalization cases alone are insufficient.

Compare throughput, first-page latency, stop/cancel/report time, peak memory,
bytes read and durability cost. Include concurrent ingestion/querying, restart
and replay, late samples, exact execution/step-range statistics, live tails versus
saved cuts, projection frontier recovery and interrupted finalization. Prove
snapshot parity, pruning/allocation behavior, cancellation and bounded paging.
Correctness is deterministic; timing/resource observations are benchmarks, never
unit-test pass/fail thresholds. Account for regressions and resolve them before
claiming the change validated. Linux and Windows both require native proof.

## Capabilities

### New Capabilities

- `storage-interface`: application persistence ownership, upstream generic
  capabilities, dependency cuts and performance/correctness proof.

### Modified Capabilities

None. Product lifecycle and query semantics are preserved under the new boundary.

## Impact

Dokime storage, query/statistics/history consumers, settings persistence,
artifact/report capture, startup composition, manifests, tests, benchmarks and
maintained documentation change. Harmos owns the generic interface, backend
support and its contract proof in a separate upstream change. Its published
revision is pinned before final validation. The excluded local-file owners and
host transaction/state/cycle ownership retain their responsibilities.
