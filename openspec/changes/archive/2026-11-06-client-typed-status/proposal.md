# Typed Client Discovery

## Why

Consumer scripts need to inspect active work and declared record columns through typed client objects.

## What Changes

- Export cycle and execution status enums with stable lowercase string values; decode summaries and accept enum filters.
- Include cycle status and running count in finite history discovery.
- Discover record declarations lazily through the existing paged catalog operations, exposing ordered typed fields.
- Update exports, typing, examples and fake-transport coverage. Preserve existing string filters and lifecycle behavior.

## Impact

Python client discovery, package exports/stubs, package/root READMEs and `docs/remote/python.md`. The owner-approved additive query extension and regenerated bindings are tracked in `query-cycle-status` and committed first on this same branch. The client retains finite snapshot discovery.
Commit only on `feat/client-typed-status`; no push, PR or full CI simulation is authorized.
