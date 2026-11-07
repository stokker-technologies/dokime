# Agreed Design

## Identity

```rust
let mut section = TraceabilitySection::new("Device Under Test");
section.id("dut"); // An edit key remains stable if its label changes.
let serial_number = TraceabilityRow::text("Serial Number", &identity.serial_number);
section.row(serial_number);
```

Share the existing document normalizer through the internal identity module. Keep section mutation statement-based like DocumentSection and row options builder-based. Explicit IDs use the same validation/normalization as document overrides. No serialized shape changes.

A variable and its response field share one name. Traceability IDs derive from labels unless explicitly set; the normalizer uses dot-separated lowercase components. Pin IDs for label-independent replacement and established external consumers.

## Audit

GUI `src/lib/traceability.ts` keys stated groups by snapshot identity plus section identifier; it does not compare authored IDs across executions. Derived host groups retain their existing source identifiers. Runtime report projection carries captured section/row metadata; it does not require the old OBC authored IDs. Runtime edits compare section and row IDs within an execution. The starter edits `bench`/`target_c`, so those are explicit stability exceptions. Identity/replacement fixtures retain explicit keys where their behavior depends on them. No compatibility or storage migration is needed.

## Scope and Ownership

One writer owns this integration worktree. Task 1 owns domain identity/traceability, all constructor consumers, runtime traceability/report expectations, starter traceability and all maintained skill copies. Task 2 owns the client example directories, starter authoring flow and sample OBC plugin operations, domain data, observers and templates. The existing host sidecar is a shipping provider, not a sample plugin, and retains its established native layout. There is no root examples directory on the base revision.

Review README impact with both tasks; preserve unaffected product claims. Do not edit historical archived changes. Migrate tests and generated bindings when contract fields actually change. No unresolved owner decisions remain.
