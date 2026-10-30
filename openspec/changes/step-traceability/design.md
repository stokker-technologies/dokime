# Design

## Agreed Contract

```rust
// Inside a declared step:
ctx.traceability().section(TraceabilitySection::new("dut", "Device Under Test"));
ctx.traceability().row("dut", TraceabilityRow::text("serial", "Serial", "SN-7"));
// A later call replaces the row with the same identifier.
ctx.traceability().row("dut", TraceabilityRow::text("serial", "Serial", "SN-8"));
```

The execution owns a mutex-protected section index, as it owns the document index. A short-lived context handle validates and changes that index, then emits through `ExecutionTransport` while holding the index lock so mutation and delivery order agree. No hook, alternate service, database or observation operation is introduced.

`section` upserts a complete section. `row` upserts a row in an existing section and emits the updated section. Unknown sections, duplicate row identifiers and reserved host-derived section identifiers are refused and logged without changing the index. Sections and rows retain insertion order when replaced. Repeated section edits in one evidence delivery apply in order; the last edit wins. Report/testing consumers fold deliveries by identifier rather than accumulating duplicates.

Existing evidence batches durably commit section replacements. Recovery, history, GUI document cards, reports, exports, plots and Python capture continue reading the existing projections. Sealed reports retain their captured content. The host plugin has no template traceability hook and retains its derived host evidence.

Criteria retain their captured-evidence-only contract and receive no mutable traceability handle. Initialization, observers and cleanup receive no traceability mutation capability; steps own discovery and editing. Document behavior itself is unchanged.

## Inventory And Storage

Owners: domain traceability validation/upsert; execution context/index and lifecycle; transport and runtime transaction; macro authoring; OBC and starter producers; runtime/server/test harness consumers; GUI/report/Python read consumers; maintained guides and byte-identical skill copies. Public wire shapes and v1 storage fields need no changes. GUI `server-state.ts` already invalidates traceability on runtime revisions, the shared document renderer consumes those refreshed projections, and report snapshots consume their frozen capture. Python `client/history.py` reads `GetCheckpointContext`; tools/python has no removed-hook authoring consumer. No alternate GUI, report or Python model is needed. No existing storage root is deleted or reset.

## Tooling Constraint

`stkr specification start step-traceability` cannot activate in this checkout because it has no `stkr.toml`. The tracked OpenSpec change and native persistent goal own the work; repository setup is outside this task.
