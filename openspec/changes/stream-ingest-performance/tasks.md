# Stream Ingest Tasks

## Implementation

- [x] 1.1 Extend native benchmark fixture and existing system bench for scalar/wide durable sidecar ingestion, CPU and bytes; capture repeated baseline.
- [x] 1.2 Audit and batch transport/publication using configured limits and one row representation; preserve ordering and bounds. Complete request constructors in OBC tests: integration/{charger,lifecycle,runs}.rs and acceptance/main.rs.
- [x] 1.3 Batch writer metadata, Arrow column construction, identity/scope and numeric summary work, replacing unbounded rank indexing with bounded binary field state and exact report replay. Integrate dev's document_statistic reader in aggregates.rs and duplicate-summary removal in writer.rs.
- [x] 1.4 Share bounded field projections across plots, remove duplicated row folding and static checkpoint rewrites, and write only dirty compact state.
- [x] 1.5 Verify touched behavior with focused tests and document measured performance and storage contracts; review README impact.

- [x] 1.6 Measure actual flush, Arrow sync, SQLite VFS sync and catalog commit counts; lock wait and admission/host/writer stages in the existing system benchmark. Separate host and sidecar CPU. Own benches/system/{ingest,ingest_sync,main}.rs; storage/{ipc,sqlite}.rs and sqlite/data/telemetry{,/flush,/writer}.rs; domains/telemetry/streams{,/admission}.rs and runtime/storage/telemetry.rs. Include spawned prepare/commit work, Unix Arrow parent-directory syncs and the distinct-value index introduced by fresh dev.
- [x] 1.7 Add deterministic 4 KiB and 64 KiB payload fixtures in OBC benchmarks, select their required benchmark-only suite catalog and measure delivered payload bytes at fresh-root durable barriers.
- [x] 1.8 Evaluate each requested trim against repeated measurements, retaining supported changes as separate commits: IPC buffering/hash work; duplicate normalization; Tail allocations; sample identity representation and exact receipt queries. Preserve durability, cadence, exact retry/replay and storage v1; no migrations.
- [x] 1.9 Update CONTRIBUTING.md and docs/product-guide.md with measurement definitions, spread and decisions; README claims reviewed. Refresh measurements after integrating dev's per-row distinct index.

- [x] 1.10 Measure the two-table rowid candidate first; retain measured rowid tables and selective document repeat/paging field indexing. The measured simplest design meets ingest and query targets, so flush batching and lazy sealed postings are unnecessary. Own storage distinct/telemetry writers and readers, bounded typed median selection, document binding field selection, integration document benchmarks, focused tests and affected guides. Preserve exact live/sealed query and report behavior, storage v1, cadence and durability; no migrations, deadlines or budget changes.

## Final Validation

- [ ] 2.1 Run focused storage/telemetry and unchanged document query/report parity tests, exact report replay, repeated all-shape ingest benchmarks, document query scaling and check:rust; inspect the forbidden-term diff. No full simulation, as explicitly requested.
- [x] 2.2 Merge fresh origin/dev into the combined candidate and revalidate affected behavior before authorized delivery.

## Workflow Follow-Up

- Authorized delivery is one PR to dev, immediately squash-merged without waiting for CI. Preserve measurement evidence outside the owned worktree, cargo clean its target and retire its worktree and branch.
