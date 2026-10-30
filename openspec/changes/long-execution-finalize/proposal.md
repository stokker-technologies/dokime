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
The rendered-data bound for existing full-history documents is an explicit owner
decision pending clarification; independent reproduction and lifecycle work proceed.
