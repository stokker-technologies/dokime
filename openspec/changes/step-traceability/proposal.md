# Step-Owned Dynamic Traceability

## Why

The owner decided on 2026-10-30 that traceability is dynamic execution content authored and edited by steps, through the same context and captured-evidence path used by documents. A separate pre-step lifecycle hook prevents steps from recording identity discovered during execution.

## What Changes

- Remove the template `traceability()` hook, its context, future, macro expansion and runtime phase without compatibility aliases.
- Expose `ctx.traceability().section(section)` and `ctx.traceability().row(section_id, row)` only to steps. Section identifiers replace a section; row identifiers replace a row within an existing section.
- Reuse execution-owned indexes, buffered evidence delivery, durable section transactions and existing traceability projections. Preserve host-derived groups and captured report behavior.
- Migrate OBC, the starter, fixtures, documentation and all authoring skill copies.

## Capabilities

### New Capabilities

- `step-traceability`: Step-owned editable sections over the existing evidence pipeline.

## Impact

Authoring is a clean API break. Observation RPCs and GUI/Python read shapes remain unchanged. Storage stays v1; no automatic data deletion, reset or migration. Existing unrelated OpenSpec changes remain untouched.

## Owner Proof Override

Run only `env -u UV_PROJECT_ENVIRONMENT mise run build:rust:focused -- check --workspace --all-targets` and `mise run format:rust`. Skip tests, other checks, strict OpenSpec validation, full lanes and CI simulation. Commit only; do not push or open a PR.
