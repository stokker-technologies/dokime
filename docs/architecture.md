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
[Cargo.toml](../Cargo.toml), `82e394dac656bf6b28a31f7429360d84f1701051`, rather than
assuming another checkout's HEAD matches this product. `Harmos:` paths resolve
inside that pinned dependency revision; other paths resolve in this repository.
The repaired generated client/plugin surfaces compile independently of the product
consumers. Runtime admission and application proof remain open in the
[gRPC plugins tasks](../openspec/changes/grpc-plugins/tasks.md).

Composition narrows callers before projecting either SDK surface. Shared payloads
have one public home; versioned wire packages remain internal and the repository
public-surface check enforces the documented allow-list. Wrong callers use the
library wrong_caller failure (refused, code -32604, action Call). Generated-only
compiler stories do not establish runtime admission or storage integration.

Paging, SDK packaging, transfers and durable lag were amended by the owner on
2026-11-13. The linked Operation Grammar records their complete amended contracts.

## Invariants

1. **Executors and Callers.** Protocol files are owned by the executor: model/v1
   contains exchanged values, host/v1 contains host operations and plugin/v1
   contains host-invoked plugin operations. Host methods require the library
   callers option (CLIENTS, PLUGINS or ALL), independently of visibility and grants.
   **Why:** A resource has one declaration even when both clients and plugins use
   it, while plugin callers cannot acquire client administration. **Where:**
   [protocol](../proto/dokime), [composition](../protocol/composition.toml) and the
   complete [identity/caller map](execution-host.md#operation-identity-and-caller-mapping).
   **Status:** definitions and generator projections are landed in this candidate;
   runtime admission, consumer adoption and wrong-caller proof remain open. Plugin
   executor methods have no callers option; their Protected bindings are host-only and generate no application client/context resource.

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
   [current host admission](../crates/dokime-runtime/src/plugin.rs).
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
   [current dispatcher](../crates/dokime-runtime/src/dispatch.rs),
   [host-only child dispatch table](../crates/dokime-operations/src/lib.rs).
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
   [current client protos](../proto/dokime/host/v1), [SDK operation](../sdk/rust/src/operation.rs).
   **Proof:** No proof yet for the complete contract; A3 must guard generation and
   settlement. Current client Run twins remain. Unary file transfers replace
   streaming transfers, with 4 MiB default chunks and an 8 MiB default message
   limit, amended by the owner on 2026-11-13. **Status:** locked direction (`operation-grammar`, `grpc-plugins`).

5. **Opaque Invocation Scope.** Carry and inherit scope unchanged; the library
   never interprets it, treats it as an authorization token or infers it from payload.
   Product handlers decide its meaning and whether it is required.
   **Why:** Generic routing must not acquire execution semantics or let a child widen authority.
   **Where:** [scope vocabulary](../openspec/changes/operation-grammar/design.md#shared-grammar-and-vocabulary),
   [product scope validation](../crates/dokime-operations/src/resources.rs),
   [SDK binding](../sdk/rust/src/context.rs).
   **Proof:** No proof yet for the full cross-provider contract; A4. Product
   `execution_isolation` in [execution_isolation.rs](../crates/dokime-runtime/tests/system/execution_isolation.rs)
   proves execution separation, not library opacity or peer inheritance.
   **Status:** locked direction (`operation-grammar`).

6. **Managers Own Live Orchestration.** Create a manager only for changing live
   state, background work or lifecycle. Library PluginManager owns entries/processes,
   GrpcManager transport/access, StorageManager storage/transfers, ServiceManager
   status/diagnostics/clock, OperationManager dispatch/invocations, StreamManager
   streams, RecordManager capture and StateManager live state; product execution,
   prompt and report managers own their lifecycles, with capture only if recordings remain.
   Local file captures are an execution-owned registry settled before evidence closes.
   **Why:** One owner must join work and settle shutdown; storage reads and pure
   rendering need no invented manager. Assembly observes native plugin changes for
   redacted diagnostics, and waits for product watches to settle after execution
   shutdown before detaching its gRPC listener. Cycles group execution-manager tasks and
   do not own an independent scheduler.
   **Where:** [manager ownership table](../openspec/changes/operation-grammar/design.md#ownership-and-managers),
   [current Host](../crates/dokime-runtime/src/host.rs),
   [composition State](../crates/dokime-runtime/src/services/state.rs),
   [execution manager](../crates/dokime-runtime/src/execution_manager.rs),
   [prompt manager](../crates/dokime-runtime/src/prompts.rs) and
   [report manager](../crates/dokime-runtime/src/reports.rs).
   **Proof:** A5: `execution_shutdown_joins_runs_after_observer_loss`,
   `prompts_settle_answers_and_shutdown_waiters` and
   `reports_publish_atomically_and_close_admission` in
   [manager_lifecycles.rs](../crates/dokime-runtime/tests/system/manager_lifecycles.rs)
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
   Evidence archives read scoped native file histories through the registered
   backend and publish their output through native uploads. Product seals,
   artifact metadata and transaction ownership remain in the product Store;
   archive stream members preserve the original Arrow bytes.
   **Where:** [boundary and settings requirements](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md),
   [current composition](../crates/dokime-runtime/Cargo.toml),
   [product settings](../crates/dokime-operations/src/host/settings/get.rs).
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
   **Status:** Access, Status, native transfers and library plugin entries are composed.
   Executions bind the admitted activation once per unit with a trusted scope and
   independent clock; lifecycle dispatch uses the library operation manager. Scoped
   sample ingestion acknowledges only after the product Arrow write and binding
   closure precedes sealing. Product file presentation stores logical paths and
   original native file references in the execution transaction. Retired SQL file
   byte objects and writers are removed; incompatible development roots are preserved
   and refused at baseline v1. Final combined validation remains a separate obligation.
   Explicit root disposal removes access and secret credentials through their
   product owners after the root lease is released, while ordinary shutdown keeps
   them for restart. Owners journal credential slots before native writes, so
   interrupted publication and failed retired-key removal retain cleanup ownership.
   Opening a root claims its backend identity for that folder in the native store
   (`services/root_claim.rs`). A folder copy carries `backend-id` and catalogs but
   not the claim: while the original exists, the copy refuses to open with
   credential records and refuses credential removal, so it can never delete the
   original's accounts. A moved root reclaims; removal releases the claim.
   Access-key cleanup checks native absence before retiring its slot journal;
   a remaining or unreadable key preserves the root for cleanup. Access and shared
   secret owners submit native creation, reads and removal to one dedicated
   process-lifetime credential thread, including root reopen and explicit discard.
   Concurrent-host stories retain independent calling-thread absence assertions.
   Native test and benchmark runners retain child scratch ownership and remove
   root credentials after child exit, including termination, before deleting scratch.
   `credentials_leave_with_discarded_roots` proves named access and secret credentials
   are unreadable after normal and failed Windows story teardown, including without
   their publication pointers.
   The locked definition now composes the library PluginEntry and plugins.Settings;
   product plugin records/settings wrappers and clock operations are deleted.
   Product plugins.Secrets is a read-only view of declared names and their
   availability. Secrets.Get admits only a calling plugin's declared names and
   reads the host-wide secret stored under exactly that identifier (owner, 2026-11-15). Administrative
   metadata is client-only. Existing stories describe behavior to preserve during
   this consumer migration; they do not prove the new definitions.

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
    [client_collects_backend_diagnostics.rs](../crates/dokime-runtime/tests/system/client_collects_backend_diagnostics.rs)
    refuses a populated foreign root and preserves its bytes.
    `hosts_refuse_occupied_roots_before_opening_evidence` in
    [storage_root_admission.rs](../crates/dokime-runtime/tests/system/storage_root_admission.rs)
    proves external-lock refusal before database creation, cloned ownership and reopening. **Status:** landed.

13. **Callee-Owned Policies and Configured Limits.** Deadlines, delays and retries
    come from operation policy/configuration; an override changes one call only.
    Retry requires declared replay safety; never increase budgets or limits to make proof pass.
    **Why:** Callers must not silently weaken failure, cancellation or resource guarantees.
    The locked unset policy means no deadline, zero cancellation grace and no retries.
    Time reads/waits use the authoritative host clock, including SDK child waits.
    **Where:** [policy requirement](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-shared-failures-and-retry-policy),
    [configured reads](../openspec/specs/query-operations/spec.md),
    [current dispatch policy](../crates/dokime-runtime/src/dispatch.rs).
    **Proof:** No proof yet for all policy/override paths; A11. `step_failure_and_retries`
    in [step_failure_and_retries.rs](../crates/dokime-runtime/tests/system/step_failure_and_retries.rs)
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
    [runtime manifest](../crates/dokime-runtime/Cargo.toml).
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
    [existing stories](../crates/dokime-runtime/tests/system).
    **Proof:** No proof yet for the complete test migration; A14. Retained stories
    still need the agreed invariant coverage. Do not add tests that quote this document.
    **Status:** locked direction (`test-stories`, `operation-grammar`).

17. **Execution Evidence Outlives Live Runs.** Execution is an open/closed/sealed
    evidence container with frozen origin; run sequencing is live host state, not a
    persisted Run object. PluginRun freezes the case/catalog/process identity;
    Import carries its own manifest and never invents a plugin or runnable case.
    **Why:** Durable evidence and recovery must survive the process that produced them.
    **Where:** [execution model](../crates/dokime-storage/src/execution.rs),
    [host creation](../crates/dokime-runtime/src/host.rs),
    [requirements](../openspec/changes/grpc-plugins/specs/host-executions/spec.md).
    **Proof:** `thermal_suite_survives_host_restart` in
    [thermal_suite_survives_host_restart.rs](../crates/dokime-runtime/tests/system/thermal_suite_survives_host_restart.rs)
    and `execution_capabilities` in
    [execution_capabilities.rs](../crates/dokime-runtime/tests/system/execution_capabilities.rs).
    **Status:** landed in the additive host; full import admission remains separate work.

18. **Attributed Outcomes.** Evaluated, declared and not-evaluated outcomes remain
    distinct; only evaluated results determine the cycle verdict. A declared Passed
    import cannot make an unevaluated cycle Passed.
    **Why:** The host must not claim it evaluated externally asserted results.
    **Where:** [outcome folding](../crates/dokime-storage/src/outcome.rs),
    [requirements](../openspec/changes/grpc-plugins/specs/host-executions/spec.md#requirement-attributed-outcomes-and-cycle-verdicts).
    **Proof:** `cycle_outcomes_keep_declared_and_unevaluated_separate` in that source
    and `execution_outcomes` in
    [execution_outcomes.rs](../crates/dokime-runtime/tests/system/execution_outcomes.rs).
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
    **Where:** [document rendering](../crates/dokime-runtime/src/documents.rs),
    [report storage](../crates/dokime-storage/src/report.rs),
    [requirement](../openspec/changes/grpc-plugins/specs/host-executions/spec.md#requirement-report-rendering-and-publication).
    **Proof:** `documents_and_reports` in
    [documents_and_reports.rs](../crates/dokime-runtime/tests/system/documents_and_reports.rs)
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
    [current failure translation](../crates/dokime-runtime/src/dispatch.rs).
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
    **Where:** [plugin entry](../crates/dokime-runtime/src/plugin.rs),
    [process owner](../crates/dokime-runtime/src/process/mod.rs),
    [requirement](../openspec/changes/grpc-plugins/specs/host-executions/spec.md#requirement-managed-and-remote-plugins).
    **Proof:** `thermal_suite_survives_host_restart` in
    [thermal_suite_survives_host_restart.rs](../crates/dokime-runtime/tests/system/thermal_suite_survives_host_restart.rs)
    runs the same suite through both placements. **Status:** landed in the additive host.

24. **Cancellation Preserves Cleanup and Evidence.** Interrupt the active invocation
    while still running required cleanup, closing sample streams and sealing attributed
    evidence; do not confuse cancelling observation with cancelling admitted work.
    **Why:** Operator cancellation must not leave physical resources or evidence unfinished.
    **Where:** [execution sequencing](../crates/dokime-runtime/src/execution.rs),
    [cancellation requirement](../openspec/changes/operation-grammar/specs/operation-contracts/spec.md#requirement-execution-stop-and-cancel).
    **Proof:** `cancelling_a_thermal_step_runs_cleanup` in
    [cancelling_a_thermal_step_runs_cleanup.rs](../crates/dokime-runtime/tests/system/cancelling_a_thermal_step_runs_cleanup.rs)
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
| Windowed interval batches stored expanded (2026-11-16) | Plugins answer host windows with columns; storage expands each part column-wise at admission, so Arrow readers and roots need no change. | [Arrow File Storage](#arrow-file-storage) |

## Product File Ownership

Product Files and Files.Uploads own client transfers and scoped plugin-host unary
transfers over the existing content store. They reuse library file/session/receipt
vocabulary and return per-call transfer limits. Published artifact, execution-file,
report, evidence-archive and appearance references use Files.Read. Files.Watch
selects exactly one published file or upload; upload watches follow durable bytes
before publication and retain the selected writer identity. Replay receipts
and writer generations survive restart. The development storage baseline remains
v1; the added upload/request tables make older layouts explicitly incompatible,
without migration or automatic reset. Library in-process storage composition is
still task 0.H. Frozen record and stream schemas retain fully qualified protobuf
message bindings alongside descriptor pages. Document headers name the selected
document even when the caller uses the default selector. Each layout section has
a matching section-identity frame before structure completion; its identity and
the structure counters describe the same frozen document.

The final library supplies composed registration and effective caller restrictions.
The product runtime still must adopt that registration; an old pin's missing API
is not a blocker at this revision. Plugin-host streams use the library's managed
child-watch contract rather than inventing a unary streaming twin.

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
and bounded projected decoder; `ipc::encode` builds a record batch message apart from
any file, with the bytes a direct append writes. `Store::open_streams` creates every
opened stream's Arrow file in parallel, syncs each header and records the streams with
their empty file entries in one transaction; a file left without an entry by a crash
holds no saved rows and is recovered to its verified header on the next open.

A flush round (`telemetry/writer.rs`) claims the head of every stream's pending
batches and prepares each stream file in parallel: concatenation, `flush_rows`
chunking, IPC encoding and column-kernel moments and previews against the file's
loaded statistics. Its commit writes and syncs every file in parallel, then saves one
catalog transaction for all of them, so a saved position never names rows a crash
could lose. Claimed rows stay pending, and so in the live tail, until that transaction
commits; any failure rolls every writer back to its committed prefix and publishes no
saved acknowledgement. Finalization appends metadata only and commits stream closure
in the same catalog transaction. Pending accepted rows form the live tail; saved cuts
exclude later positions.

Flush policy is owned by `telemetry/config.rs`. A stream reaching 65,536 rows wakes the
background round at once, and five seconds after its oldest pending row the timer does;
the round prepares without the store's locks and drops claims a synchronous flush saved
first. It writes under the locks, then syncs duplicated file handles without them, so
admission on the host's stream delivery never waits for a file sync; the round catalogs
its batches afterwards unless a flush holding the locks first synced and cataloged them,
as every synchronous flush, finalization and later round does before claiming. Pending bytes reaching 64 MiB flush inline on the admitting caller, which bounds
memory. Evidence barriers retain the writer connection through a synchronous flush and
the following mutation, and shutdown flushes and closes sample admission under that
lock. Without a runtime, a due row count also flushes inline. Host settings compose this
policy; CLI configuration exposes row, interval, pending and query bounds. Shutdown wakes
and joins every active flush worker before releasing its root. The product host runs on
a multi-thread runtime, so a brief wait on the store's blocking locks never stalls plugin
transport, clients or the flush timer.
Interval streams arrive as host-granted windows (Harmos `efc261fe`): each stored
publication is one `harmos.batch.<row>` part with `start_unix_nanos`, `period_nanos`,
`count` and the row's fields as columns. `Store::sample_interval` decodes that
Postcard part column by column straight into the frozen row's Arrow arrays, with no
per-sample protobuf, JSON or dynamic-message work, and generates per-sample
`source_ns = start + i * period` (also `receipt_ns`), consecutive positions and the
stream's next sequences. Telemetry files therefore keep one Arrow row per sample
with explicit time, position and sequence columns (about 32 bytes of metadata per
sample before compression), so every reader — evidence queries, criteria
statistics, plots, tiles, summaries, frequency selection, the time merge, exports,
archives, live views and the clients — reads interval samples unchanged, and
existing roots stay readable. Device streams keep `sample_batch`, which validates
per stream rather than per sample. Pending accepted rows are held as Arrow batches
in their stored layout: the live tail and frontier read them by filtering positions
and times, and flush rounds slice them into `flush_rows` batches with the same
file-sync-before-catalog order. Pressure and row thresholds use running counters.
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
`expressions.rs` uses `harmos_query::merge::TimeMergeReader` over retained,
projected table readers at one cut. Its AtOrBefore plan supplies the shared
time alignment for stream-to-stream and record-to-stream expressions, record-based
ranges and grouping, values at records, and paired record intervals. Intervals pair
the previous record at the current record's time before alignment to later samples.
Exact integer rank selection spills projected order keys to an ephemeral query
file once and replays those keys in bounded radix passes, avoiding repeated batch
decompression and whole-population sorting in memory. Integer ordering and
interpolation retain their integer or rational type through evidence statistics,
plot summaries and document values. Record/expression percentiles use the same
spill-backed rank selector as stream percentiles, rather than the library's
floating percentile accumulator. Category reducers likewise accumulate typed
values; plot columns retain exact totals separately from drawing coordinates.
`dokime.model.v1.Value.exact` uses the product model's
`dokime.model.v1.ExactNumber` for fractions and totals outside the 64-bit range;
ordinary integral results retain signed or unsigned integer values. The final
library has no storage wire model; the product preserves the former ExactNumber
fields (negative = 1, numerator = 2, denominator = 3) at its model boundary.
The desktop formats these values without converting them to JavaScript numbers. Document
structure carries pending bindings; result bindings, summary groups, tiles and
card completion follow the evidence cut. Literal table rows also settle at card
completion, with their declared counts still enforced. Integer
variance uses bounded exact sums of squares, so small spreads above f64 integer
precision survive. Standard deviation, RMS and chart coordinates remain floating
point; these drawing coordinates never replace the exact statistic or its label.

`Backend::open(&store)` returns the library's `StorageBackends` pair, sharing the
root's durable history. `backend.rs` and `backend/**` implement the native primary
and recovery contracts. Their catalog is `storage.sqlite3`; native sample batches
live in compressed `storage.arrow/<id>.arrow` files. SQLite retains replay digests
and column bounds, with no native sample payload or Arrow-column blobs. Each file
is synced before its catalog transaction commits; startup collects uncataloged
owned files after a failed transaction. Native row reads reconstruct Postcard
values from those Arrow columns, preserving optional messages and repeated values.
Native storage and the existing descriptor-driven evidence store remain distinct
until operations/runtime callers complete their native-port adoption. Product
protobuf values and descriptors must be mapped explicitly at those callers.

Execution archives copy finalized stream files unchanged under
`telemetry/<stream>.arrow`; `manifest.json` records paths, sizes and SHA-256, and
`evidence.pb` retains the sealed execution, records, reports and content. Cycle ZIPs
contain those execution ZIPs and a manifest. Sealed deletion records stream tombstones
in its transaction; startup completes pending physical deletion. Native file
deletion fences new readers while existing snapshots retain their original bytes;
releasing the final older reader permits physical reclamation.

The baseline remains v1. A root with the rebuilt `samples`, `sample_fields` and
`sample_batches` inventory is refused with an action to select a fresh root; no
existing root is migrated, reset or deleted. Diagnostics copy stream files too.

## Final Library Assembly and Storage Contract

This definition preserves invariant 7, **Library Machinery and Product Policy**:
"Compose library services directly, without redeclaring or forwarding them."
Invariant 11, **Distinct Storage Guarantees**, still separates accepted live rows,
durable cuts, projection frontiers and atomic conditional mutations. The physical
SQLite catalog and compressed Arrow file backend remain product-owned in
`crates/dokime-storage`; the library owns its interfaces, native operations,
StorageManager/StateManager and query engine. Its in-memory backend is test support.

The exact [operation/message/caller map](execution-host.md#operation-identity-and-caller-mapping)
is canonical.
The definition replaces these ownership paths:

| Former Owner | Final Definition Owner |
| --- | --- |
| dokime.client.v1 and dokime.plugin.host.v1 | dokime.host.v1 and its nested resource packages |
| dokime.v1 shared grab-bag values | dokime.model.v1 for exchanged values; otherwise the owning host/plugin resource |
| Product Plugin/Plugins and plugin-settings wrappers | harmos.host.v1.Plugins and harmos.host.v1.plugins.Settings |
| Product clock operation declarations | harmos.host.v1.Time |
| AnalysisRequest/AnalysisResponse | dokime.plugin.v1.AnalysesComputeRequest/AnalysesComputeResponse |
| Product generic ordered merge | harmos-query/src/merge.rs; consumer deletion is still open |

Both assembly and SDK generation read `protocol/composition.toml`. It composes
harmos.model.v1, harmos.host.v1 and harmos.plugin.v1, including nested resources,
and narrows Operations.List/Get to CLIENTS. List is host-wide browsing; Get is
arbitrary contract retrieval. Plugins receive their admitted requirements through
the host, so neither discovery operation is needed by them. Invocations remains
available for admitted peer/child calls; Time and own-entry plugins.Settings.List
remain shared. Narrowing never widens an upstream caller set.

The runtime consumer must apply this shape before listeners or plugins start:

```rust
let backends = Backend::open(&store)?;
let storage = StorageManager::with_catalog(catalog, backends, storage_settings)?;
setup.settings().storage(&storage);
setup.composition(include_str!("../../../protocol/composition.toml"))?;
setup.catalog(builtins::storage_catalog(), Lane::default());
```

This is the required assembly contract, not a claim that runtime consumers already
implement it. `primary` implements StorageBackend and `state` implements
StateBackend with identical StoreHistory. Keep one manager per root, not one per
execution. Product histories/scopes select executions under that root. Every data
operation stays in-process through the library catalog, never a storage gRPC API.
The runtime does not open the descriptor path in the composition file.

At this pin the native PayloadEncoding enum still contains only PostcardV1.
The backend consumer must encode native object/record/sample values and their
schemas truthfully; protobuf bytes must never be relabeled as Postcard. Preserve
the frozen protobuf definitions and map product values losslessly at the native
port. Arrow scans supply projected batches to the library query engine directly;
this is no permission for a duplicate durable row representation or new wire codec.


### Backend Module Ownership

| Product Modules | Required Ownership |
| --- | --- |
| backend.rs, store.rs, history.rs, administration.rs | Native ports, catalog transactions, stable history, original receipts and recovery-state persistence. Product plugin administration persistence is removed. |
| ipc.rs, ipc/**, telemetry/**, streams.rs, stream_* | Physical Arrow append/projection/flush/checkpoints and catalog pruning; saved/live boundaries and compressed recovery. |
| execution.rs, cycles.rs, plans.rs, progress.rs, traceability.rs, notes.rs, report.rs | Product objects and records reached through library native operations; no new storage interface or manager. |
| content.rs and file/upload helpers | Native staging/publication, fences, checkpoints and resumable sessions under one content owner. Wire uses library FileReference/Upload, not native backend structs. |
| expressions.rs, plot*.rs, document*.rs, numeric*.rs | Product expression binding and presentation; consume projected library query results and the library time merge. Preserve the parallel correctness repair. |
| ordered_merge.rs | Delete after expressions.rs moves to TimeMergeReader; no product generic merge remains in the completed consumer migration. |

### StorageBackend Method Inventory

The signatures and values are owned by the pinned
[StorageBackend](https://github.com/stokker-technologies/harmos/blob/82e394dac656bf6b28a31f7429360d84f1701051/crates/harmos-storage/src/backend.rs).
Every method below must be implemented for the product's native capabilities,
except the three explicit fallback optimizations. A default unsupported result is
not implementation of an exposed capability. `get_work` supplies native work
counters for product diagnostics and bounded-work evidence.

| Method | Product Obligation |
| --- | --- |
| `list_record_views` | Optional optimization; retain the shared bounded fallback. |
| `get_record_position` | Optional optimization; retain the shared bounded fallback. |
| `get_work` | Implement the product native capability; do not rely on unsupported default. |
| `advance_projection` | Optional optimization; retain the shared bounded fallback. |
| `set_object` | Implement the product native capability; do not rely on unsupported default. |
| `delete_object_receipts` | Implement the product native capability; do not rely on unsupported default. |
| `set_projection_definition` | Implement the product native capability; do not rely on unsupported default. |
| `get_projection_definition` | Implement the product native capability; do not rely on unsupported default. |
| `get_projection_status` | Implement the product native capability; do not rely on unsupported default. |
| `set_projection_output` | Implement the product native capability; do not rely on unsupported default. |
| `list_projection_rows` | Implement the product native capability; do not rely on unsupported default. |
| `delete_projection_history` | Implement the product native capability; do not rely on unsupported default. |
| `set_index_definition` | Implement the product native capability; do not rely on unsupported default. |
| `get_index_definition` | Implement the product native capability; do not rely on unsupported default. |
| `list_index_entries` | Implement the product native capability; do not rely on unsupported default. |
| `capabilities` | Implement the product native capability; do not rely on unsupported default. |
| `list_column_rows` | Implement the product native capability; do not rely on unsupported default. |
| `get_history` | Implement; required by trait. |
| `create_session` | Implement; required by trait. |
| `get_saved_position` | Implement; required by trait. |
| `delete_session` | Implement; required by trait. |
| `append_schemas` | Implement; required by trait. |
| `append_object_revisions` | Implement; required by trait. |
| `append_records` | Implement; required by trait. |
| `append_samples` | Implement; required by trait. |
| `append_file_references` | Implement; required by trait. |
| `create_snapshot` | Implement; required by trait. |
| `delete_snapshot` | Implement; required by trait. |
| `list_schemas` | Implement; required by trait. |
| `list_object_revisions` | Implement; required by trait. |
| `list_records` | Implement; required by trait. |
| `list_samples` | Implement; required by trait. |
| `list_files` | Implement; required by trait. |
| `get_object` | Implement; required by trait. |
| `get_file_bytes` | Implement; required by trait. |
| `delete_object_history` | Implement; required by trait. |
| `delete_expired_object_revisions` | Implement; required by trait. |
| `create_file_staging` | Implement; required by trait. |
| `append_file_bytes` | Implement; required by trait. |
| `publish_file_production` | Implement; required by trait. |
| `delete_file_staging` | Implement; required by trait. |
| `create_file_production` | Implement the product native capability; do not rely on unsupported default. |
| `set_file_writer_generation` | Implement the product native capability; do not rely on unsupported default. |
| `set_file_production_checkpoints` | Implement the product native capability; do not rely on unsupported default. |
| `list_file_production_checkpoints` | Implement the product native capability; do not rely on unsupported default. |
| `get_file_production` | Implement the product native capability; do not rely on unsupported default. |
| `get_file_production_bytes` | Implement the product native capability; do not rely on unsupported default. |
| `delete_file_production` | Implement the product native capability; do not rely on unsupported default. |
| `delete_file` | Implement the product native capability; do not rely on unsupported default. |
| `append_mutation_receipts` | Implement the product native capability; do not rely on unsupported default. |
| `list_mutation_receipts` | Implement the product native capability; do not rely on unsupported default. |
| `list_identities` | Implement the product native capability; do not rely on unsupported default. |

### StateBackend Method Inventory

State is the durable recovery owner, independent of the sample/catalog writer but
sharing its history. Recovery snapshots and entries retain their original required
data positions; flushing through a recovery position establishes its durability.

| Method | Product Obligation |
| --- | --- |
| `get_history` | Implement the exact pinned StateBackend contract. |
| `create_recovery_session` | Implement the exact pinned StateBackend contract. |
| `get_recovery_snapshot` | Implement the exact pinned StateBackend contract. |
| `set_recovery_snapshot` | Implement the exact pinned StateBackend contract. |
| `list_recovery_entries` | Implement the exact pinned StateBackend contract. |
| `append_recovery_entries` | Implement the exact pinned StateBackend contract. |
| `flush` | Implement the exact pinned StateBackend contract. |
| `delete_recovery_session` | Implement the exact pinned StateBackend contract. |

### Library Query Merge and Removals

The product merge module and per-execution backend facade are removed.
`TimeMergeReader` now supplies expression alignment with bounds from the product
storage settings. The native backend implements every StorageBackend method;
`advance_projection` returns `None` to use the library's bounded executor.
Record views and record-position seeking are implemented. All eight StateBackend
methods persist recovery entries, snapshots and their saved frontier in the same
native catalog. Recovery flush refuses a frontier whose required primary data is
not durable. No implemented native method reports an unsupported capability.

`harmos_query::merge::TimeMergeReader::new` takes two `Box<dyn BatchReader>`, a
native `TimeMerge` plan and a nonzero output row bound. Both readers must retain
the same cut. `TimeMerge { source, primary_time, secondary_time, mode, work }`
selects Range { end }, AtOrBefore or Intervals. Bind product source-time and record
interval columns once, push catalog pruning/projection into native backend scans,
and consume bounded batches for record ranges, values at records, grouped record
intervals and stream-to-stream alignment. No DataFusion or general product join
engine is introduced.

The remaining consumer migration removes direct Store paths in operations/runtime, product installation
records, settings wrappers and clock dispatch. The definition already deletes
product plugin service/handler modules and Store::save_plugin_administration.
Keep product schemas, report publication, expression binding, durable Arrow I/O and
all storage fault/archive stories. The existing root is never silently reset;
formats remain at baseline v1 and incompatible development roots require an
explicit fresh path.

## SDK Build Ownership

### Final Library Pin

The definition pass pins Harmos at `82e394dac656bf6b28a31f7429360d84f1701051`.
Consumer repairs follow the table below.
The pin and generated contracts do not establish consumer compilation.

| Consumer | Required Change at This Pin |
| --- | --- |
| Operations and Runtime | Use model/host/plugin identities, effective callers and direct library Plugins, plugins.Settings and Time composition. |
| Storage | Implement the native StorageBackend and StateBackend ports; register primary/state owners at assembly; replace the product ordered merge with harmos-query. |
| Rust | Adopt the internal wire layout and public root/client/plugin barrels; use generated argument-based calls. |
| Python | Resolve the pinned git dependency and regenerate private wire modules and typed client/context resources. |
| C# | Build the local NuGet feed from the same immutable revision; use its generated public namespaces and call contexts. |
| C++ | Fetch the same immutable source archive; compose the installed library types and generated resource tree. |
| Command-Line Tool | Rename the candidate binary to dokime and adopt generated client calls and library plugin records. |
| Desktop | Adopt the generated executor contracts and library plugin/settings records; preserve the concurrent plot/document repair. |

At `efc261fe` (windowed interval batches) every SDK wraps both observer forms: the
one-sample observer, called once per sample of a host window, and the window
observer, which returns a columns message mirroring the row (Rust
`Observer::window`, Python `@window_observer`, C# `WindowObserver<Row, Columns>`,
C++ `WindowObserver<Row, Columns>`), each with an optional window setting. The grant is
`SampleWindow` in Rust, Python and C++ (C# passes `Harmos.Plugins.Window`), because
`Window` already names the evidence time selection. The host
sets `PlacementSettings::interval_window` from `interval_window_ms`
(`dokime_runtime::INTERVAL_WINDOW` by default) and `PlacementSettings::interval_lag`
from `interval_lag_windows` (`dokime_runtime::INTERVAL_LAG`, 10 window lengths); the
desktop's built-in host reads both from its Interval Sampling settings at start. The
library's `Lagging`, `PoolFull` and `WindowExceedsPool` stream failures map in
`execution.rs` to Interrupted outcomes and `streams:failure` evidence that name the
stream and the setting that resolves it. Dokime keeps no demand on idle plugin streams. `Tick` and `samples.tick` are gone
from the vendored protocol and every fixture.

Each of `sdk/cpp`, `sdk/csharp` and `sdk/python` owns its mise tasks and build
recipes. Repository host-fixture tooling calls those entry points; fixtures live
under `sdk/tests/fixtures`. C++ resolves its native graph through one pinned
vcpkg manifest and exports `Dokime::Cpp` as an installed package. C# builds one
solution with central dependency versions and consumes Harmos through a local
NuGet feed. Python ships typed resource modules and generated protobuf packages.
The final pin regenerates descriptor-derived surfaces and private wire output.
SDK implementation/installed-consumer proof follows the SDK rows of the Final
Library Pin table.

The CLI embeds each SDK's `examples/plugin` and `examples/test` trees as its
starters (decision 0.J). The Python test example carries its host-driven story in
`tests/`, so `dokime new --lang python` ships the story its `mise run test` runs.
The C++ starter consumes the installed SDK archive with its supplied SHA-256,
then resolves the matching native dependencies through vcpkg.
