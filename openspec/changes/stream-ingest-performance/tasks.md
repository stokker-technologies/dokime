# Stream Ingest Tasks

## Implementation

- [x] 1.1 Extend native benchmark fixture and existing system bench for scalar/wide durable sidecar ingestion, CPU and bytes; capture repeated baseline.
- [x] 1.2 Audit and batch transport/publication using configured limits and one row representation; preserve ordering and bounds. Complete request constructors in OBC tests: integration/{charger,lifecycle,runs}.rs and acceptance/main.rs.
- [x] 1.3 Batch writer metadata, Arrow column construction, identity/scope and numeric summary work, replacing unbounded rank indexing with bounded binary field state and exact report replay. Integrate dev's document_statistic reader in aggregates.rs and duplicate-summary removal in writer.rs.
- [x] 1.4 Share bounded field projections across plots, remove duplicated row folding and static checkpoint rewrites, and write only dirty compact state.
- [x] 1.5 Verify touched behavior with focused tests and document measured performance and storage contracts; review README impact.

## Final Validation and Delivery

- [ ] 2.1 Finish the fixture-backed workflow rerun, repeat before/after benchmark and run check:rust; inspect forbidden-term diff. Measured candidate 658c45b9b passed; finish the follow-up checks for dev's object-memory-accounting integration, whose ingest/projection/transport/benchmark sources are unchanged.
- [ ] 2.2 Push verified candidate, create PR to dev and immediately squash with requested subject.
- [ ] 2.3 Preserve evidence, cargo clean owned target and retire owned worktree and branch.
