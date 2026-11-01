# Cycle Query Work Status

## Why

Snapshot cycle discovery needs to identify active work without a second live listing.

## What Changes

Add work `status` and `running_count` to cycle query rows and exact status alternatives to cycle selection. Preserve phase, ordering, paging, authorization and snapshot semantics. Publish the admitted cycle header before provider dispatch, capture lifecycle from the persisted cycle revision and count Running executions at the same metadata/catalog cut. History queries retain their existing durable cut without globally flushing active work.

## Impact

Domain query contracts, runtime history projection/selection, generated descriptors and Python/TypeScript bindings, focused runtime/server tests. The dependent client work remains tracked by `client-typed-status` in the same integration branch. Owner approved this additive extension on 2026-11-01. Commit only; no push.

The development projection body changes without a storage version bump under the project storage policy. Use fresh roots for older projections; no migration, reset or compatibility fallback is added.
