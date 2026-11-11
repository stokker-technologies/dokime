## Context

`CreateCycle` opens an empty cycle; cases join through registered case identities (`RegisterCases`) and `AppendExecutions`. A cycle's planned cases are `CycleView::cases()` in ordinal order; sealed, retired cycles are read through `RuntimeHandle::inspect_cycle`.

## Decisions

### Shapes

```rust
// dokime-operations: Execution.RerunCycle
pub struct RerunCycle {
    pub source_cycle_id: String,
    pub label: Option<String>,
    pub context: Option<MutationContext>,
}

// dokime-domain wire
pub struct RerunCycleReply {
    pub cycle_id: String,
    pub observed_cursor: Option<Cursor>,
    pub skipped: Vec<RerunCaseNotice>,
    pub dropped_parameters: Vec<RerunCaseNotice>,
}
pub struct RerunCaseNotice { pub case_id: String, pub suite_id: String, pub reason: String }

// CycleSummary gains
pub rerun_of: Option<String>,
```

`case_id` in a notice is the case's authored identifier as the plugin declares it, readable to the operator. Dropped parameters are a separate list with the same element shape, so `skipped` counts only cases that were not created.

### Resolution

For each source case, the suite resolves by `suite_id` in the installed registry and the case by template id, else by its authored identifier. Each still-declared parameter or file input is passed as a single-value override, so materialization yields exactly that case. A materialization failure, for example a value the current definition rejects, skips the case with that reason. Cases share the source cycle's plugin, and `AppendExecutions` keeps validating them.

### Lineage

The new cycle's metadata carries the attribute `rerun_of` (`dokime_runtime::RERUN_OF_ATTRIBUTE`) with the source cycle id, alongside the source's description, tags and attributes. Metadata already flows through live state, saved cycle headers and reports, so lineage needs no new field on `TestCycle`, `CycleView` or the stored cycle, and existing roots are unaffected. The server derives `CycleSummary.rerun_of` from the attribute.

### Ordering

Resolution happens before any commit, so a refused rerun creates nothing. The runtime then registers the materialized cases, creates the cycle and appends one execution per case: the same commits the desktop's create-and-append flow makes.

### Copy Cycle

Copy Cycle is untouched. It clones frozen snapshots; Rerun Cycle resolves the installed plugin. Retiring Copy Cycle is a separate owner decision.
