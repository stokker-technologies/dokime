# Operation Grammar

## Why

Product operations need consistent names, revisions, receipts, positions and
failure behavior across host, CLI, desktop and four SDKs. The execution-host
migration is present, but it does not establish the final boundary or native proof.

## What Changes

- Retain descriptor-derived resource hierarchies, whole-value Get/Set mirrors,
  ordered replay-safe Append, snapshot-then-durable-Watch and bounded paging.
- Keep explicit caller admission independent of Private/Protected/Public visibility,
  with opaque inherited scope, host-mediated Public peer calls and propagated
  cancellation/structured failures.
- Preserve callee-owned policies, accepted versus durable receipts, retained issued
  cuts, explicit resynchronization and observer-independent admitted work.
- Audit final generated inventories and application behavior after the three active
  boundary/lifecycle changes; preserve required native proof as unchecked work.

## Superseded Ownership

Executor-owned model/host/plugin packages replace the former client/context faces
and their duplicate declarations/counts. operation-boundary owns storage operations,
facade/wrappers and removal of the composition file. Harmos host operations are
in-process only. plugin-boundary owns product contracts, SDKs, registry and bundles;
plugin-lifecycle owns verbs/hooks. Plugin analyses are gone and generated code stays
tracked. The old direct library-service composition and library-owned product SDK
plans are superseded, not pending work in this change.

## Capabilities

- `operation-contracts`: resource grammar and its behavioral guarantees.
- `query-snapshots` and `query-operations`: durable position selection and bounded
  continuation without read-triggered flushes.
- `data-root-identity`: product-owned unreadable-state refusal with preserved roots.

## Impact

Protocol generation, operations/runtime consumers, SDKs, CLI and desktop have
migrated; remaining acceptance is coordinated with their current owners and the
maintained architecture/execution-host mapping. grpc-plugins retains unfinished
Effects, aggregate/media/capture behavior and stable registry replacement. No
compatibility aliases, silent root resets or delivery authority are introduced.
