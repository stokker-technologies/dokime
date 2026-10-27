# Runtime Revision Snapshots

## ADDED Requirements

### Requirement: Receipt-Acknowledged Replay

Sample and captured-record replay SHALL resume after the receiver's acknowledged position.

#### Scenario: Replacement Receiver

- **WHEN** a receiver with no receipt connects, even after an earlier receiver left a durable checkpoint
- **THEN** it rebuilds its evidence from zero

#### Scenario: Acknowledged Chunk

- **WHEN** the receiver acknowledges a chunk's final sample
- **THEN** the acknowledged position persists transactionally and replay seeks after it

#### Scenario: Locked Design Decisions Apply

- **WHEN** this requirement is implemented or reviewed
- **THEN** the following locked design decisions hold unchanged:

Sample settlement reconciles the receiver's last admitted sample identity for each stream before replay. SignalReply carries those receipts; the existing SQLite owner resolves them through the execution/stream/sample index and persists acknowledged positions transactionally. Replay seeks after that position and advances only after the receiver acknowledges the chunk's final sample. A receiver with no receipt rebuilds its evidence from zero, even when an earlier receiver left a durable checkpoint. Producer sequences, strict ordering and the bounded recent retry window remain unchanged. This signal message change requires a matched plugin/runtime build.

Captured-record deliveries carry the durable admission ordinal. Each execution session serializes live delivery and settlement through one owner and seeks the saved capture index after the receiver's acknowledged ordinal. The receiver retains a high-water mark and 64 exact recent receipts; its append-only evidence spool remains available to criteria without a per-record identity file. A replacement receiver starts at zero and rebuilds the captured evidence.

### Requirement: Shared Runtime Revisions

Runtime readers SHALL share immutable published revisions without copying whole indexes.

#### Scenario: Entry Map Update

- **WHEN** an execution, cycle, request, prompt, note-admission or finalization entry changes
- **THEN** only its logarithmic search path and the changed value are copied
- **AND** serialization retains the v1 ordered-map shape

#### Scenario: Locked Design Decisions Apply

- **WHEN** this requirement is implemented or reviewed
- **THEN** the following locked design decisions hold unchanged:

Runtime readers retain one immutable `Arc<RuntimeState>` per published revision. State cloning shares catalog, queue, resource and evidence roots; ordinary scalar fields remain values. Frequently mutated execution, cycle, request, prompt, note-admission and finalization maps use persistent AVL indexes with shared keys and values: a mutation copies only its logarithmic search path and the changed value. Bulk mutable traversal intentionally copies the visited values. Serialization retains the v1 ordered-map shape, and document/lifecycle consumers read the shared map directly. Publication remains monotonic; plot contexts retain the published Arc. Snapshot publication and plot reads clone only shared roots; entry-map updates never copy the whole index. Catalog and resource mutations copy their affected field, while unchanged fields remain shared. README usage and setup claims are unaffected.

### Requirement: Complete Saved Report Plots

Saved report plots SHALL retain their complete projected series.

#### Scenario: Integer Summary

- **WHEN** a summary window aggregates an integer series
- **THEN** means and medians are computed before conversion to plot floats

#### Scenario: Locked Design Decisions Apply

- **WHEN** this requirement is implemented or reviewed
- **THEN** the following locked design decisions hold unchanged:

Saved report plots retain their complete projected series for viewport zooming and retain the authored display budget as metadata. Intermediate projection stays disk-backed and paged. Projected scalar types remain available through summary aggregation so integer means and medians are computed before conversion to plot floats; missing X coordinates exclude rows only when a summary window requires an X coordinate. Leased sample payload fields are encoded once, with each recipient's captured scope appended to its wire row.
