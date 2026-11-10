# gRPC Plugins Tasks

## 1. Contract

- [ ] 1.1 Settle the open points in `design.md` with the owner and record each outcome there.
- [ ] 1.2 Reconcile the overlapping changes and write the spec deltas that retire superseded requirements.
- [ ] 1.3 Lock the source inventory: producers, consumers, tests, fixtures, examples, tasks, README and guidance.

## 2. Protocol

- [ ] 2.1 Define `dokime.v1`: host operations for clients and plugins, `TestCase`, `Step`, `Criterion`, `Statistics`, with checked-in protos attached to releases.
- [ ] 2.2 Add the host operations plugins need that exist only as evidence-page fields today: progress, log, traceability, document parts, record reads and statistics.
- [ ] 2.3 Replace the JSON catalog sections with `dokime.v1` messages and add `dokime proto export`.
- [ ] 2.4 Serve settings as a host operation and delete settings injection at start and the applied-versus-saved comparison.
- [ ] 2.5 Add the per-execution resource store (`ctx.set`, `ctx.get`) for Rust and remove template fields and `Own`.

## 3. Host

- [ ] 3.1 Adopt the Harmos `grpc-plugins` revision and add `Plugin::{Managed, Remote}` with the add and remove operations and entry states.
- [ ] 3.2 Move phases and criteria from the template to the test case and call `materialize` per parameter set, with rematerialization on a changed digest.
- [ ] 3.3 Build the execution manager: initialize, steps, criteria and cleanup as operation calls, with retries, cancel, interactive commands and sealing on a changed start time.
- [ ] 3.4 Open and close catalog and test case streams and ingest samples in arrival order.
- [ ] 3.5 Compute stream statistics per execution and step range.
- [ ] 3.6 Replace Settings › Plugins with managed and remote entries and their states.

## 4. SDKs

- [ ] 4.1 Rust: extract authoring into `sdk/rust` as `dokime-sdk` with no dependency on `dokime-runtime`, over the Harmos SDK, with generated step and criterion operations and derived `materialize`; `dokime-client` as the client half.
- [ ] 4.2 Move the plugin, installer and proto commands into `dokime`, add `dokime new --lang`, and delete `crates/cargo-dokime`.
- [ ] 4.3 Python: pure package with the plugin half and the generated client half; rewrite the starter and `dokime new --lang python`.
- [ ] 4.4 C#: `sdk/csharp` with both halves, a starter and `dokime new --lang csharp`.
- [ ] 4.5 C++: `sdk/cpp` with both halves, a starter and `dokime new --lang cpp`.
- [ ] 4.6 Migrate the host telemetry and OBC plugins and the samples.

## 5. Removal

- [ ] 5.1 Delete `<template>.run` and `.signal`, the signal and evidence-page protocol, the plugin walker and their transport.
- [ ] 5.2 Delete the execution hand-off layer and rename what remains as the execution manager.
- [ ] 5.3 Delete embedded plugins, the PyO3 SDK, the Python backend, stubs and wheel tasks.
- [ ] 5.4 Delete leases, sample and record delivery, replay and restart recovery during an execution.
- [ ] 5.5 Remove or rework the tests tied to the old loop; update the README, guides and authoring guidance.

## 6. Final Validation

- [ ] 6.1 Run focused runtime, server and desktop tests for plugin entries, materialization, execution order, retries, sealing, streams and criteria.
- [ ] 6.2 Run the system stories with a managed plugin and with a Python script that serves and drives its own suite.

## Workflow Follow-Up

- Tell private plugin owners the migration once a published Nightly carries the change.
- Delivery uses targeted validation and merges to dev without waiting for CI; breakage is fixed in follow-ups.
- SDK system tests run in Nightly only, not on pull requests.
