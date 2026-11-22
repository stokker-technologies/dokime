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
  history, client reads, resource values, evidence positions, administration,
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
  The command object is the registered `persisted::Command`, not an administration
  value. Its current and original payloads bind to the product's
  `dokime.host.v1.executions.ExecutionCommandReceipt`; original bytes and mutation
  identity never change when a phase advances. Restart recovery consumes bounded
  native pages at one issued cut and interrupts only unfinished commands. A root
  with live commands in the superseded administration envelope is refused rather
  than silently omitting, converting or clearing those commands.
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
  Those administration values now persist explicit E/Y scopes. Owner resolution and
  open-owner policy live in host evidence operations; the unused SQL note API/table
  are deleted. Lifecycle reads, atomic events and execution deletion remain on the
  coupled lifecycle cutover.

Touched invariants in `docs/project/architecture.md` and their purposes:

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
| 1.6, 1.7a–1.7b | `crates/dokime-storage/src/execution.rs`, `persisted.rs`, `backend.rs`, `backend/**`; `crates/dokime-operations/src/host/{storage,executions/commands}/**`; `crates/dokime-runtime/src/services/{composition,mod}.rs`: persisted objects, frozen schemas, exact scopes, native backend registration and inline command-recovery proof. Remaining execution-owned evidence-cut issuance and retention are consolidated into 2.3. |
| 2.2–2.4 | Storage `store.rs`, `cycles.rs`, `history.rs`, `resource_values.rs`, `evidence_positions.rs`, `outcome.rs`; runtime `execution.rs`, `execution_manager.rs`, `run_control.rs`, `capabilities.rs`, `host.rs`; operations `host/{cases,cycles,executions}/**`, `host/execution_state.rs`, `host/projection.rs`: lifecycle, bindings, attributed outcomes, seal and deletion policy. Command persistence in `host/executions/commands/**` also adopts exact E. |
| 3.3–3.5 | Runtime `streams.rs` and composition's sample consumer; operations `resources.rs`, `host/evidence/**`; storage `telemetry.rs`, `telemetry/{codec,writer,reader,catalog}.rs`, `ipc.rs`, `ipc/**`: admitted columnar intake, records, notes, traceability, health, native row watches and frozen decoding. Physical Arrow/catalog code survives only under the backend. |
| 4.4–4.5 | Runtime `documents.rs`, `prompts.rs`, `reports.rs`; operations `host/{documents,prompts,prompt_definitions,reports,content,files}/**`, `host/views.rs`, `host/catalog_inventory.rs`, `host/record_schemas/**`, `host/evidence/archive.rs`; storage `content.rs`, `content_name.rs`, `document_values.rs`, `report.rs`: source reads at C, explicit publication and byte-preserving archives. |
| 5.1–5.2 | Storage `query.rs`, `client_reads.rs`, `expressions.rs`, `scalar.rs`, `plot_{selection,kinds,windows}.rs`, `stream_windows.rs`, `telemetry/{aggregates,previews}.rs`; operations `host/{plots,aggregates}/**`, `host/evidence/statistics.rs`, document plot/expansion readers: native row plans and library statistics/bucket kernels; retain product presentation algorithms. |
| 5.3 | Every operations `host/**/watch.rs`, including `backend/overview`, `secrets`, `documents`, `plots/snapshots`, `content/execution_files`, lifecycle and evidence watches; `host/backend/overview/get.rs`, `host/secrets/**`, `host/plugins/secrets/**`, `host/mod.rs`: native durable sources/frontiers, separate live observation, row counts and secret metadata. |
| 6.1 and Storage Deletions | Storage `administration.rs`, `store.rs` and every superseded module above; runtime `services/{mod,state,root_claim,access_credentials,composition}.rs`; CLI `commands/doctor.rs`; desktop `native_session/storage.rs` and its `connection/{host,registry}.rs` / `app/closing.rs` consumers: storage-only admission/maintenance, no second catalog, journal, cut issuer or notification source. |
| Later Facade/Wrapper Delivery | Workspace Harmos dependencies; runtime services/state, execution/dispatch and operations/native; `protocol/composition.toml`; SDK C++/C# `tools/generate.py`; tools `build/resource_roots.py` and `build/resources/**`; product host declarations, embedded descriptors and generated consumers under all four SDKs and desktop generated host modules. Client-used library resources are Access Listener/Pairings/Certificates, Status and its Logs/TraceEvents/Diagnostics, Operations, Invocations, Schemas and Streams (D6). Plugin management belongs to `plugin-boundary`. |
| Proof | `crates/dokime-storage/tests/integration/**` including the native conformance kit; operations unit tests; runtime `tests/{integration,system,support}/**`, especially lifecycle/outcome/isolation/interruption, evidence/notes/traceability/progress, prompts/documents/reports, content, plot/query, retained/restart, health/secrets and root-admission stories; CLI doctor and native storage tests; runtime `benches/storage/**` and existing plot/document benchmarks. Replace superseded fixtures/assertions with their operation-owned equivalents. SDK/protocol/generated tests follow the later wrapper delivery. |
| Maintained Guides | `docs/project/architecture.md`, `docs/guides/execution-host.md`, `docs/index.md`, `docs/guides/query-api.md`, `docs/guides/gui.md`, `docs/project/tooling.md`, SDK guides and package READMEs, root `README.md`; change only affected contracts, layouts, commands and examples. |

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

### D6. Product Wrappers and Composition Removal (Agreed, 2026-11-19)

The client inventory below is locked for the main-line pin declared in
`Cargo.toml` at `workspace.dependencies.harmos.rev`. Declarations live under
`proto/dokime/host/v1`; each wrapper retains its existing SDK resource and method
name, declares CLIENTS explicitly and reuses the unchanged library messages,
except the product Status responses that add the host build (below).
The handler registration uses `ProtobufOperation::service()` with the product
service descriptor, because the payload descriptor belongs to the library.
Declarations and generated projections do not establish handler implementation.

The `check:repository` task in `mise.toml` invokes
`buf lint --exclude-path proto/harmos` to exclude unmodified third-party input
from the product lint scope; Harmos owns that protocol's lint policy.
`lint.ignore` only hides vendored diagnostics and still counts the vendored RPCs
against product declarations, so it cannot implement this source boundary.
D6 requires unchanged library payloads, including the four Operations List/Get messages also used by
`harmos.plugin.v1.Operations` in `proto/harmos/plugin/v1/operations.proto`.
Linting both authored and vendored RPCs together produced eight
`RPC_REQUEST_RESPONSE_UNIQUE` diagnostics: each request and response at product
List/Get and at library plugin List/Get. Product protocol rules remain enabled,
and comment ignores remain disallowed. No product payload copies are introduced.

The inventory was checked against desktop `connection/host.rs`, its generated
dispatch and `src/lib/api/native-{remote,logs,diagnostics}.ts`; CLI
`commands/{status,pairing,operation}.rs`; `dokime-client-internal`; the four SDK
connection and bound-operation implementations; and their product stories.
Generated methods without an authored caller are not evidence of use.

| Product Service/Method | Audience | Request Message | Response Message | Client Evidence |
| --- | --- | --- | --- | --- |
| `dokime.host.v1.access.Listener/Get` | CLIENTS | `harmos.host.v1.access.ListenerGetRequest` | `harmos.host.v1.access.ListenerGetResponse` | Desktop preferences; CLI pairing export |
| `dokime.host.v1.access.Listener/Set` | CLIENTS | `harmos.host.v1.access.ListenerSetRequest` | `harmos.host.v1.access.ListenerSetResponse` | Desktop preferences; CLI pairing export |
| `dokime.host.v1.access.Pairings/Create` | CLIENTS | `harmos.host.v1.access.PairingsCreateRequest` | `harmos.host.v1.access.PairingsCreateResponse` | Desktop pairing export; CLI export/revoke; remote-access story |
| `dokime.host.v1.access.Pairings/List` | CLIENTS | `harmos.host.v1.access.PairingsListRequest` | `harmos.host.v1.access.PairingsListResponse` | Desktop pairing export; CLI export/revoke; remote-access story |
| `dokime.host.v1.access.Pairings/Delete` | CLIENTS | `harmos.host.v1.access.PairingsDeleteRequest` | `harmos.host.v1.access.PairingsDeleteResponse` | Desktop pairing export; CLI export/revoke; remote-access story |
| `dokime.host.v1.access.Certificates/Create` | CLIENTS | `harmos.host.v1.access.CertificatesCreateRequest` | `harmos.host.v1.access.CertificatesCreateResponse` | Desktop certificate regeneration |
| `dokime.host.v1.Status/Get` | CLIENTS | `harmos.host.v1.StatusGetRequest` | `dokime.host.v1.StatusGetResponse` | Desktop/CLI/four SDK connection readiness; runtime status stories |
| `dokime.host.v1.Status/Watch` | CLIENTS | `harmos.host.v1.StatusWatchRequest` | `dokime.host.v1.StatusWatchResponse` (stream) | Desktop/CLI/four SDK connection readiness; runtime status stories |
| `dokime.host.v1.status.Logs/List` | CLIENTS | `harmos.host.v1.status.LogsListRequest` | `harmos.host.v1.status.LogsListResponse` | Desktop report-save and screenshot journeys page retained logs |
| `dokime.host.v1.status.Logs/Watch` | CLIENTS | `harmos.host.v1.status.LogsWatchRequest` | `harmos.host.v1.status.LogsWatchResponse` (stream) | Runtime diagnostics story; C++ client-read story |
| `dokime.host.v1.status.TraceEvents/List` | CLIENTS | `harmos.host.v1.status.TraceEventsListRequest` | `harmos.host.v1.status.TraceEventsListResponse` | Desktop log adapter; runtime diagnostics story |
| `dokime.host.v1.status.TraceEvents/Watch` | CLIENTS | `harmos.host.v1.status.TraceEventsWatchRequest` | `harmos.host.v1.status.TraceEventsWatchResponse` (stream) | Desktop log adapter; runtime diagnostics story |
| `dokime.host.v1.status.Diagnostics/Get` | CLIENTS | `harmos.host.v1.status.DiagnosticsGetRequest` | `harmos.host.v1.status.DiagnosticsGetResponse` | Desktop diagnostics archive and native host stories |
| `dokime.host.v1.Operations/List` | CLIENTS | `harmos.host.v1.OperationsListRequest` | `harmos.host.v1.OperationsListResponse` | Desktop discovery; CLI operation commands; SDK bound operations |
| `dokime.host.v1.Operations/Get` | CLIENTS | `harmos.host.v1.OperationsGetRequest` | `harmos.host.v1.OperationsGetResponse` | Desktop discovery; CLI operation commands; SDK bound operations |
| `dokime.host.v1.Invocations/Create` | CLIENTS | `harmos.host.v1.InvocationsCreateRequest` | `harmos.host.v1.InvocationsCreateResponse` | CLI and Python/C#/C++ bound operation calls |

This is 16 methods across nine services: Listener 2, Pairings 3, Certificates 1,
Status 2, Logs 2, TraceEvents 2, Diagnostics 1, Operations 2 and Invocations 1.
Logs.List was added on 2026-11-20 (owner decision): the client keeps a way to read
retained logs, because Logs.Watch opens at a boundary and emits only later events
without a snapshot; it delegates to the library's in-process `LogsList`.

Status Get/Watch answer with product responses (owner decision, 2026-11-20): a
client identifies the build of the host it talks to. `dokime.host.v1.StatusGetResponse`
and `StatusWatchResponse` keep the library `HostStatus` at field 1 and the issued
`Position` at field 2, so their wire prefix matches the library responses, and add
`HostBuild build = 3` with `source_revision` (full commit, `-dirty` suffix, or
`unknown`) and `channel` (`release`, `nightly`, empty for local). The values come
from the single build-identity derivation in `crates/build_identity.rs`, emitted by
the runtime build script; the semantic version stays in
`HostIdentity.application_version`. Requests and the health payload stay library
messages. The library status message has no product extension point, so the
product response wraps it rather than editing the vendored `harmos.host.v1.HostIdentity`.

The prior candidate table overstated Invocations.Get/Cancel/Watch,
Schemas.List and Streams.List: no authored product client calls them, so they
are not wrapped. Time.Get/Sleep are plugin context calls only and are not
client wrappers. Pairings.List, Status.Watch, Logs.List and Logs.Watch have real product
story callers even where the interactive applications do not call them.
No used generic method was missing from the candidate table.

Plugin-facing identities move to `harmos.plugin.v1.Time/{Get,Sleep}`,
`harmos.plugin.v1.Invocations/{Get,Cancel,Watch}`,
`harmos.plugin.v1.Operations/{List,Get}` and
`harmos.plugin.v1.plugins.Settings/List`. Their payload messages stay in
`harmos.host.v1` (and its `plugins` package); message reuse does not preserve
any removed host routing identity. The three virtual-clock comparisons in
`crates/dokime-runtime/src/streams.rs` must move with the Time identity.

`plugin-boundary` moves Plugins and plugins.Settings operations, contracts and
SDK projections to Dokime as product policy over Harmos activation/storage calls.
They are not wrappers around a Harmos registry or installation service.

#### Product Plugin Policy Operations (2026-11-19)

The owner requested the missing administration routes as part of this delivery.
Plugin Boundary D1/D2/D4 supersedes generic payload reuse for administration:
these declarations and their administration messages/resources are Dokime-owned,
retaining the existing field numbers and SDK method names. Shared model values
(read cuts, receipts, declarations, settings values and failures) remain Harmos
payloads. The eight Plugins methods and two plugins.Settings methods below are
CLIENTS only. Plugin own-settings reads remain the existing
`harmos.plugin.v1.plugins.Settings/List` facility and the product kind resolver;
there is no second plugin settings read route. Create retains optional `origin`
(field 5); replacement preserves it. Application Settings.Get/Set/Schema is a
separate existing product preference resource, unchanged by this inventory.

| Product Service/Method | Audience | Request Message | Response Message | Authored Callers |
| --- | --- | --- | --- | --- |
| `dokime.host.v1.Plugins/Get` | CLIENTS | `dokime.host.v1.PluginsGetRequest` | `dokime.host.v1.PluginsGetResponse` | Desktop connection/host.rs; CLI plugin/{get,remove,mod}.rs; all four SDK registration/handle readiness and teardown; runtime/desktop support clients |
| `dokime.host.v1.Plugins/List` | CLIENTS | `dokime.host.v1.PluginsListRequest` | `dokime.host.v1.PluginsListResponse` | Desktop generated dispatch and integration/rpc; CLI plugin/list.rs; Rust handle listing and generation fixture; C++ client-read/generation fixtures; runtime installation/support stories |
| `dokime.host.v1.Plugins/Watch` | CLIENTS | `dokime.host.v1.PluginsWatchRequest` | `dokime.host.v1.PluginsWatchResponse` (stream) | Desktop connection/host.rs and tests/support/server.rs; CLI readiness in plugin/mod.rs; Rust/Python/C#/C++ registration; SDK authoring support clients |
| `dokime.host.v1.Plugins/Create` | CLIENTS | `dokime.host.v1.PluginsCreateRequest` | `dokime.host.v1.PluginsCreateResponse` | Desktop connection/host.rs; CLI plugin/{register,install}.rs; all four SDK registration paths; runtime and SDK support clients |
| `dokime.host.v1.Plugins/Set` | CLIENTS | `dokime.host.v1.PluginsSetRequest` | `dokime.host.v1.PluginsSetResponse` | Desktop generated dispatch; client_installs_and_removes_managed_plugins.rs enable/disable |
| `dokime.host.v1.Plugins/Delete` | CLIENTS | `dokime.host.v1.PluginsDeleteRequest` | `dokime.host.v1.PluginsDeleteResponse` | Desktop capture support; CLI plugin/remove.rs; all four SDK registration teardown; runtime installation/support clients |
| `dokime.host.v1.Plugins/Restart` | CLIENTS | `dokime.host.v1.PluginsRestartRequest` | `dokime.host.v1.PluginsRestartResponse` | Desktop generated dispatch; client_manages_plugin_entries.rs, client_browses_catalog.rs and client_reads_interrupted_execution.rs |
| `dokime.host.v1.Plugins/Replace` | CLIENTS | `dokime.host.v1.PluginsReplaceRequest` | `dokime.host.v1.PluginsReplaceResponse` | Desktop generated dispatch; host/installation.rs startup replacement; client_installs_and_removes_managed_plugins.rs |
| `dokime.host.v1.plugins.Settings/List` | CLIENTS | `dokime.host.v1.plugins.SettingsListRequest` | `dokime.host.v1.plugins.SettingsListResponse` | Desktop generated dispatch; client_edits_settings.rs reads/reopens; SDK plugin contexts use the separate own-settings facility |
| `dokime.host.v1.plugins.Settings/Set` | CLIENTS | `dokime.host.v1.plugins.SettingsSetRequest` | `dokime.host.v1.plugins.SettingsSetResponse` | Desktop generated dispatch; client_edits_settings.rs complete conditional replacement/replay |

This adds ten product policy operations beside the 16 generic wrappers, not ten
facade-forwarding wrappers. `crates/dokime-client-internal/src/lib.rs` currently
gets its plugin resources from the library Client; migrate it to these product
resources. SDK registration owners are Rust `client/handles.rs`, Python
`_client.py`, C# `Registration.cs` and C++ `src/handles.cpp`. Tests/support
clients consume the same operations, not independent management services.

The matching library host messages still exist at the pinned revision and are
used by its in-process `harmos-operations/src/native/plugins` facade and runtime
system fixtures. Product declarations do not restore their removed wire services.
The product and library administration DTO families now overlap until the upstream
ownership move completes; retain both pending the owner's deletion decision.
Delete the obsolete upstream administration family/facade after migrating its
runtime consumers, preserving the separate own-settings facility/resolver contract.

Before this amendment Plugin Boundary assigned ownership and durable guarantees,
but did not enumerate RPC names, caller audiences, message homes or the
management-to-activation projection. This table locks names/audiences/message
homes. Runtime policy, startup/storage integration and own-settings resolver
implementation remain unchecked work; schema generation does not implement them.

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

Phase A prepares complete native units without wiring production consumers or deleting
the SQL paths. Phase B switches their consumers and removes the superseded ledger
together on the combined integration branch. Phase A preparation does not close a
coupled cutover task. The owner still decides whether NoteEvent/TraceabilityEvent
exist, whether report publication joins sealing, and whether SQL payload copies are
removed. Native preparation excludes those decisions and accepts a typed durable
intake settlement from the later 3.3/3.4 owner; it does not invent another intake feed.

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

#### Scoped Production Recovery Gap (2026-11-21)

Harmos `489dc40935c186aff38fe9dbdc77d25913932ce9` supplies G4 through
`ListFileProductions { scope, page, at }`. Its bounded, resumable pages enumerate
the library's live/recovered staging set by exact scope, returning session identity,
current writer generation, accepted/durable byte lengths and `resolved_at`.
This includes uploads whose create response was lost. G1–G3 remain available.

Cycle-seal cleanup belongs to `host/cycles/seal.rs::cleanup`, after storage recovery
and before host admission at startup, or under the administration guard on retry.
It pages at one returned cut, claims each discovered writer through
`CreateFileWriter` (recovered sessions are disconnected), abandons the returned
generation through `DeleteFileProduction`, then calls `DeleteScope` for the exact attempt scope.
Other scopes and sealed cycles retain their productions. The backend-owned SQL
discovery/abandonment in `Backend::open_with_execution` and its superseded test are
deleted; no product upload registry, timeout or second recovery owner is added.
The backend still refuses scope deletion until all its productions are settled.

G4 adoption does not implement the coupled lifecycle/evidence replacement:
2.3 finalization, 2.4 guarded execution deletion, 2.2 lifecycle/history and the
remaining 1.6 consumers and execution-owned cut retention in 2.3 stay open until their own replacement and proof.

The independent SQL command and note APIs are retired. Note ownership and
open-owner policy belong to the host evidence operations, shared by native
notes, uploads and content publication. Lifecycle reads still use the existing
execution source pending the coupled cutover. Exact execution reads return
`NotFound` for a missing owner without enumerating all execution identifiers.
The remaining note list still collects the global administration scan before
paging; scoped, bounded native Note reads belong to 1.6/3.5 rather than a second
listing implementation in this deletion. Fresh catalogs create neither SQL
table. Empty retired tables carry no undecodable state and remain untouched;
occupied retired journals refuse admission with an action to select a fresh
root, preserving their bytes. No migration, legacy payload reader or reset is
performed.

#### Desktop Brand Input Ownership

`crates/dokime-gui/src-tauri/build.rs::brand_text` reads the canonical
`packages/document/src/generated/organization/default-organization.json` written
by `tools/python/src/dokime_tools/build/organization.py::generate`. The retired
GUI-local brand directory is no longer a build input. Run `mise run generate`
before compiling runtime or desktop consumers to prepare their generated inputs,
including the runtime's `target/generated/report/report.html` template.

## Stored Envelope Coarsening at the Pinned Revision

Harmos `59822db5d5299aa24ea6d00957abcda1a4bde1d3` supplies continuation
coarsening through the existing bucket-summary transform. The continuation can
keep its source cut and field/history bindings while reducing `max_buckets`;
Harmos merges its stored summary without reconstructing raw samples or exposing
opaque continuation bytes to the product.

`crates/dokime-storage/tests/integration/backend.rs::library_bucket_continuation_coarsens_a_stored_envelope`
now passes at this pin: a four-bucket stored envelope coarsens to two buckets at
the same cut without new samples, preserving the expected counts and means.
The former upstream blocker is resolved. Product stored-envelope adoption and
its complete guarantees remain task 5.2; this pin does not close that task.

The remaining task closures retain their coupled consumer dependencies:

| Tasks | Required Completion |
| --- | --- |
| 1.6 | Migrate the remaining persisted shapes and frozen bindings with their canonical lifecycle/evidence consumers. Shape-only scaffolds do not close this task; execution-owned retained cuts are consolidated into 2.3. |
| 2.2–2.4, 3.3–3.5, 4.4–4.5, 5.1–5.3 | Switch the canonical lifecycle, intake, queries, watches, seal and preview owners together. The former coarsening gap no longer blocks their implementation; their own source and focused proof remain open. |
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

The six call sites in four files still enumerate collections: legacy artifact
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

Backend `ScanPage` readers return at least the first matching entry even when
that entry alone exceeds the page byte budget; subsequent entries end the
page before exceeding that budget. This removes the storage-side rejection of
individually oversized rows without changing accepted sizes or library scan/watch
bounds. The payload ownership overlap remains an owner
decision and no longer blocks typed note/event adoption. The pinned reference
backend's identity reader still refuses an over-budget response; the product
identity reader follows this task's explicit first-entry progress decision.
Recovery, columnar and index work limits retain their separate strict contracts.

### Maximum Note Client Retention for Owner Review

The accepted-maximum note journey has a separate client admission REWORK at
Harmos `59822db5d5299aa24ea6d00957abcda1a4bde1d3`. Dokime's runtime
`tests/support/mod.rs::client_settings` uses the same 4 MiB value for
`message_bytes` and `retained_bytes`. Harmos
`sdk/rust/src/connection/admission.rs::allocations` charges twice each string's
length plus allocation and message overhead; `connection/codec.rs::claim`
therefore refuses the create response for a 2 MiB note with
`client retained_bytes bound exceeded`. The runtime story stops at Create,
before edit/list/watch catch-up/replay/restart can be proved.

The per-message accepted size and conservative decoded-memory admission must
be reconciled by their owner at the pinned SDK boundary. Keep all bounds and
accepted sizes unchanged; do not bypass the SDK, reduce the test payload or
add a second reader. The unlanded failing journey is omitted from the passing
backend subset; the complete maximum-note journey remains open in 3.3c.

### Populated Release Measurement Baseline

The report benchmark now prepares 306 native plot views and their cut indexes
before timing the same eight-section, 116-stream, 118,784-sample, three-record save.
The previous full-save fixture already warmed 116 plot views; the 17-object
administration inventory in the original attribution belongs to its isolated example,
not this full-save fixture. Do not compare those different workloads as a scan repair.
One optimized release iteration before and one after prefix repair give these
observations over the same populated fixture (milliseconds):

| Stage | Populated Baseline | After Prefix Repair |
| --- | ---: | ---: |
| Execution Files | 2,597.203 | 373.085 |
| Notes | 1,042.240 | 254.559 |
| Artifacts | 1,055.980 | 335.724 |
| Host Create | 6,818.570 | 1,993.882 |
| Complete Save | 7,417.382 | 2,594.546 |

The release benchmark is built through the focused project task with
`--package dokime-runtime --bench storage --profile release`; no bench-profile build
is used. The executable runs once with `DOKIME_BENCH_ONCE=1` and
`--bench --test reports/save`. The baseline is recorded in
`target/benchmarks/storage/rscan-release.log`; the after-repair observation and its
compiler records are `target/benchmarks/storage/prefix-release.log` and
`prefix-release-compiler.json`. All owned compilation and validation were idle for
the measurement. These are single observations, not statistical estimates.

The single after-repair release `native-desktop` run passes nine tests. Thermal
contains 2,502 samples and one record. The earlier two release observations and the
after-repair run give these timings, not a controlled desktop comparison (milliseconds):

| Stage | Supplied Release Observation | Prior Release Observation | After Prefix Repair |
| --- | ---: | ---: | ---: |
| Execution Files | 148.195 | 196.987 | Not Captured |
| Notes | 124.468 | 148.929 | Not Captured |
| Artifacts | 166.479 | 190.125 | Not Captured |
| Host Create | 571.076 | 709.428 | 388.809 |
| Complete Save | 4,627.801 | 3,716.160 | 2,923.046 |

The after-repair picker takes 1,831.517 ms, publication 23.985 ms, transfer
46.103 ms and write/publication 202.884 ms. The prior picker was 2,053.517 ms
versus 3,316.512 ms in the supplied observation. The new run retained only the
INFO Report Created/Published events, so debug stage timings, generic native-call
admission and RPC overhead are unavailable; no second desktop run was made.
The prior run captured admission 0.0021 ms and RPC overhead 3.0639 ms. Its evidence
is `target/tests/release/windows-x86_64/system-typescript/native-desktop-2XcfS0/`
and `target/native-desktop-rscan-release.log`; the after-repair evidence is
`target/tests/release/windows-x86_64/system-typescript/native-desktop-nomNmh/`
and `target/native-desktop-prefix-release.log`.

### Requested Local Validation

The focused operations selection passes five tests; reports/views have no unit test
modules in that crate. The 18 backend tests include mixed-prefix equivalence and
payload exclusion. The native desktop story covers the report/view operation paths
and passes all nine tests. Workspace compilation, repository configuration checks,
formatting and the three requested scoped Rust lints pass. Release product, Rust SDK
consumer, host/reference and desktop fixture builds pass. Native Linux and hosted
CI/Nightly proof remain in tasks 6.5–6.6; no push or delivery is authorized here.

```powershell
mise run test -- --package dokime-operations --target lib --filterset 'test(host::storage::administration) | test(host::content) | test(host::evidence::notes) | test(host::executions::commands) | test(host::reports) | test(host::views)'
mise run test -- --package dokime-storage --target integration --story backend
mise run build -- --cargo check --workspace --all-targets
mise run check:repository
mise run format:rust -- --check
mise run lint:rust -- --package dokime-operations
mise run lint:rust -- --package dokime-storage
mise run lint:rust -- --package dokime-runtime
mise run test:typescript:system -- --profile release --select native-desktop
```

The existing `tools/python/src/dokime_tools/build/lint.py::desktop` regenerates all
frontend assets through `prepare/generated.py::prepare` for every scoped Rust lint,
even a storage-only lint. Prefer separating asset preparation from pure Rust lint
in the tooling owner. The eager export fixture and duplicate span collectors remain
in the existing ownership reviews below; no second preparation or query mechanism
is introduced by this repair.

Previously release fixture producers staged debug Rust SDK/host/content/plugin
and desktop CLI bytes, while the shipping GUI was release. Rust test harnesses
and their native host supplied through `NEXTEST_BIN_EXE` intentionally use the
unoptimized test profile; they are distinct from those auxiliary fixtures.
The reference conformance host also explicitly installed with the debug profile.
All these auxiliary producers now select the requested profile and fail if its
binary is missing. C# defaults to Release, C++ uses its release preset, and pure
Python has no compiler profile. The supplied attribution explicitly selected actual
release executables, so the producer defect does not explain that measured gap.

The earlier native run proved the helper's correction for the formatter's quoted
operation field. Setup failures from missing prepared tooling/driver and concurrent
frontend/host generation were REAL preparation errors; sequential preparation and
the final checks pass. No test, deadline or bound was weakened. The explicitly
requested release measurements and scoped Rust lint take precedence over the generic
worker prohibitions on release profiles and lint.

## Browsing View Retention Ownership

`crates/dokime-operations/src/host/views.rs::read` retains a native file index per
resource/content digest with the supplied resource scope. The redundant `view-at` index was removed
by the 2026-11-22 decision. `retain` removes newly produced files on index failure, publication-race
loss or ended observation; normal read completion, disconnect and shutdown still do not release an
indexed view. Administration uses `ObjectRetention::All`. Viewport selections change resource keys
and response changes create content revisions, so retained views still grow during a session and
across restarts until session retirement is implemented.

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

Owner decision, 2026-11-19: a retained browsing view belongs to the client session
that reads it. Disconnect, explicit close and host observation of a lost connection
end that scope. Release its indexes and native file when the last live session
referencing the content ends. At host start, release every client view left by the
previous process; no client view survives a restart. Durable evidence/report captures
retain their execution/cycle owner and survive session end and restart. Represent
client views and durable captures with distinct types and ownership, rather than
inferring their kind from a key prefix. A released cut returns the existing
resynchronization error, so the client re-reads. There is no expiry timer, size cap,
clock-based sweeper or independent connection poller.

Implementation prerequisite at the pinned Harmos revision
`a037b73e7a60e5fd4be50dbff72779c95ef7cc45`: its
`crates/harmos-runtime/src/grpc/middleware.rs::Session` is an RPC scope, not a
connection scope. `Invocation` exposes a peer address and transport extensions;
`listener.rs::Connection` owns the socket, but exports only `TcpConnectInfo` and
does not notify product middleware on EOF/drop. Releasing an RPC context would
invalidate cuts between pages while the client is connected. Keeping a map keyed
by peer address cannot observe the end of an idle connection. The library must
expose a connection-scoped identity and end notification/lease to both plaintext
and access/TLS listeners before the product can implement the complete rule.
Preserve that transport owner; do not add a second product listener or infer
connection loss from an operation error. Session-owned retirement remains deferred
while this prerequisite is unresolved. Failure cleanup is independent and follows
the document-read decision below.

Generic retained reads use only `view:{resource}:{digest}` and require the returned canonical cut
with its resource revision. The 2026-11-22 decision supersedes acceptance of position-only generic
view cuts: no repository caller constructs those cuts for this reader. Position-only execution
evidence and plot-definition cuts have independent evidence readers and never selected this view
index. Delete the `view-at` lookup and writer together; a generic caller that omits its revision
must supply the returned cut. Existing development roots are never reset or migrated. Legacy SQL-root rejection in
`crates/dokime-storage/src/store.rs` and its guard in
`crates/dokime-storage/tests/integration/resource_positions.rs` still recognize old `view-at`
journal entries, preserving their bytes while refusing admission. These are incompatible-root
guards, not native view-index readers or writers, and remain unchanged.

## Document Card Result Reads

The root document read retains one expanded declaration, execution boundary,
repeat selection and plot settings. It returns that view's cut with its bounded
structure frames. Card reads use the existing selector:

```text
Documents.Get { scope, document_id, at: root.resolved_at, card_ids: [card_id] }
```

At that cut, the host reads the saved declaration and evaluates only the selected
cards. It neither replays appended parts, expands repeats, evaluates unrelated
cards nor publishes another view. The reply includes the selected cards' section,
declaration and completion framing for the existing decoder; it does not repeat
the complete document structure. Full reads use the same result evaluator.
Continuations bind both the root cut and the structure/card selection. The saved
declaration uses the `document-read/` resource namespace, replacing the old saved
response shape without interpreting old response bytes as declarations.

Desktop captures carry the root cut through progressive assembly. Refresh first
obtains the new root cut, then refreshes cards independently through the existing
coordinator. An active card retains its old cut until it settles; its trailing
refresh selects the newest root cut without cancelling other cards. Card retry
alone uses its current root cut.

File production and its index settlement are admitted host work. The caller's
read deadline ends observation, while the host settles the current storage call
and deletes the unfinished upload or unindexed published file. Index rejection
and publication-race loss delete only the newly produced file. Existing operation
bounds remain unchanged. This failure cleanup does not implement session-owned
view retirement, which still requires the library connection-lifetime contract.
Generic views now retain only the canonical revision index, per the 2026-11-22 decision below.

Upload Append, Create, Resume and retained-view file creation read native Info once
for their chunk limit and mutation window.
The transfer owner continues to compute chunk bounds and mutation identities.

### Remaining Overlaps

These existing pairs remain for the owner's decision; the card-read change does
not retire either owner implicitly.

| Locations | Responsibilities and Callers | Preferred Removal |
| --- | --- | --- |
| `crates/dokime-operations/src/host/files/transfer.rs::write` and `host/{cycles/seal.rs,cases/storage.rs,executions/commands/mod.rs}` expiry matches | Owned upload/view writes use the transfer retry policy. Cycle seal writer recovery/abandonment and scope deletion, case registration/deletion and execution commands independently retry the same mutation-expired failure. | Delete the remaining local retry matches after those families' owners adopt transfer; those files are outside this worker's ownership. |
| `crates/dokime-operations/src/host/files/transfer.rs::publication` and `crates/dokime-operations/src/host/evidence/archive.rs::materialize` | Views and Reports.Create publish known-length/digest byte producers through transfer. Evidence.Archive and Cycles.Seal call materialize, which independently uploads an already materialized ZIP with known length/digest and failed-upload cleanup. | Delete materialize's upload loop after the evidence owner adopts the transfer publisher. Evidence handlers are outside this worker's ownership. |
| `sdk/python/tools/installed.py::main` and `tools/python/src/dokime_tools/test/archive.py::install_wheels` | SDK Install selects a dev wheel and installs examples; bindings/archive preparation selects wheels by profile. Both install the SDK in the root environment. | Delete the SDK installer's hardcoded profile selection by reusing the profile-aware artifact owner, with one owner for example preparation. This remains outside the document worker's tooling ownership. |

Generic retained revisions still hash raw protobuf bytes. Map-bearing declarations
and execution origins can encode equivalent values in different map orders, losing
content deduplication. The request-selection digest in `host/paging.rs::bytes`
already canonicalizes maps. Any consolidation must preserve native file SHA-256
integrity while the owner chooses the retained-revision contract; this change does
not add another hashing or caching mechanism.

The repair's in-process storage/task proof stays in the owned operations modules'
existing libtest harness. This differs from the generic pure-unit layer. Moving
these stories to an integration target requires the test-composition owner's
decision; this change adds no public testing API or alternate test harness.

## View Single Owners

Owner decision, 2026-11-22: consolidate mutation-expiry handling in the file transfer owner. Upload
Append, Create, Resume, Publish, Delete, Files.Delete and retained-view create/discard use the same
policy: retry only the library's mutation-expired failure, fetch the current window, preserve the
request key and issue one further storage call. A second failure is returned with the owning action;
no third attempt, delay or deadline change is allowed. Retry matches outside the assigned file/view
families remain for their owners to migrate.

Views and reports publish through `host/files/transfer.rs::publication`: one create, bounded
read/append sequence, publish and failed-upload cleanup. The byte producer remains with its caller
(view bytes or temporary report HTML). View cancellation is checked between storage calls; admitted
host settlement and unindexed-file cleanup remain in views. Reports retain their durable
cycle/execution owner and never inherit view cancellation or browsing lifetime. This supersedes the
earlier pending report/view publication-loop overlap, not the separate report-retention decisions.

The capture engine's `host/files/captures/engine.rs::attach` remains distinct: Files.Capture and
capture Stop admit public execution evidence, asynchronously read a device byte range and compute
its digest during that single read, then attach the artifact. The fixed view/report publisher
requires the digest before creation and takes a synchronous byte producer. Reusing it for captures
would require a second device scan or a changed producer/admission contract. Keep the capture path
pending that owner's decision; do not introduce an adapter or extra scan.

Desktop execution/cycle root query data owns both the current progressive native capture and the
last completed capture, plus its mapped display projection. A stable query selector exposes the
projection; cards read their declarations, root cut and page from that same query data. Unchanged
completed captures reuse the same projection. Query cancellation, generation keys, garbage
collection and cache clearing now govern the captures without a separately capped process-global
map. Native captures contain Maps, so root queries preserve their reference identity instead of
recursively structurally sharing their raw captures. Independent card queries and their existing
refresh coordinator are unchanged.

## Administration Reader Ownership Review

The scan callers enumerate collections: `content/mod.rs::validate_root` checks legacy
artifact presence; `artifact_payloads` lists committed artifacts; `execution_files`
lists an execution's logical files; and `evidence/notes/list.rs` selects notes.
`executions/commands/mod.rs::recover` owns bounded native Command pages and keyed
reads at their opening cut, preserving continuation ownership while phases advance.
`reports/get.rs` scans prepared reports only to select the latest one for a cycle;
its named prepared/published report reads already use keyed administration access.
Report capture lists files/artifacts and then uses their existing keyed Get operations.
None of these prefix scans is an exact-key lookup. File/artifact continuation reads
reopen retained views without rescanning; note continuations previously rescanned the
same prefix. Capture now selects notes once and pages that request-local image through
the same page operation, without retaining a cache across Create requests.
The remaining administration scans still accumulate complete collections despite
bounded native pages. Their note/content/report owners should stream those consumers
during the canonical cutover instead of introducing another scan registry.

| Overlap | Callers and Purpose | Preferred Removal |
| --- | --- | --- |
| `crates/dokime-storage/src/administration.rs` and `crates/dokime-operations/src/host/storage/administration.rs` | SQL keyed protobuf administration serves runtime root claims/access credentials and the secret journal; native administration serves content, notes, views and prepared reports. Commands use their own registered shape. | Delete SQL administration after migrating its remaining root/security owners with their durable replay and cleanup semantics. Keep both pending the coupled storage cutover. |
| `tools/python/src/dokime_tools/build/{desktop_fixtures,host_fixtures}.py::build` raw executable-path selection and `build/outputs.py::executables` compiler-record selection | Desktop/host fixture tasks map dev to debug and otherwise use the requested profile directory. The native Product builder selects non-test executable paths from Cargo compiler messages through `outputs.executables`. | Prefer deleting the fixture path inference after the owner agrees that fixture builds should emit compiler records consumed by the existing selector. Keep independent Product/Tests builds; do not add a second selector or make Tests depend on Product. |
| `tools/python/src/dokime_tools/test/focused.py::{selections,run}` required-feature discovery | Changed-path tests call `selections`, which derives test-target features, then `run` derives them again for `--tests`; direct package tests also call `run`. Workspace compilation consequently appends desktop `testing` twice. | Prefer making `run` the single owner for required features across explicit targets and `--tests`, then deleting required-feature derivation from `selections`. Preserve target/package boundaries and required features; tooling remains outside this worker's ownership. |

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
