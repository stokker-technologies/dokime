# Fast Long-Execution Finalization

## Why

Long executions exceed the desktop's unchanged 25-second shutdown budget.
Completion currently computes full-history aggregates and reports. The owner
requires bounded-tail Stop/Cancel and bounded-size report materialization,
including executions lasting days, with complete durable raw evidence.

## What Changes

- Measure deterministic one-hour and four-hour workloads separately for Stop,
  Cancel and HTML/document/plot/summary materialization and save.
- Profile and replace repeated history reads and end-of-run reductions with
  incremental durable work; retain exact summaries and evidence integrity.
- Join concurrent/retried finalizers; count physical drain invocations and prove
  shutdown joins an in-flight execution stop.
- Keep operation and shutdown deadlines unchanged.

## Capabilities

### New Capabilities

- `evidence-finalization`: bounded end-of-run work and joined drain ownership.

## Impact

Runtime telemetry/storage, execution lifecycle, report projection, associated
benchmarks/tests and maintained product documentation. Commit only; no push/PR.
Owner Decisions Locked On 2026-10-30: numeric plots consume time-bucket
first/last/min/max/count envelopes built during ingestion; tables retain bounded
first/last rows and identify complete evidence. Exact statistics and raw data remain
complete. Unbounded analyses are explicitly unavailable rather than scanned.
Stop, Cancel and shutdown close and drain evidence independently of background
report publication. Pending/ready/failed state is observable; downloads and saves
wait for readiness. Report failure cannot fail Stop. All presentation limits come
from configuration. Storage remains v1 with fresh roots and no migration.
