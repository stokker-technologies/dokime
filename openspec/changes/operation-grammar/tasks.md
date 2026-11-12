# Operation Grammar Tasks

Status: locked by the owner on 2026-11-12
Naming rule amended by the owner on 2026-11-12: bare verbs on the service's resource

This is the future implementation inventory, not
authorization to implement. All boxes remain open in this documentation delivery.
The owner lock is recorded; settle remaining questions when their work is reached,
without blocking independent implementation. Source inventory:
`proto/dokime/v1/`, `crates/dokime-protocol/`, `crates/dokime-operations-next/`,
`crates/dokime-runtime-next/`, `crates/dokime-storage/`, `sdk/`,
`crates/dokime-client/`, `crates/dokime-cli/`, `crates/dokime-gui/`, affected
old domain/runtime/server consumers and stories, protocol/tooling generation,
README and maintained product/query/SDK/GUI guides. The sibling library owns its
primitives, language SDKs and conformance work in its own change. Refine exact
files during implementation without dropping any mapped capability.

## 1. Shared Primitives and Benchmark Cases

- [ ] 1.1 Introduce shared positions/cut binding, one page shape, protobuf Timestamp/Duration, failures, accepted/durable receipts and retry policy into canonical declarations. Keep language SDKs and generic primitives library-owned and product SDKs building on them; verify focused protocol/dependency proof through discovered `mise run` tasks.
- [ ] 1.2 Add a reusable durable-lag case to the Nightly benchmark set, usable against a consumer backend including the product store. Measure p50, p95 and maximum under small, multi-megabyte and mixed loads against the locked payload-size-independent 250 ms p95 bound. If missed, smaller and earlier size-triggered flushes come first; accepted-data snapshots are the named fallback and are not built now. Use benchmarks for timing and deterministic tests for byte-bounded flushes/splitting.
- [ ] 1.3 Add a large-file upload/download throughput benchmark case to the Nightly set for shared unary ranged resumable transfers, usable by clients and plugins. Measure before deleting streaming transfers; do not assert throughput in correctness tests.
- [ ] 1.4 Reconcile `grpc-plugins`, `storage-interface`, `imported-executions` and existing query contracts with the locked grammar; preserve one source inventory and one desktop/new-host cutover. Coordinate sibling-library prerequisites under its owner, including self-sufficient protocol/SDK/conformance, shared transfers and a published exact revision before final validation.

## 2. Complete Proto Inventory

- [ ] 2.1 Declare the complete locked inventory: 21 client services, 118 operations/gRPC methods and 236 named request/response messages (118 requests and 118 responses), the 24 present plugin/context mapping rows and their expanded destinations, including plugin-only Effects.Get. Keep Executions.GetProgress/SetProgress/WatchProgress for its genuine multiple paths, and only paged Executions.Parameters/Steps and Cases.Parameters. Review supporting/nested payloads to establish their exact total before generation; do not confuse method-message counts with all protobuf messages.
- [ ] 2.2 Apply bare resource verbs, noun-only single-mode parts, qualified mixed-resource methods and the service-prefixed message grammar, with the regrouped services in design.md. Give every method its response wrapper, remove automatic Run generation, and declare only explicit Watch semantics (including noun-only streamed parts), plugin carriers and managed replacements with an actual cooperative final-result promise. Make every Get/Set an exact whole-writable-value/revision mirror, splitting large settings/traceability into separately revisioned sub-resources where necessary. Declare Content.AppendUploadBytes and bounded ranged Content.Get; preserve all mapped capabilities and update protocol/query examples.
- [ ] 2.3 Regenerate canonical outputs for all four SDKs, descriptors, lifecycle IDs, fingerprints and requirements together; verify focused generation/drift/routing proof. The old desktop uses its existing separate projection until section 6, with no new compatibility adapter.

## 3. New Host Resource Semantics

- [ ] 3.1 Implement durable-frontier resolution and retained returned-cut reads for evidence plus mutable metadata, membership, schemas and documents; verify deterministic targeted stories for concurrent writes, restart, seal selection and explicit unavailability, with no read-triggered barrier or accepted snapshot, and document retention behavior.
- [ ] 3.2 Implement canonical Get/List and snapshot-then-durable-Watch behavior with row/byte/work bounds, stable continuation, progress and gaps; verify focused paging-under-mutation and resynchronization stories and update the query guide in this slice.
- [ ] 3.3 Implement whole-writable-value Set with the expected revision from its exact matching Get; stale conflicts name the action get again and retry; split large writable parts into separately revisioned Get/Set sub-resources. Implement replay-safe Append for records, samples, logs, upload bytes and document parts; verify focused conflict, lost-reply and accepted/durable receipt stories (client-owned resend, original receipt for the same request identifier, changed-payload conflict) and document the writable-field contracts.
- [ ] 3.4 Implement canonical execution/case/cycle planning and control, preserving validate-only versus atomic registration, command receipts and rerun notices; verify active-only Cancel, action-named queued refusal, Stop safe boundaries, cleanup and one-execution rename in focused stories, and update execution-host guidance.
- [ ] 3.5 Implement shared caller-bound unary upload sessions with ordered Content.AppendUploadBytes at stated offsets, identical replay/different-byte conflicts, resume/status, expiry/deletion and verified publication, plus bounded ranged Content.Get downloads whose replies state total length and digest; migrate artifact and managed-plugin admission to published content and verify focused disconnect/partial/digest stories with both client and plugin callers. Update upload guidance with those semantics.
- [ ] 3.6 Complete every currently missing plugin host operation (settings, secrets, artifacts, traceability, document parts, files, effects, environment and clock-now), then migrate all 17 context bindings and lifecycle families to the locked contracts; verify requirement/fingerprint admission and focused host stories without deleting missing capabilities, and update all affected SDK examples.
- [ ] 3.7 Complete the canonical plugin/media/access/settings/secret/resource reads and actions, including explicitly unsupported capture kinds; verify focused state/revision/failure stories and update the relevant product guide sections without promising unsupported operations.
- [ ] 3.8 Separate Documents.Get, Reports.Create preview, Reports.Get published reads, Reports.Publish and bounded content delivery; verify frozen-context equivalence and that rendering/download never publishes, with focused report/content stories and matching guide updates.
- [ ] 3.9 Replace only the operations with an actual promised cooperative final result by one managed method; verify typed cancellation settlement and lost-observer ownership while proving ordinary methods have no twins, and document the explicit exceptions.

## 4. SDK Consumers

- [ ] 4.1 Keep the Rust product SDK building on the library SDK while migrating plugin/client authoring, generated contracts, descriptors and fixtures; verify focused external-consumer and new-host stories and update the Rust SDK guide without parallel legacy contracts.
- [ ] 4.2 Keep the Python product SDK building on the library SDK, including pure generated clients, authoring, context helpers and examples; verify focused fresh-package/client/plugin stories and update its README in the same slice.
- [ ] 4.3 Keep the C# product SDK building on the library SDK, including the agreed resource-store/cancellation semantics and frozen descriptors; verify focused managed/remote stories and generated-service coverage with matching SDK documentation.
- [ ] 4.4 Keep the C++ product SDK building on the library SDK, including generated clients, publication and explicit lifecycle close; verify focused managed/remote stories and update its guide alongside the implementation.

## 5. Command-Line Consumers

- [ ] 5.1 Migrate `dokime-client`, CLI discovery/dynamic calls, lifecycle commands, uploads/downloads and retained-cut paging to the canonical new host; verify focused CLI/client stories, lost replies and sample requests, and update README/product/query examples with correct current behavior.

## 6. Desktop Cutover

- [ ] 6.1 Read `docs/gui.md`, then migrate desktop/native bridge DTO consumers, plugin entry/remove shapes, execution actions, read/watch paging, uploads/ranged downloads, reports and failures to canonical generated messages and the new host in one cutover; verify focused native desktop journeys and generated-client drift while updating GUI guidance. Do not first port the desktop to the old names on the new host.
## 7. Delete Superseded Code Last

- [ ] 7.1 Remove the old loop/domain/runtime/server paths, old generated projection and duplicate protocols/DTOs together with their superseded adapters, configuration and stories after equivalent new-host consumers build and the Nightly large-file transfer benchmark has measured the replacement; verify targeted capability stories and reference checks, preserving independent coverage, required workflows and development storage roots.
- [ ] 7.2 Review every mapping destination, README claim, guide, example, SDK requirement and affected OpenSpec owner against the combined tree; verify source inventory closure and focused format/language/repository checks through discovered project tasks. Mark source complete only after all implementation items and writers have handed off; final validation remains separate.

## 8. Questions to Settle When Reached

- [ ] 8.1 When SDK work is reached, settle four-language plugin parity and all-four-versus-Rust-only generic managed-client coverage with the owner; library ownership stays locked.
- [ ] 8.2 When returned-position reads are reached, settle how long a returned position stays valid and behavior across restart, including retained mutable context and explicit unavailability.
- [ ] 8.3 When Watch handlers are reached, settle exact opening/follow, progress, gap/resynchronization, bounded delivery and actual feed semantics under the locked position/durable-visibility rule.
- [ ] 8.4 When deletion is reached, settle behavior while readers hold data, including ranged downloads and retained-cut invalidation across backends.
- [ ] 8.5 When cancellation gRPC methods are reached, identify which operations promise a cooperative final result and therefore need a single managed replacement; preserve plugin carriers and ordinary resource ownership after observer loss.

## 9. Final Validation After Source Closure

- [ ] 9.1 Run strict OpenSpec validation and the combined candidate's required format/repository/language checks; verify and report actual results without treating this proposal's documentation proof as implementation proof.
- [ ] 9.2 Verify the published exact sibling revision is pinned, all local overrides are removed, and the combined project builds with no superseded consumers; use focused artifact/dependency proof before any full lane.
- [ ] 9.3 Run the authorized final native Windows and Linux protocol/conformance/product proof for cancellation, lost replies, restart, paging under mutation, sealed rendering, unary ranged transfers and all four language consumers; report counts only from final full runs and leave missing platform proof open.
- [ ] 9.4 Review the complete source and deterministic behavior evidence against every requirement and mapping; after separately authorized delivery, verify the remote candidate and use hosted CI/Nightly for full project validation, reporting failures and pending proof without changing dev or stable branch policy.
