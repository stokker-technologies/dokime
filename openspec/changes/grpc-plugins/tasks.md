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

## 3. Host

- [ ] 3.1 Adopt the Harmos `grpc-plugins` revision and add `Plugin::{Managed, Remote}` with the add and remove operations and entry states.
- [ ] 3.2 Move phases and criteria from the template to the test case and call `materialize` per parameter set, with rematerialization on a changed digest.
- [ ] 3.3 Build the execution manager: initialize, steps, criteria and cleanup as operation calls, with retries, cancel, interactive commands and sealing on a changed start time.
- [ ] 3.4 Open and close catalog and test case streams and ingest samples in arrival order.
- [ ] 3.5 Compute stream statistics per execution and step range.
- [ ] 3.6 Replace Settings › Plugins with managed and remote entries and their states.
- [ ] 3.7 Adopt unary-only plugin operations, migrate streamed sample operations, apply internal visibility and unset-policy defaults, resolve Duration setting fields and require the placement stream window after owner confirmation; refresh vendored Harmos sources and verify descriptor parity.

## 4. SDKs

- [ ] 4.1 Rust: extract authoring into `sdk/rust` as `dokime-sdk` with no dependency on `dokime-runtime`, over the Harmos SDK, with generated step and criterion operations and derived `materialize`; `dokime-client` as the client half.
- [ ] 4.2 Move the plugin, installer and proto commands into `dokime`, add `dokime new --lang`, and delete `crates/cargo-dokime`.
- [ ] 4.3 Python: pure package with the plugin half and the generated client half; rewrite the starter and `dokime new --lang python`.
- [ ] 4.4 C#: `sdk/csharp` with both halves, a starter and `dokime new --lang csharp`.
- [ ] 4.5 C++: `sdk/cpp` with both halves, a starter and `dokime new --lang cpp`.
- [ ] 4.6 Migrate the host telemetry and OBC plugins and the samples.
- [ ] 4.7 Add `dokime plugin bundle` for any executable and build each SDK's sample plugin in parallel in CI, running one shared story against all of them.

## 5. Removal

- [ ] 5.1 Delete `<template>.run` and `.signal`, the signal and evidence-page protocol, the plugin walker and their transport.
- [ ] 5.2 Delete the execution hand-off layer and rename what remains as the execution manager.
- [ ] 5.3 Delete embedded plugins, the PyO3 SDK, the Python backend, stubs and wheel tasks.
- [ ] 5.4 Delete leases, sample and record delivery, replay and restart recovery during an execution.
- [ ] 5.5 Remove or rework the tests tied to the old loop; update the README, guides and authoring guidance.
- [ ] 5.6 Delete the template, step, criterion, suite, catalog and plugin macros and their tests; keep derives for messages, rows, records, parameters and settings.
- [ ] 5.7 Make a client-only build free of the host runtime: `dokime-client` and the SDK client halves depend on the protocol definitions, not on a Harmos feature that enables the runtime.
- [ ] 5.8 Remove the server build script's dependency on the product facade: protocol output comes from the checked-in definitions, and settings definitions get a definitions-only home.

## 6. Final Validation

- [ ] 6.1 Run focused runtime, server and desktop tests for plugin entries, materialization, execution order, retries, sealing, streams and criteria.
- [ ] 6.2 Run the system stories with a managed plugin and with a Python script that serves and drives its own suite.
- [ ] 6.3 Run the combined-tree Rust and settings/plot checks after all source handoffs; repair failures and retain native platform limitations.

## Workflow Follow-Up

- Tell private plugin owners the migration once a published Nightly carries the change.
- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
- SDK system tests run in Nightly only, not on pull requests.
- Integrate with the other open changes on `feat/grpc-plugins` in the order in its design.
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
- [ ] Pass combined formatting, repository, Python, TypeScript and Rust checks and the Tests artifact build.
- [ ] Rerun plot and settings TypeScript unit/integration tests, shell organization tests and the focused runtime captured-head test.
