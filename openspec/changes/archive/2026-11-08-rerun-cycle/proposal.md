## Why

Operators repeat a qualification campaign by running the same cases again. Rerun Execution repeats one case inside its cycle; nothing repeats a whole cycle against the plugin installed now. Copy Cycle clones the source's frozen execution snapshots, so a changed plugin cannot run them.

## What Changes

- New `Execution.RerunCycle { source_cycle_id, label }` operation, served next to `CreateCycle`, implemented by the runtime next to `create_cycle` and `append_executions`.
- The source may be Active, Sealed or archived. Its planned cases, in ordinal order, are materialized again against the currently installed suites by suite, case and parameters; the new cycle keeps the source label (or the given one) and `continue_automatically`, and every case is planned and unrun.
- Cases whose suite or case no longer exists are skipped; parameters the current case no longer declares are dropped. Both are reported. No surviving case refuses the rerun with `rerun_no_cases` and creates nothing.
- The new cycle records `rerun_of` in its metadata; `CycleSummary.rerun_of` exposes it. The source is untouched.
- GUI: Rerun Cycle in the cycle's menu, for sealed and archived cycles too; it opens the new cycle, whose Control card lists skipped cases and dropped parameters; the cycle summary shows "Rerun of <source label>" linking to the source.
- Rust client, CLI `dokime cycle rerun` and the Python `Client.rerun_cycle` expose it as thin layers.

## Capabilities

### New Capabilities
- `cycle-rerun`: Rerunning a cycle as a new planned cycle against the installed plugin.

### Modified Capabilities

## Impact

Operations catalog, domain wire, runtime planning (lineage as the `rerun_of` cycle metadata attribute), gRPC server, Rust client, CLI, Python SDK and stubs, generated TypeScript client, GUI cycle workspace, product guide and GUI guide. Copy Cycle, Rerun Execution and stored data are unchanged; no migration.
