# Test Stories Design

## Current Inventory

The old server, domain and execution-loop trees are deleted. Their historical line
numbers and workload inventory are superseded, not unfinished code-removal tasks.
Use current runtime `tests/system/` and `tests/support/`, CLI `tests/system/`, SDK
stories and desktop native/frontend tests as the acceptance inventory.

Current shared mechanics include runtime `tests/support/{credentials,bundles,
thermal,settings}.rs`, HostProcess-backed host stories, native desktop fixtures
and the GUI RPC harness. The desktop connection fixture still lives under
`src-tauri/tests/integration/connection/fixture/`; move genuinely shared mechanics
to support without creating thin wrappers or a second scenario owner.

## Story Contract

One actor performs a journey in story order with assertions on the resulting
behavior. Share mechanical host/transport/catalog setup, not scenario selection or
hidden assertions. Preserve corruption, durability, cancellation, authorization,
wire bounds and fault coverage. Delete duplicate or superseded tests rather than
restore old APIs to make their fixtures compile.

## Concrete Remaining Work

`src-tauri/tests/integration/rpc/mod.rs` still creates a large archive fixture.
Keep the smallest cases that cross their correctness boundary and move scale to
benchmarks. The new storage and desktop benchmark producers are the measurement
owners; missing retired workload coverage stays in grpc-plugins.

Direct Store-seeded client evidence, notes, telemetry, plot and fixed-document
stories are superseded by the storage-operation boundary. This includes the old
two-million-row plot-window fixture; it is not a remaining workload migration.
Sealed-history, file-transfer and archive journeys create their evidence through
real plugins and host operations. Native backend fault and storage contract tests
retain their lower-level fixtures. Generated SDK calls use the final payload names
and place optional record, statistics, progress and traceability selectors in
their owning call options.

GUI integration `shell/status.test.ts`, `cycle/document-query.test.ts`,
`plots/nominal.test.ts` and `execution/responsiveness-fixture.tsx` still use real
timers. Audit which timers control tested behavior and which only guard browser
startup. The responsiveness fixture also stamps started_at from the current date.
Use fixed fixture time and event/controlled-clock behavior; retain configured hang
guards. Tests never assert elapsed time, load, sampled memory, authored prose or
single repository constants. Benchmarks measure cost without timing assertions.

The original simulation, definition-count, plugin-settings-label and parameter-name
tests disappeared with their old owners. Their deletion obligation is complete;
any current duplicated behavior is assessed against the current stories. Source
inspection does not establish passing native proof. Final affected Windows/Linux
execution and coverage review remain separate unchecked obligations.
