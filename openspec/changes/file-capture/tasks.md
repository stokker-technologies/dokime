# Tasks

## Source Inventory

Order is by dependency. One integration branch, `feat/file-capture`, holds every slice.

### Contracts

- [ ] 1.1 `crates/dokime-domain/src/model/files/wire.rs`: add `FileSource`, `FileSelection`, `FileLocation`, `CaptureMode`, `CaptureStarted`, `CapturedFiles`, `CapturedArtifact` and `CaptureGap`; remove `FileWatchFrame` and its payloads, `FileWatchPhase`, `FileWatchPolicy`, `FileWatchSelection`, `FileWatchArmed`, `WatchFileOutcome` and the `Unspecified` variants. Accept: requests read as in the design's caller code.
- [ ] 1.2 `crates/dokime-operations/src/files`: declare `CopyFiles`, `StartCapture` and `StopCapture` with constructors used by real call sites and their policies, with replay safety declared on `StopCapture`; delete `WatchFile`, `FileWatch` and `AwaitFileWatchArmed`, and the `LOCAL_ONLY` entry.
- [ ] 1.3 `crates/dokime-runtime/src/config`: add the capture configuration with one default each; replace the literal limits and bounds in `domains/files/mod.rs` and `execution/builtins/watch.rs`.

### Host Road

- [ ] 2.1 `crates/dokime-runtime/src/domains/files`: migrate the engine to `FileSource` and `CaptureMode`, derive discovery depth from rules, select `Newest` and fail when it finds nothing; remove `SnapshotAtStop`, `Selection::New` and source waiting.
- [ ] 2.2 Implement the `Mirror` interval cadence, ignoring notifications, with the final pass kept.
- [ ] 2.3 Implement the execution-owned capture registry with idempotent stop, foreign-capture refusal, runtime-owned sealing and sealing at execution end under the capture barrier; rewrite `domains/files/operation.rs` around it.
- [ ] 2.4 Publish `dokime.files.capture_gap` records; prove a gap record exists with no stop call.

### Harness Road

- [ ] 3.1 `crates/dokime-runtime/src/execution/builtins`: route all three operations for unhosted calls, migrate the local engine to the same shapes and behavior, keep the binding-owned registry sealing at run end, and remove `Ready` from `Serve`.

### Generic Removal

- [ ] 4.1 `crates/dokime-runtime/src/execution`: delete `WatchTicket`, `Readiness`, `LocalRun.readiness`, `impl StepInstance<WatchFile>` and the `WatchFile` branches in `calls.rs` and `instances.rs`; `RunProcess` behavior is unchanged.
- [ ] 4.2 `crates/dokime-macros/src/plugin.rs`, `crates/dokime-runtime/src/catalog.rs` and `execution/context/authoring.rs`: register the new built-ins and migrate `tests/fixtures/redundant_builtin.rs`.
- [ ] 4.3 `crates/dokime-server`: delete `src/grpc/files.rs`, the `Files` service and `tests/integration/execution_artifacts/file_watch.rs`; update authorization tests that name `FileWatch`.
- [ ] 4.4 Regenerate the GUI client and `packages/dokime-client`; prove no `FileWatch` remains in generated output.

### FTP Feature

- [ ] 5.1 Add `ftp` to `dokime-runtime` and `dokime`, remove `dep:suppaftp` from `native`, add it to `testing`, enable it from `dokime-server`, and gate `domains/files/ftp.rs` and its source variant. Refuse FTP with `UNSUPPORTED` without the feature.
- [ ] 5.2 Prove with `cargo metadata`-level dependency inspection that a `dokime/plugin-sidecar` consumer excludes `suppaftp`.

### Consumers and Tests

- [ ] 6.1 `plugins/sidecars/obc`: migrate `suites/startup/sequence.rs` and the acceptance hosting stubs to the new operations.
- [ ] 6.2 `crates/dokime-runtime/tests/integration`: migrate `domains/files`, `execution_workflow/steps.rs`, `plugin/native.rs`, `plugin/watch_stop.rs` and `storage_recovery/runtime_storage/artifacts.rs`; delete stories for removed behavior, including `cancelled_source_wait`.
- [ ] 6.3 Add targeted tests for the scenarios in `specs/file-capture/spec.md`, including `many_file_stop` through `StopCapture`, capture across steps, and the Mirror cadence on a controlled clock.
- [ ] 6.4 Reconcile with the `many_file_stop` repair on `dev`: keep any engine fix that still applies and delete cancel-path fixes this change supersedes.

### Documentation

- [ ] 7.1 Rewrite the file capture part of `docs/product-guide.md` around the three use cases, and update `docs/plugins.md`.
- [ ] 7.2 Update the `dokime-authoring` skill references for plugins and suites, with their generated copies.
- [ ] 7.3 Review `README.md` and `packages/dokime-client` documentation for affected claims.
- [ ] 7.4 Review the combined tree for leftover `watch` names and superseded paths; all writers hand off and declare SOURCE COMPLETE.

## Validation

- [ ] 8.1 Run `openspec validate file-capture --strict` and targeted tests through focused `mise` tasks for changed behavior on Linux and Windows.

## Workflow Follow-Up

- Run `stkr specification finish file-capture` as the candidate's last commit, then deliver per `stkr-publish` when authorized, with the single `mise run ci:simulate` in parallel with hosted CI.
- External plugins migrate in their own repositories after this lands. The owner receives the required changes as a report.
