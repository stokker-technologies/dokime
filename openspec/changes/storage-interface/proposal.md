# Storage Interface

## Why

Application persistence has been extracted into `crates/dokime-storage/`. The
retired runtime storage folders and domain crate are gone. This change retains
backend and comparative native proof; extraction is not the remaining source task.

## Current Boundary

Protocol owns exchanged values. dokime-storage owns persisted shapes and the real
SQLite catalog/typed compressed Arrow IPC backend, implementing Harmos storage
interfaces. Its manifest has no product runtime/operations/rendering/domain edge.
Runtime retains live execution, prompt and report managers and product transition
policy. Logs, support/export/report spools, installation, desktop root leases and
backup, client pairing/downloads and tooling retain their local-file ownership.

The pinned Harmos revision and native backend include columnar projection, ordered
index reads, materialized projection frontiers, conditional updates/receipts and
compound mutation support. `backend/{columns,index,projection,objects,compound}.rs`
are source evidence, not final native proof. Retain bounded reads, snapshot and
retention behavior, replay identity, file publication and recovery frontiers.
Development formats stay v1; incompatible roots are refused and preserved.

## Superseded Extraction Plan

operation-boundary owns the complete remaining cutover: exact persisted scopes,
native retained cuts, lifecycle/seal publication, intake, watches, prompts/reports,
archive fidelity, query/bucket transforms, direct Store removal and product
wrappers over in-process Harmos operations. There is no parallel storage API or
product-owned generic merge/fold implementation here. The typed facade and
composition-file deletion also belong there. The earlier post-wave-five ordering
and instructions to call storage resources directly are superseded.

plugin-boundary owns registry/settings and bundle policy. imported-executions owns
import staging/draft publication and must establish its atomic boundary using the
canonical storage operations; extraction alone does not provide import admission.

## Comparative Proof

Keep native Windows and Linux before/adoption/after comparisons for storage,
telemetry, plots, documents, finalization and reports with identical deterministic
multi-day/million-sample data. Restore retired producers under grpc-plugins. Include
concurrent intake/queries, restart/replay, late samples, saved cuts versus live tails,
projection frontiers, interrupted finalization and exact execution/step semantics.
Report throughput, first-page/report cost, memory, bytes read and durability cost.
Do not invent historical baselines or convert timings/resource readings into
correctness assertions. Final source closure, focused proof and required native/
hosted validation remain open until the operation-boundary handoff is complete.

## Capabilities and Impact

`storage-interface` records persistence ownership and retained backend/scale proof.
The implementation spans the existing protocol/storage/operations/runtime owners,
configuration and tests/benchmarks. Final manifests, README and maintained guides
must match the canonical boundary; no extra extraction crate is proposed.
