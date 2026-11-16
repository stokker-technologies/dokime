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
- Direct `Store` consumers remain across `crates/dokime-storage/src` (store, cycles, plans,
  notes, history, client reads, resource values, evidence positions, administration,
  telemetry reader/query, stream windows, expressions, document values, plot selection,
  plot windows, plot kinds, content, report), `crates/dokime-runtime/src` (execution,
  execution manager, run control, capabilities, host, streams, documents, prompts, reports,
  access credentials, root claim) and `crates/dokime-operations/src/host` (resources, views,
  execution state, planning, cases, cycles, executions, evidence, documents, plots, prompts,
  reports, content, every `watch.rs`, backend overview).
- Duplicated kernels: moments, ranks and persisted summaries in
  `crates/dokime-storage/src/telemetry/aggregates.rs`; adaptive envelope merge in
  `telemetry/previews.rs`; scanned bins in `stream_windows.rs`; statistics, histogram and
  Tukey boxes in `plot_windows.rs`; expression arithmetic and filters in `expressions.rs`;
  payload encoding in `backend/payload.rs`.
- Offline paths: desktop archive/disposal in
  `crates/dokime-gui/src-tauri/src/native_session/storage.rs`; CLI doctor copies in
  `crates/dokime-cli/src/commands/doctor.rs`.
- Current-pin cutover dependency: `Backend::open` registers a `storage.sqlite3`
  ledger, whereas product execution/evidence/sample/seal/report writers use
  `execution.sqlite3`. Moving an append alone splits their canonical history and
  invalidates cuts and seal verification. The coupled migration needs Harmos
  compound atomic mutation at the backend boundary, now provided by Harmos
  `7fe385eacbe9a4c9a47943f6a528f97a014256ee` and implemented in
  `crates/dokime-storage/src/backend/compound.rs`. Product callers still need
  their coupled cutover; retaining two writers or
  adding a mirror is not an acceptable intermediate design. Independent retained
  views in `host/views.rs` now use Harmos objects for immutable file indexes and native
  file operations for complete protobuf payloads, including collections larger than one
  response bound. Their evidence cut issuance remains coupled to the product ledger.
- The native backend commits compound effects and applied/refused receipts together,
  refuses partial compound cuts, and implements object/scoped tombstones with retained
  reads and file reclamation. Harmos's public backend conformance kit runs in
  `crates/dokime-storage/tests/integration/backend.rs`. Maintenance copy/dispose remains
  dependent on offline/minimal-assembly storage operations; no substitute is introduced.
- Composition: `protocol/composition.toml` is read by `services/composition.rs`
  (`setup.composition`), `native::callers()` in `crates/dokime-operations/src/native.rs`,
  `sdk/cpp/tools/generate.py`, `sdk/csharp/tools/generate.py` and
  `tools/python/src/dokime_tools/build/resource_roots.py`.

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

### D1. Adoption Order (Proposed)

From the research, unchanged by the owner: register backend and catalog (landed) →
bootstrap and maintenance path → schemas, simple objects and administration → records and
trusted sample ingestion → compound execution, case, cycle, seal and report mutations →
files, archives and deletion → queries, statistics and expressions → previews, documents
and plots → watches → deletion of direct `Store` consumers and duplicated kernels → facade
adoption → product wrappers and composition removal. Each step pins the Harmos revision that
landed the groups it needs: an integration-only local override while the Harmos revision is
unpublished, replaced by the published revision before final validation.

### D2. Product Policy Over Generic Verbs (Proposed)

Execution, case, cycle, seal, report and planning policy stay in Dokime and compose:
`SetObject`/`GetObject` for single records of state; the compound mutation for multi-object
transitions (cycle membership, seal plus report publication, case deletion guards); scoped
deletion with the execution or case scope for deletion; `AppendRecords` and
`AppendSamples` with trusted execution scope for evidence (acknowledgement after the durable
receipt); `ListRows`/`ListColumnRows` and snapshots for reads; `WatchRows` for every watch.
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

Sealing an execution clears its stored envelope and every preview (time envelopes, scatter
occupancy grids, group counters, persisted summaries) in the same compound mutation that
records the seal. Reports can be created after seal (`host/reports/create.rs` reads the
current document at the sealed cut), so reports and full-range plots of a sealed execution
read the sealed cut through on-demand Harmos bucket summaries; no ordering between report
creation and clearing is required. `report-previews` (Frozen Evidence Cut, Incremental Plot
Previews) keeps its behavior: previews are maintained during execution and a sealed cut is
reconstructed from source data. The owner stated only that seal clears the stored envelope
and previews; the on-demand read path for sealed executions is this design's consequence.
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
declaration plus a handler that authorizes, maps product values and calls the typed Harmos
facade; authorization uses the existing product authorizer. Wrappers reuse the Harmos request and response messages as
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

## Risks / Trade-Offs

- [Seal clears previews, sealed full-range plots scan source data] → measure with existing
  plot benchmarks over multi-day data; report results to the owner; budgets unchanged.
- [Compound mutation unavailable on the product backend] → implement the backend capability
  in `crates/dokime-storage/src/backend` in the same task that first needs it.
- [Long migration with many callers] → each group deletes the `Store` path it replaces
  before closing, so no dual path survives a group.
- [Breaking Harmos pins] → pin a breaking revision only in the group that removes the usage;
  the build stays green at every group boundary.
- [Existing development roots become incompatible] → refuse them at baseline v1 with the
  fresh-root action; never reset or delete them.

## Migration Plan

Development storage policy applies: no migrations or compatibility. Groups land in the order
of D1 on the integration branch; rollback is the previous Harmos pin and the previous commit.
