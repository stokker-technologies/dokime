# Operation Grammar Tasks

Status: proposed, not locked. This is the future implementation inventory, not
authorization to implement. All boxes remain open in this documentation delivery.
The owner lock and unresolved decisions precede implementation. Source inventory:
`proto/dokime/v1/`, `crates/dokime-protocol/`, `crates/dokime-operations-next/`,
`crates/dokime-runtime-next/`, `crates/dokime-storage/`, `sdk/`,
`crates/dokime-client/`, `crates/dokime-cli/`, `crates/dokime-gui/`, affected
old domain/runtime/server consumers and stories, protocol/tooling generation,
README and maintained product/query/SDK/GUI guides. The sibling library owns its
primitives, SDK decision and conformance work in its own change. Refine exact
files after lock without dropping any mapped capability.

## 1. Owner Lock and Dependencies

- [ ] 1.1 Measure accepted-to-durable/read-visible lag and synchronous commit cost in the library and independent product store using the owning benchmark workflow; record the resulting design decision without adding time/load assertions to correctness tests.
- [ ] 1.2 Resolve every open question in design.md with the owner, including SDK ownership, parity, measured lag acceptance, cut retention/restart, file deletion, managed settlement and exact shared shapes; verify a dated lock and matching shared wording in both repositories before implementation.
- [ ] 1.3 Reconcile `grpc-plugins`, `storage-interface`, `imported-executions` and affected existing query contracts with the locked grammar; verify one source inventory and one desktop/new-host cutover rather than separate migrations.
- [ ] 1.4 Complete the sibling library prerequisites under its own owner: canonical primitives/policy, authoritative services, conditional managed streaming, upload/session support and the selected SDK ownership; verify targeted native proof and a published exact revision before removing integration-only overrides and pinning it for final validation.

## 2. Canonical Contracts While the Existing Desktop Still Builds

- [ ] 2.1 Introduce the locked shared positions/cut binding, page shape, time types, failures, receipts and retry policy into canonical product declarations; update generation inputs, descriptors and focused drift/routing stories together, verifying focused protocol and dependency builds through discovered `mise run` tasks.
- [ ] 2.2 Declare the complete client inventory in design.md, uniform response wrappers and plugin requirements/lifecycle IDs without automatic Run generation; regenerate canonical outputs for all four SDKs in one compiling slice and verify no generated canonical contract depends on retired names. The old desktop continues on its already separate projection until section 5, not a newly added adapter.
- [ ] 2.3 Review every one of the 132 ordinary mappings and 132 removed twins plus 24 plugin/context rows against the new inventory; verify preserved capabilities, exposure and wrapper ownership by descriptor review and focused routing proof, and update the protocol/query guide examples in the same slice.

## 3. New Host Resource Semantics

- [ ] 3.1 Implement durable-frontier resolution and retained returned-cut reads for evidence plus mutable metadata, membership, schemas and documents; verify deterministic targeted stories for concurrent writes, restart, seal selection and explicit unavailability, with no read-triggered barrier or accepted snapshot, and document retention behavior.
- [ ] 3.2 Implement canonical Get/List and snapshot-then-durable-Watch behavior with row/byte/work bounds, stable continuation, progress and gaps; verify focused paging-under-mutation and resynchronization stories and update the query guide in this slice.
- [ ] 3.3 Implement masked/whole-value Set with revision guards and replay-safe Append for records, samples, logs and document parts; verify focused conflict, lost-reply and accepted/durable receipt stories and document the writable-field contracts.
- [ ] 3.4 Implement canonical execution/case/cycle planning and control, preserving validate-only versus atomic registration, command receipts and rerun notices; verify active-only Cancel, action-named queued refusal, Stop safe boundaries, cleanup and one-execution rename in focused stories, and update execution-host guidance.
- [ ] 3.5 Implement shared caller-bound unary upload sessions, range replay/conflicts, resume/status, expiry/deletion and verified publication; migrate artifact and managed-plugin admission to published content and verify focused disconnect/partial/digest stories with both client and plugin callers. Update upload guidance with those semantics.
- [ ] 3.6 Complete every currently missing plugin host operation (settings, secrets, artifacts, traceability, document parts, files, effects, environment and clock-now), then migrate all 17 context bindings and lifecycle families to the locked contracts; verify requirement/fingerprint admission and focused host stories without deleting missing capabilities, and update all affected SDK examples.
- [ ] 3.7 Complete the canonical plugin/media/access/settings/secret/resource reads and actions, including explicitly unsupported capture kinds; verify focused state/revision/failure stories and update the relevant product guide sections without promising unsupported operations.
- [ ] 3.8 Separate GetDocument, CreateReport preview, GetReport published reads, PublishReport and bounded content delivery; verify frozen-context equivalence and that rendering/download never publishes, with focused report/content stories and matching guide updates.
- [ ] 3.9 Replace only the operations with an actual promised cooperative final result by one managed method; verify typed cancellation settlement and lost-observer ownership while proving ordinary methods have no twins, and document the explicit exceptions.

## 4. SDK and CLI Consumers

- [ ] 4.1 Apply the owner's SDK placement decision to Rust plugin/client authoring, generated contracts, descriptors and fixtures; verify focused external-consumer and new-host stories and update the Rust SDK guide without parallel legacy contracts.
- [ ] 4.2 Apply that same decision to Python, including pure generated clients, authoring, context helpers and examples; verify focused fresh-package/client/plugin stories and update its README in the same slice.
- [ ] 4.3 Apply that same decision to C#, including the agreed resource-store/cancellation semantics and frozen descriptors; verify focused managed/remote stories and generated-service coverage with matching SDK documentation.
- [ ] 4.4 Apply that same decision to C++, including generated clients, publication and explicit lifecycle close; verify focused managed/remote stories and update its guide alongside the implementation.
- [ ] 4.5 Migrate `dokime-client`, CLI discovery/dynamic calls, lifecycle commands, uploads/downloads and retained-cut paging to the canonical new host; verify focused CLI/client stories, lost replies and sample requests, and update README/product/query examples with correct current behavior.

## 5. One Desktop Cutover and Clean Removal

- [ ] 5.1 Read `docs/gui.md`, then migrate desktop/native bridge DTO consumers, plugin entry/remove shapes, execution actions, read/watch paging, uploads, reports and failures to canonical generated messages and the new host in one cutover; verify focused native desktop journeys and generated-client drift while updating GUI guidance. Do not first port the desktop to the old names on the new host.
- [ ] 5.2 Remove the old loop/domain/runtime/server paths, old generated projection and duplicate protocols/DTOs together with their superseded adapters, configuration and stories after equivalent new-host consumers build; verify targeted capability stories and reference checks, preserving independent coverage, required workflows and development storage roots.
- [ ] 5.3 Review every mapping destination, README claim, guide, example, SDK requirement and affected OpenSpec owner against the combined tree; verify source inventory closure and focused format/language/repository checks through discovered project tasks. Mark source complete only after all implementation items and writers have handed off; final validation remains separate.

## 6. Final Validation After Source Closure

- [ ] 6.1 Run strict OpenSpec validation and the combined candidate's required format/repository/language checks; verify and report actual results without treating this proposal's documentation proof as implementation proof.
- [ ] 6.2 Verify the published exact sibling revision is pinned, all local overrides are removed, and the combined project builds with no superseded consumers; use focused artifact/dependency proof before any full lane.
- [ ] 6.3 Run the authorized final native Windows and Linux protocol/conformance/product proof for cancellation, lost replies, restart, paging under mutation, sealed rendering, uploads and all four language consumers; report counts only from final full runs and leave missing platform proof open.
- [ ] 6.4 Review the complete source and deterministic behavior evidence against every requirement and mapping; after separately authorized delivery, verify the remote candidate and use hosted CI/Nightly for full project validation, reporting failures and pending proof without changing dev or stable branch policy.
