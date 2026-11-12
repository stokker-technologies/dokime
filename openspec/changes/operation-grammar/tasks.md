# Operation Grammar Tasks

Status: locked by the owner on 2026-11-12
Boundary, managers and nested resources locked by the owner on 2026-11-12
Naming rule amended by the owner on 2026-11-12: bare verbs on the service's resource

This is the future implementation inventory, not authorization to implement.
All boxes remain open in this documentation delivery. The three further owner
decisions are LOCKED on 2026-11-12. The remaining questions in design.md are
settled when their owning work is reached, without blocking independent work.
No specification lifecycle activation is part of this task.

Source inventory: `proto/dokime/v1/`, `crates/dokime-protocol/`,
`crates/dokime-operations-next/`, `crates/dokime-runtime-next/`,
`crates/dokime-storage/`, all four `sdk/` trees, `crates/dokime-client/`,
`crates/dokime-cli/`, `crates/dokime-gui/`, old domain/runtime/server consumers
and stories, protocol/tooling generation, README and maintained product/query/
SDK/GUI guides. The sibling library owns its managers, generic services, language
SDKs and conformance work in its own change; tasks below explicitly identify
those dependencies, not permission to implement them in this repository.
Refine file ownership during implementation without dropping mapped capabilities.

Dependency order is sections 1 through 7. Each slice must compile with its focused
consumers before the next; keep the existing desktop projection intact until its
single migration in section 6. Declare new services under their final nested names
before migrating any desktop consumer. Remove superseded generic implementations
as their consumers move to composition; retain no compatibility aliases or wrappers.
Final full validation is separate from source completion.

## 1. Library Prerequisites Under Their Owner

- [ ] 1.1 Reconcile the sibling change, `grpc-plugins`, `storage-interface`, `imported-executions` and query contracts with the locked boundary, managers and nested grammar. Preserve one source inventory, product policy/storage ownership and one desktop cutover; verify the existing runtime manager types and their actual service coverage before editing.
- [ ] 1.2 Library: introduce shared positions/cut binding, bounded page fields, Timestamp/Duration, failures, accepted/durable receipts and retry policy in canonical declarations; keep operations/invocations, time Get/Sleep, storage, streams, plugin protocol and SDKs library-owned. Verify focused dependency proof through discovered project tasks.
- [ ] 1.3 Library: finish Plugins under PluginManager, including managed/remote entries, verified bundle admission, persistence, state/failures, Get/List/Watch/Create/Set/Restart/Delete and process lifecycle. Preserve interruption, finalization and cleanup contracts; verify focused entry/lifecycle stories.
- [ ] 1.4 Library: add Access and nested Access.Listener under GrpcManager, including Listener.Get/Set, Access.CreatePairing, Access.CreateCertificate regeneration and key rotation, revision conflicts and access-material lifecycle; verify focused listener/pairing/certificate stories.
- [ ] 1.5 Library: add Status and nested Status.Logs/TraceEvents under ServiceManager for host identity, health, availability, logs, trace events and diagnostics archive. Add narrow host-fact registration for the host application's version and component health, without product vocabulary, schemas or policy; wire generic HostValues.Get and host-supplied value resolver registration under OperationManager, with no settings/secret stores; verify standalone registration/health/archive/value-delivery stories.
- [ ] 1.6 Library: reuse and complete the existing Storage service under StorageManager as the only client/plugin transfer mechanism (CreateFileProduction, GetFileProduction, AppendFileBytes, CreateFileWriter for fenced resume, PublishFileProduction, DeleteFileProduction, GetFileBytes and WatchFileBytes), with no Content/Uploads aliases; preserve ordered offset Append, identical replay/different-byte conflict, resume/status, explicit expiry/Delete, verified Publish and bounded ranged Get with total length/digest. Verify focused lost-reply/partial/digest stories; keep product artifact/file metadata outside the library.
- [ ] 1.7 Library: add the shared descriptor-derived package/service table and small SDK-build generator for hierarchical client roots in all four languages. Strip owner/version prefixes, derive parent path plus service leaf mechanically, apply language casing and expose composition of distinct child paths with collision errors; no hand-written per-operation code. Verify generated routing/drift and independent generic consumers.
- [ ] 1.8 Library: settle four-language plugin parity and all-four-versus-Rust-only generic managed-client coverage with the owner; close visibility and C# invocation-resource/cancellation gaps and preserve self-sufficient protocol/SDK/conformance proof. Hierarchical roots in all four languages and library SDK ownership are already locked.
- [ ] 1.9 Library: add a reusable Nightly durable-lag benchmark case against the locked payload-size-independent 250 ms p95 bound, reporting p50/p95/max for small, multi-megabyte and mixed loads and supporting the product's independent backend. Try smaller byte-bounded pieces and earlier size-triggered flushing if needed; accepted-data snapshots remain the fallback and are not built now. Test byte bounds/splitting deterministically, never elapsed time.
- [ ] 1.10 Library: add the Nightly large-file unary upload/ranged-download throughput case for both client and plugin consumers; measure the replacement before superseded streaming transfer deletion, without throughput assertions in correctness tests.
- [ ] 1.11 Publish the completed upstream prerequisites under their owner's authorization and pin the exact published library revision before switching product consumers and final combined validation. Keep integration-only overrides temporary and remove them for the pinned candidate; verify focused dependency compilation and no product vocabulary in library contracts.

## 2. Final Product Contracts and Generation

- [ ] 2.1 Declare design.md's complete per-owner inventory: whole library 13 logical services (3 nested), 67 operations, 61 gRPC methods and 67 request/response pairs; product 39 logical services (22 nested), 111 operations, 93 gRPC methods and 111 pairs. Product clients declare 32 services (20 nested), 93 methods/pairs; the mapped library client subset has 7 services (3 nested), 27 methods/pairs, for 39 composed client services, 120 methods and 240 wrappers. Reconcile all 132 ordinary rows, 132 removed Run twins, 24 plugin/context rows and 18 product host-only additions with explicit ownership; census supporting payloads separately.
- [ ] 2.2 Apply the final resource hierarchy and bare verbs to product declarations: one proto file per service in a parent folder, lowercase parent sub-package, local ServiceMethodRequest/Response wrappers, package/service-derived IDs and no authored alias. Include cross-owner Plugins.Settings and Content metadata services, Settings.Get/Set/Schema, Plots.Settings, and all nested resources in the inventory. Add product Evidence.Archive for scoped archive selection/assembly and immutable content references, followed by direct library ranged byte delivery. Keep only paged Cases.Parameters and Executions.Parameters/Steps; use one Content.ExecutionFiles.Watch with identified-file/collection selectors, preserving growth/replacement.
- [ ] 2.3 Declare Executions.Progress.Get/Watch and include the same current value in Executions.Get. Keep the plugin producer as a host-only operation with its Get/revision mirror; remove every client-facing progress Set. Declare all remaining plugin/context destinations and Effects.Get, with exact Get/Set value/revision symmetry and identified sub-resources for large writable parts.
- [ ] 2.4 Remove automatic Run generation; retain explicit Watch semantics, plugin carriers and only managed replacements with a proven cooperative final-result promise. Generate all four SDK contracts, descriptors, IDs, fingerprints and requirements together. Extend each generated client root with product nested services using the shared descriptor table/generator and compose the library root without per-operation code; verify focused generation/drift/routing proof while the old desktop projection still builds.

## 3. Product Composition and Manager Split

- [ ] 3.1 Compose and serve library Plugins, Access/Listener, Storage and Status/Logs/TraceEvents directly on the product endpoint. Register product version/component-health facts through the narrow library registration; keep only product appearance, overview and environment reads in Backend. Preserve product authorization policy and storage definitions; verify direct library-client calls against the product host with no redeclaration or forwarding wrapper.
- [ ] 3.2 Migrate product consumers to library connection, invocation dispatch, stream handling, process launch and plugin entry management, then delete their product copies, requirements, fixtures and configuration in the same slices. Verify managed/remote admission, publication and cleanup through library managers, never the protocol crate alone.
- [ ] 3.3 Migrate listener/pairing/certificate, upload handling and generic host-status consumers to the composed services, then delete product implementations and duplicate status DTOs/handlers as callers move. Retain product storage schemas, settings/secrets stores, archives, OS credential-store integration and UI declarations. Verify direct service routing, policy and failure propagation.
- [ ] 3.4 Split Host/Inner and shared services/state.rs::State into execution, prompt and report managers; add a capture manager only if recordings remain. Decide execution-versus-small-cycle manager ownership at this split by the simplest complete option. Move queueing, execution lifecycle/retries, Stop/Cancel and recovery to the execution owner, pending questions to prompt, atomic publication to report; keep rendering pure. Keep each service method thin through one operation file calling a manager or storage. Evidence, Catalog, notes, Aggregates, Plots and Documents remain storage operations with no manager; verify focused lifecycle/publication/recovery stories and tree compilation after each slice.

## 4. Product Semantics and Plugin Operations

- [ ] 4.1 Settle returned-position retention/restart and deletion with active readers when implementing those operations, including retained mutable context, ranged downloads and explicit cut unavailability. Implement durable-frontier resolution for evidence plus metadata, membership, schemas and documents with no read-triggered barrier or accepted snapshot; verify targeted concurrent-write, restart and seal-selection stories and update the query guide.
- [ ] 4.2 Settle exact Watch opening/follow, progress, gaps/resynchronization, delivery bounds and actual feeds under the locked position/durable rule. Implement Get/List and snapshot-then-durable-Watch with row/byte/work bounds and stable continuation; verify focused paging-under-mutation and resynchronization stories, including product execution-file metadata watches and library byte growth/replacement/truncation watches.
- [ ] 4.3 Implement exact whole-writable-value Get/Set revision mirrors and action-named stale conflicts; split large settings/traceability by identified value when needed. Implement replay-safe Append for records, samples, logs and document parts; verify focused conflicts, client-owned resend, original receipts for unchanged request identifiers, changed-payload conflict and accepted/durable distinctions. Document writable fields alongside each operation.
- [ ] 4.4 Implement case/cycle planning and execution control through the execution/cycle owner, preserving validate-only versus atomic registration, command receipts and rerun notices. Verify active-only Cancel, action-named queued refusal, Stop at a safe boundary, cleanup and one-execution rename; update execution-host guidance.
- [ ] 4.5 Compose library transfer sessions with product Content.Artifacts/ExecutionFiles metadata and admission. Migrate artifacts and managed-plugin admission to published content, preserving scope, retained cuts and digests; keep evidence archive selection/assembly in product Evidence.Archive and return content references for library delivery, then measure the shared large-file replacement before deleting streaming transfers. Verify focused disconnect/partial/digest stories for client and plugin callers and update upload guidance without adding a second upload implementation.
- [ ] 4.6 Complete all missing plugin host operations for settings, secrets, artifacts, progress, traceability, document parts, files/capture, effects, environment and Time.Get through generic library dispatch plus product operations. Migrate all 17 context bindings and five lifecycle families; verify requirement/fingerprint admission and host-only progress production with client Get/Watch, without deleting missing capabilities. Update SDK examples with nested names.
- [ ] 4.7 Complete product resource reads/actions, settings/secret policy and explicitly supported capture kinds under the manager split. Keep settings declarations in the plugin catalog and values served through library HostValues.Get with a product resolver; preserve secret archive behavior rather than claiming encryption. Verify focused revision/state/failure stories and update affected product guide sections.
- [ ] 4.8 Separate Documents.Get, Reports.Create pure preview rendering, Reports.Get published reads, Reports.Publish atomic manager-owned publication and library Storage.GetFileBytes byte delivery. Verify frozen-context equivalence and that rendering/download never publishes; update report/content guidance.
- [ ] 4.9 Identify operations with an actual promised cooperative final result; replace only those ordinary methods with one managed method. Verify typed cancellation settlement, lost-observer ownership, plugin carriers and absence of automatic twins; document exceptions without changing method counts.

## 5. SDK and Command-Line Consumers

- [ ] 5.1 Migrate Rust product SDK authoring, clients, descriptors and fixtures on the pinned library SDK and descriptor-derived hierarchical root; verify focused external-consumer/new-host stories and update its guide without parallel legacy contracts.
- [ ] 5.2 Migrate Python product SDK authoring, generated async clients, context helpers and examples on the library SDK and hierarchical root; verify focused fresh-package/client/plugin stories and update its README.
- [ ] 5.3 Migrate C# product SDK frozen descriptors, generated hierarchical clients and agreed invocation-resource/cancellation behavior on the library SDK; verify focused managed/remote and generated-service coverage with matching SDK documentation.
- [ ] 5.4 Migrate C++ product SDK generated hierarchical clients, publication and explicit lifecycle close on the library SDK; verify focused managed/remote stories and update its guide.
- [ ] 5.5 Migrate dokime-client and CLI discovery/dynamic calls, lifecycle actions, composed-service uploads/downloads and retained-cut paging to final names on the new host. Verify focused lost-reply/client/CLI stories and sample requests; update README/product/query examples.

## 6. Single Desktop Cutover After Final Services

- [ ] 6.1 Read docs/gui.md and migrate desktop/native bridge DTO consumers, hierarchical client routing, plugin entry/remove shapes, execution actions/progress, read/watch paging, settings/secrets, library access/status/transfers, reports and failures to final canonical services on the new host in one cutover. Verify focused native desktop journeys and generated-client drift, updating GUI guidance; do not port the desktop to intermediate names first.

## 7. Remove Remaining Superseded Paths and Close Source

- [ ] 7.1 Delete the old loop/domain/runtime/server paths, generated desktop projection, duplicate protocols/DTOs, adapters, configuration and superseded stories once their new consumers build and replacement transfer throughput is measured. Confirm all product copies listed in section 3 are gone. Preserve independent coverage, required workflows and existing storage roots; incompatible development storage uses an explicit fresh root at baseline v1, never silent deletion/reset.
- [ ] 7.2 Review every destination, per-owner service/method/message count, README claim, guide, example, SDK requirement and affected OpenSpec owner against the combined tree. Verify focused format/language/repository checks through discovered tasks. Mark source complete only when every implementation item is closed; final validation remains separate and open until performed.

## 8. Final Validation After Source Closure

- [ ] 8.1 Run strict OpenSpec validation and required combined format/repository/language checks; report actual results without treating documentation proof as implementation proof.
- [ ] 8.2 Verify the exact published sibling revision is pinned, local overrides are gone and no superseded consumers remain; verify focused dependency/artifact proof before any full lane.
- [ ] 8.3 Run authorized final native Windows and Linux protocol/conformance/product proof for direct service composition, host-fact registration, four-language hierarchical roots, cancellation, lost replies, restart, paging under mutation, sealed rendering and unary ranged transfers. Report counts only from final full runs and leave missing platform proof open.
- [ ] 8.4 Review all requirements/mappings and deterministic behavior evidence. After separately authorized delivery, verify the remote candidate and use hosted CI/Nightly for full project validation, reporting failures and pending proof without changing branch policy.
