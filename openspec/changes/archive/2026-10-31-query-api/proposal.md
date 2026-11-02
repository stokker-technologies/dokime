# Query API

## Why

The owner needs to ask for matching evidence from an execution and discover relevant historical executions without replaying everything in Python. Existing listeners and saved evidence readers provide retrieval, but no cohesive high-level query interface.

## What Changes

- Specify read-only server queries for execution/cycle history, contract-typed records, projected telemetry, downsampling and statistics.
- Specify a small typed predicate language, UTC windows, stable snapshots and bounded cursor pages with explicit completeness and readable typed failures.
- Add lazy Python query readers and optional Polars/Pandas conversion, using the existing Harmos aggregate vocabulary.
- Reuse SQLite catalog indexes and Arrow IPC batches; keep query work separate from bounded report previews and execution finalization.
- Expose the same contracts through the generic CLI operation command; avoid a second query parser.
- Preserve existing lookup, replay/follow, checkpoint, plot, browse and catalog aggregate contracts. These remain useful independent interfaces.

## Capabilities

### New Capabilities

- `query-operations`: canonical operations, configuration, authorization, errors and CLI parity.
- `query-selection`: declared-field predicates, schema validation and time windows.
- `query-snapshots`: consistent live evidence cuts, bounded paging and continuation lifecycle.
- `history-search`: indexed execution/cycle discovery across local retained history.
- `telemetry-query`: Arrow projection, exact reductions, time buckets and performance targets.

### Modified Capabilities

- `python-client`: add query readers and optional tabular conversion alongside the owned lifecycle and terminal evidence iterators.

## Impact

Owners: domain query/history/evidence/telemetry models; `crates/dokime-operations/src/query/`; runtime query/storage handlers; server operation registration/access mapping; generated Python/TypeScript descriptors; `packages/dokime-client`; generic CLI; maintained product/client documentation and focused verification fixtures.

Implementation ownership, acceptance and remaining combined review live in
`tasks.md`. The authorized implementation slices use local commits without
pushing; do not open a PR or archive the change.

## Scope Decisions

Queries address the connected server's local retained catalog, including live executions at a durable cut. Cross-server federation, external storage discovery, joins, arbitrary SQL, user code, full-text payload search, approximate quantiles, subscriptions with predicates and a GUI query builder are excluded. Existing merged history browsing stays available.

The proposal records the agreed API direction; implementation is limited to the authorized tasks above. Deployment-specific budget calibration and performance reference machines remain deferrable, as recorded in `design.md`; initial configuration defaults establish no performance claim.
