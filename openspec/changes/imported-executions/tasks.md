# Imported Executions Tasks

## 1. Origin Protocol Before the Execution Manager

- [ ] 1.1 In `proto/dokime/v1/execution.proto`, add ExecutionOrigin with PluginRun and Import variants and source-file hashes; move TestCase template/plugin/digest into PluginRun, reserve removed fields and retain parameters/steps/criteria/streams numbering.
- [ ] 1.2 Update execution and plugin-call envelopes, generated protocol bindings, materialization producers/consumers and stored execution projections to carry origin; capture plugin start time and migrate every consumer before `grpc-plugins` execution-manager task 3.3.
- [ ] 1.3 Prove both origin variants and plugin materialization/reconnect semantics with focused protocol/runtime tests during implementation; no source changes or tests belong to this authoring task.

## 2. Host Operations After SDK and Storage Dependencies

- [ ] 2.1 After the `grpc-plugins` SDK wave and `storage-interface`, extend the application execution interface with the single import call and transaction-scoped begin, write_stream, put and seal/report operations described in proposal.md; preserve existing authorization.
- [ ] 2.2 Implement named-cycle resolution/creation, Import origin, parameters, records, artifact hashes, document validation and optional importer-declared outcome; reject steps and criteria without adding a dataset model.
- [ ] 2.3 Implement staged upload through report generation, atomic publication and recovery/rollback including new cycles, membership, files and keys; preserve existing cycle data on all failure paths.
- [ ] 2.4 Serialize store-wide key reservation/publication and refuse repeated sealed keys naming the existing execution; cover concurrent submission, failed retries and lost responses with focused deterministic tests.

## 3. Columnar Storage Path

- [ ] 3.1 Use the storage interface's columnar path for one table per stream with captured schema and source timestamps; never reconstruct per-sample transport messages.
- [ ] 3.2 Persist ordinary execution evidence, source artifacts, origin, declaration attribution and seal/report capture through existing owners; retain v1 baselines and explicit fresh-root refusal.
- [ ] 3.3 Prove atomic failure/recovery and source-time plot/query/statistics behavior with focused deterministic storage and operation tests.

## 4. Client SDK Surface

- [ ] 4.1 Rust: add the composed Import value, attachments, stream tables, records, document, optional outcome and one client call returning execution/report; stay protocol-only within the product graph.
- [ ] 4.2 Python: implement the illustrated Import and import_execution shape with flat startup and explicit close; migrate Backend, DokimeClient.new_cycle, Cycle and finite-reader lifecycle APIs and stubs identified in design.md without requiring context managers.
- [ ] 4.3 C#: expose the same composition and single-call semantics in the language's idiom with explicit resource release.
- [ ] 4.4 C++: expose the same composition and single-call semantics in the language's idiom with explicit resource release.
- [ ] 4.5 Prove equivalent evidence, report downloads and failure behavior across the four SDKs using focused tests; migrate affected examples, fixtures and superseded context-manager usage.

## 5. Report and Interface Marking

- [ ] 5.1 Mark imports in history, plots, documents, report and seal with accessible origin/source hashes; retain artifact access through ordinary evidence paths.
- [ ] 5.2 Show passed/failed as declared by the importer everywhere an outcome appears; preserve no-declaration state and plugin-evaluated outcomes without fabricating criteria.
- [ ] 5.3 Verify all provenance surfaces and report failure rollback through focused behavior tests and qualitative review without asserting authored prose.

## 6. Guides and References

- [ ] 6.1 Update product, remote and SDK guides and affected README claims when implementation lands; demonstrate local parsing/post-processing, one-call import and saving its report in every SDK language.
- [ ] 6.2 Finish source lifecycle migration identified in design.md, replacing context-manager-only public entry points and migrating consumers; retain cleanup under cancellation and original/cleanup failure grouping.
- [ ] 6.3 Reconcile current Python lifecycle specs with the flat deltas here and the gRPC removal of the Python backend; do not reintroduce embedded hosting.

## 7. Final Validation and Delivery

- [ ] 7.1 After combined source closure and focused proof, validate this change strictly and perform required native/hosted validation through project tasks when authorized; report actual results.
- [ ] 7.2 Review every acceptance scenario and deliver only with owner authorization; retire the implemented change through the normal lifecycle, not during this specification-only task.
