# gRPC Plugins Tasks

## 1. Contract

- [x] 1.1 Settle the open points in `design.md` with the owner and record each outcome there.
- [x] 1.2 Reconcile the overlapping changes and write the spec deltas that retire superseded requirements.
- [x] 1.3 Lock the source inventory: producers, consumers, tests, fixtures, examples, tasks, README and guidance.
- [x] 1.2.1 Retire Python backend, native-wheel, embedded-callback, generation reload and dependency-start-order requirements; retain remote authoring, typed clients, diagnostics and focused proof requirements in replacement deltas.
- [x] 1.3.1 Protocol inventory: `proto/{dokime,harmos}/`, `crates/dokime-protocol/{src,examples,tests}`, client-service generation in `crates/dokime-server/`, document bindings under `sdk/{python,csharp,cpp}` and protocol export in `crates/dokime-cli/src/commands/proto.rs`; check generated drift and protobuf round trips.
- [x] 1.3.2 Host inventory: `crates/dokime-domain/src/model/`, `crates/dokime-operations/src/`, `crates/dokime-runtime/src/{catalog.rs,contributions.rs,domains,execution,runtime,storage,transactions,transport}`, and `crates/dokime-server/src/{grpc,host}`; migrate producers and consumers together, preserving effects, bounded queries/transfers, authorization and durability.
- [x] 1.3.3 Consumer inventory: `crates/{dokime,dokime-client,dokime-cli,dokime-packaging,cargo-dokime,dokime-gui/src-tauri}`, GUI source/generated clients and report assets, `sdk/`, `plugins/sidecars/{host,obc}` and examples/starters; migrate capabilities, origin/outcome, settings and catalog consumers with their producers.
- [x] 1.3.4 Removal/proof inventory: `crates/dokime-macros`, old execution workflow/provider/replay and embedded-plugin tests/fixtures, Python native bindings/stubs/backend and wheel tooling; retain meaningful effect/recovery/transfer/report/plot journeys under their replacement owners and use focused package targets only.
- [x] 1.3.5 Configuration/guidance inventory: workspace/package manifests and locks, build scripts, `mise.toml`, `tools/{project.toml,python}`, `.github/workflows`, README, `docs/{product-guide,tooling,gui}.md`, topic guides and `.agents/skills/dokime-{architecture,authoring,conventions}`. Update only affected current contracts; no customer or private plugin identifiers.
- [x] 1.3.6 Ownership/order: this worker owns the inventory except GUI benches, new bench files under GUI e2e, `tools/python/src/dokime_tools/test/run.py`, `tools/python/tests/integration/test_desktop_bench.py`, other bench tooling under `tools/` and `plot-refresh-scaling`; existing desktop stories and the report-browser fixture are owned here. Preserve pre-existing shared manifest/lock/tooling edits. Contract/protocol precede Harmos adoption, execution model precedes manager, SDKs precede removals; storage extraction and import admission remain later work. Each source checkbox requires focused native proof; final/platform validation stays separate.

## 2. Protocol

- [x] 2.1 Define and check in `dokime.v1` host operations, `TestCase`, `Step`, `Criterion` and `Statistics`; generate the protocol crate.
- [x] 2.1.1 Reproduce the hosted Windows interruption timeout five times by exact name; all five pass. Examined terminal-query servicing, batch acknowledgement, held mutation replies, admission watches and the frozen-origin commit; no cleanup regression or permanently unsent event was established.
- [ ] 2.2 Add the host operations plugins need that exist only as evidence-page fields today: progress, log, traceability, document parts, record reads and statistics.
- [ ] 2.3 Replace JSON catalog readers and producers with `dokime.v1` messages; add running-host, plugin URL and bundle export sources. Embedded host-source export is tracked in 2.6.
- [ ] 2.4 Serve settings as a host operation and delete settings injection at start and the applied-versus-saved comparison.
- [ ] 2.5 Add the per-execution resource store (`ctx.set`, `ctx.get`) for Rust and remove template fields and `Own`.
- [x] 2.6 Export embedded Dokime and Harmos protocol sources through `dokime proto export --out`, preserving package paths without opening storage.
- [x] 2.7 Attach the checked-in protocol sources to releases through the archive owned by `release-layout` task 2.3; the packaged-CLI export/archive path passes 11 focused release tooling tests and the complete CLI export/descriptor test on Windows. Current-source release packaging and hosted publication remain final proof.
- [ ] 2.8 Wave 2: make proto files authoritative, put generated definitions and light helpers in protocol, document cross-language semantics for each SDK, and inventory Harmos native state/recovery/storage semantics retained in `harmos-domain`; move wire mirrors out.
  - [x] Client Services: declare Plugins (9), Catalog (6), Planning (3), Execution (28), Observation (4), Reports (3), Health (1) and Backend (6) in new per-service proto files.
  - [x] Client Services: declare Appearance (1), Content (9), Documents (2), Evidence (3), History (11), Media (5), Notes (3), Plots (3), Prompts (5), Query (6), Remote (4), Runtime (2), Secrets (7), Settings (8) and Telemetry (2).
  - [x] Client Services: express all 129 served client operations plus canonical plugin List/Add, with the 131 matching cancellable Run-method routes (262 RPCs total); no served client operation is unexpressed. Internal provider-run and retired plugin evidence/lease calls have no client gRPC binding and are not client services.
  - [x] Client Services: reuse canonical values/catalog/documents/prompts and frozen origins; expose attributed outcomes, open/closed/sealed execution state, separate live state and managed/remote entries. Add Metadata tags/attributes/times and rename the colliding Prompts catalog payload to PromptCatalog; retain native percentile rank explicitly. Handler/client migration remains with 3.9–3.11 and the SDK workers.
  - [x] Client Services Isolated Proof: regenerate Rust/descriptors/reflection through the unchanged mise task in a protocol-only copy under target; focused build, seven protocol integration tests, Rust format and focused Rust lint pass. The generator proves the complete Python/C#/C++ proto set without writing SDK-owned files. The focused lint emits the existing root Clippy configuration's unreachable tokio process-path warning but exits successfully.
  - [x] Client Services Repository Proof: `mise run check -- --repository` passes; protoc 36.2 generates Python, C# and C++ for the complete Dokime/Harmos proto set under target/proto-sdk-proof. No handler or SDK source is edited.
  - [ ] Client Services Worktree Proof: generation, focused build/test and `mise run format` are blocked before compilation by the root workspace's missing sdk/rust/Cargo.toml. Merge the SDK owner's source and rerun these tasks in the combined tree; isolated proof does not close this gate.
- [x] 2.8.1 Export an authoritative descriptor set with checked-in generated messages; expose descriptor lookup for host operation declarations and prove generation drift.
- [x] 2.8.2 Document shared value, identifier, timestamp, resource, error, settings and operation semantics for all four SDKs; inventory retained Harmos native semantics against the requested revision.
- [ ] 2.8.3 Replace remaining Dokime wire mirrors and migrate their declarations and consumers; dissolve the domain owner in 5.9 after the host and SDK waves.
- [x] 2.8.3.1 Define canonical appearance/backend diagnostics messages and generate the protocol-owned embedded source inventory; prove drift and complete CLI export/descriptor agreement (five protocol tests and one focused CLI export test, focused CLI build and protocol/CLI lint on Windows). This inventory work does not close wire-mirror or handler adoption.
- [ ] 2.8.3.2 Protocol handoff: `dokime-protocol` has generated `dokime.v1` plugin messages and reflection but no client service declarations or generated client stubs. The SDK binds generated messages to discovered Harmos `Method` contracts; complete service declarations/generation in the protocol owner before promising service-wide client coverage. First declare Backend, Appearance and Plugins with canonical messages and Harmos managed-call envelopes; retain the other service areas and all handler adoption in this task.
- [x] 2.8.3.2.1 Declare Backend, Appearance and Plugins service methods and managed-call envelopes in their authoritative proto files; regenerate Rust client/server interfaces and descriptors. The service-envelope journey, generator drift and protocol package Rust check pass on Windows. This does not close the remaining services or native handler adoption.

- [ ] 2.9 Host-wave prerequisite owned here, before 3.2/3.3: define Origin with PluginRun containing frozen TestCase/plugin/template/digest/process start time and Import containing its definition/manifest; reserve removed TestCase identity fields, regenerate bindings and migrate persistence, materialization and plugin-call envelopes. No separate execution-definition or Run type.
- [x] 2.9.1 Define frozen Origin/PluginRun/Import protobuf messages, reserve TestCase identity fields and add origin to every plugin execution-call envelope; regenerate and prove round trips and origin agreement.
- [ ] 2.9.2 Migrate persisted execution definitions, catalog materialization and every execution-call consumer to the frozen origin; refuse incompatible development roots with explicit backup/reset guidance.
- [ ] 2.9.2.1 Replace `dokime-domain/src/model/execution/snapshot.rs` case/template/plugin snapshots and their storage/finalization consumers with the frozen protocol origin; keep execution identity/cycle separate and preserve existing roots on refusal.
- [ ] 2.9.2.2 Replace `model/definition/{suite,case_materialization}.rs` and runtime planning/catalog producers with parameter expansion plus pure plugin materialization; retain the returned case and provider digest/process identity together.
- [ ] 2.9.2.3 Migrate runtime execution-call producers, SDK handlers and client/report projections to the same origin/case envelopes; remove superseded snapshots/conversions in the same cutover and prove storage round trips and mismatched-origin refusal.

## 3. Host

- [ ] 3.1 Adopt the Harmos `grpc-plugins` revision and add `Plugin::{Managed, Remote}` with the add and remove operations and entry states.
- [x] 3.1.1 Adopt the exact-revision protocol owner, remove the duplicate generated Harmos Rust module and generator output, and prove vendored descriptor parity plus the five protocol journeys on Windows. Native host, SDK and tool consumer adoption remains in 3.1/3.8.2; protocol adoption precedes it to keep the tree buildable.
- [ ] 3.2 After 2.9, move phases and criteria from the template to the test case and call `materialize` per parameter set, with rematerialization on a changed PluginRun-origin digest.
- [ ] 3.3 After 2.9 and 3.12–3.15, build the manager with live queued/initializing/stepping/evaluating/cleaning-up state and finished/cancelled/interrupted termination; invoke initialize, steps, criteria and cleanup with retries and interactive commands, addressing cancel to executions only while driven by a run.
- [ ] 3.3.1 Adopt the published Harmos scoped-invocation capability at the execution-scoped plugin dispatch call site: the selected placement currently sends an empty `Request.scope`, which the SDK correctly refuses for lifecycle envelopes. The precise upstream request is in `design.md`; pure materialization, attachment, wire/native owner replacement and evidence/outcome work remain independent.
- [ ] 3.4 Open and close catalog and test case streams, retain source time, receipt time and durable position with declared axes, and ingest live samples in arrival order without claiming the host owns source clocks.
- [ ] 3.5 Compute stream statistics per execution and step range.
- [ ] 3.6 Replace Settings › Plugins with managed and remote entries and their states.
- [ ] 3.7 Adopt unary-only plugin operations, migrate streamed sample operations, apply internal visibility and unset-policy defaults, resolve Duration setting fields and require the placement stream window after owner confirmation; refresh vendored Harmos sources and verify descriptor parity.
- [x] 3.7.1 Verify that the vendored `proto/harmos/v1/plugin.proto` already matches the requested Harmos revision byte for byte; retain it without a local protocol fork.
- [ ] 3.7.2 Migrate plugin/host call sites and operation policies, stream-setting resolution, required stream window and descriptor parity after dependency adoption.
- [ ] 3.8 Wave 2: extend Harmos client-facing operation declarations/codecs to accept generated protobuf types and descriptors before adoption; preserve bounded decoding, cancellation, backpressure, service paths and streaming envelopes, then pin the published revision.
- [x] 3.8.1 Inspect the pre-client Harmos revision: `ProtobufOperation`, `OperationFactory::protobuf` and descriptor-backed admission provide server registration and internal invocation; identify the managed client's `Portable` requirement and request its replacement. The published answer and remaining admission request are tracked below.
- [x] 3.8.1.1 Obtain generated-message managed client declarations/codecs and call handles from the owner-selected published Harmos revision; prove unary, upload, download, duplex, input finish, concurrent input, cancellation settlement, schema/fingerprint refusal and encoded-size admission against its real host on Windows. Descriptor-backed client memory admission remains separately open in 3.8.1.2; no local codec/projection shim.
- [ ] 3.8.1.2 Adopt the admission bound at the revision that supplies it: Harmos owns the open decoded-memory admission follow-up as an additive `ClientSettings` bound with unchanged client call shapes. Adopt the selected generated client now with wire-message and queue bounds from configuration; this follow-up precedes neither 2.8.3 nor any other adoption task.
- [ ] 3.8.2 Pin the requested published revision in the workspace and migrate every removed facade/SDK consumer to its explicit owner without path overrides or compatibility shims.
- [ ] 3.8.2.3 Keep the existing registry Harmos dependency explicit in the four native host/client packages until their atomic owner cutover; the workspace `harmos` pin belongs to the new product SDK and has no host features. This sequencing preserves buildable commits without a new adapter.
- [x] 3.8.2.1 Replace direct native `Described` derive imports in domain/runtime with the selected published field-reading derive owner, preserving native stored shapes; domain/runtime package Rust checks, the saved-record capture journey and transaction journal journey pass on Windows.
- [x] 3.8.2.2 Replace native transaction identity declarations and the local transaction catalog macro with ordinary implementations, retaining journal identities, version refusal, replay and record collection. Schema-free v1 identities remain ordinary inherent definitions because Harmos `Identified` requires `Described`; native catalog values follow 3.8.2. The focused transaction journal journey and package Rust check pass on Windows.
- [ ] 3.9 Wave 3: move every host operation declaration and handler together, one file per operation in `dokime-operations`, with protocol requests/replies and Rust callee-owned deadlines/retries; define narrow resources without runtime dependencies, register directly, and delete `hosted!` and `Host`/`Serve` forwarding area by area.
- [ ] 3.10 Wave 3: move server logs/trace replay, support bundles, transfers/upload transactions, paging/frontier closure, watches, plot deltas/reduction, diagnostics and redaction into ordinary operations with their meaningful behavior tests.
- [ ] 3.11 Wave 3: put startup `main.rs`, listener configuration and the one client authorization hook in runtime; preserve pairing/key persistence, rotation, settings and shutdown; retain execution manager, effects, plugin entries, transactions, state and cycle transitions under the runtime host owner.

- [ ] 3.12 Host wave, before 3.3: implement Execution as identity/cycle/origin/frozen schemas/evidence/outcome/seal with open/closed/sealed only; freeze TestCase from plugin catalog or Import from script-local catalog without plugin registration/global overwrite. Migrate consumers and recovery so interrupted runs close their executions but import drafts never enter run recovery; remove superseded step-owning execution state.
- [ ] 3.13 Host wave, before 3.3: implement not-evaluated/evaluated/importer-declared outcomes and evaluated-only cycle verdict folding; count declared/unevaluated separately in summaries/reports, reject import criteria and prevent Passed for cycles with no evaluated result. Prove empty, declared-only, unevaluated-only and mixed-cycle cases deterministically.
- [ ] 3.14 Host wave, before 3.3: expose backend execution capabilities for start/rerun/cancel/inspect/export and make desktop show only supported actions; reject cancel outside an active run and import start/rerun. Prove origin/state combinations and migrate affected consumers.
- [ ] 3.15 Host wave, before 3.3: separate fixed-snapshot document rendering into temporary returned output from durable report publication/sealing for runs and imports; migrate report callers and prove render failure publishes nothing. Update affected README/guides and focused lifecycle/schema/recovery proof without implementing later import admission here.

## 4. SDKs

- [ ] 4.1 Wave 4, Rust: make `sdk/rust` the `dokime` product crate with `use dokime::prelude::*;`, plugin authoring and the absorbed client half, generated step/criterion operations and derived `materialize`; product dependencies stop at protocol and compile no host. Align Harmos's `sdk/rust` entry point as `harmos`.
- [x] 4.1.2 Interim package ownership: rename the old facade to `dokime-facade` with library `dokime`; 4.8 migrates its consumers and 5.11 deletes it. The CLI package remains `dokime-cli` and its installed binary remains `dokime`. Include macro crate discovery and the old starter dependency in this rename; focused formatting/checking accepts a package to preserve concurrent ownership. Cargo resolution and the renamed-facade operation journey pass on Windows.
- [ ] 4.2 Move the plugin, installer and proto commands into `dokime`, add `dokime new --lang`, and delete `crates/cargo-dokime`.
- [ ] 4.3 Python: flat public APIs with value composition and explicit close, no required context managers; pure package with the plugin half and the generated client half; rewrite the starter and `dokime new --lang python`.
- [ ] 4.4 C#: `sdk/csharp` with both halves, a starter and `dokime new --lang csharp`. Generate message code from the proto files during the project build (`Grpc.Tools`) and delete the committed generated sources; the proto files are the only committed source.
- [ ] 4.5 C++: `sdk/cpp` with both halves, a starter and `dokime new --lang cpp`. Generate message code from the proto files in CMake against the protobuf the build uses and delete the committed generated headers and sources. The generated Rust in `dokime-protocol` stays committed; Python generates when its package is built.
- [ ] 4.6 Migrate the host telemetry and OBC plugins and the samples.
- [ ] 4.7 Add `dokime plugin bundle` for any executable and build each SDK's sample plugin in parallel in CI, running one shared story against all of them.
- [ ] 4.8 Wave 4: redirect CLI hosting and desktop embedding from the server/facade to the runtime library; keep remote commands on the SDK client half and migrate host/distribution facade helpers to their owners.
- [ ] 4.9 Wave 4: replace server/desktop build-script facade imports with checked-in protocol output and definitions-only settings; remove build-time host compilation.

## 5. Removal

- [ ] 5.1 Delete `<template>.run` and `.signal`, the signal and evidence-page protocol, the plugin walker and their transport.
- [ ] 5.2 Delete the execution hand-off layer and rename what remains as the execution manager.
- [ ] 5.3 Delete embedded plugins, the PyO3 SDK, the Python backend, stubs and wheel tasks.
- [ ] 5.4 Delete leases, sample and record delivery, replay and restart recovery during an execution.
- [ ] 5.5 Remove or rework the tests tied to the old loop; update the README, guides and authoring guidance.
- [ ] 5.6 Delete the template, step, criterion, suite, catalog and plugin macros and their tests; keep derives for messages, rows, records, parameters and settings.
- [ ] 5.7 Wave 5: delete the separate `dokime-client` crate after its consumers use the SDK client half; remove host-enabling SDK features and prove both plugin/client dependency trees stop at protocol within the product graph.
- [ ] 5.8 Wave 5: delete `dokime-server`, per-service adapters, superseded generation/build dependencies and dead wire conversions after all behavior and consumers move.
- [ ] 5.9 Wave 5: dissolve `dokime-domain`; migrate definitions to protocol messages, admission/expansion/expression rules to their operations and persisted shapes beside storage, migrating all consumers and deleting duplicate models and superseded guidance.
- [ ] 5.10 Wave 5: remove the operations contract-only surface and `runtime` feature once no consumer needs them; preserve Harmos built-in declaration/handler co-location and enforce protocol → storage → operations → runtime → CLI/desktop dependencies without reverse edges.
- [ ] 5.11 Wave 5: retain `dokime-tracing` as small development support and reconcile manifests, examples, README and guides with the target crate ownership.

## 6. Final Validation

- [ ] 6.1 Run focused operations, runtime and desktop tests for plugin entries, materialization, execution order, retries, sealing, streams and criteria; verify protobuf handler routing, retained server behavior and host-free SDK boundaries.
- [ ] 6.2 Run the system stories with a managed plugin and with a Python script that serves and drives its own suite.
- [ ] 6.3 Run the combined-tree Rust and settings/plot checks after all source handoffs; repair failures and retain native platform limitations.
- [x] 6.3.1 Rerun Python checking after the benchmark owner repairs import ordering in `tools/python/tests/integration/test_desktop_bench.py`; the Windows check passes with 110 tooling tests. That path remains outside this worker's ownership.
- [x] 6.3.2 Clear the earlier workstation interruption with Windows whole-tree Rust checking and the Tests artifact build after restoring free space above the cutoff; three Rust selections, cargo-machete and the archive's 16 compile-contract tests pass. The first archive attempt's allocation failures are repaired by using one Cargo job. Final feature validation in 6.3 and native Linux proof remain open.

## Workflow Follow-Up

- Tell private plugin owners the migration once a published Nightly carries the change.
- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
- SDK system tests run in Nightly only, not on pull requests.
- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- This change owns the execution model before manager construction; imported-executions depends on it and follows the SDK wave and storage-interface for admission, draft publication and SDK/report integration.
- Start `storage-interface` only after wave 5: benchmark before/after with multi-day data, extend Harmos storage first, then extract `dokime-storage`; no storage extension or extraction in the gRPC waves.
- Run only focused tasks until combined source closure; final native proof remains separate.

## Combined Tree Integration

Source inventory: specification merge and validation under `openspec/`, Python
SDK export lists in `sdk/python/python/dokime/{plot.py,v1/__init__.py}`, shared
`.github/workflows/{publish,verify}.yml`, `docs/tooling.md`, and the existing
`integration-plots` benchmark and its notes in `../plot-pipeline/tasks.md`.

- [x] Merge `feat/gp-organization-pack`, retaining both sides' specification evidence and completed implementation tasks.
- [x] Repair strict OpenSpec validation and pass all 80 specifications and changes.
- [x] Run whole-tree formatting; repair Python export ordering and pass Python checking (107 tooling tests).
- [x] Pass TypeScript and repository checking, and protocol drift checking (three focused protocol tests).
- [x] Pass focused release tooling (35 tests) and workflow routing (12 tests); honor `build-run` for Windows Rust downloads and refresh Harmos alignment guidance.
- [x] Repeat release staging from clean output with both platforms' existing dev packages (17 upload entries); preserve the earlier output. Current-source packaging and release-profile/hosted proof remain separate.
- [x] Pass the combined Rust check: dev/test, desktop and bench-profile Clippy selections and cargo-machete. The unreachable `tokio::process::Command::new` configuration warning remains nonfatal.
- [x] Complete one bounded time-series profiling pass and record its findings under plot-pipeline benchmark notes without changing budgets or thresholds. All four native comparisons pass; the live increase remains measured, and no plain cause was established.

## Repository Root and Workflow Repairs

- [x] Move explicit tool configuration to `tools/config/`, remove unused root About and Typos configuration, and reconcile managed ownership.
- [x] Move contributor/security guides, environment example and screenshots under `docs/`; update readers, links, snippets and Markdown policy.
- [x] Generate notices under `target/notices/` and preserve distribution, installer and wheel notice contents.
- [x] Repair compiler cache save inputs and preserve job-level Setup cache settings.
- [x] Run repository/Python checks and formatting; exercise notices generation/staging through focused tooling tests.
- [x] Rerun the combined repository/Python entry points after the concurrent organization edits are formatted.
- [x] Complete a local artifact release staging rehearsal: clean-output `--profile dev --target all --stage-only` passed with both platforms' existing packages and 17 upload entries. Native executables identify `ca2fc48d`; full release-profile and current-source packaging proof remain separate.

## Desktop and Repository Integration

- [x] Reconcile the pending privacy guide formatter noise, merge the organization-pack specification branch and update moved-path references.
- [x] Regenerate the runtime report, bundled brand resources and desktop icons after the default appearance change.
- [x] Publish Product compiler records only after successful compilation, retaining failed output separately; prove replacement and failure behavior through tooling integration tests.
- [x] Pass combined formatting, repository, Python, TypeScript and Rust checks and the Tests artifact build.
- [x] Rerun plot and settings TypeScript unit/integration tests, shell organization tests and the focused runtime captured-head test.

## Hosted Failure Repair and Policy Adoption

- [x] Repair archived protocol compiler inputs and prove both protocol tests with `TEST_PROFILE` selecting transferred fixtures.
- [x] Align remote-operation, plot-watch and terminal journeys with the agreed contracts; pass their focused tests.
- [x] Repair native plugin counter collection and store transitions, and keep plot statistics progressing during redraws; pass the focused live-plot story.
- [x] Replace the organization-generation radius constant with an authored fixture override and pass the focused Python test.
- [x] Update the density system journey to drive the appearance combobox; pass the TypeScript checks.
- [x] Trim company policy while retaining project-owned tooling configuration.
- [x] Pass formatting, repository/Python/Rust checks, Tests Build and the focused repaired tests.
- [ ] Validate affected packaged system stories in hosted CI; no local focused packaged runner is documented. Linux proof remains hosted; no push is authorized.

## PR 411 Python Integration Repair

- [x] Reproduce the hosted JSON decode failure with the workflow environment and transferred Windows wheel; identify the expected `--features` rejection diagnostic.
- [x] Supply Cargo metadata and native compiler fixtures in the archive test; verify both platform selections and transferred files without removing collector coverage.
- [x] Capture and assert the expected unfocused `--features` rejection in the dispatcher test.
- [x] Pass the repaired Windows Python integration lane (94 tooling and 275 SDK tests), formatting, Python check and repository check; retain the repair in a local commit without pushing. Native Linux proof remains hosted.
- [x] Read both failed TypeScript System jobs and their retained traces; repair pairing origins, diagnostics filenames, installed-pack identity, update comboboxes, stale recovery controls and two-desktop startup/navigation. Restore Product-generated clients in the artifact-consuming system task.
- [x] Pass the focused Windows configuration, installed-pack, owned-backend and two-desktop stories, combined formatting and Python/repository/TypeScript checks; retain the repairs locally without pushing. Use the existing Chrome override for the report browser on this workstation. Native Linux and hosted candidate reruns remain unverified.
- [x] Read the completed head run's other failure, `CI / Required`; confirm it reports the failed test matrix and preserve its gate.

## PR 411 Plot Transport Repair

- [x] Fit contour grids by exact whole-cell merging with actual returned resolution; fit complete GRID_CHART frames and TRAJECTORY timestamp columns without changing budgets. Source inventory: runtime plot payload/preview owners and server plot fitting/wire owners, their focused integration journey, GUI contour geometry/live-data notifications and their existing plot journey, `e2e/native-desktop.mjs`, `mise.toml`, tooling/product guides and plot-kinds requirements. Pass backend proof and the existing 46-test GUI journey, including nonuniform-edge cell/band/mask geometry.
- [x] Deduplicate background refresh notifications until the plot recovers; pass the existing GUI plot integration journey including repeated errors and recovery (46 tests).
- [x] Honor the existing GUI integration task's `--select` with `--prepared`, retaining the normal regenerate/check path and proving the focused selection through that task; document the existing cycle journey filter.
- [x] Pass the deterministic focused gRPC integration journey for dense contour means, complete playback frames, trajectory admission, atomic shared watch budgets and actionable irreducible errors; pass the existing telemetry/plot-watch story and the focused mean/min/max/gap reducer unit test.
- [x] Pass the focused desktop execution story containing scene cycle with no new error notifications using the current plot implementation; reopen its three sealed execution reports. Compare semantic error-record identities over the complete history after the Library's deliberate invalid-input checks; the nonuniform-edge renderer is covered.
- [x] Repair the existing owned report-browser fixture when Chrome stays alive after DevTools close: retain closed-tab checks and the existing deadline, terminate only its owned disposable child and await physical exit. Startup and cycle cleanup pass on Windows.
- [x] Pass format, Rust/Python/TypeScript and repository checks and the Tests artifact build; preserve the benchmark worker's paths and report native Linux proof separately. The archive retry uses one Cargo job after allocation failures; its 16 compile-contract tests pass. Windows proof only; Linux and hosted reruns remain unverified.

## Protocol and SDK Branch Convergence

- [ ] Merge the protocol, Python, C# and C++ branches in order, with one merge commit each; resolve service declarations in authoritative protos and regenerate Rust output.
- [ ] Retire the native Python Cargo member, environment dependencies and binding declarations, and stale C#/C++ protocol generation/drift ownership.
- [ ] Generate all authoritative client services in Python, C# and C++; migrate canonical types and PromptCatalog, and prove plugin Add then List in each SDK's in-process client story.
- [ ] Merge the current integration branch, regenerate and run final focused proof: format, Rust/Python/repository checks, protocol stories, each SDK build/check/stories and Tests artifact build. Report host-owned failures and native platform limitations truthfully.
