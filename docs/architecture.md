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
The execution host owns current behavior. Removed implementations do not establish
proof for this tree; missing journeys remain explicit obligations.

The [Operation Grammar](../openspec/changes/operation-grammar/design.md) owns the
locked mappings and requirement deltas. Its three-level decision supersedes earlier
two-level Private wording, including the host-requested operations in
[the gRPC design](../openspec/changes/grpc-plugins/design.md). Main specifications
that still describe superseded mechanisms are migrated by those changes, not silently
interpreted as the new implementation. Missing guards are unchecked in
[Operation Grammar Tasks](../openspec/changes/operation-grammar/tasks.md).

Upstream source citations below use **Harmos** paths at the exact revision pinned by
[Cargo.toml](../Cargo.toml), `81d32d3bc9ab3e2121746923712ec9508cee3ec7`, rather than
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
   product lifecycle/client declarations are not fully landed. Records, Logs, Statistics, Prompts, Documents, Files/Captures and Backend.Environment now have separate context wrappers and no client gRPC entry methods. The host now calls the pinned library's
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
   Build-time preference export composes declared product settings and the listener
   declaration used by Access configuration, without opening a host, storage or
   credentials. Runtime startup failures retain underlying operation causes;
   desktop diagnostics do not classify every startup failure as incompatible storage.
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
   **Status:** Access and Status composition landed; transfers and plugin composition remain open (`operation-grammar`).
   Explicit root disposal removes access and secret credentials through their
   product owners after the root lease is released, while ordinary shutdown keeps
   them for restart. Owners journal credential slots before native writes, so
   interrupted publication and failed retired-key removal retain cleanup ownership.
   `credentials_leave_with_discarded_roots` proves named access and secret credentials
   are unreadable after normal and failed Windows story teardown, including without
   their publication pointers.
   Plugins read Secret Storage directly by identifier, without bindings. A plugin
   may read only identifiers it declares as `Setting.kind.secret` in its
   `dokime.v1.SettingsDeclarations` catalog section. The context secret handler
   checks the connected calling plugin's description before any store access;
   undeclared reads receive the same declaration guidance whether the secret
   exists or not. Declared but missing secrets fail. Administrative client secret
   operations retain their separate access. `plugin_reads_only_declared_secrets`
   exercises this admission through real plugin invocations.

8. **Resource Operation Grammar.** Services name resources; methods use bare verbs
   from Get/List/Watch/Set/Create/Delete/Append, with explicit domain side effects
   and noun-only single-mode parts. Multi-path parts are nested resource sub-packages;
   whole-value Set carries expected revision, Append preserves ordered immutable data,
   and wrappers are `<Service><Method>Request` and `<Service><Method>Response`.
   **Why:** One predictable contract avoids aliases, field-mask machinery and lost updates.
   **Where:** [grammar and mapping](../openspec/changes/operation-grammar/design.md#shared-grammar-and-vocabulary),
   [grammar requirements](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md).
   **Proof:** Bounded product declaration/plan/pending lists return whole sets;
   `client_browses_catalog` checks schemas and whole catalog reads across restart.
   `client_observes_revisioned_progress` proves separate context Get/Set and client
   Get/Watch declarations, whole-value revision conflicts, original receipts and
   clearing fields. Progress has no client Set or automatic streaming twin.
   `client_observes_revisioned_traceability` proves independently revisioned named
   entries, whole-value replacement, stable scoped pages and durable watches.
   No proof yet for all final declarations/revision/replay behavior; A7.
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
   Progress retains issued durable positions across restart until execution
   deletion and refuses foreign/unissued positions; the revisioned-progress
   story guards that resource. Traceability retains scoped compound cuts and
   binds bounded history pages to the same selection across restart; its plugin
   journey verifies scope attribution and refusal of foreign histories.
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
    [current storage](../crates/dokime-storage/src/store.rs).
    **Proof:** No proof yet for the combined storage contract; A10.
    **Status:** locked direction (`operation-grammar`); storage-interface retains its
    own implementation work and is not activated by this document.

12. **Refuse Incompatible Roots.** Refuse incompatible persistent state with backup
    and explicit fresh-root/reset guidance; never silently delete, reset or migrate
    development data. The new store checks its database identity/layout at baseline v1.
    The host first takes an exclusive local.lock lease and returns a typed
    OccupiedRoot error naming the path before opening SQLite if another host owns it.
    Shutdown releases the lease once owned work has settled; a host dropped without
    shutdown releases it with its last handle and its execution workers.
    **Why:** A development schema break is allowed; destroying existing evidence is not.
    **Where:** [Store::open](../crates/dokime-storage/src/store.rs),
    [Incompatible error](../crates/dokime-storage/src/error.rs),
    [current compatibility requirement](../openspec/specs/data-shape-identity/spec.md).
    **Proof:** `client_collects_backend_diagnostics` in
    [client_collects_backend_diagnostics.rs](../crates/dokime-runtime-next/tests/system/client_collects_backend_diagnostics.rs)
    refuses a populated foreign root and preserves its bytes.
    `hosts_refuse_occupied_roots_before_opening_evidence` in
    [storage_root_admission.rs](../crates/dokime-runtime-next/tests/system/storage_root_admission.rs)
    proves external-lock refusal before database creation, cloned ownership and reopening. **Status:** landed.

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
    proves actual attempt evidence, not complete policy parity. The execution fixture also reads invocation-scoped Environment and Time.Get from its initialize call.
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
    follow the direction; the native desktop is restored to workspace membership
    and uses the embedded execution host through its native SDK connection owner.
    **Status:** locked direction (`grpc-plugins`, `operation-grammar`).

15. **Generic Library and Consistent SDK Concepts.** Library protocol, SDKs and
    conformance use generic vocabulary and remain independently usable without
    product objects; language SDKs stay in the library. Product SDKs compose them,
    naming the same concepts and resource hierarchy consistently across all four languages.
    Each language ships one package for plugin development and backend clients:
    Python dokime-sdk composed with the single harmos-sdk; audience-specific builds
    are removed, and a codebase may install itself via main.py. The API should
    look simple and good. Amended by the owner on 2026-11-13.
    The Python plugin forwards independent messages and record payload types to
    Harmos's catalog builder, which owns descriptor merging and the catalog digest.
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
    **Proof:** No proof yet for the complete test migration; A14. Retained stories
    still need the agreed invariant coverage. Do not add tests that quote this document.
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

    Statistics replies include their resolved read position. Compare complete replies
    only at the same returned position; equal measurements at later positions do not
    imply equal provenance. Cached statistics retain the position actually returned
    in each read's audit evidence. `step_failure_and_retries` guards a pinned repeated
    read and a later read over the same samples, alongside recovered and exhausted
    retries. Runtime observation may publish Connecting before Ready; consumers wait
    for the state they need rather than assuming the first update completed admission.

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
    **Where:** [effect restoration requirement](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-durable-external-effect-restoration).
    **Proof:** No proof yet in the current host; completing Effects must preserve
    earliest-inverse retention, confirmation ownership, reverse restoration order
    and startup refusal on failed inverses. See the replacement-journey task.
    **Status:** locked direction; context Effects remains unimplemented.

21. **Captured Shapes Preserve Evidence.** Schema-derived stream/record identities
    are equality identities, not authored versions; saved evidence retains the
    definitions needed to decode it after plugin evolution or removal. Plugin shape
    changes do not invalidate the product root.
    **Why:** Updating acquisition code must not erase or reinterpret historical evidence.
    **Where:** [shape requirements](../openspec/specs/data-shape-identity/spec.md),
    [frozen schemas](../crates/dokime-storage/src/execution.rs).
    **Proof:** No complete proof yet for stream/record evolution and removal
    across the current host; A15 retains that obligation. **Status:** locked direction.

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

## Product File Ownership

Product Files and Files.Uploads own client transfers and scoped plugin-host unary
transfers over the existing content store. They reuse library file/session/receipt
vocabulary and return per-call transfer limits. Published artifact, execution-file,
report, evidence-archive and appearance references use Files.Read. Replay receipts
and writer generations survive restart. The development storage baseline remains
v1; the added upload/request tables make older layouts explicitly incompatible,
without migration or automatic reset. Library in-process storage composition is
still task 0.H. Frozen record and stream schemas retain fully qualified protobuf
message bindings alongside descriptor pages. Document headers name the selected
document even when the caller uses the default selector. Each layout section has
a matching section-identity frame before structure completion; its identity and
the structure counters describe the same frozen document.

Product discovery remains blocked by the pinned library's lack of an external
service method-registration API; see the exact boundary in
[Execution Host](execution-host.md#product-file-resources). Native library operations
continue to advertise their actual bindings. Plugin-host Watch also needs a child
stream carrier; ordinary invocation children have one response.

## Arrow File Storage

This implementation preserves invariant 11, **Distinct Storage Guarantees**:
accepted live rows and a saved finite cut remain different guarantees. Invariant 7,
**Library Machinery and Product Policy**, places the physical backend in
`crates/dokime-storage/`; caller adoption through library operations remains 0.H.5.

The root uses `execution.sqlite3` (plus WAL/SHM), `backend-id`, `local.lock` and
`telemetry/<owner-stream-schema-sha256>.arrow`. The file identity hashes the
execution, stream and frozen descriptor identity. Samples have no SQLite payload
or scalar-projection tables. Their only durable representation is typed compressed
Arrow IPC. SQLite stores file and batch digests/checkpoints, ranges, stream state,
incremental numeric moments, compact binary adaptive previews and per-field preview
time bounds. The derived preview payload has an explicit encoding marker; it does
not encode sample rows. Existing records, resources,
files, execution metadata and reports retain their SQLite owners.

`ipc.rs` and `ipc/projection.rs` restore the old append/checkpoint/footer machinery
and bounded projected decoder. Appends synchronize files before committing their
catalog ranges; failed appends and catalog commits restore file/hash/block
checkpoints and publish no saved acknowledgement. Finalization appends metadata
only and commits stream closure in the same catalog transaction. Evidence barriers
retain the writer connection through the flush and following mutation; shutdown
closes sample admission under that lock. Pending accepted rows form the live tail; saved cuts exclude
later positions. Flush policy is owned by `telemetry/config.rs`: 65,536 rows or five
seconds, pressure at 64 MiB, evidence barriers and shutdown. Host settings compose
this policy; CLI configuration exposes row, interval, pending and query bounds.
Shutdown wakes and joins every active timed flusher before releasing its root.
The host admits each tick response through one `sample_batch` call after validating
its sequence and source timestamps. Batching preserves every producer sample,
including late source times, and retains the same file-sync-before-catalog order.
Record scans use the evidence kind/position index, including payload-free markers;
they do not depend on a record having scalar fields.

Plot/document readers consume projected batches. The pinned library supplies
scalar evaluation, bounded accumulators, sorting and downsampling where its current
query vocabulary fits. Each document render owns one isolated reader for all its
cards and buffers its HTML output.
Desktop documents return definitions, presentation and a fixed evidence selection;
`Documents.Get` does not render and discard invisible plot payloads. Its optional
`max_frames` bounds complete frames per page alongside `max_bytes`, preserving the
same retained cut, continuation and frame-order validation. The desktop requests
32-frame pages to respect decoded-memory admission as well as wire size.
Visible plot selections belong to the desktop. It partitions them into groups of
at most 32 requests, with independent generation-owned cancellation and reconnect.
One host batch owns one reader and one frozen descriptor pool per execution;
identical selections share their computation within that batch, with distinct plot
identities and selection fingerprints. Nothing survives the batch as a query cache.
Aligned time-series columns use the existing packed `PlotFrame.time` axis instead
of repeating recursive protobuf `Value` messages for every time coordinate.
Watch recomputes when its selected saved evidence advances, rather than on every
unrelated runtime notification. This preserves invariant 11, **Distinct Storage
Guarantees**: opening or watching a plot never flushes accepted rows to manufacture
a saved cut. Memoized chart leaves retain unchanged frames across group updates.

Frequency declarations select the latest `input_samples` finite samples in source
time and durable-position order, within the requested source window and filters.
The absent declaration uses the host's transform input bound; an explicit count
outside that bound is refused. This selects transform input independently of its
taper and output resolution. Unprojected reads visit newest Arrow batch ranges
first and stop once older ranges cannot change the selection. Projected/filter
reads retain the same bounded selection while evaluating their source rows.
Grid previews retain the latest complete frames that fit their declared axes and
output resolution. A single grid whose rows and columns exceed that resolution
still refuses. Numeric frame expressions denote source-time seconds; returned
frame instants are canonical timestamps, including for playback.
Frame reduction retains typed source-time keys and formats only the bounded
returned instants; walking a long history does not allocate timestamp text per row.
`ordered_merge.rs` contains one as-of merge over two ordered
Arrow batch iterators, with no product/storage types in its signature. Expression
binding uses it for stream-to-stream and record-to-stream alignment, record-based
ranges and grouping, values at records, and paired record intervals. Intervals pair
the previous record at the current record's time before alignment to later samples.
Exact integer rank selection spills projected order keys to an ephemeral query
file once and replays those keys in bounded radix passes, avoiding repeated batch
decompression and whole-population sorting in memory. Integer ordering and
interpolation retain their integer or rational type through evidence statistics,
plot summaries and document values. Record/expression percentiles use the same
spill-backed rank selector as stream percentiles, rather than the library's
floating percentile accumulator. Category reducers likewise accumulate typed
values; plot columns retain exact totals separately from drawing coordinates. `dokime.v1.Value.exact` uses the library's
`harmos.storage.v1.ExactNumber` for fractions and totals outside the 64-bit range;
ordinary integral results retain signed or unsigned integer values. The desktop
formats these values without converting them to JavaScript numbers. Integer
variance uses bounded exact sums of squares, so small spreads above f64 integer
precision survive. Standard deviation, RMS and chart coordinates remain floating
point; these drawing coordinates never replace the exact statistic or its label.

`Backend::for_execution` implements the pinned `StorageBackend` lifecycle, retained
snapshots and native Arrow scans for an execution history. Native schema identities
name its declared streams at baseline v1 and scan columns use `_storage.*`/`values.*`.
Retained tokens bind their original cursor; changing that cursor or schema version
is refused. Generic
schema/object/record/sample ports refuse because this pin's `PayloadEncoding`
contains only `PostcardV1`, while current evidence freezes protobuf descriptors.
The generic file ports also refuse: current product File resources own a separate
file history and writer/publication positions, whose mapping belongs to 0.H.5.
No alternate row encoding or duplicate file owner is installed to conceal those
boundaries. The backend is implemented here but is not registered by the host yet.

Execution archives copy finalized stream files unchanged under
`telemetry/<stream>.arrow`; `manifest.json` records paths, sizes and SHA-256, and
`evidence.pb` retains the sealed execution, records, reports and content. Cycle ZIPs
contain those execution ZIPs and a manifest. Sealed deletion records stream tombstones
in its transaction; startup completes pending physical deletion. Retained native
backend snapshots prevent deletion until their readers release the files.

The baseline remains v1. A root with the rebuilt `samples`, `sample_fields` and
`sample_batches` inventory is refused with an action to select a fresh root; no
existing root is migrated, reset or deleted. Diagnostics copy stream files too.

## SDK Build Ownership

Each of `sdk/cpp`, `sdk/csharp` and `sdk/python` owns its mise tasks and build
recipes. Repository host-fixture tooling calls those entry points; fixtures live
under `sdk/tests/fixtures`. C++ resolves its native graph through one pinned
vcpkg manifest and exports `Dokime::Cpp` as an installed package. C# builds one
solution with central dependency versions and consumes Harmos through a local
NuGet feed. Python ships typed resource modules and generated protobuf packages.
These build changes retain the current protocol and client call shapes; the
next library pin must regenerate their descriptor-derived surfaces.

The CLI embeds the authoritative SDK starters and includes the Python starter's
host-driven tests from `sdk/tests/fixtures/python/starter`. The per-language
`examples/plugin` and `examples/test` trees required by decision 0.J are not
present yet; that example consolidation remains with the protocol regeneration.
The C++ starter consumes the installed SDK archive with its supplied SHA-256,
then resolves the matching native dependencies through vcpkg.
