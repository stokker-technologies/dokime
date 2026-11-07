# Traceability Labels and Canonical Examples

## Why

The owner agreed that traceability should use the same label-first identity convention as documents. Shipped examples must demonstrate the locked authoring flow without changing their behavior.

## What Changes

- Derive traceability section and row IDs from labels with the document normalizer; provide optional explicit IDs and remove ID-first constructors.
- Migrate every caller and maintained reference. Keep explicit IDs where edit keys or behavior fixtures depend on them.
- Apply canonical linear orchestration, operation ownership, names, units and presentation to the starter, client examples and OBC sample plugin.

## Capabilities

### New Capabilities

- `plugin-authoring`: label-derived traceability and canonical runnable examples.

## Impact

Dokime domain constructors, runtime tests, starter, OBC sample, client examples and authoring references. Wire fields and storage remain unchanged. No features or compatibility aliases are added.

The owner explicitly limits proof to focused tests, example/starter tasks, GUI focused tests if changed, check:rust and lint:rust; no full simulation. Delivery is a branch push only during the build freeze, with no PR or merge, followed by native Cargo target cleanup and a retained worktree.
