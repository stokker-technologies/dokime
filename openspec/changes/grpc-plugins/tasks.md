# gRPC Plugins Tasks

## 1. Contract

- [ ] 1.1 Settle the open points in `design.md` with the owner and record each outcome there.
- [ ] 1.2 Reconcile the overlapping changes and write the spec deltas that retire superseded requirements.
- [ ] 1.3 Lock the source inventory: producers, consumers, tests, fixtures, examples, tasks, README and guidance.

## 2. Protocol

- [x] 2.1 Define and check in `dokime.v1` host operations, `TestCase`, `Step`, `Criterion` and `Statistics`; generate the protocol crate.
- [ ] 2.2 Add the host operations plugins need that exist only as evidence-page fields today: progress, log, traceability, document parts, record reads and statistics.
- [ ] 2.3 Replace JSON catalog readers and producers with `dokime.v1` messages; add running-host, plugin URL and bundle export sources. Embedded host-source export is tracked in 2.6.
- [ ] 2.4 Serve settings as a host operation and delete settings injection at start and the applied-versus-saved comparison.
- [ ] 2.5 Add the per-execution resource store (`ctx.set`, `ctx.get`) for Rust and remove template fields and `Own`.
- [x] 2.6 Export embedded Dokime and Harmos protocol sources through `dokime proto export --out`, preserving package paths without opening storage.
- [ ] 2.7 Attach the checked-in protocol sources to releases through the archive owned by `release-layout` task 2.3.
- [ ] 2.8 Wave 2: make proto files authoritative, put generated definitions and light helpers in protocol, document cross-language semantics for each SDK, and inventory Harmos native state/recovery/storage semantics retained in `harmos-domain`; move wire mirrors out.

- [ ] 2.9 Host-wave prerequisite owned here, before 3.2/3.3: define Origin with PluginRun containing frozen TestCase/plugin/template/digest/process start time and Import containing its definition/manifest; reserve removed TestCase identity fields, regenerate bindings and migrate persistence, materialization and plugin-call envelopes. No separate execution-definition or Run type.

## 3. Host

- [ ] 3.1 Adopt the Harmos `grpc-plugins` revision and add `Plugin::{Managed, Remote}` with the add and remove operations and entry states.
- [ ] 3.2 After 2.9, move phases and criteria from the template to the test case and call `materialize` per parameter set, with rematerialization on a changed PluginRun-origin digest.
- [ ] 3.3 After 2.9 and 3.12–3.15, build the manager with live queued/initializing/stepping/evaluating/cleaning-up state and finished/cancelled/interrupted termination; invoke initialize, steps, criteria and cleanup with retries and interactive commands, addressing cancel to executions only while driven by a run.
- [ ] 3.4 Open and close catalog and test case streams, retain source time, receipt time and durable position with declared axes, and ingest live samples in arrival order without claiming the host owns source clocks.
- [ ] 3.5 Compute stream statistics per execution and step range.
- [ ] 3.6 Replace Settings › Plugins with managed and remote entries and their states.
- [ ] 3.7 Adopt unary-only plugin operations, migrate streamed sample operations, apply internal visibility and unset-policy defaults, resolve Duration setting fields and require the placement stream window after owner confirmation; refresh vendored Harmos sources and verify descriptor parity.
- [ ] 3.8 Wave 2: extend Harmos client-facing operation declarations/codecs to accept generated protobuf types and descriptors before adoption; preserve bounded decoding, cancellation, backpressure, service paths and streaming envelopes, then pin the published revision.
- [ ] 3.9 Wave 3: move every host operation declaration and handler together, one file per operation in `dokime-operations`, with protocol requests/replies and Rust callee-owned deadlines/retries; define narrow resources without runtime dependencies, register directly, and delete `hosted!` and `Host`/`Serve` forwarding area by area.
- [ ] 3.10 Wave 3: move server logs/trace replay, support bundles, transfers/upload transactions, paging/frontier closure, watches, plot deltas/reduction, diagnostics and redaction into ordinary operations with their meaningful behavior tests.
- [ ] 3.11 Wave 3: put startup `main.rs`, listener configuration and the one client authorization hook in runtime; preserve pairing/key persistence, rotation, settings and shutdown; retain execution manager, effects, plugin entries, transactions, state and cycle transitions under the runtime host owner.

- [ ] 3.12 Host wave, before 3.3: implement Execution as identity/cycle/origin/frozen schemas/evidence/outcome/seal with open/closed/sealed only; freeze TestCase from plugin catalog or Import from script-local catalog without plugin registration/global overwrite. Migrate consumers and recovery so interrupted runs close their executions but import drafts never enter run recovery; remove superseded step-owning execution state.
- [ ] 3.13 Host wave, before 3.3: implement not-evaluated/evaluated/importer-declared outcomes and evaluated-only cycle verdict folding; count declared/unevaluated separately in summaries/reports, reject import criteria and prevent Passed for cycles with no evaluated result. Prove empty, declared-only, unevaluated-only and mixed-cycle cases deterministically.
- [ ] 3.14 Host wave, before 3.3: expose backend execution capabilities for start/rerun/cancel/inspect/export and make desktop show only supported actions; reject cancel outside an active run and import start/rerun. Prove origin/state combinations and migrate affected consumers.
- [ ] 3.15 Host wave, before 3.3: separate fixed-snapshot document rendering into temporary returned output from durable report publication/sealing for runs and imports; migrate report callers and prove render failure publishes nothing. Update affected README/guides and focused lifecycle/schema/recovery proof without implementing later import admission here.

## 4. SDKs

- [ ] 4.1 Wave 4, Rust: make `sdk/rust` the `dokime` product crate with `use dokime::prelude::*;`, plugin authoring and the absorbed client half, generated step/criterion operations and derived `materialize`; product dependencies stop at protocol and compile no host. Align Harmos's `sdk/rust` entry point as `harmos`.
- [ ] 4.2 Move the plugin, installer and proto commands into `dokime`, add `dokime new --lang`, and delete `crates/cargo-dokime`.
- [ ] 4.3 Python: flat public APIs with value composition and explicit close, no required context managers; pure package with the plugin half and the generated client half; rewrite the starter and `dokime new --lang python`.
- [ ] 4.4 C#: `sdk/csharp` with both halves, a starter and `dokime new --lang csharp`.
- [ ] 4.5 C++: `sdk/cpp` with both halves, a starter and `dokime new --lang cpp`.
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

## Workflow Follow-Up

- Tell private plugin owners the migration once a published Nightly carries the change.
- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
- SDK system tests run in Nightly only, not on pull requests.
- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
- This change owns the execution model before manager construction; imported-executions depends on it and follows the SDK wave and storage-interface for admission, draft publication and SDK/report integration.
- Start `storage-interface` only after wave 5: benchmark before/after with multi-day data, extend Harmos storage first, then extract `dokime-storage`; no storage extension or extraction in the gRPC waves.
- Run only focused tasks until combined source closure; final native proof remains separate.

## Repository Root and Workflow Repairs

- [x] Move explicit tool configuration to `tools/config/`, remove unused root About and Typos configuration, and reconcile managed ownership.
- [x] Move contributor/security guides, environment example and screenshots under `docs/`; update readers, links, snippets and Markdown policy.
- [x] Generate notices under `target/notices/` and preserve distribution, installer and wheel notice contents.
- [x] Repair compiler cache save inputs and preserve job-level Setup cache settings.
- [x] Run repository/Python checks and formatting; exercise notices generation/staging through focused tooling tests.
- [x] Rerun the combined repository/Python entry points after the concurrent organization edits are formatted.
- [ ] Complete an artifact release rehearsal; local `--profile dev --stage-only` is blocked by missing Windows package artifacts.

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
