# Operation Boundary Tasks

Harmos `operation-boundary` lands first. Each Dokime group names the Harmos groups it needs
and adopts the revision that landed them: an integration-only local override while that
revision is unpublished, replaced by the published pin before final validation (refresh
locks with `mise run prepare:locks`). Every group deletes the `Store` path or kernel it
replaces, leaves the build green, commits with Conventional Commits and proves only changed
behavior with focused `mise run test`, `mise run build` and `mise run generate` invocations.
Named stories run with `mise run test -- --package dokime-runtime --target system --story
<name>`. Full lanes run only after group 14. Stored shape changes refuse incompatible roots
with the fresh-root action; no migrations or compatibility code.

## 1. Inventory and Contract

- [ ] 1.1 Lock the source inventory against design.md Context: every direct `Store`, telemetry reader, Arrow or SQL consumer outside the backend modules; every duplicated kernel; every Harmos manager, engine and internal-crate use; every composition reader; every Harmos host operation a Dokime client calls. Accept when each item has an owning group below; record surprises in design.md before editing.
- [ ] 1.2 Update `docs/architecture.md`: quote invariants 1, 6 and 7 and the Final Library Assembly and Storage Contract, state their purposes, and rewrite them as locked direction for storage through operations, statistics in Harmos, the stored envelope cleared at seal, the facade and product wrappers; update `docs/execution-host.md` caller mapping accordingly. Prove with `mise run check -- --repository`.
- [x] 1.3 Amend facade/SDK scope and product management ownership for `plugin-boundary`; review both repositories' contracts and pass strict validation of the three changes. Documentation only; implementation stays open.

## 2. Adopt the Additive Harmos Revision

The owner's storage-through-operations-2 task supersedes the former fixed-pin
instruction: all SDKs adopt Harmos
`7fe385eacbe9a4c9a47943f6a528f97a014256ee` (Harmos #272). Verified at that revision in Harmos
`crates/harmos-operations/src/storage/native/{mod,requests}.rs`: `SetSchema`,
`GetObject`, `SetObject`, `AppendObjectRevisions`, `AppendRecords`, `AppendSamples`,
`ListRows`, `ListColumnRows`, `WatchRows`, snapshots and file production/read
operations exist, together with `Mutate`, `DeleteObject` and `DeleteScope`.
ListRows supports filter/project, aggregates, histograms, time buckets, LTTB and
time merge. **Offline/minimal-assembly storage operations**, **expression transforms
scale/offset/contains**, **consolidated statistics**, the **bucket-summary kernel**
and the **typed facade** remain named upstream gaps.

- [x] 2.1 Adopt Harmos `7fe385eacbe9a4c9a47943f6a528f97a014256ee` in Cargo, Python, C++ and C#, refresh their locks and regenerate checked-in output. Prove with `mise run build -- --package dokime-runtime --check` and `mise run test -- --package dokime-storage --target integration`.
- [x] 2.2 Implement compound commit and object/scoped deletion in `crates/dokime-storage/src/backend`. Prove with Harmos's backend conformance kit, receipt replay after restart, refusal of partial compound cuts, physical file reclamation after retained reads release and preservation of incompatible roots.
- [ ] 2.3 Implement maintenance copy/dispose hooks and consistent-copy backend stories after **offline/minimal-assembly storage operations** land upstream.

## 3. Bootstrap and Maintenance

Blocked at the current pin on **offline/minimal-assembly storage operations**: root admission,
access credentials/root claims, CLI doctor copies and desktop native archive/disposal.

- [ ] 3.1 Move root admission (`crates/dokime-runtime/src/host.rs`), credential journals (`services/access_credentials.rs`) and root claims (`services/root_claim.rs`) onto storage operations on the Harmos storage-only runtime, keeping OS credential policy in Dokime. Prove with stories `storage_root_admission`, `credentials_leave_with_discarded_roots` and `client_pairs_and_rotates_remote_access`.
- [ ] 3.2 Replace desktop archive and disposal (`crates/dokime-gui/src-tauri/src/native_session/storage.rs`) and CLI doctor copies (`crates/dokime-cli/src/commands/doctor.rs`) with maintenance operations. Prove with `mise run test -- --package dokime-cli --target system` filtered to doctor and the desktop native session tests for archive and disposal.

## 4. Schemas, Objects and Administration

- [ ] 4.1 Move resources, views, execution state, catalog inventory, record schemas, notes, client reads, resource values, evidence positions and administration onto `SetSchema`, `GetObject`, `SetObject` and `ListRows`; delete their direct SQL. Prove with stories `client_browses_catalog`, `client_keeps_execution_notes`, `client_plans_parameter_selections`, `client_edits_settings` and `client_reuses_frozen_schemas`.
- [ ] 4.2 Remove direct administration deletion in `crates/dokime-operations/src/host/secrets/store.rs` and the backend overview scan in `host/backend/overview/get.rs` (counts through `list_rows`). Prove with stories `client_manages_secret_storage` and `client_inspects_health`.

- [x] 4.3 Retain catalog/document/plot/evidence view payloads through bounded Harmos file operations and immutable object indexes, preserving the first published image for each product cut even when the complete collection exceeds one response bound. Remove the unused SQL administration revision/receipt implementation and inline native view payloads. Refuse superseded roots without deleting their values. Prove retained catalog reads across host restart, oversized paged collections with `client_queries_advanced_plots`, plus focused operations/storage and affected document/plot stories.

- [x] 4.4 Replace planned-case registration, reads and deletion with Harmos objects and compound mutation; delete the SQL registration API and tables, migrate planning and execution-admission consumers, and refuse incompatible roots without resetting them. Prove `client_plans_parameter_selections`, `client_controls_execution_lifecycle` and `storage_root_admission`.

## 5. Records and Sample Ingestion

Compound atomic mutation is available at this pin. Record/sample admission, the
execution/evidence cut owner and seal verification still need their coupled product
cutover; independent operation writes cannot replace only one side of that relationship.

- [ ] 5.1 Route sample ingestion (`crates/dokime-runtime/src/streams.rs`, the product stream consumer registered in `services/composition.rs`) through `AppendSamples` under the trusted execution scope, acknowledging after the durable receipt; route evidence records, traceability and notes through `AppendRecords`. Prove with stories `execution_isolation`, `client_reads_bounded_telemetry` and `client_observes_revisioned_traceability`.

## 6. Compound Mutations and Seal

Compound atomic mutation is available at this pin. Lifecycle transitions,
resource-value/evidence publication, seal plus preview removal and report publication
remain open product implementation work. Sealed bucket-summary reads still depend on
the **bucket-summary kernel**.

- [ ] 6.1 Move execution, case, cycle and progress transitions (`crates/dokime-operations/src/host/{cases,cycles,executions}`, `crates/dokime-runtime/src/{execution,execution_manager,run_control}.rs`) onto `GetObject` and compound mutation, keeping transition rules and planning in Dokime. Prove with stories `client_controls_execution_lifecycle`, `execution_outcomes`, `client_reads_interrupted_execution` and `manager_lifecycles`.
- [ ] 6.2 Make seal one compound mutation that records the seal and clears the execution's stored envelope and every preview; reports and prompts (`crates/dokime-runtime/src/{reports,prompts}.rs`, `host/reports`) use operations, and reports of a sealed execution, whenever created, read the sealed cut through Harmos bucket summaries. Prove with stories `client_publishes_execution_report`, `client_answers_operator_prompts`, `client_browses_sealed_history` and a new story `seal_clears_stored_previews`.

- [x] 6.3 Move execution-command receipt persistence, phase updates, reads and restart recovery to execution-scoped Harmos object operations; retain the original native mutation identity/input for receipt replay and refuse superseded command journals. Prove held command reads while native history is available, command reads after host restart, interruption of unfinished receipts and identical admission receipts after phase advancement/restart with `client_controls_execution_lifecycle`. This is the command journal sub-slice of 6.1; admission's execution-state read and the execution/evidence state cutover stay open.

## 7. Files, Content and Deletion

Native file production/read operations are already composed. Compound mutation and
object/scoped deletion are available at this pin; artifact/execution-file metadata
publication and execution/case deletion remain open product implementation work.

- [ ] 7.1 Move content, artifacts, execution files and report bytes to Harmos file operations and object metadata. Prove with stories `client_transfers_execution_content` and `client_publishes_execution_report`.
- [ ] 7.2 Implement execution and case deletion with Dokime seal and ownership checks followed by Harmos scoped deletion; delete the `history.rs` and `cycles.rs` deletion paths. Prove with a new story `client_deletes_execution_scope` asserting absence at new cuts and retained-cut reads.

- [x] 7.3 Store prepared-report metadata and its native file reference through Harmos object operations; read its bytes through GetFileBytes for publication and delete the duplicate prepared HTML journal and unused live temporary-file handles. Prove publication from a prepared report across host restart with `client_publishes_execution_report` and report-manager lifecycle stories.

## 8. Queries, Statistics and Expressions

The coupled evidence source cutover above blocks replacing its Store readers with
row operations. Additional blockers are **expression transforms scale/offset/contains**
and **consolidated statistics** (including multi-percentile and box summaries).
The pinned Harmos revision already exports
`harmos_storage::writes::convert::encode_payloads`; the native backend uses it
instead of a product payload encoder.

- [ ] 8.1 Compile product expression bindings (`crates/dokime-storage/src/expressions.rs`, `document_values.rs`, `plot_selection.rs`) into Harmos query plans using filter, project, scale, offset, contains, aggregate, histogram, box summary, downsample and time merge transforms; implement `host/aggregates/get.rs` over `list_rows`. Prove with stories `client_queries_advanced_plots`, `client_queries_plot_kinds` and `client_renders_fixed_documents`.
- [ ] 8.2 Delete Dokime moments, ranks, persisted summaries and integer folds (`telemetry/aggregates.rs`), scanned bins (`stream_windows.rs`), statistics, histogram and box code (`plot_windows.rs`). Accept when no aggregation kernel remains in Dokime; prove with stories `client_reads_bounded_telemetry` and `client_queries_plot_windows`.

- [x] 8.3 Replace native Arrow-to-Postcard reconstruction with Harmos `writes::convert::encode_payloads` and delete `backend/payload.rs`; preserve bounded pages and lossless round trips. Prove with the backend conformance kit and native stream read/restart tests in `dokime-storage` integration.

## 9. Previews, Documents and Plots

Advancing persisted envelopes remains blocked on the **bucket-summary kernel**.
Compound mutation can clear every preview together with a seal. Product presentation
remains product-owned; source readers still need the evidence cutover.

- [ ] 9.1 Make zoom coarsening a Dokime operation that calls the Harmos bucket-summary transform with the stored envelope and its source cut and stores the result through Harmos object or file operations; delete the merge kernel in `telemetry/previews.rs`, keeping eligibility rules. Prove with story `client_queries_plot_windows` and a new deterministic story `stored_envelope_catches_up`.
- [ ] 9.2 Source sigma, contour, grid and FFT inputs and document and plot snapshots (`plot_kinds.rs`, `plot_windows.rs`, `host/documents`, `host/plots/snapshots`) through Harmos row operations; full-range plots of sealed executions use on-demand bucket summaries over the sealed cut. Prove with stories `client_queries_plot_kinds` and `documents_and_reports`; report sealed full-range cost from the existing plot and document benchmarks without changing budgets.

## 10. Watches

Evidence/lifecycle watches depend on the same product evidence cutover:
WatchRows can observe the registered ledger, not changes still written to the
product execution ledger. Do not add a mirror notification path.

- [ ] 10.1 Replace product storage notifications in every `watch.rs` under `crates/dokime-operations/src/host` with `WatchRows`, projecting product responses and handling resynchronization at issued cuts. Prove with stories `client_observes_revisioned_progress`, `client_observes_revisioned_traceability`, `client_browses_execution_evidence`, `client_observes_runtime_state` and `client_transfers_execution_content`.

## 11. Delete Direct Store Consumers

- [ ] 11.1 Delete the `Store` facade, telemetry reader and query entry points and every remaining consumer path so `crates/dokime-storage` holds only the backend implementation and product persisted shapes; migrate or remove tests whose code is gone. Accept when no crate outside the backend registration imports storage internals; prove with `mise run build -- --package dokime-storage --check` and `mise run test -- --package dokime-storage --target integration`.

## 12. Adopt the Harmos Facade

Blocked at the current pin on the **typed facade** and its authoring exports.

- [ ] 12.1 Adopt the revision that landed Harmos group 9; build the runtime through the Harmos builder with backend pair, catalog and settings, call Harmos only through the typed facade (`runtime.for_call(&ctx)`) instead of the operation manager (`services/state.rs`), drive plugin activation, clock binding and sample start/stop through facade calls (`execution.rs`, `dispatch.rs`), make the evidence archive decode operation-owned batch DTOs instead of engine types (`host/evidence/archive.rs`) and make the filesystem publication helper product-owned. Prove with stories `manager_lifecycles`, `execution_isolation` and `client_browses_sealed_history`.
- [ ] 12.2 Amend `docs/architecture.md` invariant 7 and Final Library Assembly and Storage Contract, quoting "Compose library services directly, without redeclaring or forwarding them" and replacing it with the owner's 2026-11-16 decision: Dokime composes Harmos through the typed in-process facade over native operations and forwards remote calls through its own wrappers; keep the purpose and record the Harmos SDK scope (design.md D7). Prove with `mise run check -- --repository`.
- [ ] 12.3 Remove direct `harmos-domain`, `harmos-query` and `harmos-storage` dependencies from every manifest, importing backend contracts through the authoring module; add the manifest dependency check covering those crates. Prove with `mise run build -- --package dokime-runtime --check`, `mise run build -- --package dokime-storage --check` and `mise run check -- --repository`.

## 13. Product Wrappers and Composition Removal

- [ ] 13.1 Adopt Harmos private host operations and declare Dokime wrappers for generic capabilities its clients use (D6), with explicit audiences and typed facade handlers. Plugin management/settings are product operations owned by `plugin-boundary`, not library administration wrappers. Migrate desktop, CLI and SDK callers together; prove focused access/status/diagnostics/plugin-management stories and direct-host/wrong-caller refusal.
- [ ] 13.2 Remove the direct `harmos-protocol` dependency once no caller uses `harmos_protocol::host` paths, taking Harmos payload values from the `harmos` SDK root, and extend the manifest dependency check to it. Prove with `mise run build -- --package dokime-protocol --check`, `mise run prepare:locks` and `mise run check -- --repository`.
- [ ] 13.3 Delete `protocol/composition.toml`, `setup.composition`, `native::callers()` and composition readers/tests in SDK generators and resource roots; use the shared Harmos generator with product descriptors under `plugin-boundary`. Coordinate one final regeneration with that change and `plugin-lifecycle`; prove focused generated consumers and protocol-generation fixtures through project tasks.
- [ ] 13.4 Update `docs/architecture.md`, `docs/execution-host.md`, the product guide and SDK READMEs for wrappers, removed composition and the new caller mapping; prove with `mise run check -- --repository`.

## 14. Integration

- [ ] 14.1 Replace every integration-only local override with the published Harmos revision, run `openspec validate operation-boundary --strict`, the format task and the language checks on the combined tree, and confirm every source item above is closed; report pending native platform or hosted proof explicitly.

## Workflow Follow-Up

- Full native Linux and Windows validation and hosted CI follow source closure under the delivery authorization.
- Archive this change after delivery and fill each new capability's Purpose in the main specs.
