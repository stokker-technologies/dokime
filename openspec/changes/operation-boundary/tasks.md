# Operation Boundary Tasks

The agreed storage mapping is design.md D8. These six ordered slices replace the
remaining family-by-family plan. Completed groundwork is retained below; it does
not establish the coupled cutover. Slices 2–5 switch together on the combined
integration branch, with one authoritative Harmos history and no dual writer,
mirror, compatibility reader or independent append migration.

Each slice deletes the superseded paths it owns and proves changed behavior with
focused `mise run build`, `mise run test` and, where needed, `mise run generate`.
Use existing story modules as acceptance targets, with deterministic new coverage
for missing guarantees; no authored-prose assertions or timing assertions. Run only
focused proof until all source items close. Incompatible roots remain preserved at
baseline v1 with an action to select a fresh root; no migrations or automatic reset.
Adopt required Harmos capabilities through integration-only local overrides while
unpublished, then replace them with published pins and refresh locks through
`mise run prepare:locks` before final validation. Generated output stays tracked.

## 1. Persisted Shapes, Catalog, Scopes and Retention

Harmos supplies **G1: restart-safe, scope-qualified retained cuts** at the revision
in 1.2a. Native backend adoption and product evidence-cut ownership remain required;
the existing product seal path does not establish restart-safe scoped retention.

- [x] 1.1 Amend facade/SDK scope and product management ownership for `plugin-boundary`; strict validation of the coordinated changes recorded the documentation agreement, not implementation.
- [x] 1.2 Adopt the additive Harmos revision across Cargo, Python, C++ and C#, refresh locks and regenerate tracked output; compound mutation, exact-scope deletion, storage-only assembly/maintenance and scale/offset/contains transforms are available.
- [x] 1.2a Pin Harmos `ee16bf8a70152b82c881dcccf5f0108a5a859091` across Cargo and all SDK source pins, refresh dependency locks and regenerate tracked bindings. G1–G3 and consolidated statistics/bucket summaries are upstream capabilities; product backend and consumer adoption remain open. The pin also supplies the typed storage facade. This delivery retains operation calls through their owning `harmos-operations` crate; facade adoption remains in 6.2. Snapshot creation and retention now use operation-owned never-retry policies.
- [x] 1.2b Adapt existing administration, retained-content and archive readers to the pinned operation-owned column kinds, IPC batches and row outputs; retain operation calls and defer the typed facade.
- [x] 1.3 Implement native compound commit and object/scoped deletion in `crates/dokime-storage/src/backend`, including atomic replay receipts, refusal of partial compound cuts and retained-reader file reclamation.
- [x] 1.4 Record the agreed entity/scope mapping, C/S seal contract, watches, intake acknowledgement, deletion policy and six-slice dependencies in design.md and the storage model in `docs/architecture.md`; implementation remains open.
- [x] 1.5 Lock the complete source inventory against design.md Context and the owners below: all Store/SQL/Arrow and telemetry consumers outside backend registration, duplicated kernels, manager/engine/internal-crate uses, composition readers, client-used Harmos host operations, tests, generated consumers and maintained guides. Assign discoveries before editing. See design.md, Locked Storage Source Inventory; facade/wrapper consumers remain assigned to their later delivery.
- [ ] 1.6 Define and register every D8 persisted shape and frozen schema graph in `crates/dokime-storage/src/{execution,backend*}` and the product catalog: exact R/C/Y/E scopes, product/plugin IDs, event envelopes, protobuf decoding bindings, Arrow-only sample payloads and enumerable preview keys. Migrate existing independent administration objects to those scopes with their consumers; no prefix hierarchy or persisted Run.
- [x] 1.6a Move scoped `Administration`, `Case` and `Plan` shapes into the product storage catalog; persist exact R/C/Y/E owners for existing object consumers, use exact C deletion for cases and retain registration replay bytes/identity. Refuse the superseded native object format before opening the execution catalog. Remaining 1.6 work includes the lifecycle/evidence/document/prompt/report/preview shapes and frozen plugin bindings; this does not close 1.6 or the coupled cutover.
- [x] 1.6b Give native uploads, artifacts, execution files, reports, archives and retained views exact R/C/Y/E file scopes. Retain each view's immutable indexes and file under its declared source owner rather than its reader, and preserve cross-owner collections under R. Refuse the superseded file/view layout before opening the execution catalog; prove exact-scope file deletion and retained readers with backend conformance and focused operations/runtime stories. Other persisted shapes and frozen plugin bindings remain in 1.6. The two archive stories still stop at the pinned Harmos `CreateSnapshot` replay-safety defect; end-to-end archive proof remains open in 6.5, with no product workaround.
- [ ] 1.7 Adopt G1 and implement durable ownership of evidence cut C for the execution lifetime, release on committed deletion and independent reader leases through deletion/restart. Prove scope-qualified retention and reclamation with focused backend integration tests, including unrelated execution deletion and incompatible-root preservation.
- [x] 1.7a Implement and prove the G1 native backend hooks: durable scoped retention, restart reopening, independent readers, transactional scope-owner release and scope-qualified row/file reclamation. Product execution ownership in 1.7 and seal publication in 2.3 remain open.

## 2. Lifecycle, Seal and Deletion

Depends on **G1** adoption for durable seals and slice 1 shapes. Existing compound mutation
and exact-scope deletion suffice for the other native verbs. This slice switches
with slices 3–5; sealed query/report readers must use the same evidence cut C.

- [x] 2.1 Replace planned-case registration, reads and deletion with native objects/compound publication in `host/cases/storage.rs`; remove the SQL registration API/tables and preserve request replay and incompatible roots.
- [ ] 2.2 Move lifecycle owners in `crates/dokime-runtime/src/{execution,execution_manager,run_control}.rs`, operations `host/{cases,cycles,executions,execution_state}` and storage `{store,cycles,history,resource_values,evidence_positions}.rs` to native objects/events and conditional compound mutations. Keep planning, ownership, attributed outcomes and transition rules in Dokime; remove coupled ledger writers/readers/cut issuance together. Prove focused lifecycle, outcome, interruption and manager-lifecycle stories.
- [ ] 2.3 Close admission, stop samples, join cleanup/captures, durably drain accepted intake, finalize files and persist Closed; fence evidence/preview writers, issue and retain C, then publish one seal mutation at receipt cut S with cycle attribution, lifecycle events, all preview deletions and optional report publication. Digest source evidence and immutable file identities at C, excluding previews/seal metadata/later reports; bound preview keys before admission. Prove atomic refusal/replay, late-write rejection, preview clearing and retained sealed reads without self-referential cuts.
- [ ] 2.4 Delete executions with ownership/seal guards and settled writers/productions followed by `DeleteScope(E)`; derive parent summaries from surviving bindings. Preserve unexecuted-only case deletion through `DeleteScope(C)` and remove superseded `history.rs`/`cycles.rs` deletion paths. Prove absence at new cuts, surviving reader leases and reclamation. Add no cycle Delete RPC or prefix cascade; single-scope deletion cannot be combined with parent updates.

- [x] 2.5 Move execution-command receipt persistence, phase updates, reads and restart recovery to execution-scoped Harmos object operations; retain the original native mutation identity/input for receipt replay and refuse superseded command journals. Prove held command reads while native history is available, command reads after host restart, interruption of unfinished receipts and identical admission receipts after phase advancement/restart with `client_controls_execution_lifecycle`. This is the command journal sub-slice of 2.2; admission's execution-state read and the coupled execution/evidence state cutover stay open.

## 3. Intake, Evidence and Native Row Watches

Harmos supplies **G2: columnar append/backend path** and **G3: compound-aware watches**
at the revision in 1.2a. Product per-row inserts/hashes must be replaced; watch
followers must include compound/conditional/deletion effects and return complete
frontiers. Slice 2 admission, bindings and cut issuance switch with this slice.

- [x] 3.1 Replace the product native payload encoder with Harmos `writes::convert::encode_payloads`, deleting `backend/payload.rs` while preserving bounded/lossless native reads.
- [x] 3.2 Move independent note lists/watches to native object history and `WatchRows`, removing retained-view files/notifications and preserving replay from an earlier issued cut. At the pinned G3 API, reread only at `DurableProgress`, since data frames carry the prior cut. Notes now persist exact E/Y scopes; owner validation, atomic events and execution deletion still await the coupled lifecycle cutover.
- [ ] 3.3 Route runtime `streams.rs`, the composition consumer, operations `resources.rs` and `host/evidence/**`, storage `notes.rs` and telemetry codec/write/read adapters through the D8 objects/records/samples. Derive E from the admitted binding, validate generation/admission/schema/stream/step/attempt, reject conflicting authored scope and preserve frozen decoding. Use atomic note/traceability state plus event mutations and capture health under its actual owner. Prove focused execution isolation, bounded telemetry, notes and traceability stories.
- [ ] 3.4 Adopt G2 and keep intake columnar from window decoding through backend append, with no per-sample protobuf/JSON/dynamic-message work, locking or SQL. Bound and retain outstanding batches/bytes; acknowledge only when `completed=true` and `durable_at` covers the batch, replaying unchanged mutation identity/payload to observe completion. Prove durable replay/restart, bounded admission and explicit failure/lag interruption with deterministic tests; make no unmeasured throughput claim.
- [ ] 3.5 Adopt G3 for evidence/progress/traceability/note/prompt/sample watches: open an image at K, register from K before draining pages, resume only by `resolved_at`, advance filtered durable progress and expose compound effects only at complete frontiers. Gap/expired cut emits `resync_required` then terminates; preserve shutdown settlement. Prove focused progress, traceability and evidence replay/deletion/gap cases; no mirror notifications.
- [x] 3.5a Implement and prove the native backend's `watch_frontier` hook for complete compound boundaries, refusal progress and deletion/history gaps; keep product watch-source adoption in 3.5 open.

## 4. Documents, Prompts, Reports, Content and Archives

Depends on **G1** adoption for retained source reads and slice 2 lifecycle/cuts. Native
sample Arrow files must be exposed through operations for byte-preserving archives.
Final archive operation-owned DTO adoption belongs to slice 6 and Harmos group 9.
This slice switches with slices 2, 3 and 5.

- [x] 4.1 Retain catalog/document/plot/evidence views through bounded native files and immutable object indexes; remove the SQL administration revision/receipt implementation and inline view payloads. Preserve first publication per product cut, oversized paging and incompatible roots.
- [x] 4.2 Move artifact/execution-file metadata, revision-based retained-file protection, archive content metadata and fixed document artifact cards to native objects; remove SQL metadata accessors and refuse superseded journals.
- [x] 4.3 Persist prepared-report metadata and native HTML references through object/file operations; delete duplicate prepared HTML storage and live temporary-file handles, preserving preparation/publication across restart.
- [ ] 4.4 Move runtime documents/prompts/reports and operations `host/{documents,prompts,prompt_definitions,reports,content,files,views}` plus `host/evidence/archive.rs` to the D8 canonical source. Replace required storage `{content,report,document_values}.rs` consumers, source artifacts/documents at C after seal and durably record answers before waking waiters. Publish report bytes before atomic metadata/reference publication; joint seal/report uses slice 2's mutation. Failed metadata publication retains reclaimable unpublished content. Prove focused prompt, document, report and sealed-history stories; rendering never publishes.
- [ ] 4.5 Join content metadata/events and execution-open admission to the native lifecycle/seal/deletion boundary; remove the old `execution_state::read` guard and evidence cut path. Archive execution/evidence/telemetry reads through operations and preserve original sample Arrow bytes, not reconstructed samples. Prove late-content refusal, retained reads, execution-content transfers and restart/archive fidelity.

## 5. Queries, Previews and Remaining Watches

Harmos supplies **groups 6–7: consolidated statistics/box summaries and
bucket-summary continuation**, and **G3** at the revision in 1.2a. Query consumers
switch with slices 2–4; no old-ledger reader remains behind new append operations.

- [ ] 5.1 Move storage query/expression/plot/stream-window modules and operations `host/{plots,aggregates}` onto native row plans (filter/project, scale/offset/contains, aggregate, histogram, box summary, downsample and time merge). Implement `host/aggregates/get.rs`; keep sigma/contour/grid/FFT and document/plot presentation as product algorithms over operation results. Prove focused advanced-plot, plot-kind/window, bounded-telemetry and fixed-document stories.
- [ ] 5.2 Adopt groups 6–7; delete duplicated moments/ranks/persisted summaries/integer folds in `telemetry/aggregates.rs`, scanned bins in `stream_windows.rs`, statistics/histogram/box kernels in `plot_windows.rs` and the adaptive merge kernel in `telemetry/previews.rs`. Persist envelopes/previews with source cuts through native objects/files, coarsen through Harmos bucket summaries with continuation, preserve eligibility rules and reconstruct sealed views on demand at C. Prove deterministic envelope advancement and seal-clearing/read behavior; no duplicate aggregation kernel remains.
- [ ] 5.3 Replace remaining storage notifications in every `host/**/watch.rs`, including documents/plots/content and `host/{backend,secrets}`, with D8 native sources and G3 frontier/resync behavior. Administration/overview counts use row operations; remove direct secrets administration deletion/scans. Compose live status separately, use native byte/production watches for files/uploads and keep media capture Watch explicitly unsupported. Prove focused runtime-state, health, secrets and content-watch stories, including filtered progress and complete compounds.

## 6. Bootstrap, Final Removal and Facade Adoption

Storage-only assembly, maintenance and removal of superseded storage paths use
the current operation-calling API at the pinned revision. They do not wait for
**Harmos group 9: typed facade and authoring exports**. Facade and wrapper adoption
remain a later delivery; coordinate their private-host changes and regeneration
with `plugin-boundary` and `plugin-lifecycle`.

- [ ] 6.1 Move host/services bootstrap, root admission, credential journals/root claims onto the storage-only runtime; keep protected credentials and cleanup policy product-owned. Implement consistent-copy/disposal backend hooks, migrate CLI doctor and desktop native storage consumers and remove direct offline paths. Prove focused root-admission, discarded-root credential, access, doctor and native archive/disposal tests.
- [ ] 6.2 In the later facade delivery, adopt group 9 and build through the Harmos builder with backend pair/catalog/settings. Replace remaining operation-manager/manager/engine use in services/state and execution/dispatch with the typed facade and authoring exports; make filesystem publication product-owned. Storage removal and current operation-owned archive DTOs belong to 6.2a and 1.2b respectively. Prove focused storage/runtime builds, backend integration and manager/isolation/archive stories.
- [ ] 6.2a Complete this storage delivery's deletions through the current operation API: remove the Store facade and every superseded direct storage consumer, execution SQL table/journal, telemetry reader/query entry point and duplicated kernel with its obsolete fixtures/configuration/guidance. Keep the native backend and persisted shapes; defer only the group 9 facade/authoring/wrapper obligations in 6.2–6.3.
- [ ] 6.3 Declare Dokime wrappers for client-used generic operations with explicit audiences and facade handlers; plugin management/settings remain product policy under `plugin-boundary`. Migrate desktop/CLI/four SDK callers, remove internal Harmos crate dependencies and direct `harmos_protocol::host` paths, and enforce the dependency fence. Delete `protocol/composition.toml`, `setup.composition`, `native::callers()` and generator/resource-root readers/tests. Regenerate tracked output once with the coordinated changes; prove focused generated consumers, protocol fixtures and access/status/diagnostics/wrong-caller behavior.
- [ ] 6.4 Update affected architecture, execution-host/caller mapping, product/SDK guides and README claims/examples/links for the complete storage/facade/wrapper cutover. Preserve unrelated documentation; remove superseded guidance and account for every slice 1 inventory item before source closure.

## Final Validation and Delivery

- [ ] 6.5 Replace all integration-only overrides with published Harmos pins, refresh locks and pass `openspec validate operation-boundary --strict`, formatting, lint and focused language/build/test proof on the combined tree. Confirm every implementation item is closed before declaring SOURCE COMPLETE; that declaration is not full validation or delivery.
- [ ] 6.6 After source closure and authorized push, report actual native Linux/Windows and hosted CI/Nightly results and pending proof. Measure sealed full-range cost using existing plot/document benchmarks without changing budgets; benchmark work follows source closure, not the implementation slices.

## Workflow Follow-Up

- Delivery authorization governs push/review/merge; these slices do not grant it.
- Archive the change only after its obligations close, filling new capability Purposes.
