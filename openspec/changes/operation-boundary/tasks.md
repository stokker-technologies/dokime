# Operation Boundary Tasks

Harmos `operation-boundary` lands first. Each Dokime group names the Harmos groups it needs
and adopts the revision that landed them: an integration-only local override while that
revision is unpublished, replaced by the published pin before final validation (refresh
locks with `mise run prepare:locks`). Every group deletes the `Store` path or kernel it
replaces, leaves the build green, commits with Conventional Commits and proves only changed
behavior with focused `mise run test`, `mise run build` and `mise run generate` invocations.
Full lanes run only after group 14. Stored shape changes refuse incompatible roots with the
fresh-root action; no migrations or compatibility code.

## 1. Inventory and Contract

- [ ] 1.1 Lock the source inventory against design.md Context: every direct `Store`, telemetry reader, Arrow or SQL consumer outside the backend modules; every duplicated kernel; every Harmos manager, engine and internal-crate use; every composition reader; every Harmos host operation a Dokime client calls. Accept when each item has an owning group below; record surprises in design.md before editing.
- [ ] 1.2 Update `docs/architecture.md`: quote invariants 1, 6 and 7 and the Final Library Assembly and Storage Contract, state their purposes, and rewrite them as locked direction for storage through operations, statistics in Harmos, the stored envelope cleared at seal, the facade and product wrappers; update `docs/execution-host.md` caller mapping accordingly. Prove with `mise run check -- --repository`.

## 2. Adopt the Additive Harmos Revision

- [ ] 2.1 Adopt the revision that landed Harmos groups 2–7 (compound mutation, deletion, storage before startup, maintenance, expression transforms, statistics, bucket summaries) with no other source change except compile fixes. Prove with `mise run build -- --package dokime-runtime --check` and `mise run test -- --package dokime-storage --target integration`.
- [ ] 2.2 Implement the compound commit, object and scoped deletion and maintenance copy/dispose hooks in `crates/dokime-storage/src/backend`. Prove with backend contract stories in `crates/dokime-storage/tests/integration` for atomic commit, conflict without effect, deletion at new cuts only and consistent copy.

## 3. Bootstrap and Maintenance

- [ ] 3.1 Move root admission (`crates/dokime-runtime/src/host.rs`), credential journals (`services/access_credentials.rs`) and root claims (`services/root_claim.rs`) onto storage operations on the Harmos storage-only runtime, keeping OS credential policy in Dokime. Prove with `mise run test -- --package dokime-runtime --target system --story storage_root_admission` and `client_pairs_and_rotates_remote_access`.
- [ ] 3.2 Replace desktop archive and disposal (`crates/dokime-gui/src-tauri/src/native_session/storage.rs`) and CLI doctor copies (`crates/dokime-cli/src/commands/doctor.rs`) with maintenance operations. Prove with the desktop storage recovery tests and `mise run test -- --package dokime-cli --target system` filtered to doctor.

## 4. Schemas, Objects and Administration

- [ ] 4.1 Move resources, views, execution state, catalog inventory, record schemas, plans, notes, client reads, resource values, evidence positions and administration onto `SetSchema`, `GetObject`, `SetObject` and `ListRows`; delete their direct SQL. Prove with the existing operation stories for these resources via `mise run test -- --package dokime-runtime --target system`, filtered to the touched stories.
- [ ] 4.2 Remove direct administration deletion in `crates/dokime-operations/src/host/secrets/store.rs` and the backend overview scan in `host/backend/overview/get.rs` (counts through `list_rows`). Prove with the secrets and overview stories.

## 5. Records and Sample Ingestion

- [ ] 5.1 Route sample ingestion (`crates/dokime-runtime/src/streams.rs`, the product stream consumer registered in `services/composition.rs`) through `AppendSamples` under the trusted execution scope, acknowledging after the durable receipt; route evidence records, traceability and notes through `AppendRecords`. Prove with `execution_isolation` and the evidence record stories.

## 6. Compound Mutations and Seal

- [ ] 6.1 Move execution, case, cycle and progress transitions (`crates/dokime-operations/src/host/{cases,cycles,executions}`, `crates/dokime-runtime/src/{execution,execution_manager,run_control}.rs`) onto `GetObject` and compound mutation, keeping transition rules and planning in Dokime. Prove with `execution_shutdown_joins_runs_after_observer_loss` and the cycle and case stories.
- [ ] 6.2 Make seal one compound mutation that records the seal, publishes report metadata and clears the execution's stored envelope and previews after the sealed report is computed; reports and prompts (`crates/dokime-runtime/src/{reports,prompts}.rs`) use operations. Prove with `reports_publish_atomically_and_close_admission`, `prompts_settle_answers_and_shutdown_waiters` and a new story `seal_clears_stored_previews`.

## 7. Files, Content and Deletion

- [ ] 7.1 Move content, artifacts, execution files and report bytes to Harmos file operations and object metadata; make the evidence archive decode operation-owned batch DTOs instead of engine types (`host/evidence/archive.rs`). Prove with the content, artifact and archive stories.
- [ ] 7.2 Implement execution and case deletion with Dokime seal and ownership checks followed by Harmos scoped deletion; delete `history.rs` and `cycles.rs` deletion paths. Prove with a deletion story asserting absence at new cuts and retained-cut reads.

## 8. Queries, Statistics and Expressions

- [ ] 8.1 Compile product expression bindings (`crates/dokime-storage/src/expressions.rs`, `document_values.rs`, `plot_selection.rs`) into Harmos query plans using filter, project, scale, offset, contains, aggregate, histogram, box summary, downsample and time merge transforms; implement `host/aggregates/get.rs` over `list_rows`. Prove with `client_queries_advanced_plots` and the document query stories.
- [ ] 8.2 Delete Dokime moments, ranks, persisted summaries and integer folds (`telemetry/aggregates.rs`), scanned bins (`stream_windows.rs`), statistics, histogram and box code (`plot_windows.rs`) and the payload encoder duplicate (`backend/payload.rs`). Accept when no aggregation kernel remains in Dokime; prove with the telemetry query and plot stories.

## 9. Previews, Documents and Plots

- [ ] 9.1 Make zoom coarsening a Dokime operation that calls the Harmos bucket-summary transform with the stored envelope and its source cut and stores the result through Harmos object or file operations; delete the merge kernel in `telemetry/previews.rs`, keeping eligibility rules. Prove with plot window and refresh stories and a deterministic catch-up story.
- [ ] 9.2 Source sigma, contour, grid and FFT inputs and document and plot snapshots (`plot_kinds.rs`, `plot_windows.rs`, `host/documents`, `host/plots/snapshots`) through Harmos row operations; full-range plots of sealed executions use on-demand bucket summaries over the sealed cut. Prove with the plot and document stories; report sealed full-range cost from the existing plot and document benchmarks without changing budgets.

## 10. Watches

- [ ] 10.1 Replace product storage notifications in every `watch.rs` under `crates/dokime-operations/src/host` with `WatchRows`, projecting product responses and handling resynchronization at issued cuts. Prove with the watch stories for executions, cycles, documents, evidence and content.

## 11. Delete Direct Store Consumers

- [ ] 11.1 Delete the `Store` facade, telemetry reader and query entry points and every remaining consumer path so `crates/dokime-storage` holds only the backend implementation and product persisted shapes; migrate or remove tests whose code is gone. Accept when no crate outside the backend registration imports storage internals; prove with `mise run build -- --package dokime-storage --check` and `mise run test -- --package dokime-storage --target integration`.

## 12. Adopt the Harmos Facade

- [ ] 12.1 Adopt the revision that landed Harmos group 9; build the runtime through the Harmos builder with backend pair, catalog and settings, hold an operation caller instead of the operation manager (`services/state.rs`), drive plugin activation, clock binding and sample start/stop through Harmos operations (`execution.rs`, `dispatch.rs`) and make the filesystem publication helper product-owned. Prove with `manager_lifecycles` stories and `execution_isolation`.
- [ ] 12.2 Remove direct `harmos-domain`, `harmos-query`, `harmos-storage` and `harmos-protocol` dependencies from every manifest, importing backend contracts through the authoring module and model values through the `harmos` root; add the manifest dependency check. Prove with `mise run build -- --package dokime-runtime --check`, `mise run check -- --repository` and `mise run prepare:locks`.

## 13. Product Wrappers and Composition Removal

- [ ] 13.1 Adopt the revision that landed Harmos group 10; declare wrapper operations under `proto/dokime/host/v1` for each Harmos host method a Dokime client calls (design.md D6), each a declaration plus a handler calling the in-process Harmos operation with its declared audience; migrate desktop, CLI and SDK callers to the wrappers. Prove with `client_pairs_and_rotates_remote_access`, `client_reads_runtime`, `client_collects_backend_diagnostics` and a story refusing direct Harmos host calls and plugin calls to client wrappers.
- [ ] 13.2 Delete `protocol/composition.toml`, `setup.composition` in `services/composition.rs`, `native::callers()`, the composition reading in `sdk/cpp/tools/generate.py`, `sdk/csharp/tools/generate.py` and `tools/python/src/dokime_tools/build/resource_roots.py`, and its tests; generators derive from `crates/dokime-protocol/src/descriptors.bin`. Prove with `mise run generate -- --protocol`, `mise run check:generated` and `tools/python/tests/integration/test_protocol_generation.py`.
- [ ] 13.3 Update `docs/architecture.md`, `docs/execution-host.md`, the product guide and SDK READMEs for wrappers, removed composition and the new caller mapping; prove with `mise run check -- --repository`.

## 14. Integration

- [ ] 14.1 Replace every integration-only local override with the published Harmos revision, run `openspec validate operation-boundary --strict`, the format task and the language checks on the combined tree, and confirm every source item above is closed; report pending native platform or hosted proof explicitly.

## Workflow Follow-Up

- Full native Linux and Windows validation and hosted CI follow source closure under the delivery authorization.
- Archive this change after delivery and fill each new capability's Purpose in the main specs.
