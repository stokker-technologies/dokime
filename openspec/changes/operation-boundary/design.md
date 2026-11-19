# Operation Boundary Design

## Context

See proposal.md for motivation and the locked owner decisions. Harmos's `operation-boundary`
change owns the upstream contracts (compound mutation, deletion, storage before full
startup, maintenance, expression transforms, statistics consolidation, bucket summaries,
the facade and private host operations). This design records how Dokime adopts them.

Current state (landed, verified 2026-11-16 in this tree; cite paths, not lines):

- Registration is landed: `crates/dokime-runtime/src/services/composition.rs` opens the
  product backend, constructs a Harmos `StorageManager`, installs it in settings and registers
  the storage catalog. Settings and plot settings already use `GetObject`, `SetObject` and
  `ListColumnRows` (`crates/dokime-operations/src/host/storage/administration.rs`); native
  transfers use operations (`crates/dokime-operations/src/host/files/`).
- Direct `Store` consumers remain across `crates/dokime-storage/src` (store, cycles,
  notes, history, client reads, resource values, evidence positions, administration,
  telemetry reader/query, stream windows, expressions, document values, plot selection,
  plot windows, plot kinds, content, report), `crates/dokime-runtime/src` (execution,
  execution manager, run control, capabilities, host, streams, documents, prompts, reports,
  access credentials, root claim) and `crates/dokime-operations/src/host` (resources, views,
  execution state, cases, cycles, executions, evidence, documents, plots, prompts,
  reports, content, every `watch.rs`, backend overview).
- Duplicated kernels: moments, ranks and persisted summaries in
  `crates/dokime-storage/src/telemetry/aggregates.rs`; adaptive envelope merge in
  `telemetry/previews.rs`; scanned bins in `stream_windows.rs`; statistics, histogram and
  Tukey boxes in `plot_windows.rs`; expression arithmetic and filters in `expressions.rs`.
  Native payload reconstruction uses Harmos's exported
  `harmos-storage/src/writes/convert.rs::encode_payloads`; no product encoder remains.
- Offline paths: desktop archive/disposal in
  `crates/dokime-gui/src-tauri/src/native_session/storage.rs`; CLI doctor copies in
  `crates/dokime-cli/src/commands/doctor.rs`.
- Current-pin cutover dependency: `Backend::open` registers a `storage.sqlite3`
  ledger, whereas product execution/evidence/sample/seal/report writers use
  `execution.sqlite3`. Moving an append alone splits their canonical history and
  invalidates cuts and seal verification. The coupled migration needs Harmos
  compound atomic mutation at the backend boundary, now provided by Harmos
  `6978e65f1d505c2fe805a34a2781e48ea6f30350` and implemented in
  `crates/dokime-storage/src/backend/compound.rs`. Product callers still need
  their coupled cutover; retaining two writers or
  adding a mirror is not an acceptable intermediate design. Independent retained
  views in `host/views.rs` now use Harmos objects for immutable file indexes and native
  file operations for complete protobuf payloads, including collections larger than one
  response bound. Their evidence cut issuance remains coupled to the product ledger.
- The native backend commits compound effects and applied/refused receipts together,
  refuses partial compound cuts, and implements object/scoped tombstones with retained
  reads and file reclamation. Harmos's public backend conformance kit runs in
  `crates/dokime-storage/tests/integration/backend.rs`. Storage-only assembly and
  maintenance operations are available at the current pin. The product backend's
  consistent-copy/disposal hooks and their CLI/desktop consumers remain open.
- Planned-case ownership is `host/cases/storage.rs`: registration replay, atomic
  publication, frozen plans and deletion use native object operations. Its consumers
  are planning, cases, execution-state projections and execution admission. The SQL
  `plans.rs` API and `cases`/`registrations` tables are deleted; execution bindings
  remain owned by the coupled lifecycle cutover. Existing development roots with
  those tables are refused at baseline v1 and preserved.
  Registered `Case`/`Plan` shapes now live in `crates/dokime-storage/src/persisted.rs`;
  cases use exact C scopes and `DeleteScope(C)`, while registration replay belongs
  to Y or R and preserves original frozen case bytes independently of deletion.
- Execution command receipts and retained command cuts use native object operations
  in `host/executions/commands`. The execution manager owns admission through durable
  publication and scheduling even if the client stops observing. Each object retains
  the original native mutation identity and input for operation-owned receipt replay. Startup recovery
  runs after binding the storage context and interrupts unfinished receipts. Command
  cuts follow native retention and refuse gaps with resynchronization guidance. Command
  objects carry the execution scope; roots holding superseded command journals
  are refused and preserved at baseline v1. Execution state and evidence remain on
  the coupled lifecycle cutover path.
- Composition: `protocol/composition.toml` is read by `services/composition.rs`
  (`setup.composition`), `native::callers()` in `crates/dokime-operations/src/native.rs`,
  `sdk/cpp/tools/generate.py`, `sdk/csharp/tools/generate.py` and
  `tools/python/src/dokime_tools/build/resource_roots.py`.
- The evidence-family inventory is `crates/dokime-runtime/src/streams.rs` and its
  consumer registration in `crates/dokime-runtime/src/services/composition.rs`;
  `crates/dokime-operations/src/resources.rs` owns evidence appends used by timing
  and plugin host operations; `crates/dokime-operations/src/host/evidence/` owns evidence
  queries and watches. The record/sample/traceability paths still depend on product
  execution cuts, step boundaries and seal verification. The statistics reader stays
  coupled to the pending **consolidated statistics** kernel; archive engine types
  remain blocked on the **typed facade**. Notes already persist through Harmos
  administration objects independently of that ledger, so their list/watch source
  can move to native cuts and `WatchRows` without splitting evidence history.
  Those administration values now persist explicit E/Y scopes. Owner validation,
  open-owner admission, atomic events and execution deletion still move together
  with the lifecycle contract before the remaining direct owner queries can be removed.

Touched invariants in `docs/architecture.md` and their purposes:

- Invariant 7 (Library Machinery and Product Policy): "Compose library services directly,
  without redeclaring or forwarding them." Purpose: avoid duplicated generic machinery and
  product policy in the library. The owner amended it on 2026-11-16: Dokime composes Harmos
  through the typed in-process facade over native operations (D5) and declares wrappers that
  forward to it (D6), reversing "directly" and the forwarding clause. The purpose holds
  because the facade adds no second policy authority and wrappers contain no generic
  machinery, only exposure. The same quote in Final Library Assembly and Storage Contract
  changes with it.
- Invariant 1 (Executors and Callers): purpose "plugin callers cannot acquire client
  administration". Preserved by the `callers` option on Dokime's own wrapper methods.
- Invariant 6 (Managers Own Live Orchestration): library managers stay the live owners but
  become internal to Harmos; Dokime drives them only through operations.

## Goals / Non-Goals

### Locked Storage Source Inventory

The storage delivery owns the following complete source sets. Directory entries
include their descendants and consumers; the operation-boundary task numbers are
the implementation owners. The current worker owns storage, runtime, operations,
CLI doctor and native session storage, together with their proof and maintained
guides. SDK, protocol, composition and facade work is inventoried here for its later
delivery; it is not permission to begin another open change.

| Owner | Source Set and Required Replacement |
| --- | --- |
| 1.6–1.7 | `crates/dokime-storage/src/execution.rs`, `persisted.rs`, `backend.rs`, `backend/**`; `crates/dokime-operations/src/host/storage/**`; `crates/dokime-runtime/src/services/composition.rs`: persisted objects, frozen schemas, exact scopes, durable evidence-cut ownership and native backend registration. |
| 2.2–2.4 | Storage `store.rs`, `cycles.rs`, `history.rs`, `resource_values.rs`, `evidence_positions.rs`, `outcome.rs`; runtime `execution.rs`, `execution_manager.rs`, `run_control.rs`, `capabilities.rs`, `host.rs`; operations `host/{cases,cycles,executions}/**`, `host/execution_state.rs`, `host/projection.rs`: lifecycle, bindings, attributed outcomes, seal and deletion policy. Command persistence in `host/executions/commands/**` also adopts exact E. |
| 3.3–3.5 | Runtime `streams.rs` and composition's sample consumer; operations `resources.rs`, `host/evidence/**`; storage `notes.rs`, `telemetry.rs`, `telemetry/{codec,writer,reader,catalog}.rs`, `ipc.rs`, `ipc/**`: admitted columnar intake, records, notes, traceability, health, native row watches and frozen decoding. Physical Arrow/catalog code survives only under the backend. |
| 4.4–4.5 | Runtime `documents.rs`, `prompts.rs`, `reports.rs`; operations `host/{documents,prompts,prompt_definitions,reports,content,files}/**`, `host/views.rs`, `host/catalog_inventory.rs`, `host/record_schemas/**`, `host/evidence/archive.rs`; storage `content.rs`, `content_name.rs`, `document_values.rs`, `report.rs`: source reads at C, explicit publication and byte-preserving archives. |
| 5.1–5.2 | Storage `query.rs`, `client_reads.rs`, `expressions.rs`, `scalar.rs`, `plot_{selection,kinds,windows}.rs`, `stream_windows.rs`, `telemetry/{aggregates,previews}.rs`; operations `host/{plots,aggregates}/**`, `host/evidence/statistics.rs`, document plot/expansion readers: native row plans and library statistics/bucket kernels; retain product presentation algorithms. |
| 5.3 | Every operations `host/**/watch.rs`, including `backend/overview`, `secrets`, `documents`, `plots/snapshots`, `content/execution_files`, lifecycle and evidence watches; `host/backend/overview/get.rs`, `host/secrets/**`, `host/plugins/secrets/**`, `host/mod.rs`: native durable sources/frontiers, separate live observation, row counts and secret metadata. |
| 6.1 and Storage Deletions | Storage `administration.rs`, `store.rs` and every superseded module above; runtime `services/{mod,state,root_claim,access_credentials,composition}.rs`; CLI `commands/doctor.rs`; desktop `native_session/storage.rs` and its `connection/{host,registry}.rs` / `app/closing.rs` consumers: storage-only admission/maintenance, no second catalog, journal, cut issuer or notification source. |
| Later Facade/Wrapper Delivery | Workspace Harmos dependencies; runtime services/state, execution/dispatch and operations/native; `protocol/composition.toml`; SDK C++/C# `tools/generate.py`; tools `build/resource_roots.py` and `build/resources/**`; product host declarations, embedded descriptors and generated consumers under all four SDKs and desktop generated host modules. Client-used library resources are Access Listener/Pairings/Certificates, Status and its Logs/TraceEvents/Diagnostics, Operations, Invocations, Schemas and Streams (D6). Plugin management belongs to `plugin-boundary`. |
| Proof | `crates/dokime-storage/tests/integration/**` including the native conformance kit; operations unit tests; runtime `tests/{integration,system,support}/**`, especially lifecycle/outcome/isolation/interruption, evidence/notes/traceability/progress, prompts/documents/reports, content, plot/query, retained/restart, health/secrets and root-admission stories; CLI doctor and native storage tests; runtime `benches/storage/**` and existing plot/document benchmarks. Replace superseded fixtures/assertions with their operation-owned equivalents. SDK/protocol/generated tests follow the later wrapper delivery. |
| Maintained Guides | `docs/architecture.md`, `docs/execution-host.md`, `docs/product-guide.md`, `docs/query-api.md`, `docs/gui.md`, `docs/tooling.md`, SDK guides and package READMEs, root `README.md`; change only affected contracts, layouts, commands and examples. |

No new owner is introduced by this inventory. In particular, native backend
registration is the only surviving direct backend consumer. Store's SQL tables,
product receipts/cuts, telemetry catalog and duplicated kernels are deletions,
not compatibility paths. The facade remains deferred at the pinned revision.

### Content and Files Source Inventory

The content family spans `host/content`, `host/evidence/archive.rs`, `host/reports`,
`host/documents`, `host/plots/snapshots`, `host/prompts`, `host/prompt_definitions`
and the overview counts in `host/backend/overview/get.rs`; their runtime owners
are `documents.rs`, `reports.rs` and `prompts.rs`. Storage readers in
`content.rs`, `report.rs`, `document_values.rs`, `expressions.rs`,
`plot_selection.rs` and plot algorithms move only where those consumers require
them. CLI `commands/doctor.rs` and desktop `native_session/storage.rs` compose
storage-only maintenance. Shared Store modules remain while other families use
them. Consolidated statistics, the bucket-summary kernel and typed facade remain
upstream gaps; expression transforms and storage-only maintenance are available
at Harmos `6978e65f1d505c2fe805a34a2781e48ea6f30350`.
Content metadata now uses scoped administration objects under `content/artifact/`
and `content/execution-file/`. Artifact commits preserve their original serialized
payload and native mutation identity; logical-file retention scans all object
revisions. Their execution-open guard still calls the coupled execution owner,
so atomic seal/deletion coordination and native cut issuance remain part of the
required shared source cutover, rather than being inferred from passing content
transfer stories.

**Goals:**

- `crates/dokime-storage` holds only the backend implementation (SQLite catalog, Arrow IPC,
  recovery) and product persisted shapes; no consumer calls it except through registration.
- One aggregation implementation (Harmos); Dokime keeps product algorithms and presentation.
- Every remotely callable operation of the Dokime host is declared in `proto/dokime`.

**Non-Goals:**

- No change to report, plot, document or query semantics beyond the sources they read and
  the envelope clearing at seal.
- No storage migrations or compatibility readers; incompatible roots are refused and the
  user chooses a fresh root.
- GUI pairing, preferences and capture folders remain shell persistence outside evidence
  storage. OS credential policy stays product-owned; only its persisted journals move.

## Decisions

### D1. Adoption Order (Agreed Storage Contract)

The six ordered slices in tasks.md replace the earlier independent-family migration:
persisted shapes/catalog/scopes and retention → lifecycle/seal/deletion → intake/evidence
and native row watches → documents/prompts/reports/content/archive ownership →
queries/previews and remaining watches → bootstrap/final removal and facade adoption.
Slices 2–5 switch together on the combined integration branch: migrate the canonical
writers, readers and cut issuance in one cutover, with no dual writer or mirror.
Already-landed independent object/file work remains groundwork, not proof of that cutover.
Each slice adopts its required Harmos capabilities through an integration-only local
override while unpublished, replaced by the published revision before final validation.

### D2. Product Policy Over Generic Verbs (Proposed)

Execution, case, cycle, seal, report and planning policy stay in Dokime and compose:
`SetObject`/`GetObject` for single records of state; the compound mutation for multi-object
transitions (cycle membership, seal plus report publication, conditional state changes); scoped
deletion with the execution or case scope for deletion; `AppendRecords` and
`AppendSamples` with trusted execution scope for evidence (acknowledgement only after
completed durable coverage); `ListRows`/`ListColumnRows` and snapshots for reads;
`WatchRows` for row sources and native file watches for bytes/production (D8).
Alternative rejected: keeping a product transaction layer in the backend, which is the
bypass.

### D3. Statistics and Product Algorithms (Proposed)

Product expressions compile into Harmos query plans (filter, project, scale, offset,
contains, aggregate, histogram, box summary, downsample, time merge). Sigma, contour, grid,
FFT and presentation calculations consume rows returned by operations. The GUI pixel
reduction and HTML pie and stack totals are presentation arithmetic and stay.
`crates/dokime-operations/src/host/aggregates/get.rs` (currently
unimplemented) is implemented over `list_rows`.

### D4. Stored Envelope and Seal (Decided by the Owner, 2026-11-16)

The zoom envelope stays stored by Dokime. Zoom coarsening is a Dokime product operation that
calls the Harmos bucket-summary transform with the stored summary and its source cut, then
persists the result through Harmos object or file operations. Dokime's own merge kernel in
`telemetry/previews.rs` is deleted. Eligibility rules (projections, `after`, cuts earlier
than the envelope, unsaved tails) remain product policy.

Sealing clears the stored envelope and every preview (time envelopes, scatter occupancy
grids, group counters, persisted summaries) in the compound mutation that publishes the
seal. D8 distinguishes the frozen evidence cut C from that mutation's publication cut S;
previews are excluded from the evidence digest. Reports can be created after seal, so
reports and full-range plots read source data at C through on-demand Harmos bucket
summaries. Rendering does not publish. `report-previews` (Frozen Evidence Cut,
Incremental Plot Previews) keeps its behavior: previews are maintained during execution
and sealed views reconstruct source data at C.
Consequence to measure: on-demand full-range cost over multi-day sealed histories; report
it from the existing plot and document benchmarks, without changing any budget.

### D5. Facade Adoption (Proposed; Facade Shape Decided by the Owner, 2026-11-16)

Dokime builds the runtime through the Harmos builder with its backend pair, catalog and
settings and calls Harmos only through the typed in-process facade over native operations
(Harmos `operation-boundary` D6):

```rust
let harmos = runtime.for_call(&ctx);
harmos.storage().append_samples(AppendSamples { mutation, scope, batch }).await?;
harmos.plugins().load(Load { kind, placement: Managed { command }, settings }).await?;
```

Each facade method dispatches its Harmos operation with the registered policy; typed values
pass without encoding; no Harmos manager or engine type is reachable. Dokime keeps the
runtime handle instead of the operation manager (`services/state.rs`), activates plugins,
binds clocks and starts and stops samples through facade calls (`execution.rs`,
`dispatch.rs`), imports backend contracts
through the authoring module and removes direct dependencies on `harmos-domain`,
`harmos-query` and `harmos-storage`. The direct `harmos-protocol` dependency goes in group 13,
after callers stop using `harmos_protocol::host` paths; Harmos payload values then reach
`dokime-protocol` through the `harmos` SDK root. The filesystem publication helper used by
`crates/dokime-gui/src-tauri/src/parameters.rs` becomes product-owned.

### D6. Product Wrappers and Composition Removal (Proposed)

Dokime wraps, under `proto/dokime/host/v1` nested resources and following the protocol
source convention, each Harmos host operation that a Dokime client (desktop, CLI, Python, C#,
C++ SDK, stories) calls today. Candidates from the current tree:

| Harmos Resource | Methods Used by Dokime Clients | Audience |
| --- | --- | --- |
| Access.Listener, Access.Pairings, Access.Certificates | Get, Set; Create, List, Delete; Create | CLIENTS |
| Status, status.Logs, status.TraceEvents, status.Diagnostics | Get, Watch; List, Watch; List, Watch; Get | CLIENTS |
| Operations, Invocations | List, Get; Create, Get, Cancel, Watch | CLIENTS |
| Schemas, Streams | List; List | CLIENTS |
| Time | Get, Sleep | CLIENTS only if a client calls it |

`plugin-boundary` moves Plugins and plugins.Settings operations, contracts and
SDK projections to Dokime as product policy over Harmos activation/storage calls.
They are not wrappers around a Harmos registry or installation service.

Decided by the owner on 2026-11-16: wrappers declare their audience with the `callers` option
on their own methods, and plugins reach time and their own settings through the Harmos plugin
protocol facilities and need no Dokime wrapper. A method no Dokime client calls is not wrapped. Each wrapper is a
declaration plus a handler that maps product values and calls the typed Harmos
facade; caller audience is enforced once, by the library's operation admission from each method's declared callers,
and the product adds only execution-scope policy for plugins. Wrappers reuse the Harmos request and response messages as
their payloads (Dokime services, Harmos messages), so no twin message is declared; the
generated clients contain those messages but no Harmos service. Under the later
`plugin-boundary` amendment, reuse is internal where needed: plugin authors see
only Dokime values and never Harmos types. Product administration payloads are
Dokime-owned. Own-settings facilities resolve product values through the registered
kind's hook; generic time, invocation and admitted discovery mechanics remain in
Harmos helpers over carrier host-call frames, with no lost capability.

`protocol/composition.toml`, `setup.composition`, `native::callers()`, the TOML reading in
both SDK generators and `resource_roots.py` are deleted. Generators read
`crates/dokime-protocol/src/descriptors.bin` and project client and plugin surfaces from the
`callers` option on Dokime's own methods.

### D7. Carrier Helpers and Dokime Author SDKs (Owner Amendment, 2026-11-16)

`plugin-boundary` supersedes the earlier library-owned author SDK assignment.
Dokime owns its contract and the only author-facing Rust/Python/C#/C++ SDKs,
registrars and starters. Harmos supplies thin internal carrier helpers for
transport, cancellation and windows, shared descriptor/codegen tooling, supported
guest ABI and generic conformance. Harmos types, attributes, namespaces, errors
and escape hatches are invisible to authors; no import aliases or copied engines.
Dependencies resolve automatically. Richer product renderers/fixes move upstream;
one self-contained contracts archive ships per release with import-closure proof.

Harmos host-client roots and host RPC clients disappear once host operations become
in-process only. Dokime clients derive from its declarations and wrappers (D6).
Installation, registry, bundles, persisted settings and management operations are
Dokime policy. The registry explicitly loads each installed plugin at startup
through the typed facade. A registered kind supplies admission/settings-resolution
hooks; Harmos retains the internal activation manager. No `PluginBackend` trait.
Coordinate one final regeneration with both other changes; generated output stays
tracked. See `plugin-boundary` for the ordered transfer and preservation rules.

### D8. Storage Mapping Contract (Agreed; Not Implemented)

Preserve invariant 17, **Execution Evidence Outlives Live Runs**, invariant 19,
**Rendering Does Not Publish**, and invariant 7's "Product transitions and seal policy
compose Harmos compound mutations." Their purposes are durable evidence independent of
live orchestration, explicit report publication and one physical transaction owner.
One authoritative Harmos history replaces the product execution ledger together with
its writers, readers and cut issuance; an append-only migration would split that history.

#### Scopes and Identities

`R=root/<backend_id>`, `C=case/<case_id>`, `Y=cycle/<cycle_id>` and
`E=execution/<execution_id>` are exact opaque scope IDs, with no library hierarchy.
Scope C below is distinct from evidence cut C. Dokime owns entity IDs, relationships,
event IDs, semantic request IDs and per-stream sequences; plugins own declared
record/stream/document/step IDs. Harmos owns history/positions, object revisions,
mutation windows, snapshot handles and file-session generations.

In the table, `O` is a registered `StoredObject` with explicit `scope()`, `A` an
immutable record schema, `S` a sample schema/stream and `F` a native published file.
Names are agreed persisted shapes, not landed API declarations. Record envelopes keep
step, operation, attempt, host receipt time and typed value; native metadata keeps
scope, source and source time. Register frozen schema graphs and retain protobuf
descriptors/message bindings for product decoding.

| Durable Entity | Harmos Representation and Key | Scope |
| --- | --- | --- |
| Root and Administration | `O Root/backend_id`; typed settings, installed plugins, credential journals and claims keep their identities; protected secrets remain OS-owned | R |
| Case | `O Case/case_id`: frozen planned case, registration identity, optional cycle association | C |
| Plan/Registration | `O Plan/request_id`: normalized selection, fingerprint, ordered case IDs and replay result; bounded large payloads use F | Y if cycle-bound; otherwise R |
| Cycle | `O Cycle/cycle_id`: metadata, sealed flag, attributed outcome counts; `A CycleEvent` for membership/history | Y |
| Execution | `O Execution/execution_id`: immutable case/cycle binding, origin/catalog reference, Open/Closed/Sealed state, outcome and seal metadata; no persisted Run | E |
| Parameters and Steps | `O Parameters/execution_id`, `O Steps/execution_id`; `A ExecutionEvent` for revisions, step boundaries and attempts | E |
| Commands and Progress | `O Command/<execution_id,command_id>`, `O Progress/execution_id`; corresponding `A ExecutionEvent` transitions | E |
| Evidence Records and Logs | `A EvidenceEvent` for host events/logs; typed `A Record/<declaration_id,shape_digest>` for plugin records; `item=event_id`, `source=declaration_id` | E |
| Samples and Streams | `O Stream/<execution_id,stream_id>` freezes binding/closure; `S Sample<Row>/<stream_id>` has `item=execution_id`, consecutive stream sequence, source/receipt times; Arrow is the sole sample payload | E |
| Notes | `O Note/note_id` with author/owner/revision; `A NoteEvent` atomically appended with create/edit; preserve owner/open-owner rules | E or Y |
| Traceability | `O Traceability/execution_id` plus `A TraceabilityEvent` in one conditional mutation | E |
| Health Events | Captured `A HealthEvent/event_id` under its actual owner; live plugin health remains live unless captured | E, Y or R |
| Prompts and Answers | `O Prompt/<execution_id,prompt_id>`, `A PromptEvent`, `A AnswerEvent`; durable answer precedes waking its waiter | E |
| Documents | `O Document/<execution_id,document_id>` for frozen/current definition; `A DocumentPart` for appended parts; large retained views use F | E |
| Prepared/Published Reports | `O PreparedReport/request_id`; immutable `O Report/<subject,report_id>` with document, evidence seal/cut and HTML F reference | E; cycle reports Y |
| Artifacts and Execution Files | `O Artifact/artifact_id`, `O ExecutionFile/<execution_id,logical_path>` and immutable F references; attachment events append atomically with metadata | E; cycle-owned artifacts Y |
| Previews and Envelopes | `O Preview/<execution_id,kind,selection>` includes source cut; every derived envelope/grid/counter/summary object has an enumerable key | E |

#### Cuts and Seal Mutation

Close admission, stop sampling, join cleanup/captures, drain every accepted batch to
durable completion, finalize files, then durably transition Open → Closed. Fence all
execution evidence and preview writers before sealing. Take `CreateSnapshot` at the
resulting durable evidence cut **C**. Digest frozen execution content, evidence/typed
rows and immutable file identities/digests at C; exclude previews, seal metadata and
subsequently published reports. Replace the current preview-inclusive digest in
`crates/dokime-storage/src/store.rs`.

```text
Mutate(seal_request, [
  CAS Execution: Closed → Sealed { evidence_cut:C, digest, retained_cut_owner },
  CAS Cycle: updated attributed summary,
  Append ExecutionEvent::Sealed, Append CycleEvent::ExecutionSealed,
  Delete each Preview/envelope/grid/counter/summary object,
  optional Create Report + Append ReportPublished
])
```

Cycle items apply only to cycle-bound executions. The durable receipt supplies
publication cut **S**; C is the frozen evidence image and S publishes its seal.
Never predict S or embed a self-referential seal. Bound and enumerate preview keys
before admitting the execution so seal/clear fits one mutation; never split it.

Open/Closed reads without `at` select latest durable data without flushing; supplied
issued cuts remain fixed. Sealed evidence, documents, plots and later reports source
data at C; current presentation/report listings may use later cuts. Publish report
bytes through native file operations before atomically publishing metadata/reference.
Joint seal-and-report publication uses the mutation above; failed metadata publication
leaves owned, reclaimable unpublished bytes. Rendering never publishes.

Retain C durably for the execution's lifetime, releasing ownership when deletion
commits. Independent reader leases survive deletion until released, then backend
reclamation removes unreferenced rows/files. **G1** blocks this guarantee today.

#### Watches

| Product Watch | Native Source |
| --- | --- |
| Cases, Cycles, Executions | `Objects(Revisions)` of typed objects; lifecycle events supply membership/removal changes |
| Progress, Traceability, Notes, Prompts | Object revisions and authoritative events; contextual notes additionally select Y |
| Records, Logs, Captured Health | `Records(schema, scope=E/Y/R)`; subscribe separately for requested context |
| Samples | `Stream(schema, scope=E, stream=id, after=0)` |
| Documents and Plot Snapshots | Document/evidence/sample sources; coalesce durable advances and recompute at one issued cut |
| Artifacts and Execution-File Listings | Metadata object revisions and attachment/removal records |
| Administration, Secrets Metadata, Backend Overview | Administration object revisions; compose live runtime/status observation separately |
| File Bytes/Uploads | `WatchFileBytes`/`WatchFileProduction` by file/writer identity and byte offset, outside `WatchRows` |

List/Get the opening image at issued cut K, register watches from K before draining
its pages, then resume exclusively with returned `resolved_at`; sample sequences and
page tokens are not watch cursors. Advance cursors on filtered durable progress too;
project compound changes only at their complete frontier. On gaps/expired cuts emit
`resync_required` and terminate; clients list a fresh image and reopen at its cut.
Preserve shutdown settlement; media capture Watch remains explicitly unsupported.
**G3** blocks complete-frontier observation today.

#### Intake Acknowledgement

Resolve inherited invocation scope through the host's admitted execution binding and
derive E there. Validate activation generation, execution admission, declared
schema/stream and step/attempt context; caller-supplied ownership grants no authority.
Set `AppendSamples.scope=E` / `AppendRecords.scope=E`, populate matching metadata and
reject conflicting authored scope. Batch by execution/schema/destination; replay a
stable mutation identity with unchanged payload.

Advance consumer acknowledgement only when `completed=true` and `durable_at` covers
the batch. Acceptance alone is insufficient; replay the same append to observe
completion. Bound outstanding batches/bytes and retain them until acknowledgement;
storage failure or lag interrupts explicitly. Decode columns once per window, without
per-sample protobuf/JSON/dynamic-message work, locking or SQL. At 128 streams × 1 kHz,
100 ms windows imply 1,280 window batches/s rather than 128,000 product calls/s; this
is workload arithmetic, not measured throughput. **G2** blocks the end-to-end columnar
path today.

#### Deletion Policy

| Owner | Selection and Policy |
| --- | --- |
| Execution | Check ownership/current seal and settle writers/productions; `DeleteScope(E)` removes objects, records, samples, notes, previews and files. Parent summaries derive from surviving execution bindings, avoiding stale copied membership. |
| Case | Preserve unexecuted-only deletion; fence admission, then `DeleteScope(C)`. Executed-case cascading deletion would change the contract. |
| Cycle | No current Delete RPC. A future operation must select execution scopes, cycle-owned case scopes and Y from explicit relationships, never prefixes. |
| Root | Stop/drain, release readers, dispose primary/recovery data offline and clean product credentials; `DeleteScope(R)` alone leaves child scopes and unscoped schemas. |

`DeleteScope` accepts one exact scope and must be the sole mutation item. Atomic
cycle-wide cascading deletion or deletion plus parent updates requires an upstream
extension; sequential scope calls cannot claim atomicity. No new cascade is introduced.

#### Upstream Blockers

| Blocker | Required Capability and Owning Slices |
| --- | --- |
| G1 | Restart-safe, scope-qualified retained cuts (slices 1, 2, 4): current snapshot registries are volatile and deletion raises a global floor. Old seals must not pin unrelated deleted executions. |
| G2 | Columnar append/backend path (slice 3): Harmos `writes/convert.rs` expands Arrow into `StreamInput` values and the product `backend/rows.rs` inserts/hashes each row. |
| G3 | Compound-aware watches (slices 3, 5): Harmos `selection.rs` followers omit compound entries and native `watch_rows.rs` issues per-entry cuts inside forbidden compound ranges. Observe compound/conditional/deletion effects only at complete frontiers. |
| Harmos Groups 6–7 | Consolidated statistics/box summaries and bucket-summary continuation (slice 5). |
| Harmos Group 9 | Typed facade, authoring exports and operation-owned batch DTOs (slice 6, including final archive DTO adoption). |

Harmos `c2cfb153ab136159f07741d1061fcb3a1191e86c` supplies G1–G3,
consolidated statistics/bucket summaries, storage-only maintenance and
scale/offset/contains transforms. The descriptions above record the original
upstream findings; product backend and consumer adoption and proof remain open.
The storage cutover keeps the current operation-calling API. Group 9's typed
facade and its dependent wrapper changes remain a later delivery. The current
pin already supplies operation-owned column kinds and row/IPC result DTOs;
existing administration, retained-content and archive operation callers adopt
those DTOs without introducing the facade.

## Stored Envelope Coarsening at the Pinned Revision

Task 5.2 requires zoom coarsening through the library bucket-summary transform,
using the stored envelope and its source cut. At the pinned revision,
`Harmos:crates/harmos-query/src/statistics/buckets.rs::Buckets::new` requires the
continuation's `max_buckets` and `width_ns` to equal the new plan exactly. A stored
four-bucket envelope therefore cannot continue under a two-bucket zoom bound, even
with unchanged fields/history/cut and no new samples. The operation forwards this
same query transform; a different caller surface does not supply coarsening.

`crates/dokime-storage/tests/integration/backend.rs::library_bucket_continuation_coarsens_a_stored_envelope`
is the deterministic failing consumer proof. The required upstream capability is
coarsening an intact continuation at its existing source cut to a smaller bucket
bound, preserving count, first/last/extrema, moments and resumable continuation.
Do not edit opaque continuation bytes, keep a second merge kernel or reconstruct
raw samples as a substitute for the agreed stored-envelope coarsening operation.
The independent numeric-kernel deletions do not close 5.2. The coupled source switch
in slices 2–5, the dependent Store deletion and complete-cutover documentation
remain open until this capability and their own implementation are complete.

The remaining task closures have these shared dependencies; none is checked off
by the independent backend/query repairs:

| Tasks | Required Completion |
| --- | --- |
| 1.6, 1.7 | Migrate the remaining persisted shapes, frozen bindings and execution-owned retained cuts with their canonical lifecycle/evidence consumers. Shape-only scaffolds do not close these tasks. |
| 2.2–2.4, 3.3–3.5, 4.4–4.5, 5.1–5.3 | Switch the canonical lifecycle, intake, queries, watches, seal and preview owners together. The missing stored-envelope coarsening capability prevents the required complete slice 5 behavior at this pin. |
| 6.1, 6.2a, 6.4 | Finish bootstrap/removal and complete-cutover guides after all canonical consumers replace the Store. Retaining the Store or claiming those consumers migrated would not close these tasks. |

## Snapshot Operation Admission at the Pinned Revision

The pinned revision repairs native snapshot admission: `CreateSnapshot` and
`RetainSnapshot` use operation-owned policies that never retry. Product snapshot
calls retain their existing deadlines and require no caller override. Archive and
execution-owned retention proof must run against this repaired pin.

## Report Capture Receipt Windows

The pinned Harmos revision `e8df78630c1cb401e8fdadf7f50fc578b8f0509e`
resolves evicted mutation receipts through the backend. Product view retention
continues to use the issued storage mutation window with a fresh request identity;
no product retries, serialization or increased receipt limits are introduced.
The merged OBC report now reaches native HTML generation and file delivery.

Scene selection owns its fixture dependencies as well as image capture: only
the complete journey and the cycles scene create the separate active observation
cycle. They retain its readiness, final outcome and seal checks. The report scene
requires the completed execution and its saved report, without starting an
unselected observation.

Report-scene acceptance remains open. Its driver awaits picker/Create/Publish/Get/
download in one WebDriver async script with the unchanged default 30-second bound;
the final merged-candidate run exceeded that bound. The native-desktop driver
submits the save and observes completion separately under its existing journey
policy. These competing workflow shapes need an owner decision: prefer the desktop
submission/observation shape, preserving every product operation deadline, rather
than increasing a timeout. This changes the check's effective workflow bound and
requires approval under the task's no-lengthening rule. Keep the existing bound
until that decision; a prior successful capture does not replace final proof.

## Administration Prefix Predicate at the Pinned Revision

Task 4.4w uses library revision `05bd25a017a54b8af7f53553927e6af24054723b`
for all Rust workspace dependencies and the Rust SDK consumer. `Cargo.toml` pins
`harmos`, `harmos-domain`, `harmos-operations`, `harmos-packaging`, `harmos-protocol`,
`harmos-query`, `harmos-runtime` and `harmos-storage`; the consumer manifest pins
`harmos-protocol` and `harmos-runtime`. Both Cargo locks are refreshed through Cargo;
`harmos-sdk-derive` follows the new source transitively. The Python tooling
pin remains `e8df78630c1cb401e8fdadf7f50fc578b8f0509e`; the Python, C# and C++ SDK
pins remain `c2cfb153ab136159f07741d1061fcb3a1191e86c` under their separate owners.
The library's single `ColumnPredicate` now carries either numeric bounds or a
nonempty case-sensitive text prefix, validates column kinds, and binds the predicate
to continuation identity. No product predicate family is introduced.

`crates/dokime-operations/src/host/storage/administration.rs::scan` passes the prefix
on `_storage.item`, the durable native key whose equality to `Administration.key`
is covered by the envelope round trip. It no longer filters decoded payloads.
`crates/dokime-storage/src/backend/columns.rs::scan` validates the predicate against
the schema before pruning, then checks the existing `storage_rows.item` metadata
with a byte-exact prefix comparison. Object chunks contain one key, so rejected
chunks never open an Arrow payload file. This needs no schema change or migration.
The shared library `finish_page`/`finish_scan` owns row filtering, page bounds and
continuation validation; output projection follows durable key pruning. Payload-projection visits
are reported through `ArrowPage.payload_rows`, including physical chunk visits.
No cache crosses requests and no bound, setting or operation deadline changes.

The six call sites in five files still enumerate collections: legacy artifact
presence, committed artifacts, execution files, command recovery, notes and latest
prepared report for a cycle. None is an exact-key lookup; named report Get already
uses keyed administration access. Their public result ordering remains key-sorted.
The deterministic backend fixture compares the old full-scan/filter result with
prefix selection across six mixed prefixes, Unicode and literal wildcard bytes,
three revisions per key, page boundaries and a later matching write beyond the
issued cut. It compares ordered entries and continuation positions, checks token
predicate binding and wrong-kind refusal, and counts rejected payload exclusion.
An absent-prefix read also runs with the Arrow directory unavailable. The focused
backend selection passes 18 tests, including this equivalence story. A full scan
projects 72 revision payload rows; each matching prefix projects 12 and an absent
prefix projects zero. Entries, page-position progression and the issued cut agree.

### Payload Storage Overlap for Owner Review

`crates/dokime-storage/src/backend/rows.rs::{append_object_revisions,append_records}`
and `backend/objects.rs::exchange` persist complete encoded envelopes in
`storage_rows.data`, while `backend/columns.rs::store` also persists their payloads
in Arrow chunks. `StorageBackend::{get_object,list_object_revisions,list_records}`
read the SQL envelopes; `StorageBackend::list_column_rows` reads the Arrow copy.
Replay comparison in `rows.rs::insert` and object-history retention also consume the
SQL envelope. Prefer deleting the SQL payload copies after migrating those readers,
replay and retention to one Arrow payload owner, proving original-byte identity and
preserving the existing SQL identity/key metadata. Keep both until the owner decides;
this prefix repair reads only SQL key metadata when pruning and introduces no new
payload store or reader.

### Populated Release Measurement Baseline

The report benchmark now prepares 306 native plot views and their cut indexes
before timing the same eight-section, 116-stream, 118,784-sample, three-record save.
The previous full-save fixture already warmed 116 plot views; the 17-object
administration inventory in the original attribution belongs to its isolated example,
not this full-save fixture. Do not compare those different workloads as a scan repair.
One optimized release iteration produces this pre-prefix-repair baseline (milliseconds):

| Stage | Populated Baseline | After Prefix Repair |
| --- | ---: | --- |
| Execution Files | 2,597.203 | Pending |
| Notes | 1,042.240 | Pending |
| Artifacts | 1,055.980 | Pending |
| Host Create | 6,818.570 | Pending |
| Complete Save | 7,417.382 | Pending |

The task-built release benchmark runs with `DOKIME_BENCH_ONCE=1` and
`--bench --test reports/save`; its observation is in
`target/benchmarks/storage/rscan-release.log`. Prefix selection is unchanged, so
there is no paired after-repair measurement yet.

The release native-desktop story's final run passes nine tests and captures both
generic native report-call events. Thermal contains 2,502 samples and one record.
The supplied prior Thermal attribution and this run give these observations, not
a controlled performance comparison (milliseconds):

| Stage | Supplied Release Observation | Current Release Observation |
| --- | ---: | ---: |
| Execution Files | 148.195 | 196.987 |
| Notes | 124.468 | 148.929 |
| Artifacts | 166.479 | 190.125 |
| Host Create | 571.076 | 709.428 |
| Complete Save | 4,627.801 | 3,716.160 |

The current picker takes 2,053.517 ms versus 3,316.512 ms previously; a smaller
total does not establish a host speedup. Current native admission totals 0.0021 ms
and RPC overhead 3.0639 ms. Evidence lives under
`target/tests/release/windows-x86_64/system-typescript/native-desktop-2XcfS0/`
and `target/native-desktop-rscan-release.log`.

Previously release fixture producers staged debug Rust SDK/host/content/plugin
and desktop CLI bytes, while the shipping GUI was release. Rust test harnesses
and their native host supplied through `NEXTEST_BIN_EXE` intentionally use the
unoptimized test profile; they are distinct from those auxiliary fixtures.
The reference conformance host also explicitly installed with the debug profile.
All these auxiliary producers now select the requested profile and fail if its
binary is missing. C# defaults to Release, C++ uses its release preset, and pure
Python has no compiler profile. The supplied attribution explicitly selected actual
release executables, so the producer defect does not explain that measured gap.

The final native run also proves the helper's correction for the formatter's quoted
operation field. Setup failures from missing prepared tooling/driver and concurrent
frontend/host generation were REAL preparation errors; sequential preparation and
the final checks pass. No test, deadline or bound was weakened. The explicitly
requested release measurements and scoped Rust lint take precedence over the generic
worker prohibitions on release profiles and lint.

## Browsing View Retention Ownership

`crates/dokime-operations/src/host/views.rs::read` retains a native file index per
resource/content digest and a separate `view-at` index per resource/evidence position.
Both carry the supplied resource scope. `retain` removes only a newly produced file
that lost a publication race; no normal read, page completion, disconnect or shutdown
releases a published view or either index. Administration uses `ObjectRetention::All`.
This explains growth during a desktop session and across restarts: viewport selections
change resource keys, response changes create content revisions, and new evidence
positions create additional cut indexes even for unchanged response bytes.

The native backend's `backend/compound.rs::delete_scope` can tombstone scoped objects
and files. `host/cases/storage.rs::delete` already calls native `DeleteScope` for C,
so deleting an unexecuted case releases that case's view ownership; `cases/get.rs`
and `cases/parameters.rs` assign their views to C. `host/executions/delete.rs` still calls the SQL ledger's
`history.rs::delete_sealed_execution`, which deletes SQL `resource_values` and
`resource_requests`, not native view indexes or native files. This is a lifecycle
REWORK finding, not permission to add a second deletion path. Native scope deletion
and SQL execution deletion coexist while the coupled storage cutover remains open;
prefer deleting the SQL execution cleanup after migrating lifecycle, seal and archive
callers to the native scope transaction.

No request/session lifetime is defined for the durable cuts: `read` intentionally
reopens indexed views and refuses unavailable cuts with a resynchronization error.
The owner must choose between explicit client/session release (requiring ownership
and cut invalidation semantics), request-only views with explicit durable captures,
or durable resource views with an explicit release operation. Prefer explicit release
of client-owned views while retaining separately owned report/evidence captures, as
part of the coupled view cutover. No retention lifetime, expiry, bound or automatic
deletion is introduced by the administration performance repair.

The same `views.rs::read` has two retained-reference lookup paths: position-only
client cuts select `view-at:{resource}:{epoch}:{sequence}`, while returned cuts with
a resource revision select `view:{resource}:{digest}`. Both resolve a native file
through `retained`; catalog, document, content, plot and report read operations call
this function with their client's cut. Prefer deleting the position-only lookup and
`view-at` index family after canonical cuts and their consumers migrate, subject to
the owner's contract decision. Keep both paths and their existing accepted cuts now.

## Administration Reader Ownership Review

The scan callers enumerate collections: `content/mod.rs::validate_root` checks legacy
artifact presence; `artifact_payloads` lists committed artifacts; `execution_files`
lists an execution's logical files; `evidence/notes/list.rs` selects notes; and
`executions/commands/mod.rs::recover` examines interrupted command receipts.
`reports/get.rs` scans prepared reports only to select the latest one for a cycle;
its named prepared/published report reads already use keyed administration access.
Report capture lists files/artifacts and then uses their existing keyed Get operations.
None of these prefix scans is an exact-key lookup. File/artifact continuation reads
reopen retained views without rescanning; note continuations previously rescanned the
same prefix. Capture now selects notes once and pages that request-local image through
the same page operation, without retaining a cache across Create requests.

| Overlap | Callers and Purpose | Preferred Removal |
| --- | --- | --- |
| `crates/dokime-storage/src/administration.rs` and `crates/dokime-operations/src/host/storage/administration.rs` | SQL keyed protobuf administration serves runtime root claims/access credentials and the secret journal; native administration serves content, notes, commands, views and prepared reports. | Delete SQL administration after migrating its remaining root/security owners with their durable replay and cleanup semantics. Keep both pending the coupled storage cutover. |
| `crates/dokime-storage/src/notes.rs` and `crates/dokime-operations/src/host/evidence/notes/{create,get,set,list,watch}.rs` | SQL note persistence has no current operation callers; native note operations own Create/Get/Set/List/Watch. Native Create/List still call SQL-backed `Store::note_owner` for owner validation. | Delete unused SQL note persistence after the owner's decision; preserve/migrate owner validation. |
| `tools/python/src/dokime_tools/build/{desktop_fixtures,host_fixtures}.py::build` raw executable-path selection and `build/outputs.py::executables` compiler-record selection | Desktop/host fixture tasks map dev to debug and otherwise use the requested profile directory. The native Product builder selects non-test executable paths from Cargo compiler messages through `outputs.executables`. | Prefer deleting the fixture path inference after the owner agrees that fixture builds should emit compiler records consumed by the existing selector. Keep independent Product/Tests builds; do not add a second selector or make Tests depend on Product. |

The storage benchmark's `main.rs` also prepares an unrelated 16,384-row export
fixture even when Criterion selects only `reports/save`. Prefer lazy fixture setup
in that benchmark owner; it is outside this worker's report-fixture ownership.

## Report Retention Ownership Review

The report-creation performance task leaves the listed overlaps intact for the owner.

| Overlap | Current Callers and Purpose | Recommended Removal and Required Work |
| --- | --- | --- |
| `crates/dokime-runtime/src/reports.rs::ReportManager::temporary` and `crates/dokime-operations/src/host/reports/get.rs` durable `PreparedReport::response` lookup | `Reports/Create` calls `Host::retain_report` through `services/state.rs`, which calls `ReportManager::retain`; `Reports/Get` calls `Host::temporary_report` through the same adapter, which calls `ReportManager::temporary`, then falls back to native administration. Runtime shutdown clears the bounded map. Both paths return the same immutable native file reference; the durable path survives eviction and restart. | Delete the runtime map and its `retain`/`temporary` methods, the two Host trait methods and state adapters, and Create's map insertion. Get should always resolve the retained native object. Keep the report manager's publication/admission responsibility until its separate canonical-source migration. |
| `crates/dokime-storage/src/report.rs::Report::html` and Create's native HTML file retained by `PreparedReport::response.file` | `Reports/Publish` downloads the complete native file and passes HTML through `Host::publish_report`, `services/state.rs`, `ReportManager::publish_prepared` and `Store::publish_report`. `Store::reports` decodes and verifies the copied HTML as part of the report seal; `Reports/Get`, `host/execution_state.rs::detail` and runtime `Host::reports` call it. `storage/content.rs::write_execution_archive` writes the complete report payload into `ExecutionArchive::reports`, and storage/history.rs deletes those report rows with their execution. Integrity, publication, document/report and lifecycle stories inspect these bytes. | Delete the copied HTML after replacing published reports with metadata, frozen document, execution seal and native file identity/digest. Update report sealing/validation, atomic publication, archive export/import and all listed readers/tests to carry and restore the native bytes. Merely removing the field would break the archive and seal contract. This is the open canonical-source migration in task 4.4, not a report-performance repair. |
| `crates/dokime-storage/src/query.rs::{cached_statistics,cache_statistics}` and `crates/dokime-storage/src/telemetry/aggregates.rs::numeric_summary` | `host/evidence/statistics.rs::select` retains complete statistics responses and audit positions on the Store. Plot `numeric_rms`, `numeric_statistics`, `time_series_window`, `histogram_window`, `box_window` and document `reduce_expression` use request-local numeric moments. Their keys and response families differ, but both retain numeric results for stream selections. | Prefer deleting the Store-wide response cache during the native query cutover, assembling responses and audit positions over the single finite query owner. Do not remove its retained-cut or audit behavior as a performance shortcut. |
| `crates/dokime-operations/src/host/reports/create.rs` and `crates/dokime-operations/src/host/views.rs::retain` upload loops | Reports/Create publishes temporary HTML through native Create/Append/Publish. `views::read` calls `views::retain`, which publishes encoded resource responses through the same upload operations and retains their references. One producer starts with a temporary file, the other with message bytes. | Prefer deleting Create's separate upload loop after the content owner exposes one bounded publication operation for either byte producer; preserve scopes, chunk limits, digests, cleanup and durable retention. This existing overlap remains unchanged. |
| `crates/dokime-storage/src/plot_windows.rs::{histogram_window,numeric_order_statistics}` and `plot_windows.rs::{box_window,reduce_numeric}` | The Histogram and BoxPlot branches of `host/plots/snapshots/list.rs::snapshot_prepared` request rank work over the same selected field/window through separate native transforms. Histogram keeps an exact native Median; box summaries retain the native box-summary floating-point result contract. | Prefer deleting the histogram's separate rank-input traversal after a shared native rank-input owner can serve both transforms. Preserve their distinct result arithmetic and prove equivalence; do not replace either with a product rank kernel or silently reuse floating-point box medians for exact histogram medians. Leave this to the owner. |
| `crates/dokime-runtime/benches/storage/main.rs::StageTimes` and `benches/storage/reports.rs::Times` | Cold numeric/document benchmarks use StageTimes to total plot-span durations; report benchmarks use Times to retain per-call source/document/plot span durations, sample counts and report debug events. Both measure plot-span lifetimes. | Prefer deleting the separate StageTimes layer after the owner agrees to use Times for cold-stage totals as well; keep both current output contracts until then. |

Landed request-local batching gives native plot batches, report capture and the
document benchmark one selection/computation owner. The singleton preparation API
is removed; its equivalence story uses the same fixed-cut batch operation. Workers
are capped by machine parallelism and the number of distinct plot selections,
independently of case admission. Reports retain original compressed source blocks
through the library's bounded query tables, so different projections keep their
original decode budgets and cut filtering while source bytes are read once per
block. The block map only finds or inserts a per-block value; that value owns
loading and spill publication, so unrelated blocks load concurrently. Retained
blocks spill rather than accumulating past the existing `StorageSettings.query_bytes`
working bound (`telemetry/config.rs`, enforced in `ipc.rs::read_query_buffer` and
`ipc/projection.rs::Projection::check_budget`). Replay borrows the Arrow value
buffer without copying it. Generic source/document/plot diagnostics are debug
operation spans; report stage events are debug-only, with one info event on creation.
The benchmark subscriber collects both spans and debug stage events.
Unprojected numeric summaries share one calculation per selected field,
window and cut; independent fields can compute concurrently. Tables and summaries
belong to the finite read and never to the host Store across Create calls.

## Risks / Trade-Offs

- [Seal clears previews, sealed full-range plots scan source data] → measure with existing
  plot benchmarks over multi-day data; report results to the owner; budgets unchanged.
- [Retained seals, columnar intake and complete watch frontiers lack upstream support] →
  resolve G1–G3 before closing their dependent slices; compound mutation alone is insufficient.
- [Long migration with coupled callers] → slices 2–5 switch together and delete every
  superseded writer/reader/cut path, with no dual writer.
- [Breaking Harmos pins] → adopt each required revision with its consumer repairs; prove
  slices with focused checks and the combined cutover before final validation.
- [Existing development roots become incompatible] → refuse them at baseline v1 with the
  fresh-root action; never reset or delete them.

## Migration Plan

Development storage policy applies: no migrations or compatibility. Implement the six
slices in D1 order; slices 2–5 switch together on the combined integration branch.
Reverting source and its Harmos pin does not make an incompatible root readable;
preserve existing roots and select a fresh root when required.
