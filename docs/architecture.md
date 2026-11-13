# Architecture

## How to Use This Document

Read this contract before proposing architecture, protocol, public API or ownership
changes. A proposal touching an invariant must quote it, state the purpose of the
mechanism it changes, and update this document in the same delivery. Never remove,
merge or replace a mechanism until its purpose is established from its handler,
tests and design, or from the owner.

**Landed** means verified in the named tree and proof, not that proof was rerun for
this documentation edit. **Locked direction** names an agreed change whose complete
contract is not implemented. Partial evidence does not establish the whole invariant.
The additive execution host and old runtime currently coexist. Paths distinguish
those owners; a proof for one is not proof for the other.

The [Operation Grammar](../openspec/changes/operation-grammar/design.md) owns the
locked mappings and requirement deltas. Its three-level decision supersedes earlier
two-level Private wording, including the host-requested operations in
[the gRPC design](../openspec/changes/grpc-plugins/design.md). Main specifications
that still describe superseded mechanisms are migrated by those changes, not silently
interpreted as the new implementation. Missing guards are unchecked in
[Operation Grammar Tasks](../openspec/changes/operation-grammar/tasks.md).

Upstream source citations below use **Harmos** paths at the exact revision pinned by
[Cargo.toml](../Cargo.toml), `26010c03516060deb5f70d17af3fbf0e49d9fbe2`, rather than
assuming another checkout's HEAD matches this product. `Harmos:` paths resolve
inside that pinned dependency revision; other paths resolve in this repository.

Paging, SDK packaging, transfers and durable lag were amended by the owner on
2026-11-13. The linked Operation Grammar records their complete amended contracts.

## Invariants

1. **Three Faces.** The plugin protocol is served by a running plugin instance and
   called only by its host; context is served by the host to a plugin during an
   invocation; the client API is served by the host to clients. The host is always
   between a plugin and anyone else; faces never replace per-operation visibility.
   **Why:** Public exposure must not grant direct instance access or client management
   to plugins. **Where:** [face inventory](../openspec/changes/operation-grammar/design.md#audience-faces-and-visibility),
   [current protocol](../proto/dokime/v1), [current host](../crates/dokime-runtime-next/src/services/mod.rs).
   **Proof:** No proof yet; task A2 must guard all three audiences and generation filters.
   **Status:** locked direction (`operation-grammar`); separate package roots and
   product declarations are not landed. The host now calls the pinned library's
   `harmos.plugin.v1.Instance` carrier.

2. **Three-Level Visibility.** Private is plugin-internal, never catalogued or on
   any wire (including host-to-plugin), has no gRPC endpoint association, and runs
   locally in its SDK with ordinary policy, retry, deadline and cancellation.
   Protected is catalogued and callable by the declaring plugin and host over their
   carrier only, never by other plugins/clients or a client-facing gRPC method;
   Public permits declared cross-plugin requirements and clients through the host,
   and may bind a client-facing method. Every SDK and host built-in chooses explicitly,
   without a default; host-only requests of plugins are Protected.
   **Why:** The owner: "a private operation never appears on the wire, is consumed only
   internally by the plugin and does not allow association to a grpc endpoint";
   "a plugin should be able to call another operation in another plugin if exposed".
   **Where:** [visibility, enum, built-in mapping and SDK audit](../openspec/changes/operation-grammar/design.md#audience-faces-and-visibility),
   [requirements](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-explicit-catalog-visibility-and-admission),
   [current host admission](../crates/dokime-runtime-next/src/plugin.rs).
   **Today:** The pinned library implements all three levels with explicit SDK
   visibility and local Private dispatch. Product template lifecycle declarations
   and host admission now use Protected in all four languages. The additive host
   still owns its dispatcher and does not yet provide the complete peer/client
   routing contract. **Proof:** No complete product proof yet; task A1 and
   4.11–4.17 retain the remaining admission and four-language obligations.
   **Status:** upstream implementation adopted; complete product guard remains locked direction.

3. **Public Child Routing.** A plugin calls a Public operation in another plugin
   through its context, the host's one registry, then the provider's invocation
   carrier. Check declared requirements, catalog descriptors/digest, fingerprints,
   visibility and admitted provider; inherit scope unchanged and propagate results,
   structured failures and caller-to-child cancellation. A requirement grants no access.
   **Why:** Exposure exists to permit plugin composition while preserving host policy.
   **Where:** [routing requirement](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-visibility-and-host-mediated-plugin-invocation),
   [current dispatcher](../crates/dokime-runtime-next/src/dispatch.rs),
   [host-only child dispatch table](../crates/dokime-operations-next/src/lib.rs).
   **Proof:** No proof yet in this repository for Public peer success plus Private
   refusal; A1 is the first task. Harmos's
   `child_calls_reach_host_and_public_peer_without_start_order` in `Harmos:crates/harmos-runtime/tests/plugin_grpc/child_calls.rs`
   proves the library route, but the additive product dispatcher currently resolves
   only its nine host operations, not peer providers. **Status:** locked direction (`operation-grammar`).

4. **Unary Operations and Deliberate Streams.** Plugin operations have one request
   and one result; carrier frames do not turn their payload into a streaming API.
   Samples have their own carrier and ordinary server streams serve Watch semantics;
   never generate an automatic streaming twin. Only a promised cooperative final
   result after cancellation justifies a managed replacement method.
   **Why:** Observation, sample delivery and durable work have different lifetimes.
   **Where:** [streaming requirement](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-explicit-streaming-contracts),
   [current client protos](../proto/dokime/v1), [SDK operation](../sdk/rust/src/operation.rs).
   **Proof:** No proof yet for the complete contract; A3 must guard generation and
   settlement. Current client Run twins remain. Unary file transfers replace
   streaming transfers, with 4 MiB default chunks and an 8 MiB default message
   limit, amended by the owner on 2026-11-13. **Status:** locked direction (`operation-grammar`, `grpc-plugins`).

5. **Opaque Invocation Scope.** Carry and inherit scope unchanged; the library
   never interprets it, treats it as an authorization token or infers it from payload.
   Product handlers decide its meaning and whether it is required.
   **Why:** Generic routing must not acquire execution semantics or let a child widen authority.
   **Where:** [scope vocabulary](../openspec/changes/operation-grammar/design.md#shared-grammar-and-vocabulary),
   [product scope validation](../crates/dokime-operations-next/src/resources.rs),
   [SDK binding](../sdk/rust/src/context.rs).
   **Proof:** No proof yet for the full cross-provider contract; A4. Product
   `execution_isolation` in [execution_isolation.rs](../crates/dokime-runtime-next/tests/system/execution_isolation.rs)
   proves execution separation, not library opacity or peer inheritance.
   **Status:** locked direction (`operation-grammar`).

6. **Managers Own Live Orchestration.** Create a manager only for changing live
   state, background work or lifecycle. Library PluginManager owns entries/processes,
   GrpcManager transport/access, StorageManager storage/transfers, ServiceManager
   status/diagnostics/clock, OperationManager dispatch/invocations, StreamManager
   streams, RecordManager capture and StateManager live state; product execution,
   prompt and report managers own their lifecycles, with capture only if recordings remain.
   **Why:** One owner must join work and settle shutdown; storage reads and pure
   rendering need no invented manager. Cycles group execution-manager tasks and
   do not own an independent scheduler.
   **Where:** [manager ownership table](../openspec/changes/operation-grammar/design.md#ownership-and-managers),
   [current Host](../crates/dokime-runtime-next/src/host.rs),
   [composition State](../crates/dokime-runtime-next/src/services/state.rs),
   [execution manager](../crates/dokime-runtime-next/src/execution_manager.rs),
   [prompt manager](../crates/dokime-runtime-next/src/prompts.rs) and
   [report manager](../crates/dokime-runtime-next/src/reports.rs).
   **Proof:** A5: `execution_shutdown_joins_runs_after_observer_loss`,
   `prompts_settle_answers_and_shutdown_waiters` and
   `reports_publish_atomically_and_close_admission` in
   [manager_lifecycles.rs](../crates/dokime-runtime-next/tests/system/manager_lifecycles.rs)
   exercise observer loss, joined cleanup, queued Cancel refusal, Stop boundaries,
   answers, shutdown and publication across restart. **Status:** landed for the
   additive host's product managers; generic library service composition remains
   locked direction under invariant 7. Media has no live recording owner here.

7. **Library Machinery and Product Policy.** Compose library services directly,
   without redeclaring or forwarding them and without rebuilding generic machinery
   on the protocol crate alone. The library owns plugin/operation/stream/storage,
   access/transfer/status machinery and SDKs; the product owns storage definitions,
   objects, operations, authorization, settings/secrets stores, schemas, UI and archives.
   **Why:** The previous approach duplicated connection, dispatch and process launch;
   product-specific policy must not become generic machinery. HostValues uses a product resolver.
   **Where:** [boundary and settings requirements](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md),
   [current composition](../crates/dokime-runtime-next/Cargo.toml),
   [product settings](../crates/dokime-operations-next/src/settings.rs).
   **Proof:** `client_pairs_and_rotates_remote_access` exercises direct library
   composition, bearer authorization of product services, revocation, restart
   and live rotation. The new host serves library Access.Listener,
   Access.Pairings and Access.Certificates through GrpcManager, with product
   credential persistence in `services/access_credentials.rs`. Its former Remote
   service and operations are removed. Status.Get/Watch, Logs, TraceEvents and
   ranged Diagnostics.Get are served by ServiceManager. `services/status.rs`
   implements Contribution for product health and diagnostic entries; generic
   Backend/Runtime services and product collection machinery are removed.
   `client_collects_backend_diagnostics` proves health events, redaction and ranged
   archives; `client_reads_runtime` proves status observation and stable restart identity.
   The old loop retains its shared protocol until atomic cutover.
   **Status:** Access and Status composition landed; transfers and plugin composition remain open (`operation-grammar`).

8. **Resource Operation Grammar.** Services name resources; methods use bare verbs
   from Get/List/Watch/Set/Create/Delete/Append, with explicit domain side effects
   and noun-only single-mode parts. Multi-path parts are nested resource sub-packages;
   whole-value Set carries expected revision, Append preserves ordered immutable data,
   and wrappers are `<Service><Method>Request` and `<Service><Method>Response`.
   **Why:** One predictable contract avoids aliases, field-mask machinery and lost updates.
   **Where:** [grammar and mapping](../openspec/changes/operation-grammar/design.md#shared-grammar-and-vocabulary),
   [grammar requirements](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md).
   **Proof:** No proof yet for final declarations/revision/replay behavior; A7.
   **Status:** locked direction (`operation-grammar`).

9. **Position-Resolved Reads.** Optional `at` selects a previously returned compatible
   position or seal; omission selects latest durable without waiting, and every reply
   states `resolved_at`. Keep pages at that cut and open Watch there before durable
   changes. Paging is only for rows that can grow unbounded; small bounded catalogs,
   plugins, pairings, suites and schemas return whole. Measured durable lag is
   accepted (small about 99 ms, large/mixed up to several hundred milliseconds
   at p95), reported without a failing threshold or accepted-data snapshot.
   Amended by the owner on 2026-11-13.
   **Why:** Consistency and read-your-write cannot depend on a read-triggered flush,
   wall time or unstable pages. Running execution reads resolve durable frontiers;
   sealed reads resolve their seal. Arbitrary historical positions are not offered.
   **Where:** [read/watch/paging requirements](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md),
   [query delta](../openspec/changes/operation-grammar/specs/query-snapshots/spec.md).
   **Proof:** No proof yet for this complete API; A8 plus Nightly task
   1.9. Measure small, multi-megabyte and mixed payloads; test splitting/positions
   deterministically, never elapsed time. **Status:** locked direction (`operation-grammar`).

10. **Caller Resend and Original Receipts.** After a lost reply the caller resends;
    the same semantic request identifier and normalized payload return the original
    receipt, and changed payload conflicts. Correlation invocation/child IDs alone
    never deduplicate; accepted and durable positions remain separate facts.
    **Why:** A failed transport does not prove that an external effect was not applied.
    **Where:** [receipt and retry requirements](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-shared-failures-and-retry-policy),
    [existing planning receipts](../crates/dokime-storage/src/plans.rs).
    **Proof:** No proof yet across the final mutation API; A9.
    **Status:** locked direction (`operation-grammar`).

11. **Distinct Storage Guarantees.** Saved finite reads differ from following new
    durable progress; scans, indexes and projection frontiers have separate contracts.
    Bulk ingestion cannot substitute for atomic conditional update, and acceptance,
    flush cadence and configured durability/synchronization are never interchangeable.
    **Why:** Merging similar APIs can silently remove consistency, bounded work or crash guarantees.
    **Where:** [storage requirements](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-distinct-storage-guarantees),
    [current storage](../crates/dokime-storage/src/store.rs),
    [existing storage backend](../crates/dokime-runtime/src/storage/sqlite).
    **Proof:** No proof yet for the combined storage contract; A10.
    **Status:** locked direction (`operation-grammar`); storage-interface retains its
    own implementation work and is not activated by this document.

12. **Refuse Incompatible Roots.** Refuse incompatible persistent state with backup
    and explicit fresh-root/reset guidance; never silently delete, reset or migrate
    development data. The new store checks its database identity/layout at baseline v1.
    **Why:** A development schema break is allowed; destroying existing evidence is not.
    **Where:** [Store::open](../crates/dokime-storage/src/store.rs),
    [Incompatible error](../crates/dokime-storage/src/error.rs),
    [current compatibility requirement](../openspec/specs/data-shape-identity/spec.md).
    **Proof:** `client_collects_backend_diagnostics` in
    [client_collects_backend_diagnostics.rs](../crates/dokime-runtime-next/tests/system/client_collects_backend_diagnostics.rs)
    refuses a populated foreign root and preserves its bytes. The older
    [format.rs](../crates/dokime-runtime/src/storage/format.rs) refuses unreadable
    built-in state, not obsolete evidence-digest markers. **Status:** landed.

13. **Callee-Owned Policies and Configured Limits.** Deadlines, delays and retries
    come from operation policy/configuration; an override changes one call only.
    Retry requires declared replay safety; never increase budgets or limits to make proof pass.
    **Why:** Callers must not silently weaken failure, cancellation or resource guarantees.
    The locked unset policy means no deadline, zero cancellation grace and no retries.
    Time reads/waits use the authoritative host clock, including SDK child waits.
    **Where:** [policy requirement](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-shared-failures-and-retry-policy),
    [configured reads](../openspec/specs/query-operations/spec.md),
    [current dispatch policy](../crates/dokime-runtime-next/src/dispatch.rs).
    **Proof:** No proof yet for all policy/override paths; A11. `step_failure_and_retries`
    in [step_failure_and_retries.rs](../crates/dokime-runtime-next/tests/system/step_failure_and_retries.rs)
    proves actual attempt evidence, not complete policy parity.
    **Status:** locked direction (`operation-grammar`).

14. **One-Way Crate Layers.** Dependency direction is protocol ← storage ← operations
    ← runtime ← CLI/desktop; SDKs consume protocol and library SDKs, never the product
    runtime. Protocol/storage never depend on upper layers; operations never depend
    on runtime, shells never redeclare contracts, and host collectors stay outside runtime.
    Keep each declaration, policy, mapping and handler together behind narrow
    resources, never callbacks into the whole runtime.
    **Why:** Host-free authoring, independent storage and thin shells must remain buildable.
    **Where:** [layer requirement](../openspec/changes/grpc-plugins/specs/crate-structure/spec.md),
    [workspace](../Cargo.toml), [SDK manifest](../sdk/rust/Cargo.toml),
    [runtime manifest](../crates/dokime-runtime-next/Cargo.toml).
    **Proof:** No proof yet for the complete migrated tree; A12. New crate manifests
    follow the direction; old crates and consumers remain.
    **Status:** locked direction (`grpc-plugins`, `operation-grammar`).

15. **Generic Library and Consistent SDK Concepts.** Library protocol, SDKs and
    conformance use generic vocabulary and remain independently usable without
    product objects; language SDKs stay in the library. Product SDKs compose them,
    naming the same concepts and resource hierarchy consistently across all four languages.
    Each language ships one package for plugin development and backend clients:
    Python dokime-sdk composed with the single harmos-sdk; audience-specific builds
    are removed, and a codebase may install itself via main.py. The API should
    look simple and good. Amended by the owner on 2026-11-13.
    **Why:** Another host must not reimplement the plugin protocol or depend on this product.
    **Where:** [SDK ownership requirement](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-library-owned-language-sdks),
    [SDK inventory](../openspec/changes/operation-grammar/design.md#sdk-ownership-and-remaining-coverage-facts),
    [product SDKs](../sdk).
    **Proof:** No proof yet for final four-language parity/hierarchy; A13.
    **Status:** locked direction (`operation-grammar`).

16. **Deterministic Stories.** Correctness tests tell one behavioral story with
    events or controlled clocks; never assert elapsed wall time, load, sampled memory,
    authored prose or one repository constant. Performance and resource measurements
    belong to benchmarks; a hang guard is not a performance assertion.
    **Why:** Correctness must not depend on workstation conditions or freeze editorial text.
    **Where:** [agreed test redesign](../openspec/changes/test-stories/design.md),
    [guard requirement](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-deterministic-behavioral-proof),
    [existing stories](../crates/dokime-runtime-next/tests/system).
    **Proof:** No proof yet for the complete test migration; A14. Existing legacy tests
    still need the agreed redesign. Do not add tests that quote this document.
    **Status:** locked direction (`test-stories`, `operation-grammar`).

17. **Execution Evidence Outlives Live Runs.** Execution is an open/closed/sealed
    evidence container with frozen origin; run sequencing is live host state, not a
    persisted Run object. PluginRun freezes the case/catalog/process identity;
    Import carries its own manifest and never invents a plugin or runnable case.
    **Why:** Durable evidence and recovery must survive the process that produced them.
    **Where:** [execution model](../crates/dokime-storage/src/execution.rs),
    [host creation](../crates/dokime-runtime-next/src/host.rs),
    [requirements](../openspec/changes/grpc-plugins/specs/host-executions/spec.md).
    **Proof:** `thermal_suite_survives_host_restart` in
    [thermal_suite_survives_host_restart.rs](../crates/dokime-runtime-next/tests/system/thermal_suite_survives_host_restart.rs)
    and `execution_capabilities` in
    [execution_capabilities.rs](../crates/dokime-runtime-next/tests/system/execution_capabilities.rs).
    **Status:** landed in the additive host; full import admission remains separate work.

18. **Attributed Outcomes.** Evaluated, declared and not-evaluated outcomes remain
    distinct; only evaluated results determine the cycle verdict. A declared Passed
    import cannot make an unevaluated cycle Passed.
    **Why:** The host must not claim it evaluated externally asserted results.
    **Where:** [outcome folding](../crates/dokime-storage/src/outcome.rs),
    [requirements](../openspec/changes/grpc-plugins/specs/host-executions/spec.md#requirement-attributed-outcomes-and-cycle-verdicts).
    **Proof:** `cycle_outcomes_keep_declared_and_unevaluated_separate` in that source
    and `execution_outcomes` in
    [execution_outcomes.rs](../crates/dokime-runtime-next/tests/system/execution_outcomes.rs).
    **Status:** landed in the additive host.

19. **Rendering Does Not Publish.** Render a fixed execution snapshot into temporary
    output without mutating evidence or publishing; explicit publication freezes the
    document/HTML durably and never changes the execution verdict or seal.
    **Why:** Preview, download and render failures must not become lifecycle mutations.
    **Where:** [document rendering](../crates/dokime-runtime-next/src/documents.rs),
    [report storage](../crates/dokime-storage/src/report.rs),
    [requirement](../openspec/changes/grpc-plugins/specs/host-executions/spec.md#requirement-report-rendering-and-publication).
    **Proof:** `documents_and_reports` in
    [documents_and_reports.rs](../crates/dokime-runtime-next/tests/system/documents_and_reports.rs)
    checks page-size equivalence, failed-render nonpublication, explicit publish and restart.
    **Status:** landed in the additive host, including report-manager ownership (6), guarded by A5.

20. **Earliest External Inverse Survives.** Durably prepare an owner-qualified inverse
    before external writes; retain the earliest inverse per scope/resource while each
    later prepare gets its own confirmation ticket. Restore resources once in reverse
    first-prepare order, keep evidence capture open, and restore pending inverses before
    recovered runs/plugins become available; a failed inverse refuses startup and stays pending.
    **Why:** Repeated writes must restore the original physical state, not an intermediate one;
    confirmation ownership and evidence still describe every call.
    **Where:** [effect transactions](../crates/dokime-runtime/src/transactions/effects.rs),
    [restoration](../crates/dokime-runtime/src/runtime/effects.rs),
    [startup ordering](../crates/dokime-runtime/src/runtime/plugins.rs),
    [requirement](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-durable-external-effect-restoration).
    **Proof:** `effect_lifecycle`, `effect_resource_order` and `effect_recovery` in
    [effects.rs](../crates/dokime-runtime/tests/integration/domains/effects.rs).
    **Status:** landed in the old runtime; preserve it when completing the additive
    host's context Effects implementation, which does not yet provide this guard.

21. **Captured Shapes Preserve Evidence.** Schema-derived stream/record identities
    are equality identities, not authored versions; saved evidence retains the
    definitions needed to decode it after plugin evolution or removal. Plugin shape
    changes do not invalidate the product root.
    **Why:** Updating acquisition code must not erase or reinterpret historical evidence.
    **Where:** [shape requirements](../openspec/specs/data-shape-identity/spec.md),
    [telemetry contracts](../crates/dokime-runtime/src/runtime/storage/telemetry.rs).
    **Proof:** `stream_revisions_decode_on_one_root` in
    [stream_revisions_decode_on_one_root.rs](../crates/dokime-runtime/tests/integration/runtime/storage/telemetry/stream_revisions_decode_on_one_root.rs)
    proves stream evolution; the complete record/removal contract has **No proof yet**;
    A15 must close that gap across the new host too. **Status:** locked direction
    (`operation-grammar`) for the complete invariant; stream retention is landed in the old runtime.

22. **Failure and Gap Honesty.** Preserve structured classification, action and causes;
    unavailable cuts, missing evidence and watch gaps are explicit failures or resync,
    never empty success. Transport loss leaves mutation outcome uncertain.
    **Why:** An operator must distinguish no data from data that could not be obtained.
    **Where:** [failure/watch requirements](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md),
    [current failure translation](../crates/dokime-runtime-next/src/dispatch.rs).
    **Proof:** No proof yet across final interfaces; A16. The current dispatcher
    collapses host child errors to a permanent host_operation_failed failure.
    **Status:** locked direction (`operation-grammar`).

23. **Placement Changes Only Process Ownership.** Managed versus remote decides
    who starts/supervises a process and how its channel is obtained; materialization,
    execution and evidence above the channel use the same contracts.
     Pending managed installation cancellation reaps its child before removing
     the installation files and restoring a previous entry. The install operation
     waits for rollback before reporting its deadline failure.
    **Why:** Hosting convenience must not fork plugin semantics.
    **Where:** [plugin entry](../crates/dokime-runtime-next/src/plugin.rs),
    [process owner](../crates/dokime-runtime-next/src/process/mod.rs),
    [requirement](../openspec/changes/grpc-plugins/specs/host-executions/spec.md#requirement-managed-and-remote-plugins).
    **Proof:** `thermal_suite_survives_host_restart` in
    [thermal_suite_survives_host_restart.rs](../crates/dokime-runtime-next/tests/system/thermal_suite_survives_host_restart.rs)
    runs the same suite through both placements. **Status:** landed in the additive host.

24. **Cancellation Preserves Cleanup and Evidence.** Interrupt the active invocation
    while still running required cleanup, closing sample streams and sealing attributed
    evidence; do not confuse cancelling observation with cancelling admitted work.
    **Why:** Operator cancellation must not leave physical resources or evidence unfinished.
    **Where:** [execution sequencing](../crates/dokime-runtime-next/src/execution.rs),
    [cancellation requirement](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-execution-stop-and-cancel).
    **Proof:** `cancelling_a_thermal_step_runs_cleanup` in
    [cancelling_a_thermal_step_runs_cleanup.rs](../crates/dokime-runtime-next/tests/system/cancelling_a_thermal_step_runs_cleanup.rs)
    guards active cancellation/cleanup. Full observer-abandonment semantics have
    **No proof yet**; A17. **Status:** locked direction (`operation-grammar`) for
    the complete invariant; active cleanup is landed in the additive host.

## Glossary

- **Face / Visibility:** Face selects server, caller and contract package; visibility
  selects who may invoke an operation. Neither replaces the other or grants authority.
- **Operation / Invocation / Attempt:** Declaration / one execution / one try within
  its retry policy; an invocation identifier is not a semantic mutation key.
- **Position:** `(history, sequence)` in one history, not wall time or a cross-store order.
- **Cut:** One atomic read selection, possibly composite, not a scalar sequence or timestamp.
- **Page Token:** A continuation bound to selection, cut, ordering and schema, not a position.
- **Snapshot:** A read view at a cut, possibly retained, not a seal or an authorization grant.
- **Accepted / Durable:** Admitted work / saved work under the configured durability
  contract; accepted is not durable and flush cadence is not synchronization proof.
- **Instance / Plugins Service:** One running plugin protocol endpoint / host-side entry
  management, not interchangeable singular and plural names for one service.
- **Manager / Service:** Live-state/lifecycle owner / callable resource boundary;
  a service does not automatically need its own manager.
- **Scope / Grant:** Opaque inherited caller context / separately checked authority;
  scope is not a grant and a declared requirement is not permission.

## Decisions and Their Reasons

| Decision | Reason | Recorded Contract |
| --- | --- | --- |
| Three visibility levels; explicit SDK choice (2026-11-12) | Preserve truly local work, host-only requests and exposed composition separately. | [operation-grammar](../openspec/changes/operation-grammar/design.md#built-in-visibility-mapping) |
| Three audience faces (2026-11-12) | Keep instance protocol, invocation context and client authority independent. | [operation-grammar](../openspec/changes/operation-grammar/design.md#audience-faces-and-visibility) |
| Managers and direct library composition (2026-11-12) | One live owner; avoid rebuilding generic machinery and leaking product policy. | [operation-grammar](../openspec/changes/operation-grammar/design.md#ownership-and-managers) |
| Nested resources, whole-value Set and Append (2026-11-12) | Predictable verbs and revision safety without field-mask machinery. | [operation-grammar](../openspec/changes/operation-grammar/design.md#shared-grammar-and-vocabulary) |
| Latest durable/no wait; measured lag accepted (amended by the owner on 2026-11-13) | Consistent reads without making readers drive flushing. | [operation-grammar](../openspec/changes/operation-grammar/design.md#read-selection-and-durability) |
| Client resends; original receipt (2026-11-12) | Recover lost replies without applying an effect twice. | [operation-grammar](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-shared-failures-and-retry-policy) |
| Independent execution host, then one cutover (2026-11-12; model locked 2026-11-10) | Prove execution before replacing desktop/CLI consumers; retain complete behavior. | [grpc-plugins](../openspec/changes/grpc-plugins/design.md) |
| Captured shapes, product-owned root compatibility (2026-11-08) | Preserve old evidence across plugin changes; refuse only unreadable product state. | [derive-data-shape-versions](../openspec/changes/archive/2026-11-08-derive-data-shape-versions/design.md) |
| Retain the earliest durable inverse (recorded 2026-11-12) | Restore original state after cancellation, repetition or crash. | [operation-grammar](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-durable-external-effect-restoration) |
| Deterministic story redesign (2026-11-10) | Correctness must be independent of workstation timing and load. | [test-stories](../openspec/changes/test-stories/design.md) |
