# Remote Plugins and the Automation Contract Tasks

Status: proposed; not locked. No task starts before 1.1 closes.

## 1. Contract

- [ ] 1.1 Decide the seven open decisions in `design.md` with the owner and record each outcome there; lock the source inventory before editing.
- [ ] 1.2 Fold `docs/remote-plugins.md` from draft PR #380 into the maintained guides once the contract is locked, and close the PR.

## 2. Plugins by URL

- [ ] 2.1 Connect to plugin URLs over `harmos.plugin.v1`, read the Dokime catalog sections and run templates as `<template>.run` and `<template>.signal` with Host-visibility child calls.
- [ ] 2.2 Replace Settings › Plugins with the URL list and its connection states, with automatic reconnect.
- [ ] 2.3 Move host telemetry per decision 4 and make OBC the reference remote plugin that tests start and connect to by URL.
- [ ] 2.4 Delete the install, bundle, feed, installer-entry and in-process Python paths listed in `design.md`, with their tests, tasks and guidance.

## 3. Automation

- [ ] 3.1 Attach the exported protos to every release under `dokime.automation.v1` and document the versioning rule.
- [ ] 3.2 Add the `grpcio` and `grpcurl` examples to the remote walkthrough; retire `dokime.Backend`.

## 4. Validation

- [ ] 4.1 Focused runtime, server and GUI tests for connection states, catalog sections and template execution; the system stories that run the OBC plugin by URL.
- [ ] 4.2 Tell private plugin owners the migration once a published Nightly carries the change.
