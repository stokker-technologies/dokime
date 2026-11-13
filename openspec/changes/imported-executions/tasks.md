# Imported Executions Tasks

## 1. Contracts Before Dependent Implementation

- [ ] 1.1 Adopt the grpc-plugins host execution model and render/publish split after its SDK wave; confirm storage-interface staged-manifest publication, conditional updates and durable receipts before host implementation. Lock producers, consumers, generated bindings, storage, recovery, UI, SDKs, exports, README/guides and focused proof inventory.
- [ ] 1.2 Resolve review issue 4 with the owner: stable cycle selector, atomic label resolution/create, ambiguous labels, closed admission and the draft-to-seal close race; record requirement scenarios before implementing cycle admission.
- [ ] 1.3 Resolve review issue 11 with the owner: disorder, timestamp ties, duplicate identity and gaps; record query/statistic scenarios before implementing ordering.
- [ ] 1.4 Resolve review issue 12 with the owner: default import windows, wide coverage, future outliers and typed query-limit failures; record scenarios before plot integration.
- [ ] 1.5 Resolve review issue 23 with the owner: validate arbitrary documents and select projection backfill or bounded raw-query preparation with explicit failure behavior before preview implementation.
- [ ] 1.6 Resolve review issue 24 with the owner: reconcile materialization/performance contracts, preview cancellation, cache policy and host work/concurrency budgets before preview implementation.
- [ ] 1.7 Resolve review issue 26 with the owner: units/calibration identity, quality counts, weighting, duplicates and first/last semantics before statistics integration.

## 2. Host Admission and Storage

- [ ] 2.1 Implement protocol/operation declarations, mappings and handlers for resumable bounded staging and atomic validated-manifest publication as an open draft; enforce checksums, limits and backpressure without live-stream admission or a run-queue entry. Prove incomplete/invalid manifest and concurrent upload behavior with deterministic focused cases.
- [ ] 2.2 Freeze Import origin, import-local schemas, parameters, records, source inventory, claimed importer/SDK, processing description, declaration, marked traceability and predecessor link; host-hash uploaded bytes, distinguish unverified claims, reject steps/attempt references/criteria and preserve whole-import semantics.
- [ ] 2.3 Implement store-scoped keys, revision-bound handles, conditional replacement/seal/discard and durable mutation receipts with request identity distinct from import key; prove lost responses, same-request replay after seal/restart, changed-payload conflicts, concurrent creation, stale revisions and sealed-key refusal naming the existing execution.
- [ ] 2.4 Implement handle-only draft access and finite revision snapshots; atomically publish seal, final document/report and cycle membership under the resolved cycle policy. Hide drafts from other clients, ordinary history/search/queries/exports; preserve prior cycle contents and draft on failed rendering or sealing.
- [ ] 2.5 Implement restart-safe staged transfer recovery, host-owned draft expiry pinned by active previews, discard and independent reclamation of all draft data/projections/attachments/metadata and orphan chunks; prove restart never invokes interrupted-run finalization on drafts and races cannot change pinned previews.

## 3. Time, Queries and Reports

- [ ] 3.1 Preserve source UTC, host receipt and durable positions with declared stream axes; validate absolute timestamps versus start plus exactly one elapsed_s/elapsed_ms/elapsed_us/elapsed_ns array, row lengths and exact integer-nanosecond conversion; reject absent/ambiguous clocks and overflow, retain measurement coverage apart from import-time history placement.
- [ ] 3.2 Reuse the plot pipeline's dokime.v1 columnar-frame message and storage-interface native columnar path; validate typed arrays, explicit validity, row count and timestamps, preserving signed/unsigned 64-bit extremes and bytes across SDKs without mandatory Arrow/dataframe dependencies.
- [ ] 3.3 Apply resolved ordering/gap/window/unit/statistical policies to queries, plots, summaries and finite draft follows; preserve fixed revision frontiers, expose source coverage and typed limit failures, and prevent cross-import name-based equivalence.
- [ ] 3.4 Build preview(document) on snapshot rendering without durable writes or publication, allowing repeated documents; apply resolved validation/preparation/budget/cancellation policy and clean temporary output. Make seal(document) render the final document at the expected revision before atomic persistence/publication; prove render failure and replacement races.

## 4. SDKs and Surfaces

- [ ] 4.1 Expose the locked flat Import -> import_execution -> draft.preview/seal/discard -> execution.report shape in Rust, Python, C# and C++; update history/search, desktop capabilities, measurement-period display and evaluated/declared/unevaluated cycle summaries while keeping drafts undiscoverable.
- [ ] 4.2 Export structured provenance with evidence; mark importer traceability/declarations and put persistence-versus-authenticity explanations in execution and cycle reports. Review affected README claims, guides and examples; replace automatic-seal and conflated retry-key guidance.
- [ ] 4.3 Add deterministic focused implementation proof for lifecycle/visibility/revision races, store identity/restart/fresh roots, shared-client conflicts, source clocks, columnar cross-language parity, provenance and corrections; respect all six open-decision gates.

## 5. Final Validation and Delivery

- [ ] 5.1 After all source work and targeted proof close, complete required format/language checks and native Windows/Linux validation through project tasks and authorized hosted CI; report pending proof truthfully.
- [ ] 5.2 Review acceptance against all 30 dispositions, keep implementation obligations open until proved, and request delivery authorization if absent before any push or integration.
